pub mod db {
    const TOKEN_DEFAULT_FILE_NAME: &str = "coselpro_token.json";
    use crate::db::credentials::db::Credentials;
    use chrono::{Duration, Utc};
    use homedir::my_home;
    use pgdatetime::Timestamp;
    use postgrest::Postgrest;
    use reqwest::Response;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_reader, json};
    use std::env::current_dir;
    use std::error::Error;
    use std::fs::{File, OpenOptions};
    use std::io::BufReader;
    use std::path::PathBuf;

    /// CoSelPro connection token structure
    /// implement save to file and load from file
    /// Timestamp of expiration to manage auto-renewal
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Token {
        token: String,
        expire: Timestamp,
        user_name: String,
    }
    impl Token {
        pub fn to_string(&self) -> &String {
            &self.token
        }
        pub fn expire(&self) -> &Timestamp {
            &self.expire
        }
        pub fn user_name(&self) -> &String {
            &self.user_name
        }

        /// Return true if token is still be active.
        /// duration defines safety time in mn before expiration is reached.
        /// duration is defaulted to 5 mn
        pub fn active(&self, safety_duration: Option<u8>) -> bool {
            let duration = Duration::minutes(safety_duration.unwrap_or_else(|| 5) as i64);
            self.expire() > &Timestamp::from((Utc::now() + duration).timestamp())
        }

        fn get_dir() -> PathBuf {
            match my_home() {
                Ok(dir) => dir.unwrap_or_else(|| PathBuf::from("")),
                Err(e) => {
                    eprintln!("Failed to get user home directory. ({e})");
                    current_dir().unwrap_or_else(|e| {
                        eprintln!("Failed to get current directory. ({e})");
                        PathBuf::from("")
                    })
                }
            }
            .join(TOKEN_DEFAULT_FILE_NAME)
        }

        /// Save token on user profile
        pub fn save(&self) -> Result<(), Box<dyn Error>> {
            let file = match OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(Self::get_dir())
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open the file{}", e);
                    return Err(Box::new(e));
                }
            };
            match serde_json::to_writer(file, &self) {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Failed to serialize the file {}", e);
                    Err(Box::new(e))
                }
            }
        }

        /// Create new token from token file on the user profile
        pub fn load() -> Option<Token> {
            let file = match File::open(Self::get_dir()) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open the file{}", e);
                    return None;
                }
            };
            let reader = BufReader::new(file);
            from_reader(reader).unwrap_or_else(|e| {
                eprintln!("Failed to deserialize the file {}", e);
                None
            })
        }

        async fn parse_response(response: Response) -> Option<Token> {
            match response.error_for_status_ref() {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Getting token: HTTP error: {}", response.status());
                    return None;
                }
            };
            match response.json::<Token>().await {
                Ok(token) => match token.save() {
                    Ok(_) => Some(token),
                    Err(_) => {
                        eprintln!("Error saving token");
                        Some(token)
                    }
                },
                Err(e) => {
                    eprintln!("Renewing token. Credential failed. {e}");
                    None
                }
            }
        }
        /// Create new token from active connection and user credentials
        pub async fn from_credentials(
            client: &Postgrest,
            credentials: &Credentials,
        ) -> Option<Token> {
            let response = match client.rpc("login",
            json!({ "username": credentials.get_login(), "pass": credentials.get_password_md5()})
                .to_string())
            .execute().await {
                Ok(response) => response,
                Err(_) => {
                    eprintln!("Getting token: Unable to connect to CoSelPro API");
                    return None
                }
            };
            Self::parse_response(response).await
        }

        /// Extend token with active connection
        pub async fn renew(&self, client: &Postgrest) -> Option<Token> {
            let response = match client
                .rpc("extend_token", "")
                .auth(&self.token)
                .execute()
                .await
            {
                Ok(response) => response,
                Err(_) => {
                    eprintln!("Renewing token: Unable to connect to CoSelPro API");
                    return None;
                }
            };
            Self::parse_response(response).await
        }
    }
}

// Cannot parallelize test as they read and write to the same file.
#[cfg(test)]
mod tests {
    const UNIT_TEST_POSTGREST_SERVER: &str = "http://proliant:3000";

    use super::*;
    use crate::db::credentials::db::Credentials;
    use db::Token;
    use postgrest::Postgrest;
    use tokio;

    #[tokio::test]
    async fn get_token() {
        let client = Postgrest::new(UNIT_TEST_POSTGREST_SERVER).schema("rest");
        let credentials = Credentials::new("consult", "consult");
        let token = Token::from_credentials(&client, &credentials).await;
        match token {
            Some(token) => assert!(token.active(None)),
            None => {
                eprintln!("Unable to extract token from server.");
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn save_load_token() {
        let client = Postgrest::new(UNIT_TEST_POSTGREST_SERVER).schema("rest");
        let credentials = Credentials::new("consult", "consult");
        match Token::from_credentials(&client, &credentials).await {
            Some(_) =>  assert!(true),
            None => {
                eprintln!("Failed to get token from server.");
                assert!(false);
            }
        };

        let token = Token::load();
        match token {
            Some(token) => assert!(token.active(None)),
            None => assert!(false),
        };
    }

    #[tokio::test]
    async fn renew_token() {
        let client = Postgrest::new(UNIT_TEST_POSTGREST_SERVER).schema("rest");
        let credentials = Credentials::new("jmeyer", "jmeyer");
        let token = Token::from_credentials(&client, &credentials)
            .await
            .unwrap();
        let renewed = token.renew(&client).await.unwrap();
        assert!(&renewed.expire() > &token.expire() && &renewed.user_name() == &token.user_name());
    }
}
