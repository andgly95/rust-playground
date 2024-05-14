use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use rusqlite::Connection;


mod game_handlers;
mod ai_handlers;
mod user_handlers;


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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL
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
            .route("/generate_chat", web::post().to(ai_handlers::generate_chat))
            .route("/generate_image", web::post().to(ai_handlers::generate_image))
            .route("/generate_speech", web::post().to(ai_handlers::generate_speech))
            .route("/transcribe_speech", web::post().to(ai_handlers::transcribe_speech))

            .route("/create_game", web::post().to(game_handlers::create_game))
            .route("/join_game", web::post().to(game_handlers::join_game))
            .route("/player_ready", web::post().to(game_handlers::player_ready))
            .route("/submit_prompt", web::post().to(game_handlers::submit_prompt))


            .route("/create_user", web::post().to(user_handlers::create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
