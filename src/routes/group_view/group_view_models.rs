use serde::{Deserialize, Serialize};

// structs
#[derive(Serialize, Deserialize)]
pub struct Worker {
    pub user_name: String,
    pub user_email: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub tag_name: String,
    pub tag_color: String,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub task_title: String,
    pub worker_name: String,
    pub start_time: String,
    pub end_time: String,
    pub description: String,
    pub project_name: String,
    pub tag_colors: Vec<String>,
}

// json format

#[derive(Deserialize)]
pub struct GetWorkerListRequest {
    pub owner_user_name: String,
    pub group_name: String,
}

#[derive(Serialize)]
pub struct GetWorkerListResponse {
    pub workers: Vec<Worker>,
}


#[derive(Deserialize)]
pub struct AddWorkerRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub worker_user_name: String,
}

#[derive(Serialize)]
pub struct AddWorkerResponse {
    pub success: bool,
    pub message: String,
}


#[derive(Deserialize)]
pub struct GetTagListRequest {
    pub owner_user_name: String,
    pub group_name: String,
}

#[derive(Serialize)]
pub struct GetTagListResponse {
    pub tags: Vec<Tag>,
}


#[derive(Deserialize)]
pub struct GetTaskListByTagListRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct GetTaskListByTagListResponse {
    pub tasks: Vec<Task>,
}


#[derive(Deserialize)]
pub struct GetTaskListByProjectNameRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
}

#[derive(Serialize)]
pub struct GetTaskListByProjectNameResponse {
    pub tasks: Vec<Task>,
}