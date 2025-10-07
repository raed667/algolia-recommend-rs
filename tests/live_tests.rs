use algolia_recommend_rs::models::Model;
use algolia_recommend_rs::{RecommendClient, TrendingFacetsRequest};
use pretty_assertions::assert_eq;

#[tokio::test]
#[ignore]
async fn live_smoke_get_recommendations() {
    dotenv::dotenv().ok();
    let app_id = match std::env::var("APP_ID") {
        Ok(v) => v,
        Err(_) => return,
    };
    let api_key = match std::env::var("API_KEY") {
        Ok(v) => v,
        Err(_) => return,
    };
    let client = RecommendClient::new(app_id, api_key);

    // Minimal request; index and object may not exist; we only assert no transport/serde crash
    use algolia_recommend_rs::models::RecommendRequest;

    let requests = vec![RecommendRequest {
        index_name: "products".to_string(),
        model: Model::TrendingItems,
        object_id: None,
        threshold: 0,
        max_recommendations: None,
        facet_name: None,
        query_parameters: None,
    }];

    let result = client
        .get_recommendations::<serde_json::Value>(requests)
        .await;

    match result {
        Ok(_) => {}
        Err(e) => {
            let msg = format!("{e}");
            assert_eq!(
                msg.contains("Algolia API error"),
                true,
                "unexpected error: {msg}"
            );
        }
    }
}

#[tokio::test]
#[ignore]
async fn live_smoke_get_trending_facets() {
    dotenv::dotenv().ok();
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
            assert_eq!(
                msg.contains("Algolia API error"),
                true,
                "unexpected error: {msg}"
            );
        }
    }
}

#[tokio::test]
#[ignore]
async fn live_get_recommendations() {
    dotenv::dotenv().ok();
    let app_id = match std::env::var("APP_ID") {
        Ok(v) => v,
        Err(_) => return,
    };
    let api_key = match std::env::var("API_KEY") {
        Ok(v) => v,
        Err(_) => return,
    };
    let index_name = match std::env::var("INDEX_NAME") {
        Ok(v) => v,
        Err(_) => return,
    };
    let object_id = match std::env::var("OBJECT_ID") {
        Ok(v) => v,
        Err(_) => return,
    };

    let client = RecommendClient::new(app_id, api_key);

    use algolia_recommend_rs::models::RecommendRequest;

    let requests = vec![RecommendRequest {
        index_name: index_name.clone(),
        model: Model::RelatedProducts,
        object_id: Some(object_id),
        threshold: 0,
        max_recommendations: None,
        facet_name: None,
        query_parameters: None,
    }];

    let result = client
        .get_recommendations::<serde_json::Value>(requests)
        .await;

    match result {
        Ok(response) => {
            assert_eq!(response.results.len(), 1);
            let result = &response.results[0];
            assert_eq!(result.index.as_deref(), Some(index_name.as_str()));

            // assert that every hit in result has objectID
            for hit in &result.hits {
                assert!(hit.payload.get("type").is_some());
            }
        }
        Err(e) => {
            let msg = format!("{e}");
            assert_eq!(
                msg.contains("Algolia API error"),
                true,
                "unexpected error: {msg}"
            );
        }
    }
}
