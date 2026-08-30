use serde_json::Value as JsonValue;

use crate::error::AppError;

use super::models::{CollectionRef, DatabaseRef, DocumentPage};

#[async_trait::async_trait]
pub trait DocumentDriver: Send + Sync {
    async fn server_version(&self) -> Result<String, AppError>;

    async fn list_databases(&self) -> Result<Vec<DatabaseRef>, AppError>;

    async fn list_collections(&self, database: &str) -> Result<Vec<CollectionRef>, AppError>;

    async fn list_documents(
        &self,
        database: &str,
        collection: &str,
        limit: i64,
        skip: i64,
    ) -> Result<DocumentPage, AppError>;

    async fn get_document(
        &self,
        database: &str,
        collection: &str,
        id: &str,
    ) -> Result<Option<JsonValue>, AppError>;

    async fn insert_document(
        &self,
        database: &str,
        collection: &str,
        document: JsonValue,
    ) -> Result<String, AppError>;

    async fn update_document(
        &self,
        database: &str,
        collection: &str,
        id: &str,
        document: JsonValue,
    ) -> Result<(), AppError>;

    async fn delete_document(
        &self,
        database: &str,
        collection: &str,
        id: &str,
    ) -> Result<(), AppError>;
}
