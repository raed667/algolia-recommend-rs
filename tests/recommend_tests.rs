use algolia_recommend_rs::models::Model;
use algolia_recommend_rs::{RecommendClient, TrendingFacetsRequest};
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

#[tokio::test]
async fn test_retry_on_5xx_then_succeed_on_next_host() {
    let primary = MockServer::start();
    let fallback = MockServer::start();

    let _m1 = primary.mock(|when, then| {
        when.method(POST)
            .path("/1/indexes/*/recommendations")
            .header("x-algolia-application-id", "APPID")
            .header("x-algolia-api-key", "KEY");
        then.status(500)
            .header("content-type", "application/json")
            .body(r#"{"message":"server error"}"#);
    });

    let m2 = fallback.mock(|when, then| {
        when.method(POST)
            .path("/1/indexes/*/recommendations")
            .header("x-algolia-application-id", "APPID")
            .header("x-algolia-api-key", "KEY");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"results":[{"hits":[{"objectID":"ok"}]}]}"#);
    });

    let hosts = vec![primary.base_url(), fallback.base_url()];
    let client = RecommendClient::with_hosts("APPID", "KEY", hosts).with_default_object_id("obj-1");

    let resp = client
        .get_recommendations::<Product>("products", vec![Model::TrendingItems])
        .await
        .expect("request ok after retry");

    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].hits.len(), 1);
    assert_eq!(resp.results[0].hits[0].object_id, "ok");
    m2.assert();
}

#[tokio::test]
async fn test_retry_on_network_error_then_succeed() {
    // First host: unroutable/closed port to force a connect error quickly
    let bad_host = String::from("http://127.0.0.1:9");
    let ok_server = MockServer::start();

    let ok = ok_server.mock(|when, then| {
        when.method(POST)
            .path("/1/indexes/*/recommendations")
            .header("x-algolia-application-id", "APPID")
            .header("x-algolia-api-key", "KEY");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"results":[{"hits":[{"objectID":"ok2"}]}]}"#);
    });

    let hosts = vec![bad_host, ok_server.base_url()];
    let client = RecommendClient::with_hosts("APPID", "KEY", hosts).with_default_object_id("obj-1");

    let resp = client
        .get_recommendations::<Product>("products", vec![Model::TrendingItems])
        .await
        .expect("request ok after network retry");

    assert_eq!(resp.results[0].hits[0].object_id, "ok2");
    ok.assert();
}

#[tokio::test]
async fn test_non_retryable_4xx_does_not_try_next_host() {
    let first = MockServer::start();
    let second = MockServer::start();

    let m1 = first.mock(|when, then| {
        when.method(POST)
            .path("/1/indexes/*/recommendations")
            .header("x-algolia-application-id", "APPID")
            .header("x-algolia-api-key", "KEY");
        then.status(400)
            .header("content-type", "application/json")
            .body(r#"{"message":"bad request"}"#);
    });

    let m2 = second.mock(|when, then| {
        when.method(POST).path("/1/indexes/*/recommendations");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"results":[{"hits":[]}]}"#);
    });

    let hosts = vec![first.base_url(), second.base_url()];
    let client = RecommendClient::with_hosts("APPID", "KEY", hosts).with_default_object_id("obj-1");

    let err = client
        .get_recommendations::<Product>("products", vec![Model::TrendingItems])
        .await
        .expect_err("should fail with 400 and not retry");

    let msg = format!("{err}");
    assert!(msg.contains("Algolia API error"));
    assert!(msg.contains("400"));

    // Ensure second host was not called
    assert_eq!(m1.hits(), 1);
    assert_eq!(m2.hits(), 0);
}
