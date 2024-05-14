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
    player_id: String,
}

#[derive(Serialize)]
struct JoinGameResponse {
    game_uuid: String,
}


#[derive(Serialize, Deserialize)]
struct PlayerReadyRequest {
    game_uuid: String,
    player_id: String,
}

#[derive(Serialize, Deserialize)]
struct Player {
    id: String,
    username: String,
    score: i32,
    ready: bool,
}

#[derive(Serialize, Deserialize)]
struct GameState {
    game_id: String,
    status: String,
    current_round: i32,
    total_rounds: i32,
    players: Vec<Player>,
    current_prompt: String,
    current_image: String,
    submitted_prompts: Vec<(String, String)>,
    submitted_guesses: Vec<(String, String, String)>,
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

    let initial_state = GameState {
        game_id: game_uuid.clone(),
        status: "waiting".to_string(),
        current_round: 1,
        total_rounds: 3,
        players: vec![],
        current_prompt: "".to_string(),
        current_image: "".to_string(),
        submitted_prompts: vec![],
        submitted_guesses: vec![],
    };

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
        params![game_uuid, serde_json::to_string(&initial_state).unwrap()],
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

    let mut game_state: GameState = match conn.query_row(
        "SELECT state FROM games WHERE uuid = ?1",
        params![game_uuid],
        |row| {
            let state_json: String = row.get(0)?;
            serde_json::from_str(&state_json).map_err(|_| rusqlite::Error::InvalidQuery)
        },
    ) {
        Ok(state) => state,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    let username: String = match conn.query_row(
        "SELECT username FROM users WHERE id = ?1",
        params![game_data.player_id],
        |row| row.get(0),
    ) {
        Ok(username) => username,
        Err(_) => "".to_string(),
    };

    let player = Player {
        id: game_data.player_id.clone(),
        username,
        score: 0,
        ready: false,
    };

    game_state.players.push(player);

    if game_state.players.len() == 2 && game_state.players.iter().all(|p| p.ready) {
        game_state.status = "in_progress".to_string();
    }

    match conn.execute(
        "UPDATE games SET state = ?1 WHERE uuid = ?2",
        params![serde_json::to_string(&game_state).unwrap(), game_uuid],
    ) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error updating game state: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().json(game_state)
}



pub async fn player_ready(game_data: web::Json<PlayerReadyRequest>) -> impl Responder {

    let conn = match Connection::open("game_database.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error connecting to database: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut game_state: GameState = match conn.query_row(
        "SELECT state FROM games WHERE uuid = ?1",
        params![game_data.game_uuid],
        |row| {
            let state_json: String = row.get(0)?;
            serde_json::from_str(&state_json).map_err(|_| rusqlite::Error::InvalidQuery)
        },
    ) {
        Ok(state) => state,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    if let Some(player) = game_state.players.iter_mut().find(|p| p.id == game_data.player_id) {
        player.ready = true;
    }

    if game_state.players.len() == 2 && game_state.players.iter().all(|p| p.ready) {
        game_state.status = "in_progress".to_string();
    }


    HttpResponse::Ok().json(game_state)
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