use crate::{
    model::{CrtShEntry, Subdomain},
    Error,
};
use reqwest::blocking::Client;
use std::{
    env,collections::HashSet, time::Duration};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};

/// Base URL of the certificate transparency source used for subdomain
/// enumeration. crt.sh is the default, but it is a free public service with no
/// SLA, so it is overridable to point at a mirror, a local fixture, or a
/// different transparency log.
fn crt_sh_base_url() -> String {
    env::var("TRICODER_CRTSH_URL")
        .unwrap_or_else(|_| "https://crt.sh".to_string())
}

pub fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let url = format!("{}/?q=%25.{}&output=json", crt_sh_base_url(), target);
    let response = http_client.get(&url).send()?;

    // crt.sh answers with an HTML error page or an empty body when it is down
    // or rate-limiting, which reqwest cannot decode as JSON. Surface that as a
    // readable error instead of an opaque JSON decode failure.
    let status = response.status();
    if !status.is_success() {
        return Err(Error::UpstreamUnavailable(format!(
            "certificate transparency source returned HTTP {} for {url}",
            status.as_u16()
        )));
    }

    let body = response.text()?;
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

    // clean and dedup results
    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .flat_map(|entry| {
            entry
                .name_value
                .split('\n')
                .map(|subdomain| subdomain.trim().to_string())
                .collect::<Vec<String>>()
        })
        .filter(|subdomain: &String| subdomain != target)
        .filter(|subdomain: &String| !subdomain.contains('*'))
        .collect();
    subdomains.insert(target.to_string());

    let subdomains: Vec<Subdomain> = subdomains
        .into_iter()
        .map(|domain| Subdomain {
            domain,
            open_ports: Vec::new(),
        })
        .filter(resolves)
        .collect();

    Ok(subdomains)
}

pub fn resolves(domain: &Subdomain) -> bool {
    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_secs(4);

    let dns_resolver = Resolver::new(
        ResolverConfig::default(),
        opts,
    )
    .expect("subdomain resolver: building DNS client");
    dns_resolver.lookup_ip(domain.domain.as_str()).is_ok()
}
