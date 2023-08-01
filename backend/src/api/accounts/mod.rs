mod getters;
mod setters;

use actix_web::web::ServiceConfig;

pub fn init(service: &mut ServiceConfig) {
    service
        .service(getters::list)
        .service(getters::get_full_user)
        .service(setters::password_change)
        .service(setters::update_active)
        .service(setters::update_core)
        .service(setters::password_change)
        .service(setters::new);
}
