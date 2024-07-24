use serde::{Deserialize, Serialize};

// New structs for the required APIs

#[derive(Deserialize)]
pub struct GetProjectDetailRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
}

#[derive(Serialize)]
pub struct GetProjectDetailResponse {
    pub project_name: String,
    pub project_description: String,
    pub tags: Vec<String>,
}


#[derive(Deserialize)]
pub struct AddProjectRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
    pub project_descr: String,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct AddProjectResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
    pub new_project_name: String,
    pub new_project_descr: String,
    pub new_tags: Vec<String>,
}

#[derive(Serialize)]
pub struct UpdateProjectResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct DeleteProjectRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
}

#[derive(Serialize)]
pub struct DeleteProjectResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct GetTaskDetailRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
}

#[derive(Serialize)]
pub struct GetTaskDetailResponse {
    pub tasks: Vec<Task>,
}

// Structs used within the responses
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


#[derive(Deserialize)]
pub struct AddTaskRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
    pub worker_name: String,
    pub task_title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Serialize)]
pub struct AddTaskResponse {
    pub success: bool,
    pub message: String,
}


#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
    pub task_title: String,
    pub new_task_title: String,
    pub new_worker_name: String,
    pub new_description: String,
    pub new_start_time: String,
    pub new_end_time: String,
}

#[derive(Serialize)]
pub struct UpdateTaskResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct DeleteTaskRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub project_name: String,
    pub task_title: String,
}

#[derive(Serialize)]
pub struct DeleteTaskResponse {
    pub success: bool,
    pub message: String,
}