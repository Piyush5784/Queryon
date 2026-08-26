use queryon_lib::domain::connection::service::open_document_connection;
use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};

fn test_uri() -> Option<String> {
    std::env::var("MONGODB_TEST_URI").ok()
}

fn test_profile(uri: &str) -> ConnectionProfile {
    ConnectionProfile {
        id: "mongodb-test".to_string(),
        name: "MongoDB Test".to_string(),
        engine: Engine::MongoDb,
        host: uri.to_string(),
        port: 0,
        database: "NeonDbClient".to_string(),
        user: String::new(),
        password: String::new(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

#[tokio::test]
async fn connects_lists_databases_and_reports_version() {
    let Some(uri) = test_uri() else {
        eprintln!("skipping: MONGODB_TEST_URI not set");
        return;
    };

    let profile = test_profile(&uri);
    let (driver, server_version, default_database) = open_document_connection(&profile)
        .await
        .expect("should connect to live Atlas cluster");

    assert!(!server_version.is_empty());
    assert_eq!(default_database, "NeonDbClient");

    let databases = driver.list_databases().await.expect("should list databases");
    assert!(!databases.is_empty(), "cluster should have at least one database");
}

#[tokio::test]
async fn lists_collections_and_documents_round_trip() {
    let Some(uri) = test_uri() else {
        eprintln!("skipping: MONGODB_TEST_URI not set");
        return;
    };

    let profile = test_profile(&uri);
    let (driver, _version, _default_db) = open_document_connection(&profile)
        .await
        .expect("should connect");

    let probe_collection = "_queryon_driver_probe_test";

    let mongo_client = mongodb::Client::with_uri_str(&uri).await.expect("raw client");
    let db = mongo_client.database("NeonDbClient");
    let coll = db.collection::<mongodb::bson::Document>(probe_collection);
    coll.insert_one(mongodb::bson::doc! { "kind": "integration-test", "value": 7 })
        .await
        .expect("seed insert");

    let collections = driver
        .list_collections("NeonDbClient")
        .await
        .expect("should list collections");
    assert!(collections.iter().any(|c| c.name == probe_collection));

    let page = driver
        .list_documents("NeonDbClient", probe_collection, 10, 0)
        .await
        .expect("should list documents");
    assert_eq!(page.documents.len(), 1);
    assert!(!page.has_more);

    let doc_id = page.documents[0].id.clone();
    let fetched = driver
        .get_document("NeonDbClient", probe_collection, &doc_id)
        .await
        .expect("should fetch document")
        .expect("document should exist");
    assert_eq!(fetched.get("kind").and_then(|v| v.as_str()), Some("integration-test"));

    db.collection::<mongodb::bson::Document>(probe_collection)
        .drop()
        .await
        .expect("cleanup should succeed");

    let collections_after = driver
        .list_collections("NeonDbClient")
        .await
        .expect("should list collections after cleanup");
    assert!(!collections_after.iter().any(|c| c.name == probe_collection));
}
