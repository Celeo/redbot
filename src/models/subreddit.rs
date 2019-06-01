//! Struct-based access to various subreddit APIs.
//!
//! Get a subreddit struct with:
//!
//! ```
//! let subreddit = api.get_subreddit("name")?;
//! ```

use crate::{Api, ApiError, QueryListingRequest, Value};

/// Maps to a single subreddit. Contains methods for reading and
/// writing to subreddit-specific APIs.
#[derive(Clone)]
pub struct Subreddit<'a> {
    /// Rerefence to the source `Api` struct. Used for calling API endpoints.
    pub api: &'a Api,
    /// Name of the subreddit.
    pub name: String,
}

impl<'a> Subreddit<'a> {
    /// Get the top `count` posts from the subreddit.
    ///
    /// # Arguments
    ///
    /// * `count` - number of posts to retrieve
    ///
    /// # Examples
    ///
    /// ```
    /// let posts = subreddit.get_top(25)?;
    /// ```
    pub fn get_top(&self, count: u64) -> Result<Vec<Value>, ApiError> {
        let (mp, times) = if count > 100 {
            (100, count / 100)
        } else {
            (count, 1)
        };
        let path = format!("r/{}/top", self.name);
        let ql = QueryListingRequest::new(&path, mp, times);
        let posts = self.api.query_listing(ql)?;
        Ok(posts.iter().take(count as usize).cloned().collect())
    }
}

#[cfg(test)]
mod tests {}
