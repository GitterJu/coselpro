pub mod db {
    use crate::db::coselpro::db;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct XRisk {
        risk: Option<String>,
        symbol: Option<String>,
        risk_id: Option<i32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct XCompany {
        company_id: i32,
        company: String,
        man_status_id: Option<i32>,
        man_status: Option<String>,
        man_risk: Option<XRisk>,
        sup_status_id: Option<i32>,
        sup_status: Option<String>,
        sup_risk: Option<XRisk>,
        cst_status_id: Option<i32>,
        cst_status: Option<String>,
        cst_risk: Option<XRisk>,
        reliability: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct XCompanyRequest {
        pub company: String,
        pub division_ids: Option<Vec<i32>>,
        pub xcompany_type_ids: Option<Vec<i32>>,
    }
    impl XCompanyRequest {
        pub fn to_string(&self) -> String {
            match serde_json::to_string(&self) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error serializing XCompanyRequest: {}", e);
                    "{\"xcompany\":null, \"division_ids\":null, \"xcompany_type_ids\":null}"
                        .to_string()
                }
            }
        }
    }

    impl db::CoSelPro {
        pub async fn x_company(self, x_company_request: XCompanyRequest) -> Option<XCompany> {
            let response = match self
                .client
                .clone()
                .schema("rest")
                .rpc("xcompany", x_company_request.to_string())
                .auth(&self.token.to_string())
                .execute()
                .await
            {
                Ok(response) => response,
                Err(error) => {
                    eprintln!("While running xcompany query. {error}");
                    return None;
                }
            };
            let valid_response = match response.error_for_status() {
                Ok(response) => response,
                Err(error) => {
                    eprintln!("REST Error from CoSelPro server. {error}");
                    return None;
                }
            };
            let response_content = match valid_response.text().await {
                Ok(s) => s,
                Err(error) => {
                    eprintln!("Failed to extract CoSelPro response content. {error}");
                    return None;
                }
            };
            dbg!(&response_content);
            match serde_json::from_str::<XCompany>(response_content.as_str()) {
                Ok(xcompany_response) => Some(xcompany_response),
                Err(e) => {
                    eprintln!("Incorrect server response format. {e}");
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::coselpro::db::CoSelPro;
    use crate::db::credentials::db::Credentials;
    use crate::db::crosses::db::XCompanyRequest;

    const UNIT_TEST_POSTGREST_SERVER: &str = "http://proliant:3000";
    #[tokio::test(flavor = "multi_thread")]
    async fn xcompany() {
        let credentials =
            Credentials::new(UNIT_TEST_POSTGREST_SERVER, "consult", "consult").unwrap();
        let api = CoSelPro::from_credentials(credentials, None).await.unwrap();
        let xcp = XCompanyRequest {
            company: "ti".to_string(),
            division_ids: None,
            xcompany_type_ids: None,
        };
        let r_xcp = &api.x_company(xcp).await;
        match r_xcp {
            Some(_) => assert!(true),
            None => assert!(false),
        };

        /*
        let xcp2 = XCompanyRequest {
            company: "AD".to_string(),
            division_ids: None,
            xcompany_type_ids: None,
        };
        let r_xcp2 = &api.x_company(xcp2).await;
        */
    }
}
