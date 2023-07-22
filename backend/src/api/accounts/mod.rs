mod getters;

use actix_web::web::ServiceConfig;

pub fn init(service: &mut ServiceConfig) {
    service.service(getters::list);
}
