use crate::repositories::todo_repository;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug)]
pub struct AppState {
    pub app_name: String,
    pub repository: Arc<dyn todo_repository::TodoRepository>,
}