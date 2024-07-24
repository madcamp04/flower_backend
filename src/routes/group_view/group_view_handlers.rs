use actix_web::{web, HttpResponse, HttpRequest, Responder};
use sqlx::{MySqlPool, Row};
use time::PrimitiveDateTime;
use log::{error, info};
// use time::PrimitiveDateTime;
use super::group_view_models::{
    GetWorkerListRequest, GetWorkerListResponse, Worker,
    AddWorkerRequest, AddWorkerResponse,
    GetTagListRequest, GetTagListResponse, Tag,
    AddTagRequest, AddTagResponse,
    UpdateTagRequest, UpdateTagResponse,
    DeleteTagRequest, DeleteTagResponse,
    GetTaskListByTagListRequest, GetTaskListByTagListResponse, Task,
    GetTaskListByProjectNameRequest, GetTaskListByProjectNameResponse,
    GetProjectListRequest, GetProjectListResponse, Project
};

// Default handler for group selection root
pub async fn group_view_get() -> impl Responder {
    HttpResponse::Ok().body("Hello, this is the Group View endpoint.")
}

// Handler to get the worker list
pub async fn get_worker_list(
    pool: web::Data<MySqlPool>,
    _: HttpRequest,
    request: web::Json<GetWorkerListRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    
    // Get the group_id with group_name from Groups_
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
            return HttpResponse::BadRequest().json(GetWorkerListResponse { workers: Vec::new() });
        }
    };

    // Get the worker list from GroupUserMapping_ where group_id is the one from the previous query
    let workers_result = sqlx::query!(
        "SELECT u.user_name, u.user_email 
         FROM GroupUserMapping_ gum
         JOIN Users_ u ON gum.user_id = u.user_id
         WHERE gum.group_id = ? AND u.user_name != ?",
        group_id, owner_user_name
    )
    .fetch_all(pool.get_ref())
    .await;

    match workers_result {
        Ok(records) => {
            let workers: Vec<Worker> = records.into_iter().map(|record| Worker {
                user_name: record.user_name,
                user_email: record.user_email,
            }).collect();

            HttpResponse::Ok().json(GetWorkerListResponse { workers })
        },
        Err(e) => {
            error!("Failed to fetch workers for group_id {}: {}", group_id, e);
            HttpResponse::InternalServerError().json(GetWorkerListResponse { workers: Vec::new() })
        }
    }
}

