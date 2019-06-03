//! Structs for use in making and viewing [listing] requests and responses.
//!
//! [listing]: https://www.reddit.com/dev/api#listings
//!
//! # Examples
//!
//! Simple:
//!
//! ```rust,no_run,ignore
//! let ql = QueryListingRequest::new("r/rust/hot", 1, 1);
//! ```
//!
//! More complex:
//!
//! ```rust,no_run,ignore
//! let ql = QueryListingRequest::new("r/rust/hot", 25, 2)
//!     .after(Some("t3_aaaaa"))
//!     .count(12)
//!     .show_all(false);
//! ```

/// Builder struct for constructing requests to a listing endpoint.
#[derive(Clone, Debug)]
pub struct QueryListingRequest<'a> {
    /// The relative URL path
    pub path: &'a str,
    /// The optional URL query parameters to supply
    pub params: &'a [(&'a str, &'a str)],
    /// The optional fullname to start at
    pub after: Option<&'a str>,
    /// The number received so far
    pub count: u64,
    /// The number of items to get per request
    pub limit: u64,
    /// The number of requests to make
    pub requests: u64,
    /// Wether to show all items (true) or follow hidden items settings (false)
    pub show_all: bool,
}

impl<'a> QueryListingRequest<'a> {
    /// Construct a new builder.
    pub fn new(path: &'a str, limit: u64, requests: u64) -> Self {
        QueryListingRequest {
            path,
            params: &[],
            after: None,
            count: 0,
            limit,
            requests,
            show_all: true,
        }
    }

    /// Override the `path` field.
    pub fn path(mut self, path: &'a str) -> Self {
        self.path = path;
        self
    }

    /// Override the `params` field.
    pub fn params(mut self, params: &'a [(&'a str, &'a str)]) -> Self {
        self.params = params;
        self
    }

    /// Override the `after` field.
    pub fn after(mut self, after: Option<&'a str>) -> Self {
        self.after = after;
        self
    }

    /// Override the `count` field.
    pub fn count(mut self, count: u64) -> Self {
        self.count = count;
        self
    }

    /// Override the `limit` field.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = limit;
        self
    }

    /// Override the `requests` field.
    pub fn requests(mut self, requests: u64) -> Self {
        self.requests = requests;
        self
    }

    /// Override the `show_all` field.
    pub fn show_all(mut self, show_all: bool) -> Self {
        self.show_all = show_all;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::QueryListingRequest;

    #[test]
    fn simple() {
        let path = "p";
        let limit = 1;
        let requests = 2;

        let ql = QueryListingRequest::new(path, limit, requests);

        assert_eq!(ql.path, path);
        assert_eq!(ql.limit, limit);
        assert_eq!(ql.requests, requests);
        assert_eq!(ql.params, &[]);
        assert_eq!(ql.after, None);
        assert_eq!(ql.count, 0);
        assert_eq!(ql.show_all, true);
    }

    #[test]
    fn with_builders() {
        let path = "p";
        let limit = 1;
        let requests = 2;
        let params = vec![("a", "b")];
        let after = Some("t3_aaa");
        let count = 3;
        let show_all = false;

        let ql = QueryListingRequest::new(path, limit, requests)
            .params(&params)
            .after(after)
            .count(count)
            .show_all(show_all);

        assert_eq!(ql.path, path);
        assert_eq!(ql.limit, limit);
        assert_eq!(ql.requests, requests);
        assert_eq!(ql.params, params.as_slice());
        assert_eq!(ql.after, after);
        assert_eq!(ql.count, count);
        assert_eq!(ql.show_all, show_all);
    }
}
