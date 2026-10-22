use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateRolePermissionsRequest {
    #[validate(length(max = 200, message = "Quá nhiều quyền"))]
    pub permission_codes: Vec<String>,
}
