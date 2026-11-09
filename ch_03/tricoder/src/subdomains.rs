use crate::{
    model::{CrtShEntry, Subdomain},
    Error,
};
use futures::stream;
use futures::StreamExt;
use reqwest::Client;
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::{GenericConnection, GenericConnectionProvider, TokioRuntime},
    AsyncResolver,
};

type DnsResolver = AsyncResolver<GenericConnection, GenericConnectionProvider<TokioRuntime>>;

/// Base URL of the certificate transparency source. crt.sh is the default but
/// is a free public service with no SLA, so it is overridable to point at a
/// mirror, a local fixture, or a different log.
fn crt_sh_base_url() -> String {
    std::env::var("TRICODER_CRTSH_URL").unwrap_or_else(|_| "https://crt.sh".to_string())
}

pub async fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let url = format!("{}/?q=%25.{}&output=json", crt_sh_base_url(), target);
    let response = http_client.get(&url).send().await?;

    // crt.sh answers with an HTML error page or an empty body when it is down or
    // rate-limiting, which reqwest cannot decode as JSON. Surface that as a
    // readable error instead of an opaque decode failure.
    let status = response.status();
    if !status.is_success() {
        return Err(Error::UpstreamUnavailable(format!(
            "certificate transparency source returned HTTP {} for {url}",
            status.as_u16()
        )));
    }

    let body = response.text().await?;
    if body.trim().is_empty() {
        return Err(Error::UpstreamUnavailable(format!(
            "certificate transparency source returned an empty body for {url}"
        )));
    }

    let entries: Vec<CrtShEntry> = serde_json::from_str(&body).map_err(|e| {
        Error::UpstreamUnavailable(format!(
            "certificate transparency source returned non-JSON for {url}: {e}"
        ))
    })?;

    let mut dns_resolver_opts = ResolverOpts::default();
    dns_resolver_opts.timeout = Duration::from_secs(4);

    let dns_resolver = AsyncResolver::tokio(
        ResolverConfig::default(),
        dns_resolver_opts,
    )
    .expect("subdomain resolver: building DNS client");

    // clean and dedup results
    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .map(|entry| {
            entry
                .name_value
                .split("\n")
                .map(|subdomain| subdomain.trim().to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .filter(|subdomain: &String| subdomain != target)
        .filter(|subdomain: &String| !subdomain.contains("*"))
        .collect();
    subdomains.insert(target.to_string());

    let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
        .map(|domain| Subdomain {
            domain,
            open_ports: Vec::new(),
        })
        .filter_map(|subdomain| {
            let dns_resolver = dns_resolver.clone();
            async move {
                if resolves(&dns_resolver, &subdomain).await {
                    Some(subdomain)
                } else {
                    None
                }
            }
        })
        .collect()
        .await;

    Ok(subdomains)
}

pub async fn resolves(dns_resolver: &DnsResolver, domain: &Subdomain) -> bool {
    dns_resolver.lookup_ip(domain.domain.as_str()).await.is_ok()
}
