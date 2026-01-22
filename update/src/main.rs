use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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

    // Update criteria
    let match_value = "Test3";
    let new_value = "UPDATED_VALUE";

    // -----------------------------
    // 1. Read source data (A:C)
    // -----------------------------
    let read_url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/Sheet1!A:C",
        spreadsheet_id
    );

    let resp: Value = client
        .get(&read_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json()
        .await?;

    let rows = match resp["values"].as_array() {
        Some(r) => r,
        None => {
            println!("No data found");
            return Ok(());
        }
    };

    println!("restult {:?}", rows);

    // -----------------------------
    // 2. Find matching rows & update source
    // -----------------------------
    for (index, row) in rows.iter().enumerate() {
        let col_c = row.get(2).and_then(|v| v.as_str()).unwrap_or("");

        if col_c == match_value {
            let sheet_row = index + 2; // because data starts at A2

            let update_url = format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{}/values/Sheet1!B{}?valueInputOption=RAW",
                spreadsheet_id,
                sheet_row
            );

            client
                .put(&update_url)
                .bearer_auth(access_token)
                .json(&json!({
                    "values": [[new_value]]
                }))
                .send()
                .await?;

            println!(
                "✅ Updated row {} (C='{}') → B='{}'",
                sheet_row, match_value, new_value
            );
        }
    }

    println!("QUERY is updated.");

    Ok(())
}
