use actix_web::{web, HttpResponse, HttpRequest, Responder};
use sqlx::MySqlPool;
use log::{error, info};
use time::{PrimitiveDateTime, macros::format_description};
use super::project_view_models::{
    GetProjectDetailRequest, GetProjectDetailResponse,
    AddProjectRequest, AddProjectResponse,
    UpdateProjectRequest, UpdateProjectResponse,
    DeleteProjectRequest, DeleteProjectResponse,
    GetTaskDetailRequest, GetTaskDetailResponse, Task,
    AddTaskRequest, AddTaskResponse,
    UpdateTaskRequest, UpdateTaskResponse,
    DeleteTaskRequest, DeleteTaskResponse,
};

// Default handler for project selection root
pub async fn project_view_get() -> impl Responder {
    HttpResponse::Ok().body("Hello, this is the Project View endpoint.")
}

// Handler to get project details
pub async fn get_project_detail(
    pool: web::Data<MySqlPool>,
    _: HttpRequest,
    request: web::Json<GetProjectDetailRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(GetProjectDetailResponse {
                project_name: "".to_string(),
                project_description: "".to_string(),
                tags: Vec::new(),
            });
        }
    };

    // Get project details from Projects_
    let project_result = sqlx::query!(
        "
        SELECT p.project_name, p.project_description
        FROM Projects_ p
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let (project_name, project_description) = match project_result {
        Ok(record) => (record.project_name, record.project_description),
        Err(_) => {
            info!("Project not found: {}", project_name);
            return HttpResponse::BadRequest().json(GetProjectDetailResponse {
                project_name: "".to_string(),
                project_description: "".to_string(),
                tags: Vec::new(),
            });
        }
    };

    // Get project tags from TagProjectMapping_ and Tags_
    let tags_result = sqlx::query!(
        "
        SELECT t.tag_name
        FROM Tags_ t
        JOIN TagProjectMapping_ tpm ON t.tag_id = tpm.tag_id
        JOIN Projects_ p ON tpm.project_id = p.project_id
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_all(pool.get_ref())
    .await;

    let tags: Vec<String> = match tags_result {
        Ok(records) => records.into_iter().map(|record| record.tag_name).collect(),
        Err(_) => {
            info!("Tags not found for project: {}", project_name);
            Vec::new()
        }
    };

    HttpResponse::Ok().json(GetProjectDetailResponse {
        project_name,
        project_description,
        tags,
    })
}

// Handler to add a project
pub async fn add_project(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<AddProjectRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;
    let project_descr: &String = &request.project_descr;
    let tags = &request.tags;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for add_project");
            return HttpResponse::BadRequest().json(AddProjectResponse {
                success: false,
                message: "Session ID not found".to_string(),
            });
        }
    };

    let session_result = sqlx::query!(
        "SELECT u.user_name FROM Sessions_ s
         JOIN Users_ u ON s.user_id = u.user_id
         WHERE s.session_id = ? AND s.expires_at > NOW()",
        session_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let current_user_name = match session_result {
        Ok(session) => session.user_name,
        Err(_) => {
            info!("Invalid or expired session ID: {}", session_id);
            return HttpResponse::BadRequest().json(AddProjectResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(AddProjectResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(AddProjectResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Add project to Projects_
    let insert_result = sqlx::query!(
        "INSERT INTO Projects_ (group_id, project_name, project_description) VALUES (?, ?, ?)",
        group_id, project_name, project_descr
    )
    .execute(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        error!("Failed to add project to group {}: {}", group_id, e);
        return HttpResponse::InternalServerError().json(AddProjectResponse {
            success: false,
            message: "Failed to add project".to_string(),
        });
    }

    // Get the project_id of the newly added project
    let project_id_result = sqlx::query!(
        "SELECT project_id FROM Projects_ WHERE group_id = ? AND project_name = ?",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let project_id = match project_id_result {
        Ok(record) => record.project_id,
        Err(_) => {
            info!("Project not found after insertion: {}", project_name);
            return HttpResponse::InternalServerError().json(AddProjectResponse {
                success: false,
                message: "Project not found after insertion".to_string(),
            });
        }
    };

    // Add tags to TagProjectMapping_
    for tag_name in tags {
        let tag_id_result = sqlx::query!(
            "SELECT tag_id FROM Tags_ WHERE group_id = ? AND tag_name = ?",
            group_id, tag_name
        )
        .fetch_one(pool.get_ref())
        .await;

        let tag_id = match tag_id_result {
            Ok(record) => record.tag_id,
            Err(_) => {
                info!("Tag not found: {}", tag_name);
                continue;
            }
        };

        let insert_tag_mapping_result = sqlx::query!(
            "INSERT INTO TagProjectMapping_ (project_id, tag_id) VALUES (?, ?)",
            project_id, tag_id
        )
        .execute(pool.get_ref())
        .await;

        if let Err(e) = insert_tag_mapping_result {
            error!("Failed to add tag mapping for project {}: {}", project_id, e);
        }
    }

    HttpResponse::Ok().json(AddProjectResponse {
        success: true,
        message: "Project added successfully".to_string(),
    })
}

pub async fn update_project(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<UpdateProjectRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;
    let new_project_name = &request.new_project_name;
    let new_project_descr = &request.new_project_descr;
    let new_tags = &request.new_tags;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for update_project");
            return HttpResponse::BadRequest().json(UpdateProjectResponse {
                success: false,
                message: "Session ID not found".to_string(),
            });
        }
    };

    let session_result = sqlx::query!(
        "SELECT u.user_name FROM Sessions_ s
            JOIN Users_ u ON s.user_id = u.user_id
            WHERE s.session_id = ? AND s.expires_at > NOW()",
        session_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let current_user_name = match session_result {
        Ok(session) => session.user_name,
        Err(_) => {
            info!("Invalid or expired session ID: {}", session_id);
            return HttpResponse::BadRequest().json(UpdateProjectResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(UpdateProjectResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(UpdateProjectResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Get project_id and current details using project_name from Projects_
    let project_details_result = sqlx::query!(
        "
        SELECT p.project_id, p.project_name, p.project_description 
        FROM Projects_ p
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let (project_id, current_project_name, current_project_descr) = match project_details_result {
        Ok(record) => (record.project_id, record.project_name, record.project_description),
        Err(_) => {
            info!("Project not found: {}", project_name);
            return HttpResponse::BadRequest().json(UpdateProjectResponse {
                success: false,
                message: "Project not found".to_string(),
            });
        }
    };

    // Determine the new project name and description, maintaining current values if new ones are empty
    let final_project_name = if new_project_name.is_empty() {
        &current_project_name
    } else {
        new_project_name
    };

    let final_project_descr = if new_project_descr.is_empty() {
        &current_project_descr
    } else {
        new_project_descr
    };

    // Update project details in Projects_
    let update_result = sqlx::query!(
        "
        UPDATE Projects_
        SET project_name = ?, project_description = ?
        WHERE project_id = ?
        ",
        final_project_name, final_project_descr, project_id
    )
    .execute(pool.get_ref())
    .await;

    if let Err(e) = update_result {
        error!("Failed to update project {}: {}", project_id, e);
        return HttpResponse::InternalServerError().json(UpdateProjectResponse {
            success: false,
            message: "Failed to update project".to_string(),
        });
    }

    // If new_tags is not empty, clear existing tags and add new ones
    if !new_tags.is_empty() {
        let clear_tags_result = sqlx::query!(
            "DELETE FROM TagProjectMapping_ WHERE project_id = ?",
            project_id
        )
        .execute(pool.get_ref())
        .await;

        if let Err(e) = clear_tags_result {
            error!("Failed to clear tags for project {}: {}", project_id, e);
            return HttpResponse::InternalServerError().json(UpdateProjectResponse {
                success: false,
                message: "Failed to clear project tags".to_string(),
            });
        }

        // Add new tags to TagProjectMapping_
        for tag_name in new_tags {
            let tag_id_result = sqlx::query!(
                "SELECT tag_id FROM Tags_ WHERE group_id = ? AND tag_name = ?",
                group_id, tag_name
            )
            .fetch_one(pool.get_ref())
            .await;

            let tag_id = match tag_id_result {
                Ok(record) => record.tag_id,
                Err(_) => {
                    info!("Tag not found: {}", tag_name);
                    continue;
                }
            };

            let insert_tag_mapping_result = sqlx::query!(
                "INSERT INTO TagProjectMapping_ (project_id, tag_id) VALUES (?, ?)",
                project_id, tag_id
            )
            .execute(pool.get_ref())
            .await;

            if let Err(e) = insert_tag_mapping_result {
                error!("Failed to add tag mapping for project {}: {}", project_id, e);
            }
        }
    }

    HttpResponse::Ok().json(UpdateProjectResponse {
        success: true,
        message: "Project updated successfully".to_string(),
    })
}

pub async fn delete_project(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<DeleteProjectRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for delete_project");
            return HttpResponse::BadRequest().json(DeleteProjectResponse {
                success: false,
                message: "Session ID not found".to_string(),
            });
        }
    };

    let session_result = sqlx::query!(
        "SELECT u.user_name FROM Sessions_ s
         JOIN Users_ u ON s.user_id = u.user_id
         WHERE s.session_id = ? AND s.expires_at > NOW()",
        session_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let current_user_name = match session_result {
        Ok(session) => session.user_name,
        Err(_) => {
            info!("Invalid or expired session ID: {}", session_id);
            return HttpResponse::BadRequest().json(DeleteProjectResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(DeleteProjectResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(DeleteProjectResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Get project_id using project_name from Projects_
    let project_id_result = sqlx::query!(
        "
        SELECT p.project_id 
        FROM Projects_ p
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let project_id = match project_id_result {
        Ok(record) => record.project_id,
        Err(_) => {
            info!("Project not found: {}", project_name);
            return HttpResponse::BadRequest().json(DeleteProjectResponse {
                success: false,
                message: "Project not found".to_string(),
            });
        }
    };

    // Begin a transaction
    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(DeleteProjectResponse {
                success: false,
                message: "Failed to start transaction".to_string(),
            });
        }
    };

    // Delete tasks associated with the project
    let delete_tasks_result = sqlx::query!(
        "
        DELETE FROM Tasks_ 
        WHERE project_id = ?
        ",
        project_id
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = delete_tasks_result {
        error!("Failed to delete tasks for project {}: {}", project_id, e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(DeleteProjectResponse {
            success: false,
            message: "Failed to delete tasks".to_string(),
        });
    }

    // Delete tag mappings associated with the project
    let delete_tag_mappings_result = sqlx::query!(
        "
        DELETE FROM TagProjectMapping_ 
        WHERE project_id = ?
        ",
        project_id
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = delete_tag_mappings_result {
        error!("Failed to delete tag mappings for project {}: {}", project_id, e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(DeleteProjectResponse {
            success: false,
            message: "Failed to delete tag mappings".to_string(),
        });
    }

    // Delete the project
    let delete_project_result = sqlx::query!(
        "
        DELETE FROM Projects_ 
        WHERE project_id = ?
        ",
        project_id
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = delete_project_result {
        error!("Failed to delete project {}: {}", project_id, e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(DeleteProjectResponse {
            success: false,
            message: "Failed to delete project".to_string(),
        });
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(DeleteProjectResponse {
            success: false,
            message: "Failed to commit transaction".to_string(),
        });
    }

    HttpResponse::Ok().json(DeleteProjectResponse {
        success: true,
        message: "Project deleted successfully".to_string(),
    })
}


// Handler to get task details
pub async fn get_task_detail(
    pool: web::Data<MySqlPool>,
    _: HttpRequest,
    request: web::Json<GetTaskDetailRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(GetTaskDetailResponse { tasks: Vec::new() });
        }
    };

    // Get project_id using project_name from Projects_
    let project_id_result = sqlx::query!(
        "
        SELECT p.project_id 
        FROM Projects_ p
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let project_id = match project_id_result {
        Ok(record) => record.project_id,
        Err(_) => {
            info!("Project not found: {}", project_name);
            return HttpResponse::BadRequest().json(GetTaskDetailResponse { tasks: Vec::new() });
        }
    };

    // Get tasks for the project
    let tasks_result = sqlx::query!(
        "
        SELECT t.title AS task_title, u.user_name AS worker_name, t.start_time, t.end_time, t.description, p.project_name, GROUP_CONCAT(ta.tag_color SEPARATOR ',') AS tag_colors
        FROM Tasks_ t
        JOIN Users_ u ON t.worker_user_id = u.user_id
        JOIN Projects_ p ON t.project_id = p.project_id
        LEFT JOIN TagProjectMapping_ tpm ON t.project_id = tpm.project_id
        LEFT JOIN Tags_ ta ON ta.tag_id = tpm.tag_id
        WHERE t.project_id = ?
        GROUP BY t.task_id
        ",
        project_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match tasks_result {    
        Ok(records) => {
            let tasks: Vec<Task> = records.into_iter().map(|record| Task {
                task_title: record.task_title,
                worker_name: record.worker_name,
                start_time: record.start_time.to_string(),
                end_time: record.end_time.to_string(),
                description: record.description,
                project_name: record.project_name,
                tag_colors: record.tag_colors.expect("TAG COLORS EMPTY, NEVER").split(',').map(|s| s.to_string()).collect(),
            }).collect();

            HttpResponse::Ok().json(GetTaskDetailResponse { tasks })
        },
        Err(e) => {
            error!("Failed to fetch tasks for project_id {}: {}", project_id, e);
            HttpResponse::InternalServerError().json(GetTaskDetailResponse { tasks: Vec::new() })
        }
    }
}


pub async fn add_task(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<AddTaskRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;
    let worker_name = &request.worker_name;
    let task_title = &request.task_title;
    let description = &request.description;

    let format = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    );

    // Parse start_time and end_time
    let start_time = 
        PrimitiveDateTime::parse(&request.start_time, &format)
        .expect("Failed to parse start time");
    
    let end_time = 
        PrimitiveDateTime::parse(&request.end_time, &format)
        .expect("Failed to parse end time");

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for add_task");
            return HttpResponse::BadRequest().json(AddTaskResponse {
                success: false,
                message: "Session ID not found".to_string(),
            });
        }
    };

    let session_result = sqlx::query!(
        "SELECT u.user_name FROM Sessions_ s
         JOIN Users_ u ON s.user_id = u.user_id
         WHERE s.session_id = ? AND s.expires_at > NOW()",
        session_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let current_user_name = match session_result {
        Ok(session) => session.user_name,
        Err(_) => {
            info!("Invalid or expired session ID: {}", session_id);
            return HttpResponse::BadRequest().json(AddTaskResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(AddTaskResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(AddTaskResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Get project_id using project_name from Projects_
    let project_id_result = sqlx::query!(
        "
        SELECT p.project_id 
        FROM Projects_ p
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let project_id = match project_id_result {
        Ok(record) => record.project_id,
        Err(_) => {
            info!("Project not found: {}", project_name);
            return HttpResponse::BadRequest().json(AddTaskResponse {
                success: false,
                message: "Project not found".to_string(),
            });
        }
    };

    // Get worker_user_id using worker_name from Users_
    let worker_id_result = sqlx::query!(
        "
        SELECT u.user_id 
        FROM Users_ u
        WHERE u.user_name = ?
        ",
        worker_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let worker_user_id = match worker_id_result {
        Ok(record) => record.user_id,
        Err(_) => {
            info!("Worker not found: {}", worker_name);
            return HttpResponse::BadRequest().json(AddTaskResponse {
                success: false,
                message: "Worker not found".to_string(),
            });
        }
    };

    // Add task to Tasks_
    let insert_result = sqlx::query!(
        "
        INSERT INTO Tasks_ (project_id, worker_user_id, title, description, start_time, end_time) 
        VALUES (?, ?, ?, ?, ?, ?)",
        project_id, worker_user_id, task_title, description, start_time, end_time
    )
    .execute(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        error!("Failed to add task to project {}: {}", project_id, e);
        return HttpResponse::InternalServerError().json(AddTaskResponse {
            success: false,
            message: "Failed to add task".to_string(),
        });
    }

    HttpResponse::Ok().json(AddTaskResponse {
        success: true,
        message: "Task added successfully".to_string(),
    })
}

pub async fn update_task(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<UpdateTaskRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;
    let task_title = &request.task_title;
    let new_task_title = &request.new_task_title;
    let new_worker_name = &request.new_worker_name;
    let new_description = &request.new_description;
    let new_start_time = &request.new_start_time;
    let new_end_time = &request.new_end_time;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for update_task");
            return HttpResponse::BadRequest().json(UpdateTaskResponse {
                success: false,
                message: "Session ID not found".to_string(),
            });
        }
    };

    let session_result = sqlx::query!(
        "SELECT u.user_name FROM Sessions_ s
         JOIN Users_ u ON s.user_id = u.user_id
         WHERE s.session_id = ? AND s.expires_at > NOW()",
        session_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let current_user_name = match session_result {
        Ok(session) => session.user_name,
        Err(_) => {
            info!("Invalid or expired session ID: {}", session_id);
            return HttpResponse::BadRequest().json(UpdateTaskResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(UpdateTaskResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(UpdateTaskResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Get project_id using project_name from Projects_
    let project_id_result = sqlx::query!(
        "
        SELECT p.project_id 
        FROM Projects_ p
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let project_id = match project_id_result {
        Ok(record) => record.project_id,
        Err(_) => {
            info!("Project not found: {}", project_name);
            return HttpResponse::BadRequest().json(UpdateTaskResponse {
                success: false,
                message: "Project not found".to_string(),
            });
        }
    };

    // Get current task details
    let task_details_result = sqlx::query!(
        "
        SELECT t.task_id, t.title, t.worker_user_id, t.description, t.start_time, t.end_time
        FROM Tasks_ t
        WHERE t.project_id = ? AND t.title = ?
        ",
        project_id, task_title
    )
    .fetch_one(pool.get_ref())
    .await;

    let (task_id, current_task_title, current_worker_user_id, current_description, current_start_time, current_end_time) = match task_details_result {
        Ok(record) => (
            record.task_id,
            record.title,
            record.worker_user_id.expect("worker id should exist"),
            record.description,
            record.start_time,
            record.end_time,
        ),
        Err(_) => {
            info!("Task not found: {}", task_title);
            return HttpResponse::BadRequest().json(UpdateTaskResponse {
                success: false,
                message: "Task not found".to_string(),
            });
        }
    };

    // Determine the new task title, worker ID, description, start time, and end time
    let final_task_title = if new_task_title.is_empty() {
        &current_task_title
    } else {
        new_task_title
    };

    let final_worker_user_id = if new_worker_name.is_empty() {
        current_worker_user_id
    } else {
        // Get worker_user_id using new_worker_name from Users_
        let worker_id_result = sqlx::query!(
            "
            SELECT u.user_id 
            FROM Users_ u
            WHERE u.user_name = ?
            ",
            new_worker_name
        )
        .fetch_one(pool.get_ref())
        .await;

        match worker_id_result {
            Ok(record) => record.user_id,
            Err(_) => {
                info!("Worker not found: {}", new_worker_name);
                return HttpResponse::BadRequest().json(UpdateTaskResponse {
                    success: false,
                    message: "Worker not found".to_string(),
                });
            }
        }
    };

    let final_description = if new_description.is_empty() {
        &current_description
    } else {
        new_description
    };

    let final_start_time = if new_start_time.is_empty() {
        current_start_time
    } else {
        match PrimitiveDateTime::parse(new_start_time, format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")) {
            Ok(time) => time,
            Err(_) => {
                return HttpResponse::BadRequest().json(UpdateTaskResponse {
                    success: false,
                    message: "Invalid start time format".to_string(),
                });
            }
        }
    };

    let final_end_time = if new_end_time.is_empty() {
        current_end_time
    } else {
        match PrimitiveDateTime::parse(new_end_time, format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")) {
            Ok(time) => time,
            Err(_) => {
                return HttpResponse::BadRequest().json(UpdateTaskResponse {
                    success: false,
                    message: "Invalid end time format".to_string(),
                });
            }
        }
    };

    // Update task details in Tasks_
    let update_result = sqlx::query!(
        "
        UPDATE Tasks_
        SET title = ?, worker_user_id = ?, description = ?, start_time = ?, end_time = ?
        WHERE task_id = ?
        ",
        final_task_title, final_worker_user_id, final_description, final_start_time, final_end_time, task_id
    )
    .execute(pool.get_ref())
    .await;

    if let Err(e) = update_result {
        error!("Failed to update task {}: {}", task_id, e);
        return HttpResponse::InternalServerError().json(UpdateTaskResponse {
            success: false,
            message: "Failed to update task".to_string(),
        });
    }

    HttpResponse::Ok().json(UpdateTaskResponse {
        success: true,
        message: "Task updated successfully".to_string(),
    })
}

pub async fn delete_task(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<DeleteTaskRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;
    let task_title = &request.task_title;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for delete_task");
            return HttpResponse::BadRequest().json(DeleteTaskResponse {
                success: false,
                message: "Session ID not found".to_string(),
            });
        }
    };

    let session_result = sqlx::query!(
        "SELECT u.user_name FROM Sessions_ s
         JOIN Users_ u ON s.user_id = u.user_id
         WHERE s.session_id = ? AND s.expires_at > NOW()",
        session_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let current_user_name = match session_result {
        Ok(session) => session.user_name,
        Err(_) => {
            info!("Invalid or expired session ID: {}", session_id);
            return HttpResponse::BadRequest().json(DeleteTaskResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(DeleteTaskResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get group_id using group_name from Groups_
    let group_id_result = sqlx::query!(
        "
        SELECT g.group_id 
        FROM Groups_ g
        JOIN Users_ u ON g.owner_user_id = u.user_id
        WHERE g.group_name = ? AND u.user_name = ?
        ",
        group_name, owner_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let group_id = match group_id_result {
        Ok(record) => record.group_id,
        Err(_) => {
            info!("Group not found: {}", group_name);
            return HttpResponse::BadRequest().json(DeleteTaskResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Get project_id using project_name from Projects_
    let project_id_result = sqlx::query!(
        "
        SELECT p.project_id 
        FROM Projects_ p
        WHERE p.group_id = ? AND p.project_name = ?
        ",
        group_id, project_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let project_id = match project_id_result {
        Ok(record) => record.project_id,
        Err(_) => {
            info!("Project not found: {}", project_name);
            return HttpResponse::BadRequest().json(DeleteTaskResponse {
                success: false,
                message: "Project not found".to_string(),
            });
        }
    };

    // Get task_id using task_title from Tasks_
    let task_id_result = sqlx::query!(
        "
        SELECT t.task_id 
        FROM Tasks_ t
        WHERE t.project_id = ? AND t.title = ?
        ",
        project_id, task_title
    )
    .fetch_one(pool.get_ref())
    .await;

    let task_id = match task_id_result {
        Ok(record) => record.task_id,
        Err(_) => {
            info!("Task not found: {}", task_title);
            return HttpResponse::BadRequest().json(DeleteTaskResponse {
                success: false,
                message: "Task not found".to_string(),
            });
        }
    };

    // Begin a transaction
    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(DeleteTaskResponse {
                success: false,
                message: "Failed to start transaction".to_string(),
            });
        }
    };

    // Delete the task
    let delete_task_result = sqlx::query!(
        "
        DELETE FROM Tasks_ 
        WHERE task_id = ?
        ",
        task_id
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = delete_task_result {
        error!("Failed to delete task {}: {}", task_id, e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(DeleteTaskResponse {
            success: false,
            message: "Failed to delete task".to_string(),
        });
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(DeleteTaskResponse {
            success: false,
            message: "Failed to commit transaction".to_string(),
        });
    }

    HttpResponse::Ok().json(DeleteTaskResponse {
        success: true,
        message: "Task deleted successfully".to_string(),
    })
}