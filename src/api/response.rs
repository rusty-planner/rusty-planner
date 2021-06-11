use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SimpleResponse {
    pub code: u16,
    pub message: String,
}
