//! This crate is used to query the [Reddit API](https://www.reddit.com/dev/api).
//!
//! First, create a [`Config`](struct.Config.html) struct. Then, use it to create an
//! [`Api`](struct.Api.html) struct, which exposes several methods for querying
//! the API.
//!
//! # Example
//!
//! ```
//! use redbot::{Api, Config, Value};
//!
//! fn main() {
//!     let config = Config::load_config("config.json").expect("Could not load confiog");
//!     let mut api = Api::new(config);
//!     api.do_login().expect("Could not perform login");
//!
//!     let mut resp = match api.query("GET", "api/v1/me/karma", None, None) {
//!         Ok(resp) => resp,
//!         Err(err) => panic!(err),
//!     };
//!     let karma_breakdown: Value = match resp.json() {
//!         Ok(data) => data,
//!         Err(err) => panic!(err),
//!     };
//!
//!     println!("{:?}", karma_breakdown);
//! }
//! ```

use log::debug;

pub use reqwest::Method;
use reqwest::{
    self,
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
};
use serde::Deserialize;
pub use serde_json::Value;
use std::collections::HashMap;
use std::{fs::File, io::prelude::*};

pub mod query_listing;
pub use query_listing::QueryListingRequest;

pub mod errors;
pub use errors::ApiError;
pub mod models;
pub use models::subreddit::Subreddit;

const RATE_LIMIT_HEADER_NAMES: [&str; 3] = [
    "X-Ratelimit-Used",
    "X-Ratelimit-Remaining",
    "X-Ratelimit-Reset",
];

/// Program configuration - contains the required values
/// to communicate with the Reddit OAuth API for a token.
///
/// The `username` and `password` fields are the same login
/// strings that you'd use to log into the account on the
/// Reddit website. The `user_agent` field is for setting
/// the 'User Agent' header value to use when communicating
/// with the API, as per the API usage requirements found
/// [here](https://github.com/reddit-archive/reddit/wiki/API#rules).
/// The `client_id` and `client_secret` fields are for a
/// 'script' type application that you create on the Reddit
/// website, [here](https://www.reddit.com/prefs/apps/).
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Config {
    /// Account username
    pub username: String,
    /// Account password
    pub password: String,
    /// User agent to use
    pub user_agent: String,
    /// App client id
    pub client_id: String,
    /// App client secret
    pub client_secret: String,
}

impl Config {
    /// Attempt to load the configuration from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - relative path to the file
    ///
    /// # Examples
    ///
    /// A file called 'config.json' is has the content:
    ///
    /// ```json
    /// {
    ///   "username": "my-bot-account",
    ///   "password": "hunter2",
    ///   "user_agent": "linux:reddit-rust:v.0.0.1 (bot by /u/my-main-account)",
    ///   "client_id": "foo",
    ///   "client_secret": "bar"
    /// }
    /// ```
    ///
    /// Retrieve the config with:
    ///
    /// ```
    /// let config = Config::load_config("config.json")?;
    /// ```
    pub fn load_config(path: &str) -> Result<Self, ApiError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let c = serde_json::from_str::<Config>(&contents)?;
        Ok(c)
    }
}

/// Reddit API access. This is the struct that you'll be using to
/// interact with the API.
pub struct Api {
    config: Config,
    client: reqwest::Client,
    access_token: Option<AccessTokenResponse>,
    /// The account's whoami info
    pub whoami: Option<Value>,
}

impl Api {
    /// Create a new API client.
    ///
    /// # Arguments
    ///
    /// * `config` - the configuration
    ///
    /// # Examples
    ///
    /// ```
    /// let config = Config::load_config().expect("Could not load config");
    /// let mut api = Api::new(config);
    /// ```
    pub fn new(config: Config) -> Self {
        debug!("New API object created");
        Api {
            config,
            client: reqwest::Client::new(),
            access_token: None,
            whoami: None,
        }
    }