// Handler to add a worker
pub async fn add_worker(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<AddWorkerRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let worker_user_name = &request.worker_user_name;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for add_worker");
            return HttpResponse::BadRequest().json(AddWorkerResponse {
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
            return HttpResponse::BadRequest().json(AddWorkerResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(AddWorkerResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Check if worker_user_name exists in Users_ table
    let worker_result = sqlx::query!(
        "SELECT user_id FROM Users_ WHERE user_name = ?",
        worker_user_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let worker_user_id = match worker_result {
        Ok(record) => record.user_id,
        Err(_) => {
            info!("Worker not found: {}", worker_user_name);
            return HttpResponse::BadRequest().json(AddWorkerResponse {
                success: false,
                message: "Worker not found".to_string(),
            });
        }
    };

    // Get the group_id with group_name from Groups_
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
            return HttpResponse::BadRequest().json(AddWorkerResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Add worker to GroupUserMapping_ with writeable set to 0
    let insert_result = sqlx::query!(
        "INSERT INTO GroupUserMapping_ (group_id, user_id, writeable) VALUES (?, ?, false)",
        group_id, worker_user_id
    )
    .execute(pool.get_ref())
    .await;

    match insert_result {
        Ok(_) => HttpResponse::Ok().json(AddWorkerResponse {
            success: true,
            message: "Worker added successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to add worker to group {}: {}", group_id, e);
            HttpResponse::InternalServerError().json(AddWorkerResponse {
                success: false,
                message: "Failed to add worker".to_string(),
            })
        }
    }
}

// Handler to get the tag list
pub async fn get_tag_list(
    pool: web::Data<MySqlPool>,
    _: HttpRequest,
    request: web::Json<GetTagListRequest>,
) -> impl Responder {
    let group_name = &request.group_name;
    let owner_user_name = &request.owner_user_name;

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
            return HttpResponse::BadRequest().json(GetTagListResponse { tags: Vec::new() });
        }
    };

    // Get all tags with group_id
    let tags_result = sqlx::query!(
        "SELECT tag_name, tag_color FROM Tags_ WHERE group_id = ?",
        group_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match tags_result {
        Ok(records) => {
            let tags: Vec<Tag> = records.into_iter().map(|record| Tag {
                tag_name: record.tag_name, // Handle Option<String>
                tag_color: record.tag_color, // Handle Option<String>
            }).collect();

            HttpResponse::Ok().json(GetTagListResponse { tags })
        },
        Err(e) => {
            error!("Failed to fetch tags for group_id {}: {}", group_id, e);
            HttpResponse::InternalServerError().json(GetTagListResponse { tags: Vec::new() })
        }
    }
}

// Handler to add tag
pub async fn add_tag(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<AddTagRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let tag_name = &request.tag_name;
    let tag_color = &request.tag_color;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for add_tag");
            return HttpResponse::BadRequest().json(AddTagResponse {
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
            return HttpResponse::BadRequest().json(AddTagResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(AddTagResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get the group_id with group_name from Groups_
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
            return HttpResponse::BadRequest().json(AddTagResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Add tag to Tags_ table
    let insert_result = sqlx::query!(
        "INSERT INTO Tags_ (group_id, tag_name, tag_color) VALUES (?, ?, ?)",
        group_id, tag_name, tag_color
    )
    .execute(pool.get_ref())
    .await;

    match insert_result {
        Ok(_) => HttpResponse::Ok().json(AddTagResponse {
            success: true,
            message: "Tag added successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to add tag to group {}: {}", group_id, e);
            HttpResponse::InternalServerError().json(AddTagResponse {
                success: false,
                message: "Failed to add tag".to_string(),
            })
        }
    }
}

pub async fn update_tag(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<UpdateTagRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let tag_name = &request.tag_name;
    let new_tag_name = &request.new_tag_name;
    let new_tag_color = &request.new_tag_color;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for update_tag");
            return HttpResponse::BadRequest().json(UpdateTagResponse {
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
            return HttpResponse::BadRequest().json(UpdateTagResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(UpdateTagResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get the group_id with group_name from Groups_
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
            return HttpResponse::BadRequest().json(UpdateTagResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Check if tag_name exists in Tags_ table under the group
    let tag_result = sqlx::query!(
        "SELECT tag_id, tag_name, tag_color FROM Tags_ WHERE group_id = ? AND tag_name = ?",
        group_id, tag_name
    )
    .fetch_one(pool.get_ref())
    .await;

    let (tag_id, current_tag_name, current_tag_color) = match tag_result {
        Ok(record) => (record.tag_id, record.tag_name, record.tag_color),
        Err(_) => {
            info!("Tag not found: {}", tag_name);
            return HttpResponse::BadRequest().json(UpdateTagResponse {
                success: false,
                message: "Tag not found".to_string(),
            });
        }
    };

    // Determine the new tag name and color, maintaining current values if new ones are empty
    let final_tag_name = if new_tag_name.is_empty() {
        &current_tag_name
    } else {
        new_tag_name
    };

    let final_tag_color = if new_tag_color.is_empty() {
        &current_tag_color
    } else {
        new_tag_color
    };

    // Update the tag in the Tags_ table
    let update_result = sqlx::query!(
        "UPDATE Tags_ SET tag_name = ?, tag_color = ? WHERE tag_id = ?",
        final_tag_name, final_tag_color, tag_id
    )
    .execute(pool.get_ref())
    .await;

    match update_result {
        Ok(_) => HttpResponse::Ok().json(UpdateTagResponse {
            success: true,
            message: "Tag updated successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to update tag {}: {}", tag_id, e);
            HttpResponse::InternalServerError().json(UpdateTagResponse {
                success: false,
                message: "Failed to update tag".to_string(),
            })
        }
    }
}

pub async fn delete_tag(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    request: web::Json<DeleteTagRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let tag_name = &request.tag_name;

    // Get the current user name using session ID in the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for delete_tag");
            return HttpResponse::BadRequest().json(DeleteTagResponse {
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
            return HttpResponse::BadRequest().json(DeleteTagResponse {
                success: false,
                message: "Invalid or expired session ID".to_string(),
            });
        }
    };

    // Assert owner_user_name == current user name
    if owner_user_name != &current_user_name {
        return HttpResponse::BadRequest().json(DeleteTagResponse {
            success: false,
            message: "Unauthorized action".to_string(),
        });
    }

    // Get the group_id with group_name from Groups_
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
            return HttpResponse::BadRequest().json(DeleteTagResponse {
                success: false,
                message: "Group not found".to_string(),
            });
        }
    };

    // Check if the tag exists and get its ID
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
            return HttpResponse::BadRequest().json(DeleteTagResponse {
                success: false,
                message: "Tag not found".to_string(),
            });
        }
    };

    // Check if any projects are associated with this tag
    let project_ids_result = sqlx::query!(
        "SELECT project_id FROM TagProjectMapping_ WHERE tag_id = ?",
        tag_id
    )
    .fetch_all(pool.get_ref())
    .await;

    let project_ids: Vec<i32> = match project_ids_result {
        Ok(records) => records.into_iter().map(|record| record.project_id).collect(),
        Err(_) => {
            // No projects associated with this tag
            Vec::new()
        }
    };

    for project_id in &project_ids {
        let tag_count_result = sqlx::query!(
            "SELECT COUNT(*) as tag_count FROM TagProjectMapping_ WHERE project_id = ?",
            project_id
        )
        .fetch_one(pool.get_ref())
        .await;

        let tag_count = match tag_count_result {
            Ok(record) => record.tag_count,
            Err(_) => {
                return HttpResponse::InternalServerError().json(DeleteTagResponse {
                    success: false,
                    message: "Failed to check tag count for project".to_string(),
                });
            }
        };

        if tag_count <= 1 {
            return HttpResponse::BadRequest().json(DeleteTagResponse {
                success: false,
                message: "Cannot delete the tag as it is the only tag for a project".to_string(),
            });
        }
    }

    // Remove all mappings from TagProjectMapping_ table related to this tag
    let delete_mappings_result = sqlx::query!(
        "DELETE FROM TagProjectMapping_ WHERE tag_id = ?",
        tag_id
    )
    .execute(pool.get_ref())
    .await;

    if let Err(e) = delete_mappings_result {
        error!("Failed to delete tag mappings for tag_id {}: {}", tag_id, e);
        return HttpResponse::InternalServerError().json(DeleteTagResponse {
            success: false,
            message: "Failed to delete tag mappings".to_string(),
        });
    }

    // Remove the tag from Tags_ table
    let delete_tag_result = sqlx::query!(
        "DELETE FROM Tags_ WHERE tag_id = ?",
        tag_id
    )
    .execute(pool.get_ref())
    .await;

    match delete_tag_result {
        Ok(_) => HttpResponse::Ok().json(DeleteTagResponse {
            success: true,
            message: "Tag deleted successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to delete tag {}: {}", tag_id, e);
            HttpResponse::InternalServerError().json(DeleteTagResponse {
                success: false,
                message: "Failed to delete tag".to_string(),
            })
        }
    }
}

// Handler to get task list by tag list
pub async fn get_task_list_by_tag_list(
    pool: web::Data<MySqlPool>,
    _: HttpRequest,
    request: web::Json<GetTaskListByTagListRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let tags = &request.tags;
    info!("get_task_list_by_tag_list");
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
            return HttpResponse::BadRequest().json(GetTaskListByTagListResponse { tasks: Vec::new() });
        }
    };

    if tags.is_empty() {
        // Tag list is empty, get all tasks under the group
        let tasks_result = sqlx::query!(
            "SELECT t.title AS task_title, u.user_name AS worker_name, t.start_time, t.end_time, t.description, p.project_name, GROUP_CONCAT(ta.tag_color SEPARATOR ',') AS tag_colors
             FROM Tasks_ t
             JOIN Users_ u ON t.worker_user_id = u.user_id
             JOIN Projects_ p ON t.project_id = p.project_id
             JOIN Tags_ ta ON ta.group_id = ?
             LEFT JOIN TagProjectMapping_ tpm ON ta.tag_id = tpm.tag_id
             WHERE p.group_id = ?
             GROUP BY t.task_id",
            group_id, group_id
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

                HttpResponse::Ok().json(GetTaskListByTagListResponse { tasks })
            },
            Err(e) => {
                error!("Failed to fetch tasks for group_id {}: {}", group_id, e);
                HttpResponse::InternalServerError().json(GetTaskListByTagListResponse { tasks: Vec::new() })
            }
        }
    } else {
        // Tag list is not empty
        info!("tag_list is not empty: {:?}", tags);
        // let tag_names = tags.join(",");
        let tag_names_placeholder = tags.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let tag_names_query_str = format!(
            "SELECT tag_id FROM Tags_ WHERE group_id = ? AND tag_name IN ({})",
            tag_names_placeholder
        );
        
        info!("Constructed query: {}", tag_names_query_str);
        // Execute the query with dynamically provided parameters
        let mut tag_names_query = sqlx::query(&tag_names_query_str);
        tag_names_query = tag_names_query.bind(group_id); // Assuming group_id is defined
        for tag_name in tags {
            tag_names_query = tag_names_query.bind(tag_name);
        }

        let tag_ids_result = tag_names_query.fetch_all(pool.get_ref()).await;

        let tag_ids: Vec<i32> = match tag_ids_result {
            Ok(records) => records.into_iter().map(|record| record.get("tag_id")).collect(),
            Err(e) => {
                info!("Error executing query: {}", e);
                info!("Tags not found in group: {}", group_name);
                return HttpResponse::BadRequest().json(GetTaskListByTagListResponse { tasks: Vec::new() });
            }
        };

        if tag_ids.is_empty() {
            info!("tag_ids is empty");
            return HttpResponse::BadRequest().json(GetTaskListByTagListResponse { tasks: Vec::new() });
        }
        info!("get project_id_list with tag_id_list");
        // 2. get project_id_list that is mapped with tag_id in the tag_id_list.
        let tag_ids_placeholder = tag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let tag_query_str = format!(
            "SELECT DISTINCT project_id FROM TagProjectMapping_ WHERE tag_id IN ({})",
            tag_ids_placeholder
        );
    
        // Execute the query with dynamically provided parameters
        let mut tag_query = sqlx::query(&tag_query_str);
        for tag_id in tag_ids {
            tag_query = tag_query.bind(tag_id);
        }

        let project_ids_result = tag_query.fetch_all(pool.get_ref()).await;

        let project_ids: Vec<i32> = match project_ids_result {
            Ok(records) => records.into_iter().map(|record| record.get("project_id")).collect(),
            Err(_) => {
                info!("Projects not found for tags in group: {}", group_name);
                return HttpResponse::BadRequest().json(GetTaskListByTagListResponse { tasks: Vec::new() });
            }
        };

        if project_ids.is_empty() {
            return HttpResponse::Ok().json(GetTaskListByTagListResponse { tasks: Vec::new() });
        }

        info!("4. get list of tasks");
        // 4. get list of tasks whose project_id is in project_id_list
        // Dynamically construct the IN clause
        let placeholders = project_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query_str = format!(
            "SELECT t.title AS task_title, u.user_name AS worker_name, t.start_time, t.end_time, t.description, p.project_name, GROUP_CONCAT(ta.tag_color SEPARATOR ',') AS tag_colors
            FROM Tasks_ t
            JOIN Users_ u ON t.worker_user_id = u.user_id
            JOIN Projects_ p ON t.project_id = p.project_id
            LEFT JOIN TagProjectMapping_ tpm ON t.project_id = tpm.project_id
            LEFT JOIN Tags_ ta ON tpm.tag_id = ta.tag_id AND ta.group_id = ?
            WHERE p.project_id IN ({})
            GROUP BY t.task_id", placeholders
        );

        // Execute the query with dynamically provided parameters
        let mut query: sqlx::query::Query<sqlx::MySql, sqlx::mysql::MySqlArguments> = sqlx::query(&query_str).bind(group_id);
        for project_id in project_ids {
            query = query.bind(project_id);
        }

        let tasks_result = query.fetch_all(pool.get_ref()).await;
        
        match tasks_result {
            Ok(records) => {
                let tasks: Vec<Task> = records.into_iter().map(|record| Task {
                    task_title: record.get("task_title"),
                    worker_name: record.get("worker_name"),
                    start_time: record.get::<PrimitiveDateTime, _>("start_time").to_string(),
                    end_time: record.get::<PrimitiveDateTime, _>("end_time").to_string(),
                    description: record.get("description"),
                    project_name: record.get("project_name"),
                    tag_colors: record.get::<Option<String>, _>("tag_colors").unwrap_or_default().split(',').map(|s| s.to_string()).collect(),
                }).collect();

                HttpResponse::Ok().json(GetTaskListByTagListResponse { tasks })
            },
            Err(e) => {
                info!("Failed to fetch tasks for group_id {}: {}", group_id, e);
                HttpResponse::InternalServerError().json(GetTaskListByTagListResponse { tasks: Vec::new() })
            }
        }
    }
}


// Handler to get task list by project name
pub async fn get_task_list_by_project_name(
    pool: web::Data<MySqlPool>,
    _: HttpRequest,
    request: web::Json<GetTaskListByProjectNameRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;
    let project_name = &request.project_name;

    if project_name.is_empty() {
        return HttpResponse::BadRequest().json(GetTaskListByProjectNameResponse { tasks: Vec::new() });
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
            return HttpResponse::BadRequest().json(GetTaskListByProjectNameResponse { tasks: Vec::new() });
        }
    };

    // Get projects under the group containing project_name as substring
    let projects_result = sqlx::query!(
        "SELECT project_id FROM Projects_ WHERE group_id = ? AND project_name LIKE ?",
        group_id, format!("%{}%", project_name)
    )
    .fetch_all(pool.get_ref())
    .await;

    let project_ids: Vec<i32> = match projects_result {
        Ok(records) => records.into_iter().map(|record| record.project_id).collect(),
        Err(_) => {
            info!("Projects not found for group: {}", group_name);
            return HttpResponse::BadRequest().json(GetTaskListByProjectNameResponse { tasks: Vec::new() });
        }
    };

    // Dynamically construct the IN clause
    let placeholders = project_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query_str = format!(
        "SELECT t.title AS task_title, u.user_name AS worker_name, t.start_time, t.end_time, t.description, p.project_name, GROUP_CONCAT(ta.tag_color SEPARATOR ',') AS tag_colors
        FROM Tasks_ t
        JOIN Users_ u ON t.worker_user_id = u.user_id
        JOIN Projects_ p ON t.project_id = p.project_id
        LEFT JOIN TagProjectMapping_ tpm ON t.project_id = tpm.project_id
        LEFT JOIN Tags_ ta ON tpm.tag_id = ta.tag_id AND ta.group_id = ?
        WHERE p.project_id IN ({})
        GROUP BY t.task_id", placeholders
    );

    // Execute the query with dynamically provided parameters
    let mut query = sqlx::query(&query_str).bind(group_id);
    for project_id in project_ids {
        query = query.bind(project_id);
    }

    let tasks_result = query.fetch_all(pool.get_ref()).await;
    
    match tasks_result {
        Ok(records) => {
            let tasks: Vec<Task> = records.into_iter().map(|record| Task {
                task_title: record.get("task_title"),
                worker_name: record.get("worker_name"),
                start_time: record.get::<PrimitiveDateTime, _>("start_time").to_string(),
                end_time: record.get::<PrimitiveDateTime, _>("end_time").to_string(),
                description: record.get("description"),
                project_name: record.get("project_name"),
                tag_colors: record.get::<Option<String>, _>("tag_colors").unwrap_or_default().split(',').map(|s| s.to_string()).collect(),
            }).collect();

            HttpResponse::Ok().json(GetTaskListByTagListResponse { tasks })
        },
        Err(e) => {
            error!("Failed to fetch tasks for group_id {}: {}", group_id, e);
            HttpResponse::InternalServerError().json(GetTaskListByTagListResponse { tasks: Vec::new() })
        }
    }
}

pub async fn get_project_list(
    pool: web::Data<MySqlPool>,
    _: HttpRequest,
    request: web::Json<GetProjectListRequest>,
) -> impl Responder {
    let owner_user_name = &request.owner_user_name;
    let group_name = &request.group_name;

    // Get group_id using group_name and owner_user_name from Groups_
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
            return HttpResponse::BadRequest().json(GetProjectListResponse { projects: Vec::new() });
        }
    };

    // Get the project list from Projects_ where group_id is the one from the previous query
    let projects_result = sqlx::query!(
        "SELECT project_id, project_name, project_description 
         FROM Projects_
         WHERE group_id = ?",
        group_id
    )
    .fetch_all(pool.get_ref())
    .await;

    let projects = match projects_result {
        Ok(records) => records,
        Err(e) => {
            error!("Failed to fetch projects for group_id {}: {}", group_id, e);
            return HttpResponse::InternalServerError().json(GetProjectListResponse { projects: Vec::new() });
        }
    };

    // For each project, find all the tag_ids that are mapped with the corresponding project_id
    let mut projects_with_tags = Vec::new();

    for project in projects {
        let tag_ids_result = sqlx::query!(
            "SELECT tag_id 
             FROM TagProjectMapping_
             WHERE project_id = ?",
            project.project_id
        )
        .fetch_all(pool.get_ref())
        .await;

        let tag_ids = match tag_ids_result {
            Ok(records) => records.into_iter().map(|record| record.tag_id).collect::<Vec<_>>(),
            Err(e) => {
                error!("Failed to fetch tag_ids for project_id {}: {}", project.project_id, e);
                Vec::new()
            }
        };

        let tag_colors = if !tag_ids.is_empty() {
            let query_str = format!(
                "SELECT tag_color FROM Tags_ WHERE tag_id IN ({})",
                tag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ")
            );
        
            // Execute the query with dynamically provided parameters
            let mut query = sqlx::query(&query_str);
            for tag_id in tag_ids.clone() {
                query = query.bind(tag_id);
            }
        
            let tag_colors_result = query.fetch_all(pool.get_ref()).await;
        
            match tag_colors_result{
                Ok(records) => records.into_iter().map(|record| record.get(0)).collect::<Vec<_>>(),
                Err(e) => {
                    info!("Failed to fetch tag_colors for tag_ids {:?}: {}", tag_ids, e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        projects_with_tags.push(Project {
            project_name: project.project_name,
            tag_colors,
        });
    }

    HttpResponse::Ok().json(GetProjectListResponse { projects: projects_with_tags })
}
