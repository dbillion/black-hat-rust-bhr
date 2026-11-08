use crate::error::Error;
use async_trait::async_trait;
use fantoccini::{Client, ClientBuilder};
use select::{
    document::Document,
    predicate::{Class, Name, Predicate},
};
use tokio::sync::Mutex;

pub struct QuotesSpider {
    webdriver_client: Mutex<Client>,
}

/// WebDriver endpoint. Overridable so the browser-driven spider can be pointed
/// at a local chromedriver container instead of a host process.
fn webdriver_url() -> String {
    std::env::var("CRAWLER_WEBDRIVER_URL").unwrap_or_else(|_| "http://localhost:4444".to_string())
}

/// Base URL of the target site. The book hardcodes quotes.toscrape.com, which
/// cannot be crawled locally; this mirrors it in Docker so the JS-rendered path
/// can be exercised against a target I control.
fn site_base() -> String {
    std::env::var("CRAWLER_QUOTES_BASE").unwrap_or_else(|_| "https://quotes.toscrape.com".to_string())
}

impl QuotesSpider {
    pub async fn new() -> Result<Self, Error> {
        let mut caps = serde_json::map::Map::new();
        let chrome_opts = serde_json::json!({ "args": ["--headless", "--disable-gpu"] });
        caps.insert("goog:chromeOptions".to_string(), chrome_opts);
        let webdriver_client = ClientBuilder::rustls()
            .capabilities(caps)
            .connect(&webdriver_url())
            .await?;

        Ok(QuotesSpider {
            webdriver_client: Mutex::new(webdriver_client),
        })
    }
}

#[derive(Debug, Clone)]
pub struct QuotesItem {
    quote: String,
    author: String,
}

#[async_trait]
impl super::Spider for QuotesSpider {
    type Item = QuotesItem;

    fn name(&self) -> String {
        String::from("quotes")
    }

    fn start_urls(&self) -> Vec<String> {
        vec![format!("{}/js", site_base())]
    }

    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        let mut items = Vec::new();
        let html = {
            let webdriver = self.webdriver_client.lock().await;
            webdriver.goto(&url).await?;
            webdriver.source().await?
        };

        let document = Document::from(html.as_str());

        let quotes = document.select(Class("quote"));
        for quote in quotes {
            let mut spans = quote.select(Name("span"));
            let quote_span = spans.next().unwrap();
            let quote_str = quote_span.text().trim().to_string();

            let author = spans
                .next()
                .unwrap()
                .select(Class("author"))
                .next()
                .unwrap()
                .text()
                .trim()
                .to_string();

            items.push(QuotesItem {
                quote: quote_str,
                author,
            });
        }

        let next_pages_link = document
            .select(
                Class("pager")
                    .descendant(Class("next"))
                    .descendant(Name("a")),
            )
            .filter_map(|n| n.attr("href"))
            .map(|url| self.normalize_url(url))
            .collect::<Vec<String>>();

        Ok((items, next_pages_link))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        println!("{}", item.quote);
        println!("by {}\n", item.author);
        Ok(())
    }
}

impl QuotesSpider {
    fn normalize_url(&self, url: &str) -> String {
        let url = url.trim();

        if url.starts_with("/") {
            return format!("{}{}", site_base(), url);
        }

        return url.to_string();
    }
}
