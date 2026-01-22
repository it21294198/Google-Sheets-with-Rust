## Overview

1. Create a Google Cloud project.
2. Enable Google Sheets API.
3. Create a Service Account & download credentials JSON.
4. Share your Google Sheet with the service account email.
5. Write a Rust program to insert data.
---
### Google Cloud setup (one-time)
---
1. Create project

2. Enable API
    ```
    APIs & Services → Library
    ```
    Enable Google Sheets API.

3. Create Service Account
    ```
    IAM & Admin → Service Accounts
    Create → give any name
    Create JSON key → download it (e.g. credentials.json)
    ```
    Put `credentials.json` on root folder.
4. Share Google Sheet
    ```
    Open your Google Sheet → Share → add:
    ```

    ```
    xxxx@xxxx.iam.gserviceaccount.com
    ```
    (Give Editor access)

5. Get Spreadsheet ID

    From this URL:
    ```
    https://docs.google.com/spreadsheets/d/1AbCDefGHIJKLmnoPQRstuVWxyz/edit
    ```

    Spreadsheet ID is:
    ```
    1AbCDefGHIJKLmnoPQRstuVWxyz
    ```
    Add Spreadsheet ID to `.env` on root folder.
   ```
   SPREADSHEET_ID = 
   ```
6. Run it
   
    * Write Program
    ```
    cargo run -p write
    ```
    * Read Program
    ```
    cargo run -p read
    ```
    * Query a text
    ```
    cargo run -p query
    ```
    * Update a text
    ```
    cargo run -p update
    ```

If everything is correct → your data appears in Google Sheets.