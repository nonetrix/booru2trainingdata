//! Clients for the supported booru APIs.
//!
//! Every service is looked up by post id and reduced to a [`Post`]. Services
//! report "no usable post" by printing a reason and returning `Ok(None)`, so the
//! CLI can keep asking for ids instead of aborting.

use std::time::Duration;

use anyhow::{Context, Result};
use html_escape::decode_html_entities;
use reqwest::Url;
use reqwest::blocking::{Client, Response};
use serde_json::Value;

use crate::post::Post;

/// Sent on every request; both services ask for a descriptive user agent.
pub const USER_AGENT: &str = "Booru2Training data a0.3";

/// Most anime models are trained on this one.
pub const DANBOORU_URL: &str = "https://danbooru.donmai.us";
/// Current Gelbooru API; needs an API key and user id.
pub const GELBOORU_URL: &str = "https://gelbooru.com/";
/// rule34.xxx and friends still run the unauthenticated older API.
pub const GELBOORU_OLD_URL: &str = "https://rule34.xxx/";

/// A service that can resolve a post id.
pub trait Booru {
    /// Service name as shown in the id prompt.
    fn label(&self) -> &'static str;

    /// Look up `id`, returning `Ok(None)` when the service has no usable post.
    fn fetch(&self, id: &str) -> Result<Option<Post>>;
}

/// Shared HTTP plumbing: one connection pool and one user agent for the session.
#[derive(Clone)]
pub struct Http {
    client: Client,
}

impl Http {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .build()
            .context("building the HTTP client")?;
        Ok(Self { client })
    }

    /// GET `url`, failing only on transport errors.
    pub fn get(&self, url: Url) -> Result<Response> {
        self.client
            .get(url.clone())
            .send()
            .with_context(|| format!("GET {url}"))
    }
}

/// Danbooru's `/posts/:id.json` endpoint.
pub struct Danbooru {
    http: Http,
    base: Url,
}

impl Danbooru {
    pub fn new(base: &str, http: &Http) -> Result<Self> {
        Ok(Self {
            http: http.clone(),
            base: parse_base(base)?,
        })
    }

    fn parse(json: &Value) -> Option<Post> {
        let id = json.get("id").and_then(scalar)?;
        // `file_url` is null for posts that cannot be served without an account.
        let file_url = json.get("file_url").and_then(Value::as_str)?;
        let file_ext = json
            .get("file_ext")
            .and_then(Value::as_str)
            .map_or_else(|| Post::ext_from_url(file_url), str::to_owned);

        Some(Post {
            id,
            file_url: file_url.to_owned(),
            file_ext,
            tags: text(json.get("tag_string")),
        })
    }
}

impl Booru for Danbooru {
    fn label(&self) -> &'static str {
        "Danbooru"
    }

    fn fetch(&self, id: &str) -> Result<Option<Post>> {
        let url = join(&self.base, &format!("posts/{id}.json"))?;
        let response = self.http.get(url)?;
        if !response.status().is_success() {
            println!("Request failed.");
            return Ok(None);
        }

        let json: Value = response.json().context("parsing the Danbooru reply")?;
        Ok(Self::parse(&json))
    }
}

/// Gelbooru credentials; the current API rejects requests without them.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub api_key: String,
    pub user_id: String,
}

/// `index.php?page=dapi&s=post&q=index`, as served by Gelbooru and its clones.
///
/// The API shape differs by deployment: Gelbooru wraps posts in `{"post": [...]}`
/// and wants credentials, while rule34.xxx returns a bare array and wants none.
/// One client covers both; `auth` decides whether credentials are sent.
pub struct Gelbooru {
    http: Http,
    base: Url,
    auth: Option<Credentials>,
}

impl Gelbooru {
    pub fn new(base: &str, auth: Option<Credentials>, http: &Http) -> Result<Self> {
        Ok(Self {
            http: http.clone(),
            base: parse_base(base)?,
            auth,
        })
    }

    fn request_url(&self, id: &str) -> Result<Url> {
        let mut url = join(&self.base, "index.php")?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("page", "dapi");
            query.append_pair("s", "post");
            query.append_pair("q", "index");
            query.append_pair("json", "1");
            query.append_pair("id", id);
            if let Some(auth) = &self.auth {
                query.append_pair("api_key", &auth.api_key);
                query.append_pair("user_id", &auth.user_id);
            }
        }
        Ok(url)
    }

    fn parse(json: &Value) -> Option<Post> {
        let post = first_post(json)?;
        let id = post.get("id").and_then(scalar)?;
        let file_url = post.get("file_url").and_then(Value::as_str)?;
        let tags = post
            .get("tags")
            .and_then(Value::as_str)
            .or_else(|| post.get("tag_string").and_then(Value::as_str))
            .unwrap_or_default();
        // Gelbooru HTML-escapes tag text, so `girls'_frontline` arrives as
        // `girls&#039;_frontline` and would otherwise end up in the caption.
        let tags = decode_html_entities(tags).into_owned();

        Some(Post {
            id,
            file_url: file_url.to_owned(),
            file_ext: Post::ext_from_url(file_url),
            tags,
        })
    }
}

impl Booru for Gelbooru {
    fn label(&self) -> &'static str {
        "Gelbooru"
    }

    fn fetch(&self, id: &str) -> Result<Option<Post>> {
        let response = self.http.get(self.request_url(id)?)?;
        if !response.status().is_success() {
            println!(
                "Request failed. Status Code: {}",
                response.status().as_u16()
            );
            return Ok(None);
        }

        let json: Value = response.json().context("parsing the Gelbooru reply")?;
        let post = Self::parse(&json);
        if post.is_none() {
            println!("Post not found or API structure error.");
        }
        Ok(post)
    }
}

fn parse_base(base: &str) -> Result<Url> {
    let trimmed = base.trim().trim_end_matches('/');
    Url::parse(trimmed).with_context(|| format!("invalid service URL {trimmed:?}"))
}

/// Append `path` to the base URL, keeping any path the base already has.
fn join(base: &Url, path: &str) -> Result<Url> {
    let mut url = base.clone();
    let root = url.path().trim_end_matches('/').to_owned();
    url.set_path(&format!("{root}/{path}"));
    Ok(url)
}

/// The first post of a Gelbooru reply, in either API shape.
fn first_post(json: &Value) -> Option<&Value> {
    match json {
        // Legacy API: a bare array of posts.
        Value::Array(posts) => posts.first(),
        // Current API: {"post": [ ... ]} (a single object for some clones).
        Value::Object(_) => match json.get("post")? {
            Value::Array(posts) => posts.first(),
            Value::Object(_) => json.get("post"),
            _ => None,
        },
        _ => None,
    }
}

/// A JSON scalar as text: ids are numbers on Danbooru and strings on Gelbooru.
fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

/// A JSON string field, or an empty tag list when the service omits it.
fn text(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().to_owned()
}
