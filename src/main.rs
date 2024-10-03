mod ai_config;
mod api;
mod db;
mod errors;

use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let pool = db::establish_connection().await;
    let _ = db::prepare_db(&pool).await;
    let app_state = ai_config::AppState::new().await;
    let application = move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(Files::new("/tmp", "./tmp").show_files_listing())
            .service(Files::new("/models", "./models").show_files_listing())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(app_state.clone()))
            .configure(api::config)
    };

    HttpServer::new(application)
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
