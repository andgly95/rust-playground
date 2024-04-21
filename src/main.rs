use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use actix_web::http::header;
use serde::{Deserialize, Serialize};
use std::env;

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

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
    finish_reason: String,
    index: i32,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Deserialize, Debug)]
struct Response {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageRequestPayload {
    model: String,
    prompt: String,
    size: String,
    quality: String,
    n: i32,
}

#[derive(Deserialize, Debug)]
struct ImageData {
    url: String,
}

#[derive(Deserialize, Debug)]
struct ImageResponse {
    data: Vec<ImageData>,
}

async fn send_request(payload: &RequestPayload) -> Result<String, reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await?;

    let result = response.text().await?;
    Ok(result)
}

async fn generate_chat(payload: web::Json<RequestPayload>) -> impl Responder {
    let response_json = match send_request(&payload).await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let response: Response = serde_json::from_str(&response_json).unwrap();
    let generated_chat = &response.choices[0].message.content;

    HttpResponse::Ok().body(generated_chat.to_string())
}

async fn send_image_request(payload: &ImageRequestPayload) -> Result<String, reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/images/generations";

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await?;

    let result = response.text().await?;
    Ok(result)
}

async fn generate_image(payload: web::Json<ImageRequestPayload>) -> impl Responder {
    let response_json = match send_image_request(&payload).await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let response: ImageResponse = serde_json::from_str(&response_json).unwrap();
    let image_url = &response.data[0].url;

    HttpResponse::Ok().body(image_url.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://guess-ai.app")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/generate_chat", web::post().to(generate_chat))
            .route("/generate_image", web::post().to(generate_image))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}