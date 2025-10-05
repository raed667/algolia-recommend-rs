use crate::error::{Error, Result};
use crate::models::{
    Model, RecommendRequest, RecommendResponse, TrendingFacetsRequest, TrendingFacetsResponse,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Client, StatusCode};
use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const DEFAULT_SCHEME: &str = "https";
const RECOMMEND_PATH: &str = "/1/indexes/*/recommendations";
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug)]
pub struct RecommendClient {
    app_id: String,
    api_key: String,
    http: Client,
    base_url: String,
    hosts: Vec<String>,
    host_cursor: Arc<AtomicUsize>,
    default_object_id: Option<String>,
}

impl RecommendClient {
    pub fn new(app_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        let app_id_str = app_id.into();
        let hosts = get_default_hosts(&app_id_str);
        Self::with_hosts(app_id_str, api_key, hosts)
    }

    pub fn with_custom_host(
        app_id: impl Into<String>,
        api_key: impl Into<String>,
        host: impl Into<String>,
    ) -> Self {
        let app_id_str = app_id.into();
        let host_str: String = host.into();
        let base_url = format!("{DEFAULT_SCHEME}://{host_str}");
        Self::with_hosts(app_id_str, api_key, vec![base_url])
    }

    pub fn with_base_url(
        app_id: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("failed to build reqwest client");
        let base = base_url.into();
        Self {
            app_id: app_id.into(),
            api_key: api_key.into(),
            http,
            base_url: base.clone(),
            hosts: vec![base],
            host_cursor: Arc::new(AtomicUsize::new(0)),
            default_object_id: None,
        }
    }

    pub fn with_hosts(
        app_id: impl Into<String>,
        api_key: impl Into<String>,
        hosts: Vec<String>,
    ) -> Self {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("failed to build reqwest client");
        let base_url = hosts
            .first()
            .cloned()
            .unwrap_or_else(|| String::from("https://"));
        Self {
            app_id: app_id.into(),
            api_key: api_key.into(),
            http,
            base_url,
            hosts,
            host_cursor: Arc::new(AtomicUsize::new(0)),
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

    async fn post_json<B: Serialize, R: serde::de::DeserializeOwned>(&self, body: &B) -> Result<R> {
        // Start from a rotating cursor to distribute load across hosts
        let total_hosts = self.hosts.len();
        let start = if total_hosts == 0 {
            0
        } else {
            self.host_cursor.fetch_add(1, Ordering::Relaxed) % total_hosts
        };

        let mut last_error: Option<Error> = None;
        for attempt in 0..std::cmp::max(1, total_hosts) {
            let idx = (start + attempt) % std::cmp::max(1, total_hosts);
            let base = self
                .hosts
                .get(idx)
                .cloned()
                .unwrap_or_else(|| self.base_url.clone());
            let url = format!("{base}{RECOMMEND_PATH}");

            let req = self.http.post(&url).headers(self.headers()).json(body);

            match req.send().await {
                Ok(res) => {
                    let status = res.status();
                    let text = res.text().await?;
                    if status.is_success() {
                        let parsed = serde_json::from_str::<R>(&text)?;
                        return Ok(parsed);
                    }

                    // Retry on 5xx and 429 by moving to next host
                    if status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(Error::Api {
                            status: status.as_u16(),
                            message: serde_json::from_str::<serde_json::Value>(&text)
                                .ok()
                                .and_then(|v| {
                                    v.get("message")
                                        .and_then(|m| m.as_str())
                                        .map(|s| s.to_string())
                                }),
                            body: text,
                        });
                        continue;
                    } else {
                        return Err(Error::Api {
                            status: status.as_u16(),
                            message: serde_json::from_str::<serde_json::Value>(&text)
                                .ok()
                                .and_then(|v| {
                                    v.get("message")
                                        .and_then(|m| m.as_str())
                                        .map(|s| s.to_string())
                                }),
                            body: text,
                        });
                    }
                }
                Err(e) => {
                    // Retry on network/connect/timeout errors
                    if e.is_connect() || e.is_timeout() || e.is_request() {
                        last_error = Some(Error::Http(e));
                        continue;
                    } else {
                        return Err(Error::Http(e));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Api {
            status: 0,
            message: Some("all hosts failed".to_string()),
            body: String::new(),
        }))
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

fn get_default_hosts(app_id: &str) -> Vec<String> {
    // https://github.com/algolia/algoliasearch-client-javascript/blob/main/packages/recommend/src/recommendClient.ts
    // https://github.com/algolia/algoliasearch-client-javascript/blob/main/packages/client-common/src/transporter/createTransporter.ts
    vec![
        format!("https://{app_id}-dsn.algolia.net"),
        format!("https://{app_id}.algolia.net"),
        format!("https://{app_id}-1.algolianet.com"),
        format!("https://{app_id}-2.algolianet.com"),
        format!("https://{app_id}-3.algolianet.com"),
    ]
}
