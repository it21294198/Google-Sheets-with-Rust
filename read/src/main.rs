use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use dotenv::dotenv;
use std::env;

// ---------------- SERVICE ACCOUNT ----------------

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    client_email: String,
    private_key: String,
    token_uri: String,
}

// ---------------- JWT CLAIMS ----------------

#[derive(Debug, Serialize)]
struct Claims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: i64,
    iat: i64,
}

// ---------------- READ RESPONSE ----------------

#[derive(Debug, Deserialize)]
struct SheetResponse {
    values: Option<Vec<Vec<String>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load service account JSON
    let sa: ServiceAccount =
        serde_json::from_str(&fs::read_to_string("credentials.json")?)?;

    // Time
    let now = Utc::now();

    // JWT claims
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

    // Exchange JWT for access token
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
        .expect("No access token returned");

    // ---------------- CONFIG ----------------

    // Load environment variables from .env file.
    // The .ok() handles cases where the file might not be present (e.g., in production).
    dotenv().ok();

    // Google Sheet info
    let spreadsheet_id = env::var("SPREADSHEET_ID")
        .expect("SPREADSHEET_ID must be set in the .env file on root directory");

    let read_range = "Sheet1!A1:C10";

    // ---------------- READ DATA ----------------

    let read_url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        spreadsheet_id, read_range
    );

    let response: SheetResponse = client
        .get(&read_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json()
        .await?;

    println!("\nData read from Google Sheet:");

    if let Some(rows) = response.values {
        for row in rows {
            println!("{:?}", row);
        }
    } else {
        println!("Sheet is empty");
    }

    Ok(())
}
