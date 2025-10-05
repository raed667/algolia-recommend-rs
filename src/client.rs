use crate::error::{Error, Result};
use crate::models::{
    Model, RecommendRequest, RecommendResponse, TrendingFacetsRequest, TrendingFacetsResponse,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Client, StatusCode};
use serde::Serialize;

const DEFAULT_SCHEME: &str = "https";
const DEFAULT_HOST_SUFFIX: &str = ".algolia.net";
const RECOMMEND_PATH: &str = "/1/indexes/*/recommendations";

#[derive(Clone, Debug)]
pub struct RecommendClient {
    app_id: String,
    api_key: String,
    http: Client,
    base_url: String,
    default_object_id: Option<String>,
}

impl RecommendClient {
    pub fn new(app_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        let app_id_str = app_id.into();
        let base_url = format!("{DEFAULT_SCHEME}://{app_id_str}{DEFAULT_HOST_SUFFIX}");
        Self::with_base_url(app_id_str, api_key, base_url)
    }

    pub fn with_custom_host(
        app_id: impl Into<String>,
        api_key: impl Into<String>,
        host: impl Into<String>,
    ) -> Self {
        let app_id_str = app_id.into();
        let host_str: String = host.into();
        let base_url = format!("{DEFAULT_SCHEME}://{host_str}");
        Self::with_base_url(app_id_str, api_key, base_url)
    }

    pub fn with_base_url(
        app_id: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let http = Client::builder()
            .user_agent("algolia-recommend-rs/0.1")
            .build()
            .expect("failed to build reqwest client");
        Self {
            app_id: app_id.into(),
            api_key: api_key.into(),
            http,
            base_url: base_url.into(),
            default_object_id: None,
        }
    }

    pub fn with_default_object_id(mut self, object_id: impl Into<String>) -> Self {
        self.default_object_id = Some(object_id.into());
        self
    }

    pub fn set_default_object_id(&mut self, object_id: impl Into<String>) {
        self.default_object_id = Some(object_id.into());
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-algolia-application-id",
            HeaderValue::from_str(&self.app_id).unwrap(),
        );
        headers.insert(
            "x-algolia-api-key",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers
    }

    fn endpoint(&self) -> String {
        format!("{base}{path}", base = self.base_url, path = RECOMMEND_PATH)
    }

    async fn post_json<B: Serialize, R: serde::de::DeserializeOwned>(&self, body: &B) -> Result<R> {
        let res = self
            .http
            .post(self.endpoint())
            .headers(self.headers())
            .json(body)
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            let message = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                });
            return Err(Error::Api {
                status: status.as_u16(),
                message,
                body: text,
            });
        }
        let parsed = serde_json::from_str::<R>(&text)?;
        Ok(parsed)
    }

    // Public API
    pub async fn get_recommendations<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        index_name: impl Into<String>,
        models: Vec<Model>,
    ) -> Result<RecommendResponse<T>> {
        let index_name = index_name.into();
        // Build requests internally; forbid trending-facets here
        let mut requests: Vec<RecommendRequest> = Vec::with_capacity(models.len());
        for model in models {
            match model {
                Model::TrendingFacets => {
                    return Err(Error::Api {
                        status: StatusCode::BAD_REQUEST.as_u16(),
                        message: Some(
                            "trending-facets must be requested via get_trending_facets".to_string(),
                        ),
                        body: String::new(),
                    });
                }
                Model::TrendingItems => {
                    requests.push(RecommendRequest {
                        index_name: index_name.clone(),
                        model,
                        object_id: None,
                        threshold: None,
                        max_recommendations: None,
                        facet_name: None,
                        query_parameters: None,
                    });
                }
                Model::BoughtTogether | Model::RelatedProducts | Model::LookingSimilar => {
                    let oid = self.default_object_id.as_ref().ok_or_else(|| Error::Api {
                        status: StatusCode::BAD_REQUEST.as_u16(),
                        message: Some("default objectID not set; call with_default_object_id or set_default_object_id".to_string()),
                        body: String::new(),
                    })?;
                    requests.push(RecommendRequest {
                        index_name: index_name.clone(),
                        model,
                        object_id: Some(oid.clone()),
                        threshold: None,
                        max_recommendations: None,
                        facet_name: None,
                        query_parameters: None,
                    });
                }
            }
        }
        #[derive(Serialize)]
        struct Body<'a> {
            requests: &'a [RecommendRequest],
        }
        let body = Body {
            requests: &requests,
        };
        self.post_json::<_, RecommendResponse<T>>(&body).await
    }

    pub async fn get_trending_facets(
        &self,
        requests: Vec<TrendingFacetsRequest>,
    ) -> Result<TrendingFacetsResponse> {
        if requests
            .iter()
            .any(|r| !matches!(r.model, Model::TrendingFacets))
        {
            return Err(Error::Api {
                status: StatusCode::BAD_REQUEST.as_u16(),
                message: Some("all requests must use model=trending-facets".to_string()),
                body: String::new(),
            });
        }
        #[derive(Serialize)]
        struct Body<'a> {
            requests: &'a [TrendingFacetsRequest],
        }
        let body = Body {
            requests: &requests,
        };
        self.post_json::<_, TrendingFacetsResponse>(&body).await
    }
}
