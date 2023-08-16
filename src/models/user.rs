use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JwkKeys {
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Deserialize)]
pub struct JwksResp {
    pub keys: Vec<JwkKeys>,
}
