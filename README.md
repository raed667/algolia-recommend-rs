# Algolia Recommend Rust Client

<p align="center">
  <!-- Stability -->
  <a href="https://crates.io/crates/algolia-recommend-rs"><img alt="Stability: beta" src="https://img.shields.io/badge/stability-beta-f4d03f.svg" /></a>
  <!-- Version -->
  <a href="https://crates.io/crates/algolia-recommend-rs"><img alt="Crates.io" src="https://img.shields.io/crates/v/algolia-recommend-rs"></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/algolia-recommend-rs"><img alt="Crates.io" src="https://img.shields.io/crates/d/algolia-recommend-rs"></a>
  <!-- Tests -->
  <a href="https://github.com/raed667/algolia-recommend-rs/actions/workflows/ci.yml"><img src="https://github.com/raed667/algolia-recommend-rs/actions/workflows/ci.yml/badge.svg" /></>
  <!-- codecov <a href="https://codecov.io/gh/raed667/algolia-recommend-rs"><img src="https://codecov.io/gh/raed667/algolia-recommend-rs/branch/main/graph/badge.svg?token=6IH3LQRXNH"/></a> -->
  <!-- Docs -->
  <a href="https://docs.rs/algolia-recommend-rs"><img src="https://docs.rs/algolia-recommend-rs/badge.svg"/></a>
  <!-- license -->
  <a href="https://crates.io/crates/algolia-recommend-rs"><img alt="Crates.io" src="https://img.shields.io/crates/l/algolia-recommend-rs"></a>
</p>

<p align="center">
    <b>algolia-recommend-rs</b> is an unofficial Rust Client for <a href="https://www.algolia.com/doc/rest-api/recommend/get-recommendations">Algolia Recommend</a>
    <br>
     <a href="https://docs.rs/algolia-recommend-rs/latest/algolia-recommend-rs/"><strong>docs.rs/algolia-recommend-rs</strong></a>
</p>

## 📦 Install

```sh
$ cargo add algolia-recommend-rs
```

## ⚡️ Quick start

```rust
use algolia_recommend_rs::{RecommendClient, models::{RecommendRequest, Model, TrendingFacetsRequest}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Product {
    title: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RecommendClient::new("ALGOLIA_APPLICATION_ID", "ALGOLIA_API_KEY");

    // Build recommendation requests
    let requests = vec![
        RecommendRequest {
            index_name: "products".to_string(),
            model: Model::BoughtTogether,
            object_id: Some("example-object-id".to_string()),
            threshold: Some(0),
            max_recommendations: None,
            facet_name: None,
            query_parameters: None,
        },
        RecommendRequest {
            index_name: "products".to_string(),
            model: Model::TrendingItems,
            object_id: None,
            threshold: Some(0),
            max_recommendations: None,
            facet_name: None,
            query_parameters: None,
        },
    ];

    // Fetch recommendations
    let recs = client.get_recommendations::<Product>(requests).await?;
    println!("results: {}", recs.results.len());

    for result in recs.results.iter() {
        for hit in result.hits.iter() {
            println!("objectID={} score={:?}", hit.object_id, hit.score);
        }
    }

    // Fetch trending facets
    let trending = client
        .get_trending_facets(vec![TrendingFacetsRequest::new("products", "category")])
        .await?;
    println!("trending results: {}", trending.results.len());

    Ok(())
}
```

## 🦀 Notes

- The library is lenient in (de)serialization to stay forward-compatible with Algolia responses.
- Provide `queryParameters` via `RecommendRequest.query_parameters` / `TrendingFacetsRequest.query_parameters` as raw JSON.

## 📜 License

[MIT](LICENSE)
