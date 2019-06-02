//! Struct-based access to various user APIs.
//!
//! Get a user struct with:
//!
//! TODO

use crate::{Api, ApiError};

#[derive(Clone)]
pub struct User<'a> {
    /// Rerefence to the source `Api` struct. Used for calling API endpoints.
    pub api: &'a Api,
    /// User's name.
    pub name: String,
    // ...
}

impl<'a> User<'a> {
    pub fn send_message(&self, message: &str) -> Result<(), ApiError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
