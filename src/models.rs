use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    BoughtTogether,
    RelatedProducts,
    TrendingItems,
    TrendingFacets,
    LookingSimilar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendRequest {
    #[serde(rename = "indexName")]
    pub index_name: String,
    pub model: Model,

    // For models that require an objectID
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "objectID")]
    pub object_id: Option<String>,

    #[serde()]
    pub threshold: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxRecommendations")]
    pub max_recommendations: Option<u32>,

    // Specific to trending facets (not used here but kept for forward-compat)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "facetName")]
    pub facet_name: Option<String>,

    // Arbitrary query parameters passthrough (see docs)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "queryParameters")]
    pub query_parameters: Option<Value>,
}

impl RecommendRequest {
    pub fn bought_together(index_name: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            index_name: index_name.into(),
            model: Model::BoughtTogether,
            object_id: Some(object_id.into()),
            threshold: 0,
            max_recommendations: None,
            facet_name: None,
            query_parameters: None,
        }
    }

    pub fn related_products(index_name: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            index_name: index_name.into(),
            model: Model::RelatedProducts,
            object_id: Some(object_id.into()),
            threshold: 0,
            max_recommendations: None,
            facet_name: None,
            query_parameters: None,
        }
    }

    pub fn trending_items(index_name: impl Into<String>) -> Self {
        Self {
            index_name: index_name.into(),
            model: Model::TrendingItems,
            object_id: None,
            threshold: 0,
            max_recommendations: None,
            facet_name: None,
            query_parameters: None,
        }
    }

    pub fn looking_similar(index_name: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            index_name: index_name.into(),
            model: Model::LookingSimilar,
            object_id: Some(object_id.into()),
            threshold: 0,
            max_recommendations: None,
            facet_name: None,
            query_parameters: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingFacetsRequest {
    pub model: Model,
    #[serde(rename = "indexName")]
    pub index_name: String,
    #[serde(rename = "facetName")]
    pub facet_name: String,

    #[serde()]
    pub threshold: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxRecommendations")]
    pub max_recommendations: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "queryParameters")]
    pub query_parameters: Option<Value>,
}

impl TrendingFacetsRequest {
    pub fn new(index_name: impl Into<String>, facet_name: impl Into<String>) -> Self {
        Self {
            model: Model::TrendingFacets,
            index_name: index_name.into(),
            facet_name: facet_name.into(),
            threshold: 0,
            max_recommendations: None,
            query_parameters: None,
        }
    }
}

// Generic recommendations response (excluding trending-facets)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: serde::de::DeserializeOwned"))]
pub struct RecommendResponse<T> {
    pub results: Vec<RecommendResult<T>>,
}

// A single hit always contains an objectID provided by Algolia and may include a relevance score.
// The remainder of the user-defined payload is flattened into `payload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: serde::de::DeserializeOwned"))]
pub struct Hit<T> {
    #[serde(rename = "objectID")]
    pub object_id: String,
    #[serde(default)]
    #[serde(rename = "_score")]
    pub score: Option<f64>,
    #[serde(flatten)]
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: serde::de::DeserializeOwned"))]
pub struct RecommendResult<T> {
    #[serde(default)]
    pub hits: Vec<Hit<T>>,
    #[serde(default)]
    pub index: Option<String>,
    #[serde(default)]
    #[serde(rename = "nbHits")]
    pub nb_hits: Option<u32>,
    #[serde(default)]
    #[serde(rename = "queryID")]
    pub query_id: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

// Trending facets response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingFacetsResponse {
    pub results: Vec<TrendingFacetsResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingFacetsResult {
    #[serde(default)]
    pub index: Option<String>,
    #[serde(default)]
    pub facet: Option<String>,
    #[serde(default)]
    #[serde(rename = "facetHits")]
    pub facet_hits: Vec<TrendingFacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingFacetValue {
    pub value: String,
    #[serde(default)]
    pub count: u64,
    #[serde(default)]
    pub highlighted: Option<String>,
}
