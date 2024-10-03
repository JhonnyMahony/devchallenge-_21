use std::sync::{Arc, Mutex};

use super::models::{Category, CreateCategory, UpdateCategory};
use super::utils::reindex_calls_for_category;
use crate::ai_config::AppState;
use crate::db::establish_connection;
use crate::errors::AppResult;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;

// Get all categories
#[get("/category")]
pub async fn get_categories(pool: web::Data<PgPool>) -> AppResult<impl Responder> {
    let categories = sqlx::query_as::<_, Category>("SELECT id, title, points FROM category")
        .fetch_all(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(categories))
}

// Create a new category
#[post("/category")]
pub async fn create_category(
    app_state: web::Data<Arc<Mutex<AppState>>>,
    pool: web::Data<PgPool>,
    new_category: web::Json<CreateCategory>,
) -> AppResult<impl Responder> {
    let category = match sqlx::query_as::<_, Category>(
        r#"
    INSERT INTO category (title, points)
    VALUES ($1, $2)
    RETURNING * 
    "#,
    )
    .bind(&new_category.title)
    .bind(&new_category.points)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(category) => category,
        Err(_) => return Ok(HttpResponse::UnprocessableEntity().finish()),
    };

    let candidate_labels: Vec<String> = {
        let title_iter = std::iter::once(category.title.clone());
        let points_iter = category
            .points
            .clone()
            .unwrap_or_else(|| vec![])
            .into_iter(); // Handle None case
        title_iter.chain(points_iter) // Combine both title and points
    }
    .collect();

    let app_state = match app_state.lock() {
        Ok(state) => state,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let zero_shot = &app_state.zero_shot;
    reindex_calls_for_category(&pool, None, &category.title, candidate_labels, zero_shot).await?;

    Ok(HttpResponse::Ok().json(category))
}

// Update an existing category
#[put("/category/{category_id}")]
pub async fn update_category(
    app_state: web::Data<Arc<Mutex<AppState>>>,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    updated_category: web::Json<UpdateCategory>,
) -> AppResult<impl Responder> {
    let title =
        match sqlx::query_as::<_, Category>("SELECT id, title, points FROM category WHERE id = $1")
            .bind(*id)
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(category) => category.title,
            Err(_) => return Ok(HttpResponse::UnprocessableEntity().finish()),
        };
    let category = match sqlx::query_as::<_, Category>(
        r#"
    UPDATE category
    SET title = COALESCE($1, title), points = COALESCE($2, points)
    WHERE id = $3
    RETURNING *
    "#,
    )
    .bind(&updated_category.title)
    .bind(&updated_category.points)
    .bind(*id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(category) => category,
        Err(_) => return Ok(HttpResponse::UnprocessableEntity().finish()),
    };

    let candidate_labels: Vec<String> = {
        let title_iter = std::iter::once(category.title.clone());
        let points_iter = category
            .points
            .clone()
            .unwrap_or_else(|| vec![])
            .into_iter(); // Handle None case
        title_iter.chain(points_iter) // Combine both title and points
    }
    .collect();

    let app_state = match app_state.lock() {
        Ok(state) => state,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let zero_shot = &app_state.zero_shot;
    reindex_calls_for_category(
        &pool,
        Some(&title),
        &category.title,
        candidate_labels,
        zero_shot,
    )
    .await?;

    Ok(HttpResponse::Ok().json(category))
}

// Delete a category
#[delete("/category/{id}")]
pub async fn delete_category(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> AppResult<impl Responder> {
    let result = sqlx::query(
        r#"
    WITH deleted_category AS (
        DELETE FROM category
        WHERE id = $1
        RETURNING title
    )
    UPDATE call
    SET categories = array_remove(categories, (SELECT title::text FROM deleted_category))
    "#,
    )
    .bind(*id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

use actix_web::{test, App};

#[actix_web::test]
async fn test_get_categories() {
    let pool = establish_connection().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get_categories),
    )
    .await;

    let req = test::TestRequest::get().uri("/category").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

// Test DELETE /category/{id}
#[actix_web::test]
async fn test_delete_category() {
    let pool = establish_connection().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(delete_category),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/category/1") // Assuming category 1 exists for the test
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}
