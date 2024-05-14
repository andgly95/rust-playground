use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use rusqlite::Connection;


mod game_handlers;
mod user_handlers;
use user_handlers::{create_user, CreateUserRequest};


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

#[derive(Serialize, Deserialize, Debug)]
struct TextToSpeechRequestPayload {
    model: String,
    input: String,
    voice: String,
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

#[derive(Deserialize, Debug)]
struct AnthropicResponse {
    completion: String,
    stop_reason: String,
    truncated: bool,
    log_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageRequestPayload {
    model: String,
    prompt: String,
    size: String,
    quality: String,
    n: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpeechToTextRequestPayload {
    model: String,
    file: String,
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
    let api_key = if payload.model.starts_with("claude") {
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set")
    } else {
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set")
    };

    let client = reqwest::Client::new();
    let url = if payload.model.starts_with("claude") {
        "https://api.anthropic.com/v1/complete"
    } else {
        "https://api.openai.com/v1/chat/completions"
    };

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

    let generated_chat = if payload.model.starts_with("claude") {
        let response: AnthropicResponse = serde_json::from_str(&response_json).unwrap();
        response.completion.to_string()
    } else {
        let response: Response = serde_json::from_str(&response_json).unwrap();
        response.choices[0].message.content.to_string()
    };

    HttpResponse::Ok().body(generated_chat)
}


async fn transcribe_speech(mut payload: Multipart) -> actix_web::Result<HttpResponse> {
    let mut file_contents = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_contents.extend_from_slice(&data);
        }
    }

    let model = "whisper-1".to_string();
    let file = "recording.wav".to_string();

    let payload = SpeechToTextRequestPayload {
        model,
        file,
    };

    let transcription = send_speech_to_text_request(&payload, &file_contents)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(transcription))
}

async fn send_speech_to_text_request(payload: &SpeechToTextRequestPayload, file_contents: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/audio/transcriptions";

    let part = reqwest::multipart::Part::bytes(file_contents.to_vec())
        .file_name(payload.file.clone())
        .mime_str("audio/wav")?;

    let form = reqwest::multipart::Form::new()
        .text("model", payload.model.clone())
        .part("file", part);

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    let result = response.text().await?;
    Ok(result)
}


async fn send_text_to_speech_request(payload: &TextToSpeechRequestPayload) -> Result<Vec<u8>, reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/audio/speech";

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await?;

    let audio_data = response.bytes().await?;
    Ok(audio_data.to_vec())
}

async fn generate_speech(payload: web::Json<TextToSpeechRequestPayload>) -> impl Responder {
    let audio_data = match send_text_to_speech_request(&payload).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok()
        .content_type("audio/mpeg")
        .body(audio_data)
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

    let conn = Connection::open("game_database.db").expect("Failed to open database connection");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_codes (
            code TEXT PRIMARY KEY,
            game_uuid TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create game_codes table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            uuid TEXT PRIMARY KEY,
            state TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create games table");


    let conn = Connection::open("game_database.db").expect("Failed to open database connection");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create users table");


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
            .route("/generate_speech", web::post().to(generate_speech))
            .route("/transcribe_speech", web::post().to(transcribe_speech))
            .route("/create_game", web::post().to(game_handlers::create_game))
            .route("/join_game", web::post().to(game_handlers::join_game))
            .route("/create_user", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
