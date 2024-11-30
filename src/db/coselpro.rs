pub mod db {
    use crate::db::credentials::db::Credentials;
    use crate::db::token::db::Token;
    use postgrest::{Builder, Postgrest};
    use std::fmt;

    type Result<T> = std::result::Result<T, CoSelProDbError>;
    #[derive(Debug, Clone)]
    pub enum CoSelProDbError {
        NewToken,
        RenewToken,
    }
    impl fmt::Display for CoSelProDbError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                CoSelProDbError::NewToken => {
                    write!(f, "CoSelPro Database Error: Failed to get new token")
                }
                CoSelProDbError::RenewToken => {
                    write!(f, "CoSelPro Database Error: Failed to renew token")
                } //_ => write!(f, "CoSelPro Database Error")
            }
        }
    }

    /// CoSelPro API implementing Postgrest API
    /// Manage authentication
    /// Exposes CoSelPro functions
    pub struct CoSelPro {
        client: Postgrest,
        token: Token,
    }
    impl CoSelPro {
        /// Get username from token
        pub fn user_name(&self) -> &str {
            &self.token.user_name()
        }

        /// Create CoSelProAPI from Postgrest client and active Token
        pub fn from_token(client: Postgrest, token: Token) -> Result<CoSelPro> {
            let token = match token.active(Some(0u8)) {
                true => Ok(token),
                false => Err(CoSelProDbError::NewToken),
            }?;
            Ok(CoSelPro { client, token })
        }

        /// Create new CoSelPro from Postgrest client and credentials.
        pub async fn from_credentials(
            client: Postgrest,
            credentials: &Credentials,
        ) -> Result<CoSelPro> {
            let token = match Token::from_credentials(&client, credentials).await {
                Ok(token) => token,
                Err(_) => return Err(CoSelProDbError::NewToken),
            };
            Self::from_token(client, token)
        }

        /// Create new CoSelPro from uri and credentials
        pub async fn from_uri_credentials(
            uri: &str,
            credentials: &Credentials,
        ) -> Result<CoSelPro> {
            let client = Postgrest::new(uri);
            CoSelPro::from_credentials(client, credentials).await
        }

        /// Force CoSelPro token renewal
        pub async fn renew_token(&self) -> Result<CoSelPro> {
            match self.token.renew(&self.client).await {
                Ok(token) => Ok(CoSelPro {
                    client: self.client.clone(),
                    token,
                }),
                Err(_) => Err(CoSelProDbError::RenewToken)?,
            }
        }
        pub fn from(self, table: &str) -> Builder {
            self.client
                .schema("rest")
                .from(table)
                .auth(&self.token.to_string())
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
    //#[tokio::test(flavor = "multi_thread")]
    #[tokio::test]
    async fn get_coselpro_api() {
        let cred = Credentials::new("consult", "consult");
        let api = CoSelPro::from_uri_credentials(UNIT_TEST_POSTGREST_SERVER, &cred).await;
        match api {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        };
    }

    //#[tokio::test(flavor = "multi_thread")]
    #[tokio::test]
    async fn get_coselpro_renew_token() {
        let cred = Credentials::new("jmeyer", "jmeyer");
        let api = CoSelPro::from_uri_credentials(UNIT_TEST_POSTGREST_SERVER, &cred)
            .await
            .unwrap();
        let renewed = api.renew_token().await.unwrap();
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
        let credentials = Credentials::new("consult", "consult");
        let api = CoSelPro::from_uri_credentials(UNIT_TEST_POSTGREST_SERVER, &credentials)
            .await
            .unwrap()
            .from("users")
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
