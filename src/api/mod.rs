mod call;
mod category;
mod models;
mod utils;

use actix_web::web::{self, service};
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(category::get_categories)
            .service(category::create_category)
            .service(category::update_category)
            .service(category::delete_category) //.service(call::get_call)
            .service(call::create_call)
            .service(call::get_call),
    );
}
