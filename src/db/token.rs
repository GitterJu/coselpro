pub mod db {
    const TOKEN_DEFAULT_FILE_NAME:&str = "coselpro_token.json";
    use std::env::current_dir;
    use std::error::Error;
    use std::fs::OpenOptions;
    use std::io::BufReader;
    use chrono::{Duration, Utc};
    use homedir::my_home;
    use pgdatetime::Timestamp;
    use postgrest::Postgrest;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_reader, json};
    use crate::db::credentials::db::Credentials;

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
        pub fn new(login: String, user_name: String) -> Token {
            Token {
                token: login,
                expire: Timestamp::from((Utc::now() + Duration::minutes(15)).timestamp()),
                user_name,
            }
        }
        pub fn token(&self) -> &String {
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
        pub fn active(&self, safety_duration:Option<u8>) -> bool {
            let duration = Duration::minutes(safety_duration.unwrap_or_else(||5) as i64);
            self.expire() > &Timestamp::from((Utc::now() + duration).timestamp())
        }

        /// Save token on user profile
        pub fn save(&self) -> Result<(), Box<dyn Error>> {
            let file_path = my_home()?
                .unwrap_or_else(|| current_dir().expect("unable to get current dir"))
                .join(TOKEN_DEFAULT_FILE_NAME);
            dbg!(&file_path);
            let file = OpenOptions::new().write(true).create(true).open(file_path)?;

            serde_json::to_writer(file, &self)?;
            Ok(())
        }

        /// Create new token from token file on the user profile
        pub fn load() -> Result<Token, Box<dyn Error>> {
            let file_path = my_home()?
                .unwrap_or_else(|| current_dir().expect("unable to get current dir"))
                .join(TOKEN_DEFAULT_FILE_NAME);
            dbg!(&file_path);
            let file = OpenOptions::new().read(true).open(file_path)?;
            let reader = BufReader::new(file);
            let token = from_reader(reader)?;
            Ok(token)
        }

        /// Create new token from active connection and user credentials
        pub async fn from_credentials(client:&Postgrest, credentials:&Credentials) -> Option<Token> {
            let response = match client.rpc("login",
            json!({ "username": credentials.get_login(), "pass": credentials.get_password_md5(),})
                .to_string())
            .execute().await {
                Ok(response) => response,
                Err(_) => {
                    eprintln!("Getting token: Unable to connect to CoSelPro API");
                    return None
                }
            };
            match response.error_for_status_ref() {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Getting token: HTTP error: {}", response.status());
                    return None
                }
            };
            match response.json::<Token>().await {
                Ok(token) => {
                    match token.save() {
                        Ok(_) => Some(token),
                        Err(_) => None
                    }
                },
                Err(e) => {
                    eprintln!("Getting token: Credential failed. {e}");
                    None
                },
            }
        }

        /// Extend token with active connection
        pub async fn renew(&self, client:&Postgrest) -> Option<Token> {
            let response = match client.rpc("extend_token", "")
            .execute().await {
                Ok(response) => response,
                Err(_) => {
                    eprintln!("Renewing token: Unable to connect to CoSelPro API");
                    return None
                }
            };
            match response.error_for_status_ref() {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Renewing token. HTTP error: {}", response.status());
                    return None
                }
            };
            match response.json::<Token>().await {
                Ok(token) => {
                    match token.save() {
                        Ok(_) => Some(token),
                        Err(_) => None
                    }
                },
                Err(e) => {
                    eprintln!("Renewing token. Credential failed. {e}");
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    const UNIT_TEST_POSTGREST_SERVER: &str = "http://proliant:3000";

    use super::*;
    use db::Token;
    use postgrest::Postgrest;
    use tokio;
    use crate::db::credentials::db::Credentials;

    #[tokio::test(flavor = "multi_thread")]
    async fn get_token() {
        let client = Postgrest::new(UNIT_TEST_POSTGREST_SERVER).schema("rest");
        let credentials = Credentials::new("consult", "consult");
        let token = Token::from_credentials(&client, &credentials).await;
        dbg!(&token);
        match token {
            Some(token) => assert!(token.active(None)),
            None => {
                eprintln!("Unable to extract token from server.");
                assert!(false);
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn save_token() {
        let client = Postgrest::new(UNIT_TEST_POSTGREST_SERVER).schema("rest");
        let credentials = Credentials::new("consult", "consult");
        match Token::from_credentials(&client, &credentials).await.unwrap()
            .save() {
            Ok(_) => assert!(true),
            Err(error) => {
                eprintln!("Unable to save token from server response: {:?}", error);
                assert!(false);
            }
        };
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn load_token() {
        let token = Token::load();
        dbg!(&token);
        match token {
            Ok(token) => assert!(token.active(None)),
            Err(error) => assert!(false, "{:?}", error),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn renew_token() {
        let client = Postgrest::new(UNIT_TEST_POSTGREST_SERVER).schema("rest");
        let credentials = Credentials::new("consult", "consult");
        let token = Token::from_credentials(&client, &credentials).await.unwrap();
        let renewed = token.renew(&client).await.unwrap();
        assert!(renewed.expire() > token.expire());
    }
}
