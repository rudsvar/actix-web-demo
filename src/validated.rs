//! Utility types for error handling.

use std::ops::Deref;
use validator::Validate;

/// A collection of validation errors. A wrapper type is used to be able to implement traits on it.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationErrors(validator::ValidationErrors);

/// Having an instance of [`Validated<T>`] guarantees that the inner `T` has been validated.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Validated<T> {
    data: T,
}

impl<T: Validate> Validated<T> {
    /// Try to construct a new validated value.
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

#[cfg(test)]
mod tests {
    use crate::validated::Validated;
    use validator::Validate;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Validate)]
    struct AlwaysValid;

    #[test]
    fn always_valid_succeeds() {
        let data = AlwaysValid;
        assert_eq!(Ok(Validated { data }), Validated::new(data));
    }

    #[derive(Clone, Debug, Validate, PartialEq, Eq)]
    struct Email {
        #[validate(email)]
        email: String,
    }

    #[test]
    fn valid_email_succeeds() {
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
    fn invalid_email_fails() {
        let email = Email {
            email: "not_an_email".to_string(),
        };
        assert!(matches!(Validated::new(email), Err(_)));
    }
}
