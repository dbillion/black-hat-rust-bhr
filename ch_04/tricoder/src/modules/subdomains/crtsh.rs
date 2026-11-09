use crate::{
    modules::{Module, SubdomainModule},
    Error,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct Crtsh {}

impl Crtsh {
    pub fn new() -> Self {
        Crtsh {}
    }
}

impl Module for Crtsh {
    fn name(&self) -> String {
        String::from("subdomains/crtsh")
    }

    fn description(&self) -> String {
        String::from("Use crt.sh/ to find subdomains")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CrtShEntry {
    name_value: String,
}

fn base_url() -> String {
    // A free public service with no SLA. Overridable so the pipeline can be
    // exercised against a local fixture instead of a flaky upstream.
    std::env::var("TRICODER_CRTSH_URL").unwrap_or_else(|_| "https://crt.sh".to_string())
}

#[async_trait]
impl SubdomainModule for Crtsh {
    async fn enumerate(&self, domain: &str) -> Result<Vec<String>, Error> {
        let url = format!("{}/?q=%25.{}&output=json", base_url(), domain);
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

        let crtsh_entries: Vec<CrtShEntry> = match res.json().await {
            Ok(info) => info,
            Err(e) => {
                return Err(Error::InvalidHttpResponse {
                    module: self.name(),
                    detail: format!("{url} returned a body that is not valid JSON: {e}"),
                })
            }
        };

        // clean and dedup results
        let subdomains: HashSet<String> = crtsh_entries
            .into_iter()
            .map(|entry| {
                entry
                    .name_value
                    .split("\n")
                    .map(|subdomain| subdomain.trim().to_string())
                    .collect::<Vec<String>>()
            })
            .flatten()
            .filter(|subdomain: &String| !subdomain.contains("*"))
            .collect();

        Ok(subdomains.into_iter().collect())
    }
}
