use actix_web::{web, HttpResponse, Scope};
use crate::services::aggregator_service::AggregatorService;

pub fn init_aggregator_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/aggregate")
             .route(web::get().to(aggregate)), 
    );
}

async fn aggregate(service: web::Data<AggregatorService>) -> HttpResponse {
    match service.fetch_data().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().json("Failed to fetch aggregated data"),
    }
}