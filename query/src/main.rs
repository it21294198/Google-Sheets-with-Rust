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

    // Filter value for QUERY
    let filter_value = "Test6".to_string();

    // -----------------------------
    // 1. Write QUERY formula dynamically
    // -----------------------------
    let query_formula = format!(
        "=QUERY(A:C, \"SELECT A,B WHERE C='{}'\")",
        filter_value
    );

    let write_url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/Sheet1!E1?valueInputOption=USER_ENTERED",
        spreadsheet_id
    );

    client
        .put(&write_url)
        .bearer_auth(access_token)
        .json(&json!({
            "values": [[query_formula]]
        }))
        .send()
        .await?;

    // -----------------------------
    // 2. Read QUERY result
    // -----------------------------
    let read_url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/Sheet1!E:F",
        spreadsheet_id
    );

    let resp: Value = client
        .get(&read_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json()
        .await?;

    // -----------------------------
    // 3. Process result
    // -----------------------------
    if let Some(rows) = resp["values"].as_array() {
        for row in rows {
            let col1 = row.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let col2 = row.get(1).and_then(|v| v.as_str()).unwrap_or("");

            println!("Result → {} | {}", col1, col2);
        }
        // print!("Result → {:?}",rows);
    } else {
        println!("No matching rows found");
    }

    Ok(())
}
