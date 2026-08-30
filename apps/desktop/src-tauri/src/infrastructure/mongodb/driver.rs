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

fn json_to_document(value: JsonValue) -> Result<Document, AppError> {
    let bson = Bson::try_from(value)
        .map_err(|e| AppError::new(format!("Failed to convert JSON to document: {e}")))?;
    match bson {
        Bson::Document(doc) => Ok(doc),
        _ => Err(AppError::new("Document must be a JSON object.")),
    }
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

fn bson_as_f64(doc: &Document, key: &str) -> f64 {
    match doc.get(key) {
        Some(Bson::Double(v)) => *v,
        Some(Bson::Int32(v)) => *v as f64,
        Some(Bson::Int64(v)) => *v as f64,
        _ => 0.0,
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

            let stats = db.run_command(doc! { "collStats": &name }).await.ok();
            let (storage_size, avg_obj_size, index_count, total_index_size) = stats
                .map(|s| {
                    (
                        bson_as_f64(&s, "storageSize"),
                        bson_as_f64(&s, "avgObjSize"),
                        bson_as_f64(&s, "nindexes") as u32,
                        bson_as_f64(&s, "totalIndexSize"),
                    )
                })
                .unwrap_or((0.0, 0.0, 0, 0.0));

            refs.push(CollectionRef {
                name,
                estimated_count: count as f64,
                storage_size_bytes: storage_size,
                avg_document_size_bytes: avg_obj_size,
                index_count,
                total_index_size_bytes: total_index_size,
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

    async fn insert_document(
        &self,
        database: &str,
        collection: &str,
        document: JsonValue,
    ) -> Result<String, AppError> {
        let coll = self.client.database(database).collection::<Document>(collection);
        let doc = json_to_document(document)?;
        let result = coll.insert_one(doc).await.map_err(mongo_err)?;
        match result.inserted_id {
            Bson::ObjectId(oid) => Ok(oid.to_hex()),
            other => Ok(other.to_string()),
        }
    }

    async fn update_document(
        &self,
        database: &str,
        collection: &str,
        id: &str,
        document: JsonValue,
    ) -> Result<(), AppError> {
        let coll = self.client.database(database).collection::<Document>(collection);
        let filter = parse_id_filter(id);
        let mut doc = json_to_document(document)?;
        doc.remove("_id");

        let result = coll
            .replace_one(filter, doc)
            .await
            .map_err(mongo_err)?;
        if result.matched_count == 0 {
            return Err(AppError::new("Document not found — it may have been deleted."));
        }
        Ok(())
    }

    async fn delete_document(
        &self,
        database: &str,
        collection: &str,
        id: &str,
    ) -> Result<(), AppError> {
        let coll = self.client.database(database).collection::<Document>(collection);
        let filter = parse_id_filter(id);
        let result = coll.delete_one(filter).await.map_err(mongo_err)?;
        if result.deleted_count == 0 {
            return Err(AppError::new("Document not found — it may have already been deleted."));
        }
        Ok(())
    }
}
