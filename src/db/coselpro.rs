pub mod db {
    const DEFAULT_COSELPRO_SCHEMA: &str = "rest";
    use crate::db::coselpro::db::CoSelProDbError::ExpiredToken;
    use crate::db::credentials::db::Credentials;
    use crate::db::token::db;
    use crate::db::token::db::Token;
    use postgrest::{Builder, Postgrest};
    use std::fmt;

    pub type Result<T> = std::result::Result<T, CoSelProDbError>;
    #[derive(Debug, Clone)]
    pub enum CoSelProDbError {
        NewToken(db::TokenError),
        RenewToken(db::TokenError),
        ExpiredToken,
    }
    impl fmt::Display for CoSelProDbError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let err_det = match self {
                CoSelProDbError::NewToken(tk_err) => {
                    ("Failed to get new token!", tk_err.to_string())
                }
                CoSelProDbError::RenewToken(tk_err) => {
                    ("Failed to renew token!", tk_err.to_string())
                }
                ExpiredToken => ("Token expired!", String::from("inactive")),
            };
            write!(f, "CoSelPro Database Error: {} {}", err_det.0, err_det.1)
        }
    }

    /// CoSelPro API implementing Postgrest API
    /// Manage authentication
    /// Exposes CoSelPro functions
    pub struct CoSelPro {
        pub client: Postgrest,
        pub token: Token,
        pub schema: String,
    }
    impl CoSelPro {
        /// Get username from token
        pub fn user_name(&self) -> &str {
            &self.token.user_name()
        }

        /// Create new CoSelPro from Postgrest client and credentials.
        pub async fn from_credentials(
            credentials: Credentials,
            schema: Option<String>,
        ) -> Result<CoSelPro> {
            let client = credentials.client();
            let token = match Token::from_credentials(credentials).await {
                Ok(token) => token,
                Err(e) => return Err(CoSelProDbError::NewToken(e)),
            };
            match schema {
                Some(schema) => Ok(CoSelPro {
                    client,
                    token,
                    schema,
                }),
                None => Ok(CoSelPro {
                    client,
                    token,
                    schema: DEFAULT_COSELPRO_SCHEMA.to_string(),
                }),
            }
        }

        /// Force CoSelPro token renewal
        pub async fn renew(&self) -> Result<CoSelPro> {
            match self.token.renew(self.client.clone()).await {
                Ok(token) => Ok(CoSelPro {
                    client: self.client.clone(),
                    token,
                    schema: self.schema.clone(),
                }),
                Err(e) => Err(CoSelProDbError::RenewToken(e))?,
            }
        }

        fn get_token(&self) -> Result<&Token> {
            match &self.token.active(Some(0u8)) {
                true => Ok(&self.token),
                false => Err(ExpiredToken),
            }
        }
        pub fn from(self, table: &str) -> Result<Builder> {
            let token = self.get_token()?;
            Ok(self
                .client
                .clone()
                .schema(&self.schema)
                .from(table)
                .auth(token.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::coselpro::db::CoSelPro;
    use crate::db::credentials::db::Credentials;
    use serde::{Deserialize, Serialize};
    use std::fmt;

    const UNIT_TEST_POSTGREST_SERVER: &str = "http://proliant:3000";
    #[tokio::test(flavor = "multi_thread")]
    async fn get_coselpro_api() {
        let credentials =
            Credentials::new(UNIT_TEST_POSTGREST_SERVER, "consult", "consult").unwrap();
        let api = CoSelPro::from_credentials(credentials, None).await;
        match api {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_coselpro_renew_token() {
        let credentials = Credentials::new(UNIT_TEST_POSTGREST_SERVER, "jmeyer", "jmeyer").unwrap();
        let api = CoSelPro::from_credentials(credentials, None).await.unwrap();
        let renewed = api.renew().await.unwrap();
        assert_eq!(renewed.user_name(), api.user_name());
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct User {
        pub user_id: u32,
        pub user_login: String,
        pub user_name: String,
        pub email: Option<String>,
        pub phone: Option<String>,
    }
    impl fmt::Display for User {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}\t", self.user_id)
        }
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn read_users() {
        let credentials =
            Credentials::new(UNIT_TEST_POSTGREST_SERVER, "consult", "consult").unwrap();
        let api = CoSelPro::from_credentials(credentials, None)
            .await
            .unwrap()
            .from("users")
            .unwrap()
            .select("*")
            .execute()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .text()
            .await
            .unwrap();
        let val: Vec<User> = serde_json::from_str(&api).unwrap();
        dbg!(&api);
        val.iter().for_each(|item| println!("{:?}", item));
    }
}
