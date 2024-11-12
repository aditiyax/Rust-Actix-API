use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Task {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[validate(length(min = 5, message = "Title must have minimum of 5 characters"))]
    pub title: String,
    #[validate(length(min = 10, message = "Body must have aleast 10 characters"))]
    pub body: String
}

#[derive(Serialize, Deserialize)]
 pub struct TaskAggregate {
    pub status: String,
    pub count:  i64,
 }