use std::env;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestPayload {
    model: String,
    messages: Vec<Message>,
}

async fn send_request(api_key: &str, payload: &RequestPayload) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let json_payload = serde_json::to_string(&payload).unwrap();
    println!("Request Payload: {}", json_payload);

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(json_payload)
        .send()
        .await?;

    let result = response.text().await?;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a poetic assistant, skilled in explaining complex programming concepts with creative flair.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Compose a poem that explains the concept of recursion in programming.".to_string(),
        },
    ];

    let payload = RequestPayload {
        model: "gpt-3.5-turbo".to_string(),
        messages,
    };

    let response_json = send_request(&api_key, &payload).await?;
    println!("Response JSON: {}", response_json);

    Ok(())
}