    /// Uses the values from the config to get an access token
    /// from the OAuth endpoint, and stores it in the struct.
    ///
    /// This method should be called after creating the struct,
    /// and before attempting to query any inforamtion from the API.
    ///
    /// # Examples
    ///
    /// ```
    /// if let Err(err) = api.do_login() {
    ///     panic!("Could not get an access token: {}", err);
    /// }
    /// ```
    pub fn do_login(&mut self) -> Result<(), ApiError> {
        // urls
        #[cfg(not(test))]
        let url = "https://www.reddit.com";
        #[cfg(test)]
        let url = &mockito::server_url();

        debug!("Performing login");
        let mut form = HashMap::new();
        form.insert("grant_type", "password");
        form.insert("username", &self.config.username);
        form.insert("password", &self.config.password);
        let mut resp = self
            .client
            .post(&format!("{}/api/v1/access_token", url))
            .header("User-Agent", self.config.user_agent.clone())
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&form)
            .send()?;
        debug!("Login response code = {}", resp.status().as_str());
        let data = resp.json::<AccessTokenResponse>()?;
        debug!("Access token is {}", data.token);
        self.access_token = Some(data);
        let whoami = self.get_whoami()?;
        debug!("Returned whoami is {:?}", whoami);
        self.whoami = Some(whoami);
        Ok(())
    }

    /// Returns the account's username from the 'api/v1/me' endpoint.
    fn get_whoami(&self) -> Result<Value, ApiError> {
        let mut resp = self.query("GET", "api/v1/me", None, None)?;
        let data: Value = resp.json()?;
        Ok(data)
    }

    /// Returns the username from the stored whoami data.
    fn get_username(&self) -> Option<String> {
        Some(self.whoami.as_ref()?["name"].as_str().unwrap().to_owned())
    }

    /// Generate headers for the request.
    /// Always includes the User Agent header, and includes
    /// the OAuth token if available.
    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.config.user_agent).unwrap(),
        );
        if self.access_token.is_some() {
            let auth_header = HeaderValue::from_str(&format!(
                "bearer {}",
                self.access_token.as_ref().unwrap().token
            ))
            .unwrap();
            headers.insert(AUTHORIZATION, auth_header);
        }
        headers
    }

    /// Macros and replacements for the URL path and the
    /// appending to the root OAuth API URL.
    fn reformat_path(&self, path: &str) -> String {
        // urls
        #[cfg(not(test))]
        let url = "https://oauth.reddit.com";
        #[cfg(test)]
        let url = &mockito::server_url();

        let path = if path.contains("{username}") {
            debug!("Replacing 'username' macro");
            path.replace("{username}", &self.get_username().unwrap())
        } else {
            path.to_owned()
        };
        format!("{}/{}", url, path)
    }

    /// Processing of the response headers.
    fn process_response_headers(&self, headers: &HeaderMap) {
        for header_name in &RATE_LIMIT_HEADER_NAMES {
            if let Some(value) = headers.get(*header_name) {
                debug!(">> Header {}: {}", header_name, value.to_str().unwrap());
            }
        }
    }

    /// Query the Reddit API.
    ///
    /// # Arguments
    ///
    /// * `method` - A string representing an HTTP method, capable of being parsed by
    /// [reqwest](https://docs.rs/reqwest/latest/reqwest/struct.Method.html#method.from_bytes), i.e. "GET", "POST", etc.
    /// * `path` - A relative URL path (everything after reddit.com/)
    /// * `query` - An optional collection of query parameters
    /// * `form_data` - An optional collection of form data to submit
    ///
    /// # Examples
    ///
    /// Get data from an endpoint:
    /// ```
    /// match api.query("GET", "some/endpoint", None, None) {
    ///     Ok(data) => println!("{}", data),
    ///     Err(err) => panic!(err),
    /// };
    /// ```
    ///
    /// Post data to an endpoint:
    ///
    /// ```
    /// let mut post_data = HashMap::new();
    /// post_data.insert("id", "t3_aaaaaa");
    /// match api.query("POST", "api/save", None, Some(post_data)) {
    ///     Ok(data) => println!("{}", data),
    ///     Err(err) => panic!(err),
    /// }
    /// ```
    pub fn query(
        &self,
        method: &str,
        path: &str,
        query: Option<Vec<(&str, &str)>>,
        form_data: Option<HashMap<&str, &str>>,
    ) -> Result<reqwest::Response, ApiError> {
        let method = Method::from_bytes(method.as_bytes()).unwrap();
        let path = self.reformat_path(path);
        let req = self
            .client
            .request(method, &path)
            .headers(self.get_headers());
        let req = match query {
            Some(q) => req.query(&q),
            None => req,
        };
        debug!("{:?}", req);
        let resp = match form_data {
            Some(fd) => req.form(&fd).send()?,
            None => req.send()?,
        };
        self.process_response_headers(&resp.headers());
        Ok(resp)
    }

    /// Query the Reddit API via a listing endpoint.
    ///
    /// Information on listing endpoints can be found in the
    /// [offial docs](https://www.reddit.com/dev/api#listings).
    ///
    /// # Arguments
    ///
    /// * `ql` - A [`QueryListingRequest`](query_listing/struct.QueryListingRequest.html) struct
    ///
    /// # Examples
    ///
    /// ```
    /// let ql = QueryListingRequest::new("r/rust/hot", 1, 1);
    /// let data: Vec<Value> = api.query_listing(ql).unwrap();
    /// println!("{:?}", data);
    /// ```
    pub fn query_listing(&self, ql: QueryListingRequest) -> Result<Vec<Value>, ApiError> {
        debug!("Listing request call: {:?}", ql);
        let method = Method::GET;
        let path = self.reformat_path(&ql.path);
        let headers = self.get_headers();

        let req = self.client.request(method, &path).headers(headers);
        let mut all_resp: Vec<Value> = Vec::new();
        let mut after = match ql.after {
            Some(a) => a.to_owned(),
            None => String::new(),
        };
        let mut count = ql.count;

        for _ in 0..ql.requests {
            let req = req.try_clone().unwrap();
            let req = if ql.params.is_empty() {
                req.query(ql.params)
            } else {
                req
            };
            let mut listing_parms = vec![("limit", ql.limit.to_string())];
            if !after.is_empty() {
                listing_parms.push(("after", after));
            }
            if count > 0 {
                listing_parms.push(("count", format!("{}", count)));
            }
            if ql.show_all {
                listing_parms.push(("show", "all".to_owned()));
            }
            let req = req.query(&listing_parms);
            let mut resp = req.send()?;
            if resp.status().is_client_error() || resp.status().is_server_error() {
                return Err(ApiError::from(format!(
                    "Server error, code {}",
                    resp.status().as_str()
                )));
            }
            let data: Value = resp.json()?;
            after = data["data"]["after"].as_str().unwrap().to_owned();
            for item in data["data"]["children"].as_array().unwrap() {
                count += 1;
                all_resp.push(item.clone());
            }
        }
        Ok(all_resp)
    }

    /// Search for subreddits matching the parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - subreddit (partial) name
    ///
    /// # Examples
    ///
    /// ```
    /// let names = match api.search_for_subreddit("rust") {
    ///     Ok(names) => names,
    ///     Err(err) => panic!(err),
    /// }
    /// ```
    pub fn search_for_subreddit(&self, name: &str) -> Result<Vec<Subreddit>, ApiError> {
        let mut resp = self.query(
            "GET",
            "api/search_reddit_names",
            Some(vec![("query", name), ("exact", "false")]),
            None,
        )?;
        let status = resp.status();
        if status.is_client_error() || status.is_server_error() {
            return Err(ApiError::from(format!(
                "Server error, code {}",
                status.as_str(),
            )));
        }
        let data: Value = resp.json()?;
        Ok(data["names"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .map(|e| Subreddit {
                api: &self,
                name: e.to_owned(),
            })
            .collect::<Vec<Subreddit>>())
    }

    /// Get a subreddit by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - subreddit name
    ///
    /// # Examples
    ///
    /// ```
    /// let subreddit = match api.get_subreddit("rust") {
    ///     Ok(sr) => sr,
    ///     Err(err) => panic!(err),
    /// }
    /// ```
    pub fn get_subreddit(&self, name: &str) -> Result<Subreddit, ApiError> {
        let matching = self.search_for_subreddit(name)?;
        for sr in matching {
            if sr.name == name {
                return Ok(sr);
            }
        }
        Err(ApiError::from(String::from("Subreddit not found")))
    }
}

