//! Error handling. Includes a custom error type `ApiError` that
//! contains conversions from the underlying error types that the
//! libraries that this library relies on can generate.

use reqwest;
use std::error;
use std::fmt;

/// Wrapper for errors.
#[derive(Debug, PartialEq)]
pub struct ApiError {
    pub source: String,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.source.is_empty() {
            write!(f, "API error: {}", self.message)
        } else {
            write!(f, "API error from '{}': {}", self.source, self.message)
        }
    }
}

impl error::Error for ApiError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError {
            source: "reqwest::Error".to_owned(),
            message: format!("{:?}", error).to_owned(),
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        ApiError {
            source: "std::io::Error".to_owned(),
            message: format!("{:?}", error).to_owned(),
        }
    }
}

impl From<serde_json::error::Error> for ApiError {
    fn from(error: serde_json::error::Error) -> Self {
        ApiError {
            source: "serde_json::error::Error".to_owned(),
            message: format!("{:?}", error).to_owned(),
        }
    }
}

impl From<http::method::InvalidMethod> for ApiError {
    fn from(error: http::method::InvalidMethod) -> Self {
        ApiError {
            source: "http::method::InvalidMethod".to_owned(),
            message: format!("{:?}", error).to_owned(),
        }
    }
}

impl From<String> for ApiError {
    fn from(error: String) -> Self {
        ApiError {
            source: String::new(),
            message: error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;

    #[test]
    fn from_string() {
        let msg = String::from("some erorr");
        let expected = ApiError {
            source: String::new(),
            message: msg.clone(),
        };
        let actual = ApiError::from(msg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn display_no_source() {
        let e = ApiError {
            source: String::new(),
            message: String::from("something"),
        };
        let expected = String::from("API error: something");
        let actual = format!("{}", e);

        assert_eq!(actual, expected);
    }

    #[test]
    fn display_with_source() {
        let e = ApiError {
            source: String::from("somewhere"),
            message: String::from("something"),
        };
        let expected = String::from("API error from 'somewhere': something");
        let actual = format!("{}", e);

        assert_eq!(actual, expected);
    }

    // 'From' impl's tested by nature of successfully compiling
}
