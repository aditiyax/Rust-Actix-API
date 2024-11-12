use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Tasks {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub data: Vec<Tasks>,
    pub meta: Meta,
    pub _link: Link,    
}

#[derive(Debug, Serialize)]
pub struct Meta {
    pub offset: u64,
    pub limit: i64,
    pub total_results: u64,
    pub search_criteria: Option<String>,
    pub sort_by: Option<String>, 
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub first: LinkHref,
    pub last: LinkHref,
    pub previous: Option<LinkHref>,
    pub next: Option<LinkHref>,
    pub self_link: LinkHref,
}

#[derive(Debug, Serialize)]
pub struct LinkHref{
     pub href: String,
}