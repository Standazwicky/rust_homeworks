use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use actix_files::Files;
use std::sync::Arc;


#[derive(Serialize, Deserialize)]
struct Message {
    id: i32,
    username: String,
    content: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
}

#[derive(Serialize, Deserialize)]
struct UserDeleteRequest {
    username: String,
}

async fn get_messages(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let rows = sqlx::query!(
        r#"
        SELECT messages.id, users.username, messages.content, messages.timestamp
        FROM messages
        JOIN users ON messages.user_id = users.id
        "#
    )
    .fetch_all(pool.get_ref().as_ref())
    .await
    .unwrap();

    let messages: Vec<Message> = rows
        .into_iter()
        .map(|row| Message {
            id: row.id,
            username: row.username,
            content: row.content,
            timestamp: row.timestamp.unwrap().to_string(),
        })
        .collect();
        
    HttpResponse::Ok().json(messages)
}

async fn delete_user(
    pool: web::Data<Arc<PgPool>>,
    user_info: web::Json<UserDeleteRequest>,
) -> impl Responder {
    let username = &user_info.username;
    let user_id_result = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
        .fetch_one(pool.get_ref().as_ref())
        .await;
        
    match user_id_result {
        Ok(record) => {
            let user_id = record.id;
            sqlx::query!("DELETE FROM messages WHERE user_id = $1", user_id)
                .execute(pool.get_ref().as_ref())
                .await
                .unwrap();
            
            sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
                .execute(pool.get_ref().as_ref())
                .await
                .unwrap();
                
            HttpResponse::Ok().json("User and associated messages deleted successfully.")
        }
        Err(_) => HttpResponse::NotFound().json("User not found."),
    }
}


pub async fn run(db_pool: Arc<PgPool>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .route("/messages", web::get().to(get_messages))
            .route("/delete_user", web::post().to(delete_user))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
