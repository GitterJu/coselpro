pub mod coselpro {
    use std::error::Error;
    use postgrest::Postgrest;
    use crate::db::credentials::db::Credentials;
    use crate::db::token::db::Token;

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
            dbg!(&self.token);
            &self.token.user_name()
        }

        pub fn test(&self) -> bool {
            true
        }

        /// Create CoSelProAPI from Postgrest client and active Token
        pub fn from_token(client:Postgrest, token: Token) -> Result<CoSelPro, Box<dyn Error>> {
            dbg!(&token);
            let token = match token.active(Some(0u8)) {
                true => Ok(token),
                false => Err("token not active")
            }?;
            dbg!(&token);
            Ok(
                CoSelPro {
                    client,
                    token
                })
        }
        pub async fn from_credentials(client:Postgrest, credentials:&Credentials) -> Result<CoSelPro, Box<dyn Error>> {
            let token = match Token::from_credentials(&client, credentials).await {
                Some(token) => token,
                None => Err("Unable to obtain token.")?
            };
            Self::from_token(client, token)
        }
        pub async fn from_uri_credentials(uri:&str, credentials: &Credentials) -> Result<CoSelPro, Box<dyn Error>> {
            let client = Postgrest::new(uri);
            CoSelPro::from_credentials(client, credentials).await
        }
        pub async fn renew_token(&self) -> Result<CoSelPro, Box<dyn Error>> {
            match self.token.renew(&self.client).await {
                Some(token) => Ok(CoSelPro {
                    client: self.client.clone(),
                    token
                }),
                None => Err("Unable to renew token")?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::coselpro::coselpro::CoSelPro;
    use crate::db::credentials::db::Credentials;

    const UNIT_TEST_POSTGREST_SERVER: &str = "http://proliant:3000";
    #[tokio::test(flavor = "multi_thread")]
    async fn get_coselpro_api() {

        let cred = Credentials::new("consult", "consult");
        let api = CoSelPro::from_uri_credentials(UNIT_TEST_POSTGREST_SERVER, &cred).await;
        match  api {
            Ok(api) => assert!(true),
            Err(error) => assert!(false),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_coselpro_renew_token() {

        let cred = Credentials::new("consult", "consult");
        let api = CoSelPro::from_uri_credentials(UNIT_TEST_POSTGREST_SERVER, &cred)
            .await.unwrap();
        let renewed = api.renew_token().await.unwrap();
        assert_eq!(renewed.user_name(), api.user_name());
    }
}