# Algolia Recommend Rust Client

Minimal async Rust client for the Algolia Recommend API (unofficial).

- get_recommendations<T>: fetches recommendations.
- get_trending_facets: fetches trending facet values.

See API reference: [Algolia Recommend API](https://www.algolia.com/doc/rest-api/recommend/get-recommendations).

## Installation

Add to Cargo.toml:

```toml
algolia-recommend = "0.1"
```

## Usage

```rust
use algolia_recommend::{RecommendClient, RecommendRequest, TrendingFacetsRequest};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Product {
    title: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RecommendClient::new("ALGOLIA_APPLICATION_ID", "ALGOLIA_API_KEY");

    // Recommendations with typed hits
    let recs = client
        .get_recommendations::<Product>(
            "products",
            vec![
                Model::BoughtTogether,
                Model::RelatedProducts,
                Model::TrendingItems,
                Model::LookingSimilar,
            ],
        )
        .await?;
    println!("results: {}", recs.results.len());

    for result in recs.results.iter() {
        for hit in result.hits.iter() {
            println!("objectID={} score={:?}", hit.object_id, hit.score);
        }
    }

    // 2) Trending facets
    let trending = client
        .get_trending_facets(vec![TrendingFacetsRequest::new("products", "category")])
        .await?;
    println!("trending results: {}", trending.results.len());

    Ok(())
}
```

## Notes

- The library is lenient in (de)serialization to stay forward-compatible with Algolia responses.
- Provide `queryParameters` via `RecommendRequest.query_parameters` / `TrendingFacetsRequest.query_parameters` as raw JSON.

## License

MIT
