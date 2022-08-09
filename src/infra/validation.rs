//! Utility types for error handling.

use serde::{Deserialize, Serialize};
use std::ops::Deref;
use validator::Validate;

/// A collection of validation errors. A wrapper type is used to be able to implement traits on it.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationErrors(validator::ValidationErrors);

/// Having an instance of [`Validated<T>`] guarantees that the inner `T` has been validated.
///
/// # Examples
///
/// Valid objects can be constructed with [`Validated::new`].
///
/// ```
/// # use actix_web_demo::infra::validation::Validated;
/// # use validator::Validate;
/// #
/// #[derive(Validate)]
/// struct Email {
///     #[validate(email)]
///     email: String
/// }
///
/// let email = Email {
///     email: "foo@bar.com".to_string(),
/// };
///
/// assert!(Validated::new(email).is_ok())
/// ```
///
/// Invalid objects will be stopped by the constructor.
///
/// ```
/// # use actix_web_demo::infra::validation::Validated;
/// # use validator::Validate;
/// #
/// #[derive(Validate)]
/// struct Email {
///     #[validate(email)]
///     email: String
/// }
///
/// let email = Email {
///     email: "not an email".to_string(),
/// };
///
/// assert!(Validated::new(email).is_err())
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Validated<T> {
    data: T,
}

impl<T: Validate> Validated<T> {
    /// Tries to construct a new validated value.
    pub fn new(data: T) -> Result<Self, ValidationErrors> {
        match data.validate() {
            Ok(_) => Ok(Self { data }),
            Err(e) => Err(ValidationErrors(e)),
        }
    }
}

impl<T> Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'de, T: Deserialize<'de> + Validate> Deserialize<'de> for Validated<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let t = T::deserialize(deserializer)?;
        match Validated::new(t) {
            Ok(t) => Ok(t),
            Err(e) => Err(D::Error::custom(e.0.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::infra::validation::Validated;
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Validate)]
    struct AlwaysValid;

    #[test]
    fn always_valid_succeeds() {
        let data = AlwaysValid;
        assert_eq!(Ok(Validated { data }), Validated::new(data));
    }

    #[derive(Clone, Debug, Validate, PartialEq, Eq, Deserialize)]
    struct Email {
        #[validate(email)]
        email: String,
    }

    #[test]
    fn validating_valid_email_succeeds() {
        let email = Email {
            email: "foo@bar.com".to_string(),
        };
        assert_eq!(
            Ok(Validated {
                data: email.clone()
            }),
            Validated::new(email)
        );
    }

    #[test]
    fn validating_invalid_email_fails() {
        let email = Email {
            email: "not_an_email".to_string(),
        };
        assert!(matches!(Validated::new(email), Err(_)));
    }

    #[test]
    fn parsing_valid_email_succeeds() {
        let email: Result<Validated<Email>, _> = serde_json::from_str(
            r#"
                {
                    "email": "foo@bar.baz"
                }
            "#,
        );
        assert_eq!(
            Validated {
                data: Email {
                    email: "foo@bar.baz".to_string()
                }
            },
            email.unwrap()
        );
    }

    #[test]
    fn parsing_invalid_email_fails() {
        let email: Result<Validated<Email>, _> = serde_json::from_str(
            r#"
                {
                    "email": "not_an_email"
                }
            "#,
        );
        assert!(email.is_err())
    }
}
