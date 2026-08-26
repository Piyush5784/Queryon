use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson, Document};
use mongodb::options::FindOptions;
use mongodb::Client;
use serde_json::Value as JsonValue;

use crate::domain::document::{CollectionRef, DatabaseRef, DocumentDriver, DocumentPage, DocumentSummary};
use crate::error::AppError;

use futures_util::TryStreamExt;

const MAX_PAGE_SIZE: i64 = 500;

pub struct MongoDbDriver {
    client: Client,
}

impl MongoDbDriver {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

fn mongo_err(e: mongodb::error::Error) -> AppError {
    AppError::new(crate::error::describe_mongodb_error(&e))
}

fn document_to_json(doc: Document) -> Result<JsonValue, AppError> {
    serde_json::to_value(Bson::Document(doc))
        .map_err(|e| AppError::new(format!("Failed to convert document to JSON: {e}")))
}

fn preview_of(doc: &Document) -> String {
    let mut parts = Vec::new();
    for (key, value) in doc.iter() {
        if key == "_id" {
            continue;
        }
        parts.push(format!("{key}: {}", preview_value(value)));
        if parts.len() >= 4 {
            break;
        }
    }
    if parts.is_empty() {
        "{ }".to_string()
    } else {
        format!("{{ {} }}", parts.join(", "))
    }
}

fn preview_value(value: &Bson) -> String {
    match value {
        Bson::String(s) => {
            if s.len() > 40 {
                format!("\"{}…\"", &s[..40])
            } else {
                format!("\"{s}\"")
            }
        }
        Bson::Document(_) => "{…}".to_string(),
        Bson::Array(_) => "[…]".to_string(),
        other => other.to_string(),
    }
}

fn document_id_string(doc: &Document) -> String {
    match doc.get("_id") {
        Some(Bson::ObjectId(oid)) => oid.to_hex(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn parse_id_filter(id: &str) -> Document {
    match ObjectId::parse_str(id) {
        Ok(oid) => doc! { "_id": oid },
        Err(_) => doc! { "_id": id },
    }
}

#[async_trait::async_trait]
impl DocumentDriver for MongoDbDriver {
    async fn server_version(&self) -> Result<String, AppError> {
        let db = self.client.database("admin");
        let info = db
            .run_command(doc! { "buildInfo": 1 })
            .await
            .map_err(mongo_err)?;
        let version = info
            .get_str("version")
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        Ok(version)
    }

    async fn list_databases(&self) -> Result<Vec<DatabaseRef>, AppError> {
        let names = self.client.list_database_names().await.map_err(mongo_err)?;
        Ok(names.into_iter().map(|name| DatabaseRef { name }).collect())
    }

    async fn list_collections(&self, database: &str) -> Result<Vec<CollectionRef>, AppError> {
        let db = self.client.database(database);
        let names = db.list_collection_names().await.map_err(mongo_err)?;

        let mut refs = Vec::with_capacity(names.len());
        for name in names {
            let coll = db.collection::<Document>(&name);
            let count = coll.estimated_document_count().await.unwrap_or(0);
            refs.push(CollectionRef {
                name,
                estimated_count: count as f64,
            });
        }
        Ok(refs)
    }

    async fn list_documents(
        &self,
        database: &str,
        collection: &str,
        limit: i64,
        skip: i64,
    ) -> Result<DocumentPage, AppError> {
        let start = std::time::Instant::now();
        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        let skip = skip.max(0);

        let coll = self.client.database(database).collection::<Document>(collection);
        let find_options = FindOptions::builder()
            .limit(limit + 1)
            .skip(skip as u64)
            .build();

        let mut cursor = coll
            .find(doc! {})
            .with_options(find_options)
            .await
            .map_err(mongo_err)?;

        let mut documents = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(mongo_err)? {
            documents.push(doc);
        }

        let has_more = documents.len() as i64 > limit;
        documents.truncate(limit as usize);

        let summaries = documents
            .iter()
            .map(|doc| DocumentSummary {
                id: document_id_string(doc),
                preview: preview_of(doc),
            })
            .collect();

        Ok(DocumentPage {
            documents: summaries,
            has_more,
            duration_ms: start.elapsed().as_millis() as u32,
        })
    }

    async fn get_document(
        &self,
        database: &str,
        collection: &str,
        id: &str,
    ) -> Result<Option<JsonValue>, AppError> {
        let coll = self.client.database(database).collection::<Document>(collection);
        let filter = parse_id_filter(id);
        let found = coll.find_one(filter).await.map_err(mongo_err)?;
        match found {
            Some(doc) => Ok(Some(document_to_json(doc)?)),
            None => Ok(None),
        }
    }
}
