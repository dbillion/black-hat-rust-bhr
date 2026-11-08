import pathlib
p = pathlib.Path('src/spiders/github.rs'); t = p.read_text()

old = '''    fn start_urls(&self) -> Vec<String> {
        vec!["https://api.github.com/orgs/google/public_members?per_page=100&page=1".to_string()]
    }'''
new = '''    fn start_urls(&self) -> Vec<String> {
        // Overridable so the crawl loop can be exercised against a local fixture.
        // The public GitHub API is rate-limited and its search endpoints are the
        // slowest part of any run, so measuring through it measures the limit,
        // not the crawler.
        let base = std::env::var("CRAWLER_GITHUB_BASE")
            .unwrap_or_else(|_| "https://api.github.com/orgs/google/public_members".to_string());
        vec![format!("{}?per_page=100&page=1", base)]
    }'''
assert old in t, 'start_urls anchor missing'
t = t.replace(old, new, 1)

# guard the unwrap on captures
old2 = '''            let captures = self.page_regex.capture(&url).unwrap();'''
if old2 not in t:
    old2b = '''            let captures = self.page_regex.captures(&url).unwrap();'''
    new2b = '''            // A URL without ?page=N has no captures, and unwrapping it panics.
            // Treat that as "no next page" instead.
            let captures = match self.page_regex.captures(&url) {
                Some(c) => c,
                None => return Ok((items, Vec::new())),
            };'''
    assert old2b in t, 'captures anchor missing'
    t = t.replace(old2b, new2b, 1)
p.write_text(t); print('github.rs patched')
