pub mod db {
    use crate::db::credentials::db::Credentials;
    use crate::db::token::db::TokenError::TokenParsingError;
    use chrono::{Duration, Utc};
    use pgdatetime::Timestamp;
    use postgrest::Postgrest;
    use reqwest::Response;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::fmt;

    type Result<T> = std::result::Result<T, TokenError>;
    #[derive(Debug, Clone)]
    pub enum TokenError {
        TokenParsingError(String),
    }
    impl fmt::Display for TokenError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let tp = match self {
                TokenParsingError(str) => ("parse", str),
            };
            write!(
                f,
                "CoSelPro Token Error: Failed to {} token! {}",
                tp.0, tp.1
            )
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

        async fn parse_response(response: Response) -> Result<Token> {
            match response.error_for_status_ref() {
                Ok(_) => {}
                Err(e) => return Err(TokenParsingError(e.to_string())),
            };
            match response.json::<Token>().await {
                Ok(token) => Ok(token),
                Err(e) => Err(TokenParsingError(e.to_string())),
            }
        }

        /// Create new token from active connection and user credentials
        pub async fn from_credentials(credentials: &Credentials) -> Result<Token> {
            let client = Postgrest::new(credentials.get_uri());
            let response = match client.rpc("login",
            json!({ "username": credentials.get_login(), "pass": credentials.get_password_md5()})
                .to_string())
            .execute().await {
                Ok(response) => response,
                Err(e) => {
                    return Err(TokenParsingError(e.to_string()));
                }
            };
            Self::parse_response(response).await
        }

        /// Extend token with active connection
        pub async fn renew(&self, client: &Postgrest) -> Result<Token> {
            let response = match client
                .rpc("extend_token", "")
                .auth(&self.token)
                .execute()
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    return Err(TokenParsingError(e.to_string()));
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

    #[tokio::test(flavor = "multi_thread")]
    async fn get_token() {
        let credentials = Credentials::new(UNIT_TEST_POSTGREST_SERVER, "consult", "consult");
        let token = Token::from_credentials(&credentials).await;
        match token {
            Ok(token) => assert!(token.active(None)),
            Err(e) => {
                eprintln!("{e}");
                assert!(false);
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn renew_token() {
        let credentials = Credentials::new(UNIT_TEST_POSTGREST_SERVER, "jmeyer", "jmeyer");
        let token = Token::from_credentials(&credentials).await.unwrap();
        let renewed = token
            .renew(&Postgrest::new(credentials.get_uri()))
            .await
            .unwrap();
        assert!(&renewed.expire() > &token.expire() && &renewed.user_name() == &token.user_name());
    }
}
