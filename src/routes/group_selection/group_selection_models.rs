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