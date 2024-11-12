use actix_web::{
    delete, get, post, put, web,
    web::{Data, Json, Path},
    HttpResponse,
};
use actix_web_grants::proc_macro::has_any_role;
use log::warn;
use mongodb::Client;
use serde::Deserialize;
use validator::Validate;

use actix_web:: Responder;
use crate::services::{aggregator_service, task_service::TaskService};



use crate::{
    models::{error_model::ApiErrorType, task_model::Task, TaskAggregate},
    services::task_service,
};


pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(create_task);
    cfg.service(get_task);
    cfg.service(update_task);
    cfg.service(delete_task);
    cfg.service(get_all_tasks);
    cfg.service(create_task).service(aggregate_tasks);
}


#[post("/tasks")]
pub async fn create_task(
    client: Data<Client>,
    new_task: Json<Task>,
) -> Result<HttpResponse, ApiErrorType> {

    println!("Here");
    let is_valid = new_task.validate();
    match is_valid {
        Ok(_) => task_service::create_task(&client, new_task).await,
        Err(err) => {
            warn!("Payload validation Error on add task: {}", err);
            // Validation error.
            Err(ApiErrorType::ValidationError {
                validation_error: err,
                object: "Task".to_string(),
            })
        }
    }
}

  async fn aggregate_tasks(task_service: web::Data<TaskService>) -> impl Responder { 
    match task_service.aggregate_tasks().await {
        Ok(aggregator_tasks)  => HttpResponse::Ok().json(aggregated_tasks),
        Err(_) => HttpResponse::InternalServerError().json(ApiErrorType::AggregatorError),
    }   
}

#[get("/tasks/{id}")]
pub async fn get_task(
    client: Data<Client>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    task_service::get_task_by_id(&client, path).await
}

#[put("/tasks/{id}")]
pub async fn update_task(
    client: Data<Client>,
    path: Path<String>,
    update_task: Json<Task>,
) -> Result<HttpResponse, ApiErrorType> {
    task_service::update_task(&client, path, update_task).await
}

#[delete("/tasks/{id}")]
pub async fn delete_task(
    client: Data<Client>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    task_service::delete_task(&client, path).await
}

#[derive(Deserialize)]
pub struct Pagination {
    pub offset: Option<u64>,
    pub limit: Option<i64>,
}

#[get("/tasks")]
#[has_any_role("USER")] 
pub async fn get_all_tasks(
    client: Data<Client>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, ApiErrorType> {
    task_service::get_all_tasks(&client, &pagination.0).await
}

