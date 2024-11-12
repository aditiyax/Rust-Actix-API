use std::env;
use actix_cors::Cors;
use actix_web::dev::ServiceRequest;
use actix_web::{
    error::Error, error::InternalError, error::JsonPayloadError, http, web, HttpRequest, HttpResponse,
};
use actix_web::{middleware, web::Data, web::JsonConfig, App, HttpServer};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use chrono::{SecondsFormat, Utc};
use dotenvy::dotenv;
use log::{info, warn};

use models::error_model::ApiError;
use models::task_model::{Task, TaskAggregate}; // Assuming TaskAggregate is added to handle aggregation
use services::task_service::TaskService;
use services::aggregator_service::AggregatorService;
use crate::auth::claims::Claims;
use crate::config::db;

mod api;
mod auth;
mod config;
mod constants;
mod handler;
mod models;
mod repository;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize Log4rs
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    info!("Initializing application...");

    // Load .env file
    dotenv().ok();

    // Initialize MongoDB connection
    let client = db::init().await;

    // Initialize TaskService with MongoDB collection
    let task_service = TaskService::new(client.database("users").collection::<Task>("tasks"));

    // Get server host and port number from the environment file
    let server_host = env::var("SERVER.HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port: u16 = env::var("SERVER.PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080);

    info!("Starting Actix-web server on {}:{}", server_host, server_port);

    // Config and start Actix-web server
    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validator);
        App::new()
            // Configure CORS
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_origin("http://localhost:8080")
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            // Configure compression
            .wrap(middleware::Compress::default())
            // Configure app data
            .app_data(Data::new(client.clone()))
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(task_service.clone())) // Pass TaskService into app state
            // Configure un-secure controllers
            .configure(api::init_auth_api)
            .configure(api::init_ping_api)
            .configure(api::init_location_api)
            // Configure secure controller with JWT authentication under '/api' scope
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .guard(check_auth)
                    .configure(api::init_user_api)
                    .configure(api::init_hello_api)
                    .configure(api::init_task_api)
                    .configure(api::init_aggregator_api) 
            )
           
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", "0.3.0")))
            .wrap(middleware::Logger::default())
    })
    .bind((server_host, server_port))
    .unwrap_or_else(|_| panic!("Error binding to port '{:?}'", server_port))
    .run()
    .await
}

fn json_error_handler(err: JsonPayloadError, _req: &HttpRequest) -> Error {
    let detail = err.to_string();
    let resp = match &err {
        JsonPayloadError::ContentType => HttpResponse::UnsupportedMediaType().json(ApiError {
            status: 415,
            time: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            message: "Unsupported media type".to_owned(),
            debug_message: Some(detail),
            sub_errors: Vec::new(),
        }),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => HttpResponse::UnprocessableEntity().json(ApiError {
            status: 422,
            time: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            message: "Unprocessable payload".to_owned(),
            debug_message: Some(detail),
            sub_errors: Vec::new(),
        }),
        _ => HttpResponse::BadRequest().json(ApiError {
            status: 400,
            time: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            message: "Bad request. Missing parameter and/or wrong payload.".to_owned(),
            debug_message: Some(detail),
            sub_errors: Vec::new(),
        }),
    };
    InternalError::from_response(err, resp).into()
}

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    warn!("Validating JWT auth");
    let result = Claims::decode_jwt(credentials.token());
    match result {
        Ok(claims) => {
            req.attach(claims.permissions);
            Ok(req)
        }
        Err(e) => Err((e, req)),
    }
}

use actix_web::guard::GuardContext;

fn check_auth(ctx: &GuardContext) -> bool {
    if let Some(auth_header) = ctx.head().headers().get(http::header::AUTHORIZATION) {
        return auth_header.to_str().is_ok();
    }
    false
}
