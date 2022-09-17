//! Utilities for extracting headers.

/// Supported authentication methods.
pub(crate) enum Auth {
    Basic(BasicAuth),
    Bearer(BearerAuth),
}

impl Auth {
    pub(crate) fn from_header(header: &str) -> Option<Self> {
        let (prefix, rest) = header.split_once(' ')?;
        let auth = match prefix {
            "Bearer" => Auth::Bearer(BearerAuth::from_header(rest)?),
            "Basic" => Auth::Basic(BasicAuth::from_header(rest)?),
            _ => return None,
        };
        Some(auth)
    }
}

/// Basic authentication.
pub(crate) struct BasicAuth {
    username: String,
    password: String,
}

impl BasicAuth {
    pub(crate) fn from_header(header: &str) -> Option<Self> {
        let decoded = base64::decode(header).ok()?;
        let decoded = String::from_utf8(decoded).ok()?;
        let (username, password) = decoded.split_once(':')?;
        let basic_auth = Self {
            username: username.to_string(),
            password: password.to_string(),
        };
        Some(basic_auth)
    }

    pub(crate) fn username(&self) -> &str {
        self.username.as_ref()
    }

    pub(crate) fn password(&self) -> &str {
        self.password.as_ref()
    }
}

/// Bearer authentication.
pub(crate) struct BearerAuth {
    token: String,
}

impl BearerAuth {
    pub(crate) fn from_header(header: &str) -> Option<Self> {
        let bearer_auth = Self {
            token: header.to_owned(),
        };
        Some(bearer_auth)
    }

    pub(crate) fn token(&self) -> &str {
        self.token.as_ref()
    }
}
