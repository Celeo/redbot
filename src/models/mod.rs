//! Struct-oriented access to the Reddit API.
//!
//! The modules contained here are wrappers around various Reddit APIs
//! that allow interacting with the API without requiring the developer
//! to look at the API docs to find the endpoints that they want to use.
//! Instead, they are able to create and interact with the structs defined
//! in this module, relying on this library to abstract-away those
//! individual API calls to make interacting with the API simpler.

pub mod comment;
pub mod post;
pub mod subreddit;
pub mod user;
