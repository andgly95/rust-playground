// ai_handlers.rs
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestPayload {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextToSpeechRequestPayload {
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
pub struct ImageRequestPayload {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct EmbeddingRequestPayload {
    model: String,
    input: Vec<String>,
}

async fn send_request(payload: &RequestPayload) -> Result<String, reqwest::Error> {
    let api_key = if payload.model.starts_with("claude") {
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set")
    } else {
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set")
    };

    let url = if payload.model.starts_with("claude") {
        "https://api.anthropic.com/v1/complete"
    } else {
        "https://api.openai.com/v1/chat/completions"
    };

    let response = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

pub async fn generate_chat(payload: web::Json<RequestPayload>) -> impl Responder {
    let response_json = match send_request(&payload).await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let generated_chat = if payload.model.starts_with("claude") {
        let response: AnthropicResponse = serde_json::from_str(&response_json).unwrap();
        response.completion
    } else {
        let response: Response = serde_json::from_str(&response_json).unwrap();
        response.choices[0].message.content.to_string()
    };

    HttpResponse::Ok().body(generated_chat)
}

async fn send_speech_to_text_request(
    payload: &SpeechToTextRequestPayload,
    file_contents: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let url = "https://api.openai.com/v1/audio/transcriptions";

    let part = reqwest::multipart::Part::bytes(file_contents.to_vec())
        .file_name(payload.file.clone())
        .mime_str("audio/wav")?;

    let form = reqwest::multipart::Form::new()
        .text("model", payload.model.clone())
        .part("file", part);

    let response = reqwest::Client::new()
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

pub async fn transcribe_speech(mut payload: Multipart) -> actix_web::Result<HttpResponse> {
    let mut file_contents = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_contents.extend_from_slice(&data);
        }
    }

    let payload = SpeechToTextRequestPayload {
        model: "whisper-1".to_string(),
        file: "recording.wav".to_string(),
    };

    let transcription = send_speech_to_text_request(&payload, &file_contents)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(transcription))
}

async fn send_text_to_speech_request(
    payload: &TextToSpeechRequestPayload,
) -> Result<Vec<u8>, reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let url = "https://api.openai.com/v1/audio/speech";

    let response = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();

    Ok(response)
}

pub async fn generate_speech(payload: web::Json<TextToSpeechRequestPayload>) -> impl Responder {
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
    let url = "https://api.openai.com/v1/images/generations";

    let response = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

pub async fn generate_image(payload: web::Json<ImageRequestPayload>) -> impl Responder {
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

async fn send_embedding_request(
    payload: &EmbeddingRequestPayload,
) -> Result<String, reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let url = "https://api.openai.com/v1/embeddings";

    let response = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

pub async fn get_embeddings(payload: web::Json<EmbeddingRequestPayload>) -> impl Responder {
    let response_json = match send_embedding_request(&payload).await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().body(response_json)
}

pub async fn calculate_similarity(prompt: web::Json<String>, guess: web::Json<String>) -> String {
    let payload = EmbeddingRequestPayload {
        model: "text-embedding-ada-002".to_string(),
        input: vec![prompt.to_string(), guess.to_string()],
    };

    let response_json = match send_embedding_request(&payload).await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error: {}", e);
            return "Error".to_string();
        }
    };

    let response: serde_json::Value = serde_json::from_str(&response_json).unwrap();
    let embeddings = response["data"].as_array().unwrap();
    let prompt_embedding: Vec<f64> = embeddings[0]["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let guess_embedding: Vec<f64> = embeddings[1]["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();

    let similarity = cosine_similarity(&prompt_embedding, &guess_embedding);
    let score = (similarity * 50.0 + 50.0).round() as u32;

    score.to_string()
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f64 = a.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let magnitude_b: f64 = b.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    dot_product / (magnitude_a * magnitude_b)
}
