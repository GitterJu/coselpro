pub mod connection {
    const TOKEN_DEFAULT_FILE_NAME:&str = "coselpro_token.json";
    use md5;
    use pgdatetime::Timestamp;
    use postgrest::Postgrest;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_reader, json};
    use std::env::current_dir;
    use std::error::Error;
    use std::fs::{OpenOptions};
    use std::io::{stdin, stdout, BufReader, Write};

    extern crate rpassword;
    use chrono::{Duration, Utc};
    use homedir::my_home;
    use rpassword::read_password;

    #[derive(Debug)]
    pub struct Credentials {
        login: String,
        password: String,
    }
    impl Credentials {
        pub fn new(login: &str, password: &str) -> Credentials {
            Credentials {
                login: login.to_string(),
                password: password.to_string(),
            }
        }
        pub fn get_login(&self) -> &String {
            &self.login
        }
        pub fn get_password_md5(&self) -> String {
            let mut wrd: String = self.password.clone();
            wrd.push_str(&self.get_login());
            let mut mp = format!("{:x}", md5::compute(&wrd[..])).to_string();
            mp.insert_str(0, "md5");
            mp
        }
        /// Prompt user in console for login and password and returns a credential
        pub fn from_console_prompt() -> Result<Credentials, Box<dyn Error>> {
            println!("Issue CoSelPro connection credentials:");

            print!("login: ");
            let _ = stdout().flush();
            let mut login = String::new();
            stdin().read_line(&mut login)?;
            login = login.trim().to_lowercase().to_string();

            print!("password: ");
            let _ = stdout().flush();
            let pwd = read_password()?;

            let creds = Credentials::new(&login, &pwd.trim().to_string());
            Ok(creds)
        }
    }

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
                user_name:user_name,
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
                    eprintln!("Unable to connect to CoSelPro API");
                    return None
                }
            };
            match response.error_for_status_ref() {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Getting token. HTTP error: {}", response.status());
                    return None
                }
            };
            match response.json::<Token>().await {
                Ok(token) => Some(token),
                Err(e) => {
                    eprintln!("Getting token. Credential failed. {e}");
                    None
                }
            }
        }
    }

    /// CoSelPro API implementing Postgrest API
    /// Manage authentication
    /// Exposes CoSelPro functions
    pub struct CoSelProAPI {
        client: Postgrest,
        token: Token,
    }
    impl CoSelProAPI {

        /// Get user name from token
        pub fn user_name(&self) -> &str {
            dbg!(&self.token);
            &self.token.user_name
        }

        pub fn test(&self) -> bool {
            true
        }

        /// Create CoSelProAPI from Postgrest client and active Token
        pub fn from_token(client:Postgrest, token: Token) -> Result<CoSelProAPI, Box<dyn Error>> {
            dbg!(&token);
            let token = match token.active(Some(0u8)) {
                true => Ok(token),
                false => Err("token not active")
            }?;
            dbg!(&token);
            Ok(
                CoSelProAPI {
                    client,
                    token
                })
        }
        pub async fn from_credentials(client:Postgrest, credentials:&Credentials) -> Result<CoSelProAPI, Box<dyn Error>> {
            let token = match Token::from_credentials(&client, credentials).await {
                Some(token) => token,
                None => Err("Unable to obtain token.")?
            };
            Self::from_token(client, token)
        }
        pub async fn from_uri_credentials(uri:&str, credentials: &Credentials) -> Result<CoSelProAPI, Box<dyn Error>> {
            let client = Postgrest::new(uri);
            CoSelProAPI::from_credentials(client, credentials).await
        }
    }
}
#[cfg(test)]
mod tests {
    const UNIT_TEST_POSTGREST_SERVER: &str = "http://proliant:3000";

    use chrono::Duration;
    use super::*;
    use connection::*;
    use postgrest::Postgrest;
    use tokio;

    #[test]
    fn get_password() {
        let cred = Credentials::new("consult", "consult");
        assert_eq!(
            cred.get_password_md5(),
            "md55e73b42456347af1be4be2d0c8eda64a"
        );
    }
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
    async fn get_coselpro_api() {

        let cred = Credentials::new("consult", "consult");
        dbg!(&cred);
        let api = CoSelProAPI::from_uri_credentials(UNIT_TEST_POSTGREST_SERVER, &cred).await;
        match  api {
            Ok(api) => assert!(true),
            Err(error) => assert!(false),
        };
    }
}
