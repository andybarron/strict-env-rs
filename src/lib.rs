#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::unwrap_used,
    missing_crate_level_docs,
    missing_docs
)]

//! Use this crate to parse environment variables into any type that
//! implements [`FromStr`](std::str::FromStr).
//!
//! # Basic usage
//! ```
//! # fn main() -> Result<(), strict_env::Error> {
//! std::env::set_var("PORT", "9001");
//! let port: u16 = strict_env::parse("PORT")?;
//! assert_eq!(port, 9001);
//! # Ok(())
//! # }
//! ```
//!
//! # Usage with remote types
//! If you need to parse a type that originates from an external crate
//! and does not implement [`FromStr`](std::str::FromStr), you can wrap
//! the value in a newtype that implements the trait.
//! ```
//! // std::time::Duration does not implement FromStr!
//! struct ConfigDuration(std::time::Duration);
//!
//! // Custom implementation using the awesome humantime crate
//! impl std::str::FromStr for ConfigDuration {
//!     type Err = humantime::DurationError;
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         let inner = humantime::parse_duration(s)?;
//!         Ok(Self(inner))
//!     }
//! }
//!
//! // Now we can use strict_env! (But we might have to use the turbofish.)
//! # fn main() -> Result<(), strict_env::Error> {
//! std::env::set_var("CACHE_DURATION", "2 minutes");
//! let cache_duration = strict_env::parse::<ConfigDuration>("CACHE_DURATION")?.0;
//! assert_eq!(cache_duration.as_secs(), 120);
//! # Ok(())
//! # }
//! ```

use std::{
    env::{self, VarError},
    ffi::OsString,
    str::FromStr,
};

/// Parse an environment variable into a value that implements
/// [`FromStr`](std::str::FromStr).
///
/// # Errors
/// Returns an error if the requested environment variable is missing
/// or empty, contains invalid UTF-8, or has a value that cannot be
/// parsed into the target type.
pub fn parse<T: FromStr>(name: &str) -> Result<T, Error>
where
    T::Err: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let value_result = env::var(name);
    let value = match value_result {
        Ok(value) => {
            if value.is_empty() {
                return Err(Error::Missing {
                    name: name.to_owned(),
                });
            }
            value
        }
        Err(err) => match err {
            VarError::NotPresent => {
                return Err(Error::Missing {
                    name: name.to_owned(),
                })
            }
            VarError::NotUnicode(value) => {
                return Err(Error::InvalidUtf8 {
                    name: name.to_owned(),
                    value,
                })
            }
        },
    };
    let parse_result = T::from_str(&value);
    let parsed = match parse_result {
        Ok(parsed) => parsed,
        Err(err) => {
            return Err(Error::InvalidValue {
                name: name.to_owned(),
                value,
                source: err.into(),
            })
        }
    };
    Ok(parsed)
}

/// Like [`parse`](crate::parse), but allows the environment variable to
/// be missing or empty.
///
/// The parsed object is wrapped in an [`Option`](Option) to allow this.
///
/// # Errors
/// Returns an error if the requested environment variable contains invalid
/// UTF-8 or has a value that cannot be parsed into the target type.
pub fn parse_optional<T: FromStr>(name: &str) -> Result<Option<T>, Error>
where
    T::Err: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let result = parse(name);
    match result {
        Ok(parsed) => Ok(Some(parsed)),
        Err(Error::Missing { .. }) => Ok(None),
        Err(err) => Err(err),
    }
}

/// Like [`parse`](crate::parse), but falls back to a default value when
/// the environment variable is missing or empty.
///
/// The target type must implement [`Default`](Default).
///
/// # Errors
/// Returns an error if the requested environment variable contains invalid
/// UTF-8 or has a value that cannot be parsed into the target type.
pub fn parse_or_default<T: FromStr + Default>(name: &str) -> Result<T, Error>
where
    T::Err: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    parse_optional(name).map(Option::unwrap_or_default)
}

#[derive(Debug, thiserror::Error)]
/// Error type for this library.
pub enum Error {
    /// The requested environment variable was missing or empty.
    #[error("Missing or empty environment variable {name:?}")]
    Missing {
        /// Name of the requested environment variable.
        name: String,
    },
    #[error("Invalid UTF-8 in environment variable {name:?}")]
    /// The environment variable exists, but its value is not valid
    /// UTF-8.
    InvalidUtf8 {
        /// Name of the requested environment variable.
        name: String,
        /// Value of the environment variable.
        value: OsString,
    },
    #[error("Error parsing environment variable {name:?}: {source}")]
    /// The environment variable exists and is valid UTF-8, but it
    /// could not be parsed into the target type.
    InvalidValue {
        /// Name of the requested environment variable.
        name: String,
        /// Value of the environment variable.
        value: String,
        #[source]
        /// The underlying error that occurred during parsing.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[allow(
    clippy::missing_const_for_fn,
    clippy::unwrap_used,
    clippy::wildcard_imports
)]
#[cfg(test)]
mod tests {
    use crate::*;
    use os_str_bytes::OsStrBytes;
    use serial_test::serial;
    use std::ffi::OsStr;

    mod parse {
        use super::*;

        #[test]
        #[serial]
        fn valid() {
            let _guard = EnvGuard::with("TEST_VAR", "255");
            let value: u8 = parse("TEST_VAR").unwrap();
            assert_eq!(value, 255);
        }

