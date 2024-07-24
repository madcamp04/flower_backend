use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetGroupListRequest {
    // if needed
}

#[derive(Serialize)]
pub struct Group {
    pub group_name: String,
    pub writeable: bool,
    pub owner_username: String,
}

// list of groups
#[derive(Serialize)]
pub struct GetGroupListResponse {
    pub groups: Vec<Group>,
}

#[derive(Deserialize)]
pub struct AddGroupRequest {
    // if needed
    pub group_name: String,
}
// list of groups
#[derive(Serialize)]
pub struct AddGroupResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct UpdateGroupRequest {
    pub owner_user_name: String,
    pub group_name: String,
    pub new_group_name: String,
}
// list of groups
#[derive(Serialize)]
pub struct UpdateGroupResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct DeleteGroupRequest {
    pub owner_user_name: String,
    pub group_name: String,
}
// list of groups
#[derive(Serialize)]
pub struct DeleteGroupResponse {
    pub success: bool,
    pub message: String,
}