/// the program's API access information.
#[derive(Debug, Deserialize, PartialEq)]
struct AccessTokenResponse {
    #[serde(alias = "access_token")]
    token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
}

#[cfg(test)]
mod tests {
    use super::{AccessTokenResponse, Api, Config, QueryListingRequest};
    use mockito::mock;
    use std::fs::File;
    use std::io::Write;
    use tempfile;

    fn get_config() -> Config {
        Config {
            username: String::new(),
            password: String::new(),
            user_agent: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
        }
    }

    fn get_api() -> Api {
        let config = get_config();
        Api::new(config)
    }

    fn get_sample_atr() -> String {
        String::from(
            "{\"access_token\":\"aaaaa\",\"token_type\":\"bbbbb\", \
             \"expires_in\":10000,\"scope\":\"ccccc\"}",
        )
    }

    #[test]
    fn load_config_from_disk() {
        let original_content = "{\"username\":\"a\",\"password\":\"b\", \
                                \"user_agent\":\"c\",\"client_id\":\"d\",\"client_secret\":\"e\"}";
        let dir = tempfile::tempdir().unwrap();
        let file_name = "reddit_api-config.json";
        let file_path = dir.path().join(file_name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", original_content).unwrap();

        let config = Config::load_config(&file_path.as_os_str().to_str().unwrap()).unwrap();

        assert_eq!(config.username, "a");
        assert_eq!(config.password, "b");
        assert_eq!(config.user_agent, "c");
        assert_eq!(config.client_id, "d");
        assert_eq!(config.client_secret, "e");
    }

    #[test]
    fn access_token_response_serialize() {
        let atr: AccessTokenResponse = serde_json::from_str(&get_sample_atr()).unwrap();

        assert_eq!(atr.token, String::from("aaaaa"));
        assert_eq!(atr.token_type, String::from("bbbbb"));
        assert_eq!(atr.expires_in, 10000);
        assert_eq!(atr.scope, String::from("ccccc"));
    }

    #[test]
    fn new_api() {
        let config = get_config();
        let api = get_api();

        assert_eq!(api.config, config);
        assert_eq!(api.access_token, None);
        assert_eq!(api.whoami, None);
    }

    #[test]
    fn do_login() {
        let _m1 = mock("POST", "/api/v1/access_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_sample_atr())
            .create();
        let _m2 = mock("GET", "/api/v1/me")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"test-name\"}")
            .create();

        let mut api = get_api();
        api.do_login().unwrap();
        let username = api.get_username().unwrap();

        assert_eq!(username, "test-name");
        _m1.assert();
        _m2.assert();
    }

    #[test]
    fn query_listing() {
        let body = "{\"data\":{\"kind\":\"Listing\",\"after\":\"t3_ccccc\",\"children\": \
                    [{\"data\":{\"id\":\"aaaaa\"},\"kind\":\"t3\"},{\"data\":{\"id\":\"bbbbb\"}, \
                    \"kind\":\"t3\"},{\"data\":{\"id\":\"ccccc\"},\"kind\":\"t3\"}]}}";
        let _m1 = mock("GET", "/some/endpoint?limit=3&show=all")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        let ql = QueryListingRequest::new("some/endpoint", 3, 1);
        let values = get_api().query_listing(ql).unwrap();

        assert_eq!(values.len(), 3);
        _m1.assert();
    }

    #[test]
    fn search_for_subreddit() {
        let body = "{\"names\":[\"rust1\",\"rust2\",\"rust3\"]}";
        let _m1 = mock("GET", "/api/search_reddit_names?query=rust&exact=false")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        let api = get_api();
        let srs = api.search_for_subreddit("rust").unwrap();

        assert_eq!(srs.len(), 3);
        _m1.assert();
    }

    #[test]
    fn get_subreddit() {
        let body = "{\"names\":[\"rust\",\"rust1\",\"rust2\"]}";
        let _m1 = mock("GET", "/api/search_reddit_names?query=rust1&exact=false")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        let api = get_api();
        let sr = api.get_subreddit("rust1").unwrap();

        assert_eq!(sr.name, "rust1");
        _m1.assert();
    }
}
