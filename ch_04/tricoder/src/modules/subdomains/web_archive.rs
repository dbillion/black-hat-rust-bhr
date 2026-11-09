use crate::{
    modules::{Module, SubdomainModule},
    Error,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

pub struct WebArchive {}

impl WebArchive {
    pub fn new() -> Self {
        WebArchive {}
    }
}

impl Module for WebArchive {
    fn name(&self) -> String {
        String::from("subdomains/webarchive")
    }

    fn description(&self) -> String {
        String::from("Use web.archive.org to find subdomains")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WebArchiveResponse(Vec<Vec<String>>);

fn base_url() -> String {
    // A free public service with no SLA. Overridable so the pipeline can be
    // exercised against a local fixture instead of a flaky upstream.
    std::env::var("TRICODER_WEBARCHIVE_URL").unwrap_or_else(|_| "https://web.archive.org".to_string())
}

#[async_trait]
impl SubdomainModule for WebArchive {
    async fn enumerate(&self, domain: &str) -> Result<Vec<String>, Error> {
        let url = format!("{}/cdx/search/cdx?matchType=domain&fl=original&output=json&collapse=urlkey&url={}", base_url(), domain);
        let res = reqwest::get(&url).await?;

        // These are free public services with no SLA; answer failures with a
        // named error that says which URL failed and what it returned, instead
        // of a generic "Invalid HTTP response".
        let status = res.status();
        if !status.is_success() {
            return Err(Error::InvalidHttpResponse {
                module: self.name(),
                detail: format!("{url} returned HTTP {}", status.as_u16()),
            });
        }

        let web_archive_urls: WebArchiveResponse = match res.json().await {
            Ok(info) => info,
            Err(e) => {
                return Err(Error::InvalidHttpResponse {
                    module: self.name(),
                    detail: format!("{url} returned a body that is not valid JSON: {e}"),
                })
            }
        };

        let subdomains: HashSet<String> = web_archive_urls
            .0
            .into_iter()
            .flatten()
            .filter_map(|url| {
                Url::parse(&url)
                    .map_err(|err| {
                        log::error!("{}: error parsing url: {}", self.name(), err);
                        err
                    })
                    .ok()
            })
            .filter_map(|url| url.host_str().map(|host| host.to_string()))
            .collect();

        Ok(subdomains.into_iter().collect())
    }
}
