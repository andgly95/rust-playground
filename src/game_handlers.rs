// game_handlers.rs
use actix_web::{web, HttpResponse, Responder};
use rand::Rng;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
struct CreateGameResponse {
    game_code: String,
}

#[derive(Deserialize)]
pub struct JoinGameRequest {
    game_code: String,
}

#[derive(Serialize)]
struct JoinGameResponse {
    game_uuid: String,
}

pub async fn create_game() -> impl Responder {
    let mut game_code;
    let game_uuid = Uuid::new_v4().to_string();

    let conn = match Connection::open("game_database.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error connecting to database: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    loop {
        game_code = generate_game_code();

        let count: i32 = match conn.query_row(
            "SELECT COUNT(*) FROM game_codes WHERE code = ?1",
            params![game_code],
            |row| row.get(0),
        ) {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Error checking game code uniqueness: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        if count == 0 {
            break;
        }
    }

    match conn.execute(
        "INSERT INTO game_codes (code, game_uuid) VALUES (?1, ?2)",
        params![game_code, game_uuid],
    ) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error inserting game code: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    match conn.execute(
        "INSERT INTO games (uuid, state) VALUES (?1, ?2)",
        params![game_uuid, serde_json::to_string(&initial_game_state()).unwrap()],
    ) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error inserting game: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().json(CreateGameResponse { game_code })
}

pub async fn join_game(game_data: web::Json<JoinGameRequest>) -> impl Responder {
    let conn = match Connection::open("game_database.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error connecting to database: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let game_uuid: String = match conn.query_row(
        "SELECT game_uuid FROM game_codes WHERE code = ?1",
        params![game_data.game_code],
        |row| row.get(0),
    ) {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!("Error retrieving game UUID: {}", e);
            return HttpResponse::NotFound().finish();
        }
    };

    HttpResponse::Ok().json(JoinGameResponse { game_uuid })
}

fn generate_game_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const CODE_LENGTH: usize = 5;

    let mut rng = rand::thread_rng();
    let game_code: String = (0..CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    game_code
}

fn initial_game_state() -> serde_json::Value {
    serde_json::json!({
        "current_round": 1,
        "total_rounds": 3,
        "current_prompts": [],
        "current_images": [],
        "is_submitting_prompts": false,
        "is_generating_images": false,
        "is_guessing_images": false
    })
}