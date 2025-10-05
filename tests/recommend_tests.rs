use algolia_recommend::models::Model;
use algolia_recommend::{RecommendClient, TrendingFacetsRequest};
use httpmock::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Product {}

#[tokio::test]
async fn test_get_recommendations_excludes_trending_facets_and_parses_hits() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/1/indexes/*/recommendations")
            .header("x-algolia-application-id", "APPID")
            .header("x-algolia-api-key", "KEY");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
                "results": [
                    { "hits": [ { "objectID": "a" }, { "objectID": "b" } ] }
                ]
            }"#,
            );
    });

    let client = RecommendClient::with_base_url("APPID", "KEY", server.base_url())
        .with_default_object_id("obj-1");

    let resp = client
        .get_recommendations::<Product>(
            "products",
            vec![
                Model::BoughtTogether,
                Model::RelatedProducts,
                Model::TrendingItems,
                Model::LookingSimilar,
            ],
        )
        .await
        .expect("request ok");

    mock.assert();
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].hits.len(), 2);
    assert_eq!(resp.results[0].hits[0].object_id, "a");
}

#[tokio::test]
async fn test_get_trending_facets_parses_facet_hits() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/1/indexes/*/recommendations")
            .header("x-algolia-application-id", "APPID")
            .header("x-algolia-api-key", "KEY");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
                "results": [
                    { "facetHits": [ { "value": "Book", "count": 42 } ] }
                ]
            }"#,
            );
    });

    let client = RecommendClient::with_base_url("APPID", "KEY", server.base_url());

    let resp = client
        .get_trending_facets(vec![TrendingFacetsRequest::new("products", "category")])
        .await
        .expect("request ok");

    mock.assert();
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].facet_hits.len(), 1);
    assert_eq!(resp.results[0].facet_hits[0].value, "Book");
    assert_eq!(resp.results[0].facet_hits[0].count, 42);
}

#[tokio::test]
async fn test_non_2xx_api_error_is_mapped() {
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(POST).path("/1/indexes/*/recommendations");
        then.status(403)
            .header("content-type", "application/json")
            .body(r#"{"message":"invalid api key"}"#);
    });

    let client = RecommendClient::with_base_url("APPID", "KEY", server.base_url());

    let err = client
        .get_recommendations::<Product>("products", vec![Model::TrendingItems])
        .await
        .expect_err("should error");

    let msg = format!("{err}");
    assert!(msg.contains("Algolia API error"));
    assert!(msg.contains("403"));
}

#[tokio::test]
async fn test_malformed_json_yields_serde_error() {
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(POST).path("/1/indexes/*/recommendations");
        then.status(200)
            .header("content-type", "application/json")
            .body("not-json");
    });

    let client = RecommendClient::with_base_url("APPID", "KEY", server.base_url());

    let err = client
        .get_recommendations::<Product>("products", vec![Model::TrendingItems])
        .await
        .expect_err("should error");

    let msg = format!("{err}");
    assert!(msg.contains("serde error"));
}