        #[test]
        #[serial]
        fn missing() {
            let _guard = EnvGuard::without("TEST_VAR");
            let error = parse::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::Missing { .. }));
        }

        #[test]
        #[serial]
        fn empty() {
            let _guard = EnvGuard::with("TEST_VAR", "");
            let error = parse::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::Missing { .. }));
        }

        #[test]
        #[serial]
        fn invalid_utf8() {
            let value = invalid_utf8_string();
            let _guard = EnvGuard::with("TEST_VAR", value);
            let error = parse::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::InvalidUtf8 { .. }));
        }

        #[test]
        #[serial]
        fn invalid_value() {
            let _guard = EnvGuard::with("TEST_VAR", "256");
            let error = parse::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::InvalidValue { .. }));
        }
    }

    mod parse_optional {
        use super::*;

        #[test]
        #[serial]
        fn valid() {
            let _guard = EnvGuard::with("TEST_VAR", "255");
            let value: u8 = parse_optional("TEST_VAR").unwrap().unwrap();
            assert_eq!(value, 255);
        }

        #[test]
        #[serial]
        fn missing() {
            let _guard = EnvGuard::without("TEST_VAR");
            let option = parse_optional::<u8>("TEST_VAR").unwrap();
            assert_eq!(option, None);
        }

        #[test]
        #[serial]
        fn empty() {
            let _guard = EnvGuard::with("TEST_VAR", "");
            let option = parse_optional::<u8>("TEST_VAR").unwrap();
            assert_eq!(option, None);
        }

        #[test]
        #[serial]
        fn invalid_utf8() {
            let invalid_unicode_bytes = [b'f', b'o', b'o', 0x80];
            let invalid_unicode = OsStr::from_raw_bytes(&invalid_unicode_bytes[..]).unwrap();
            let _guard = EnvGuard::with("TEST_VAR", &invalid_unicode);
            let error = parse_optional::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::InvalidUtf8 { .. }));
        }

        #[test]
        #[serial]
        fn invalid_value() {
            let _guard = EnvGuard::with("TEST_VAR", "256");
            let error = parse_optional::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::InvalidValue { .. }));
        }
    }

    mod parse_or_default {
        use super::*;

        #[test]
        #[serial]
        fn valid() {
            let _guard = EnvGuard::with("TEST_VAR", "255");
            let value: u8 = parse_or_default("TEST_VAR").unwrap();
            assert_eq!(value, 255);
        }

        #[test]
        #[serial]
        fn missing() {
            let _guard = EnvGuard::without("TEST_VAR");
            let value: u8 = parse_or_default::<u8>("TEST_VAR").unwrap();
            assert_eq!(value, 0);
        }

        #[test]
        #[serial]
        fn empty() {
            let _guard = EnvGuard::with("TEST_VAR", "");
            let value: u8 = parse_or_default::<u8>("TEST_VAR").unwrap();
            assert_eq!(value, 0);
        }

        #[test]
        #[serial]
        fn invalid_utf8() {
            let invalid_unicode_bytes = [b'f', b'o', b'o', 0x80];
            let invalid_unicode = OsStr::from_raw_bytes(&invalid_unicode_bytes[..]).unwrap();
            let _guard = EnvGuard::with("TEST_VAR", &invalid_unicode);
            let error = parse_or_default::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::InvalidUtf8 { .. }));
        }

        #[test]
        #[serial]
        fn invalid_value() {
            let _guard = EnvGuard::with("TEST_VAR", "256");
            let error = parse_or_default::<u8>("TEST_VAR").unwrap_err();
            assert!(matches!(error, Error::InvalidValue { .. }));
        }
    }

    mod error {
        use super::*;

        #[test]
        fn is_send() {
            assert_send::<Error>();
        }
        #[test]
        fn is_sync() {
            assert_sync::<Error>();
        }
        #[test]
        fn is_static() {
            assert_static::<Error>();
        }
        #[test]
        fn missing() {
            let error = Error::Missing {
                name: "TEST_VAR".into(),
            };
            assert_eq!(
                error.to_string(),
                "Missing or empty environment variable \"TEST_VAR\"",
            );
        }
        #[test]
        fn invalid_utf8() {
            let error = Error::InvalidUtf8 {
                name: "TEST_VAR".into(),
                value: invalid_utf8_string(),
            };
            assert_eq!(
                error.to_string(),
                "Invalid UTF-8 in environment variable \"TEST_VAR\"",
            );
        }
        #[test]
        fn invalid_value() {
            let source = "".parse::<u8>().unwrap_err();
            let error = Error::InvalidValue {
                name: "TEST_VAR".into(),
                value: "".into(),
                source: source.into(),
            };
            assert_eq!(
                error.to_string(),
                "Error parsing environment variable \"TEST_VAR\": cannot parse integer from empty string",
            );
        }
    }

    // utils

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_static<T: 'static>() {}

    fn invalid_utf8_string() -> OsString {
        let bytes = [b'f', b'o', b'o', 0x80];
        std::str::from_utf8(&bytes).unwrap_err();
        OsStr::from_raw_bytes(&bytes[..]).unwrap().to_os_string()
    }

    struct EnvGuard {
        vars: Vec<(OsString, OsString)>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                vars: std::env::vars_os().collect(),
            }
        }
        fn with(name: &str, value: impl AsRef<OsStr>) -> Self {
            let guard = Self::new();
            std::env::set_var(name, value);
            guard
        }
        fn without(name: impl AsRef<OsStr>) -> Self {
            let guard = Self::new();
            std::env::remove_var(name);
            guard
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (var, _) in std::env::vars_os() {
                std::env::remove_var(var);
            }
            assert_eq!(std::env::vars_os().count(), 0);
            for (var, value) in &self.vars {
                std::env::set_var(var, value);
            }
        }
    }
}
