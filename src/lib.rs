const FUNCTION_LOGIN = "login";
const LOGIN_FIELD = "login";
const PWD_FIELD = "pass";

use postgrest::Postgrest;
use serde_json::{ Map, Value};

#[tokio::main]
async fn get_token(uri:&str, login:&str, pwd:&str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Postgrest::new(uri);
    let credentials = vec![vec![LOGIN_FIELD, login], vec![PWD_FIELD, &pwd]];
    credentials.iter().map(|c|{})
    let resp = client.rpc(FUNCTION_LOGIN, r#"{"login":{}}"#).await?;
    println!("{}", resp.text().await?);
    Ok(())
}