//! Struct-based access to various user APIs.
//!
//! Get a user struct with:
//!
//! ```rust,no_run,ignore
//! let user = api.get_user("some-username")?;
//! ```

use crate::{Api, ApiError};
use serde_json::Value;

#[derive(Clone)]
pub struct User<'a> {
    /// Rerefence to the source `Api` struct. Used for calling API endpoints.
    pub api: &'a Api,
    /// User's "about" information.
    pub about: Value,
}

impl<'a> User<'a> {
    /// Get the user's name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run,ignore
    /// let name = user.name();
    /// ```
    pub fn name(&self) -> String {
        self.about["data"]["name"].as_str().unwrap().to_owned()
    }

    // TODO
    pub fn send_message(&self, _message: &str) -> Result<(), ApiError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::User;
    use crate::Api;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref API: Api = Api::new(std::default::Default::default());
    }

    #[test]
    fn name() {
        let u = User {
            api: &API,
            about: serde_json::from_str("{\"data\":{\"name\":\"test\"}}").unwrap(),
        };

        assert_eq!(u.name(), "test")
    }
}
