use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub is_admin: bool,
    pub exp: usize,
}

impl JwtClaims {
    pub fn new(id: u32, name: Option<String>, is_admin: bool, exp: usize) -> Self {
        Self {
            id,
            name,
            is_admin,
            exp,
        }
    }
}
