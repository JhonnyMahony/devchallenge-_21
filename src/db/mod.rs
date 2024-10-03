use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Executor};
use std::env;

pub async fn establish_connection() -> sqlx::PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database")
}

pub async fn prepare_db(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    pool.execute_many(include_str!("schema.sql"));
    Ok(())
}
