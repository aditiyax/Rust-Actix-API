use actix_web::web::Data;
use futures::TryStreamExt;
use mongodb::options::FindOptions;
use mongodb::{
    bson::doc, 
    error::Error,
    results::{DeleteResult, UpdateResult},
    Client, Collection,
};
use nanoid::nanoid;

use crate::models::task_model::Task;
use crate::{constants, models::task_list_response::Tasks};

// Function to create a new task in MongoDB
pub async fn create_task(client: &Data<Client>, new_task: Task) -> Result<Option<Task>, Error> {
    let new_doc = Task {
        id: Some(nanoid!()),  // Generate a unique ID for the new task
        title: new_task.title,
        body: new_task.body,
    };

    let collection = client
        .database(constants::MONGO_DATABASE)
        .collection::<Task>(constants::MONGO_TASK_COLLECTION);
    
    let result = collection.insert_one(new_doc.clone(), None).await?;

    collection.find_one(doc! { "_id": result.inserted_id }, None).await
}

// Function to retrieve a task by its ID from MongoDB
pub async fn get_task(client: &Data<Client>, id: &String) -> Result<Option<Task>, Error> {
    let filter = doc! { "_id": id };
    
    let collection = client
        .database(constants::MONGO_DATABASE)
        .collection::<Task>(constants::MONGO_TASK_COLLECTION);
    
    collection.find_one(filter, None).await
}

// Function to update a task by its ID
pub async fn update_task(
    client: &Data<Client>,
    id: &String,
    updated_task: Task,
) -> Result<UpdateResult, Error> {
    let filter = doc! { "_id": id };
    let update_doc = doc! {
        "$set": {
            "title": updated_task.title,
            "body": updated_task.body
        },
    };

    let collection = client
        .database(constants::MONGO_DATABASE)
        .collection::<Task>(constants::MONGO_TASK_COLLECTION);
    
    collection.update_one(filter, update_doc, None).await
}

// Function to delete a task by its ID
pub async fn delete_task(client: &Data<Client>, id: &String) -> Result<DeleteResult, Error> {
    let filter = doc! { "_id": id };

    let collection = client
        .database(constants::MONGO_DATABASE)
        .collection::<Task>(constants::MONGO_TASK_COLLECTION);
    
    collection.delete_one(filter, None).await
}

// Function to retrieve all tasks with pagination
pub async fn get_all_tasks(
    client: &Data<Client>,
    offset: u64,
    limit: i64,
) -> Result<Vec<Tasks>, Error> {
    let collection = client
        .database(constants::MONGO_DATABASE)
        .collection::<Task>(constants::MONGO_TASK_COLLECTION);
    
    let find_options = FindOptions::builder()
        .skip(offset)
        .limit(limit)
        .sort(doc! { "title": 1 })  // Sorting tasks by title
        .build();
    
    let mut cursors = collection.find(None, find_options).await?;
    let mut tasks: Vec<Tasks> = Vec::new();

    while let Some(task) = cursors.try_next().await? {
        tasks.push(Tasks {
            id: task.id.unwrap_or_else(|| "".to_string()),
            title: task.title,
            body: task.body,
        });
    }
    
    Ok(tasks)
}

// Function to get the total count of tasks
pub async fn get_tasks_size(client: &Data<Client>) -> Result<u64, Error> {
    let collection = client
        .database(constants::MONGO_DATABASE)
        .collection::<Task>(constants::MONGO_TASK_COLLECTION);
    
    collection.count_documents(doc! {}, None).await
}

