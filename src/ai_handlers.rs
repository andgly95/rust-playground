// ai_handlers.rs
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::env;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};


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

pub async fn send_request(payload: &RequestPayload) -> Result<String, reqwest::Error> {
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
        response.completion.to_string()
    } else {
        let response: Response = serde_json::from_str(&response_json).unwrap();
        response.choices[0].message.content.to_string()
    };

    HttpResponse::Ok().body(generated_chat)
}

pub async fn send_speech_to_text_request(
    payload: &SpeechToTextRequestPayload,
    file_contents: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {    
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

pub async fn transcribe_speech(mut payload: Multipart) -> actix_web::Result<HttpResponse> {
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

pub async fn send_text_to_speech_request(
    payload: &TextToSpeechRequestPayload,
) -> Result<Vec<u8>, reqwest::Error> {
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

pub async fn send_image_request(payload: &ImageRequestPayload) -> Result<String, reqwest::Error> {
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