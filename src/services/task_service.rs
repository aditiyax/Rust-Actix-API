use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use log::{error, warn};
use mongodb::bson::{doc, Bson};
use mongodb::error::Error;
use mongodb::Client;
use mongodb::Collection;
use futures::stream::StreamExt;

use crate::api::task_api::Pagination;
use crate::constants;
use crate::models::task_list_response::{Link, LinkHref, Meta, TaskListResponse};
use crate::models::error_model::ApiErrorType;
use crate::models::task_model::{Task, TaskAggregate};
use crate::repository::task_repo;


// Add a new task to MongoDB
pub async fn create_task(
    client: &Data<Client>,
    new_task: Json<Task>,
) -> Result<HttpResponse, ApiErrorType> {
    let data = Task {
        id: None,
        title: new_task.title.to_owned(),
        body: new_task.body.to_owned(),
    };
    let task_detail = task_repo::create_task(client, data).await;
    match task_detail {
        Ok(Some(task)) => Ok(HttpResponse::Created().json(task)),
        Ok(None) => Err(ApiErrorType::InternalServerError),
        Err(err) => {
            error!("Error: {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

// Get a task by given id from MongoDB database
pub async fn get_task_by_id(
    client: &Data<Client>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        warn!("Task with id - {} not found for get task by ID", id);
        return Err(ApiErrorType::BadRequest);
    }
    let task_detail = task_repo::get_task(client, &id).await;
    handle_optional_task_response(task_detail)
}

// Update a task for a given unique task id.
pub async fn update_task(
    client: &Data<Client>,
    path: Path<String>,
    update_task: Json<Task>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        return Err(ApiErrorType::BadRequest);
    };
    let data = Task {
        id: Some(String::from(&id)),
        title: update_task.title.to_owned(),
        body: update_task.body.to_owned(),
    };

    let update_result = task_repo::update_task(client, &id, data).await;
    match update_result {
        Ok(update) => {
            if update.matched_count == 1 {
                let updated_task_info = task_repo::get_task(client, &id).await;
                handle_optional_task_response(updated_task_info)
            } else {
                warn!("Task with id -{} not found to update task by ID", id);
                Err(ApiErrorType::TaskNotFound)
            }
        }
        Err(err) => {
            error!("Error: {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

// Delete a task for a given unique task id.
pub async fn delete_task(
    client: &Data<Client>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        return Err(ApiErrorType::TaskNotFound);
    };
    let result = task_repo::delete_task(client, &id).await;
    match result {
        Ok(res) => {
            if res.deleted_count == 1 {
                Ok(HttpResponse::NoContent().finish())
            } else {
                warn!("Task with id -{} not found for delete task by ID", id);
                Err(ApiErrorType::TaskNotFound)
            }
        }
        Err(err) => {
            error!("Error : {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

// Fetch all tasks from the database
pub async fn get_all_tasks(
    client: &Data<Client>,
    pagination: &Pagination,
) -> Result<HttpResponse, ApiErrorType> {
    let offset = pagination.offset.unwrap_or(constants::DEFAULT_OFFSET_SIZE);
    let limit = pagination.limit.unwrap_or(constants::DEFAULT_LIMIT_SIZE);
    let task_list = task_repo::get_all_tasks(client, offset, limit).await;
    let task_count = task_repo::get_tasks_size(client).await.unwrap_or(0);
    let last_offset = (task_count / (limit as u64)) * limit as u64;

    let next_offset = i64::try_from(offset).unwrap_or(0) + limit;
    let previous_offset = i64::try_from(offset).unwrap_or(0) - limit;

    match task_list {
        Ok(t) => {
            let response = TaskListResponse {
                data: t,
                meta: Meta {
                    offset,
                    limit,
                    total_results: task_count,
                    search_criteria: None,
                    sort_by: None,
                },
                _link: Link {
                    first: LinkHref {
                        href: format!("/api/tasks?offset={}&limit={}", 0, limit).to_string(),
                    },
                    last: LinkHref {
                        href: format!("/api/tasks?offset={}&limit={}", last_offset, limit)
                            .to_string(),
                    },
                    previous: if previous_offset < 0 {
                        None
                    } else {
                        Some(LinkHref {
                            href: format!("/api/tasks?offset={}&limit={}", previous_offset, limit)
                                .to_string(),
                        })
                    },
                    next: if (next_offset as u64) > last_offset {
                        None
                    } else {
                        Some(LinkHref {
                            href: format!("/api/tasks?offset={}&limit={}", next_offset, limit)
                                .to_string(),
                        })
                    },
                    self_link: LinkHref {
                        href: format!("/api/tasks?offset={}&limit={}", offset, limit).to_string(),
                    },
                },
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            error!("Error : {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

fn handle_optional_task_response(
    task: Result<Option<Task>, Error>,
) -> Result<HttpResponse, ApiErrorType> {
    match task {
        Ok(Some(task)) => Ok(HttpResponse::Ok().json(task)),
        Ok(None) => Err(ApiErrorType::TaskNotFound),
        Err(err) => {
            error!("Error: {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

// TaskService struct
#[derive(Clone)] // Deriving Clone to fix the error
pub struct TaskService {
    repo: TaskRepository,
}

impl TaskService {
    pub fn new (collection: Collection<Task>) -> Self {
        Self {
            repo: TaskRepository::new(collection),
        }
    }

    pub async fn aggregate_tasks(&self) -> Result<Vec<TaskAggregate>, mongodb::error::Error> {
        let pipeline = vec![
            doc! {"$group": {"_id": "$status", "count": {"$sum": 1}}},
        ];

        let cursor = self.repo.collection.aggregate(pipeline, None).await?;
        let mut results = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(bson) => {
                    if let Ok(agg) = bson::from_bson(Bson::Document(bson)) {
                        results.push(agg);
                    }
                }
                Err(e) => eprintln!("Error while aggregating: {}", e),
            }
        }
        Ok(results)
    }
}
