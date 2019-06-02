//! Struct-based access to various comment APIs.
//!
//! Get a comment struct with:
//!
//! TODO

use super::user::User;
use crate::{Api, ApiError};

#[derive(Clone)]
pub struct Comment<'a> {
    /// Rerefence to the source `Api` struct. Used for calling API endpoints.
    pub api: &'a Api,
    /// Comment's user.
    pub user: User<'a>,
    /// The Comment's link, if a link-type Comment.
    pub link: Option<String>,
    /// The Comment's text, if a text-type Comment.
    pub text: Option<String>,
}

impl<'a> Comment<'a> {
    pub fn reply(&self, message: &str) -> Result<Comment, ApiError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
