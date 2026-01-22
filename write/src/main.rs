use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    client_email: String,
    private_key: String,
    token_uri: String,
}

#[derive(Debug, Serialize)]
struct Claims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: i64,
    iat: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load service account
    let sa: ServiceAccount =
        serde_json::from_str(&fs::read_to_string("credentials.json")?)?;

    // JWT claims
    let now = Utc::now();
    let claims = Claims {
        iss: &sa.client_email,
        scope: "https://www.googleapis.com/auth/spreadsheets",
        aud: &sa.token_uri,
        iat: now.timestamp(),
        exp: (now + Duration::minutes(60)).timestamp(),
    };

    // Sign JWT
    let key = EncodingKey::from_rsa_pem(sa.private_key.as_bytes())?;
    let jwt = encode(&Header::new(Algorithm::RS256), &claims, &key)?;

    // Exchange for access token
    let client = Client::new();
    let token_resp: serde_json::Value = client
        .post(&sa.token_uri)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ])
        .send()
        .await?
        .json()
        .await?;

    let access_token = token_resp["access_token"]
        .as_str()
        .unwrap();

    // Load environment variables from .env file.
    // The .ok() handles cases where the file might not be present (e.g., in production).
    dotenv().ok();

    // Google Sheet info
    let spreadsheet_id = env::var("SPREADSHEET_ID")
        .expect("SPREADSHEET_ID must be set in the .env file on root directory");
    let range = "Sheet1!A1";

    // Data
    let body = json!({
        "values": [
            ["Test1", "Test2", "Test3"],
            ["Test4", "Test5", "Test6"]
        ]
    });

    // Write to sheet
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption=RAW",
        spreadsheet_id, range
    );

    client
        .put(&url)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await?;

    println!("✅ Data written to Google Sheet!");

    Ok(())
}