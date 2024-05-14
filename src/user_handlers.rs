// user_handlers.rs
use actix_web::{web, HttpResponse, Responder};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    user_id: String,
    token: String,
}

pub async fn create_user(user_data: web::Json<CreateUserRequest>) -> impl Responder {
    let user_id = Uuid::new_v4().to_string();
    let token = generate_jwt_token(&user_id);

    let conn = match Connection::open("game_database.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error connecting to database: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match conn.execute(
        "INSERT INTO users (id, username) VALUES (?1, ?2)",
        params![user_id, user_data.username],
    ) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error inserting user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().json(CreateUserResponse { user_id, token })
}

fn generate_jwt_token(user_id: &str) -> String {
    // TODO: Implement JWT token generation logic
    // For now, you can return a dummy token
    format!("dummy_token_{}", user_id)
}