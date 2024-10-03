use std::sync::{Arc, Mutex};

use super::models::{Call, CallId, Category};
use super::utils::{
    categories, download_audio_file, emotional_tone, name_and_locations, transcribe_audio,
};
use crate::ai_config::AppState;
use crate::db::establish_connection;
use crate::errors::AppResult;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct CreateCallRequest {
    audio_url: String,
}

// Create a new call
#[post("/call")]
pub async fn create_call(
    pool: web::Data<PgPool>,
    app_state: web::Data<Arc<Mutex<AppState>>>,
    new_call: web::Json<CreateCallRequest>,
) -> AppResult<impl Responder> {
    let audio_url = &new_call.audio_url;
    let file_path = match download_audio_file(audio_url).await {
        Ok(path) => path,
        Err(_) => return Ok(HttpResponse::UnprocessableEntity().finish()),
    };

    let app_state = match app_state.lock() {
        Ok(state) => state,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let transcriber = &app_state.transcriber;
    let sentiment = &app_state.sentiment;
    let ner = &app_state.ner;
    let zero_shot = &app_state.zero_shot;

    // Transcribe audio
    let transcribed_text = transcribe_audio(format!("./tmp/{}", file_path), transcriber).await;
    // Define emotional tone
    let emotional_tone = emotional_tone(transcribed_text.clone(), sentiment).await?;
    // Extract names and locations using NER (stubbed)
    let (name, location) = name_and_locations(transcribed_text.clone(), ner).await?;
    // Parse categories based on text (you can extend this to match actual topics)
    let category = sqlx::query_as::<_, Category>("SELECT * FROM category")
        .fetch_all(pool.get_ref())
        .await?;

    let categories = categories(transcribed_text.clone(), category, zero_shot).await?;

    let call = sqlx::query_as::<_, CallId>(
        r#"
    INSERT INTO call (name, location, emotional_tone, text, categories, id)
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING id
    "#,
    )
    .bind(name.map(|name| name.join(" ")))
    .bind(location.map(|loc| loc.join(" ")))
    .bind(emotional_tone)
    .bind(transcribed_text)
    .bind(&categories as &[String])
    .bind(file_path)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(call))
}

// Get a specific call by ID
#[get("call/{id}")]
pub async fn get_call(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    let result = sqlx::query_as::<_, Call>(
        r#"
    SELECT id, name, location, emotional_tone, text, categories
    FROM call
    WHERE id = $1
    "#,
    )
    .bind(*id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(call) => HttpResponse::Ok().json(call),
        Err(_) => HttpResponse::Accepted().finish(),
    }
}

use actix_web::{test, App};

// Test GET /call/{id}
#[actix_web::test]
async fn test_get_call() {
    let pool = establish_connection().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get_call),
    )
    .await;

    let call_id = Uuid::new_v4(); // Assume you have a valid UUID for testing
    let req = test::TestRequest::get()
        .uri(&format!("/call/{}", call_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
