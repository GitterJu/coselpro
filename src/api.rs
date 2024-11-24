pub mod connection {
    use md5;
    use pgdatetime::Timestamp;
    use postgrest::Postgrest;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_reader, json};
    use std::env::current_dir;
    use std::error::Error;
    use std::fs::{File, OpenOptions};
    use std::io::{stdin, stdout, Write};
    extern crate rpassword;
    use chrono::{Duration, Utc};
    use homedir::my_home;
    use rpassword::read_password;

    #[derive(Serialize, Deserialize, Debug)]
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
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Token {
        token: String,
        expire: Timestamp,
    }
    impl Token {
        pub fn new(login: &str) -> Token {
            Token {
                token: login.to_string(),
                expire: Timestamp::from((Utc::now() + Duration::minutes(15)).timestamp()),
            }
        }
        pub fn token(&self) -> &str {
            &self.token
        }
        pub fn expire(&self) -> &Timestamp {
            &self.expire
        }
        pub fn active(&self) -> bool {
            self.expire() > &Timestamp::from(Utc::now().timestamp())
        }
        pub fn save(&self) -> Result<(), Box<dyn Error>> {
            let file_path = my_home()?
                .unwrap_or_else(|| current_dir().expect("unable to get current dir"))
                .join(".coselpro_token.json");

            let file = OpenOptions::new().write(true).open(file_path)?;

            serde_json::to_writer(file, &self)?;
            Ok(())
        }
    }

    /// CoSelPro API implementing Postgrest API
    /// Manage authentication
    /// Exposes CoSelPro functions
    pub struct CoSelProAPI {
        client: Postgrest,
        credentials: Credentials,
        user_name: String,
    }
    impl CoSelProAPI {
        pub fn new(uri: &str, credentials: Credentials) -> CoSelProAPI {
            CoSelProAPI {
                client: Postgrest::new(uri),
                credentials: credentials,
                user_name: "test".to_string(),
            }
        }
        pub fn user_name(&self) -> &String {
            &self.user_name
        }
    }
    pub async fn get_token(
        client: &Postgrest,
        credentials: &Credentials,
    ) -> reqwest::Result<Token> {
        client.rpc("login",
            json!({ "username": credentials.get_login(), "pass": credentials.get_password_md5(),})
                .to_string())
            .execute().await?
            .error_for_status()?
            .json::<Token>().await
    }
    pub fn prompt_console_credentials() -> Result<Credentials, Box<dyn Error>> {
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
        Result::Ok(creds)
    }
}
#[cfg(test)]
mod tests {
    const UNIT_TEST_POSTGREST_SERVER: &str = "http://proliant:3000";
    use super::*;
    use connection::*;
    use postgrest::Postgrest;
    use std::path::Path;
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
        let token = connection::get_token(&client, &credentials).await;
        dbg!(&token);
        match token {
            Ok(token) => assert!(token.active()),
            Err(error) => {
                eprintln!("Unable to extract token from server response: {:?}", error);
                assert!(false);
            }
        }
    }
}
