//! Struct-based access to various post APIs.
//!
//! Get a post struct with:
//!
//! TODO

use super::user::User;
use crate::Api;

/// Maps to a single post, either link or text.
#[derive(Clone)]
pub struct Post<'a> {
    /// Rerefence to the source `Api` struct. Used for calling API endpoints.
    pub api: &'a Api,
    /// Post's user.
    pub user: User<'a>,
    /// Title's post.
    pub title: String,

    /// The post's link, if a link-type post.
    pub link: Option<String>,
    /// The post's text, if a text-type post.
    pub text: Option<String>,
}

impl<'a> Post<'a> {}

#[cfg(test)]
mod tests {}
