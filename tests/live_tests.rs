use algolia_recommend::models::Model;
use algolia_recommend::{RecommendClient, TrendingFacetsRequest};

#[tokio::test]
#[ignore]
async fn live_smoke_get_recommendations() {
    let app_id = match std::env::var("APP_ID") {
        Ok(v) => v,
        Err(_) => return,
    };
    let api_key = match std::env::var("API_KEY") {
        Ok(v) => v,
        Err(_) => return,
    };
    let client = RecommendClient::new(app_id, api_key).with_default_object_id("test-record-123");

    // Minimal request; index and object may not exist; we only assert no transport/serde crash
    let result = client
        .get_recommendations::<serde_json::Value>("products", vec![Model::TrendingItems])
        .await;

    match result {
        Ok(_) => {}
        Err(e) => {
            let msg = format!("{e}");
            assert!(msg.contains("Algolia API error"), "unexpected error: {msg}");
        }
    }
}

#[tokio::test]
#[ignore]
async fn live_smoke_get_trending_facets() {
    let app_id = match std::env::var("APP_ID") {
        Ok(v) => v,
        Err(_) => return,
    };
    let api_key = match std::env::var("API_KEY") {
        Ok(v) => v,
        Err(_) => return,
    };
    let client = RecommendClient::new(app_id, api_key);

    let result = client
        .get_trending_facets(vec![TrendingFacetsRequest::new("products", "category")])
        .await;

    match result {
        Ok(_) => {}
        Err(e) => {
            let msg = format!("{e}");
            assert!(msg.contains("Algolia API error"), "unexpected error: {msg}");
        }
    }
}
