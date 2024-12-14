use cnctd_server::{
    bad_request, internal_server_error, not_found, unauthorized, success_data, success_msg,
    router::{error::{ErrorCode, ErrorResponse}, response::SuccessResponse, HttpMethod},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::router::rest::Resource;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataIn {
    id: Option<String>, // Changed to String for simplicity (no UUID dependency)
    name: Option<String>,
    order_number: Option<i32>,
    public_flag: Option<bool>,
    token: Option<String>,
}

enum Operation {
    ById,
    Create,
    UpdateName,
    Unrecognized,
}

impl Operation {
    fn from_str(s: &str) -> Self {
        match s {
            "by-id" => Operation::ById,
            "create" => Operation::Create,
            "update-name" => Operation::UpdateName,
            _ => Operation::Unrecognized,
        }
    }

    fn from_option(s: Option<String>) -> Self {
        match s {
            Some(op) => Self::from_str(&op),
            None => Operation::Unrecognized,
        }
    }

    fn requires_auth(&self) -> bool {
        !matches!(self, Operation::Unrecognized)
    }
}

pub async fn route_new_resource(
    method: HttpMethod,
    operation: Option<String>,
    data_val: Value,
    auth_token: Option<String>,
    _client_id: Option<String>, // Client ID is unused in this simplified version
) -> Result<SuccessResponse, ErrorResponse> {
    let operation = Operation::from_option(operation);
    let data: DataIn = serde_json::from_value(data_val.clone()).map_err(|e| bad_request!(e))?;

    if operation.requires_auth() {
        Resource::authenticate(auth_token.clone()).map_err(|e| unauthorized!(e))?;
    }

    match method {
        HttpMethod::GET => match operation {
            Operation::ById => {
                let id = data.id.ok_or_else(|| bad_request!("id required"))?;
                let resource = json!({ "id": id, "name": "Example Resource" }); // Simulated resource
                Ok(success_data!(resource))
            }
            _ => Err(bad_request!("Invalid operation for GET")),
        },
        HttpMethod::POST => match operation {
            Operation::Create => {
                let name = data.name.ok_or_else(|| bad_request!("name required"))?;
                let new_resource = json!({ "id": "12345", "name": name }); // Simulated resource creation
                Ok(success_data!(new_resource))
            }
            _ => Err(bad_request!("Invalid operation for POST")),
        },
        HttpMethod::PUT => match operation {
            Operation::UpdateName => {
                let id = data.id.ok_or_else(|| bad_request!("id required"))?;
                let new_name = data.name.ok_or_else(|| bad_request!("name required"))?;
                let updated_resource = json!({ "id": id, "name": new_name }); // Simulated update
                Ok(success_data!(updated_resource))
            }
            _ => Err(bad_request!("Invalid operation for PUT")),
        },
        HttpMethod::DELETE => match operation {
            Operation::ById => {
                let id = data.id.ok_or_else(|| bad_request!("id required"))?;
                Ok(success_msg!(format!("Resource with id {} deleted", id)))
            }
            _ => Err(bad_request!("Invalid operation for DELETE")),
        },
        _ => Err(bad_request!("Invalid HTTP method")),
    }
}
