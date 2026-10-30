use axum::http::{HeaderMap, header::COOKIE};

use super::session::LIFETIME_SECS;

const LOGIN_LIFETIME_SECS: i64 = 600;

pub(crate) struct Cookie {
    name: &'static str,
    path: &'static str,
    max_age_secs: i64,
}

pub(crate) const SESSION: Cookie = Cookie {
    name: "crabbage_session",
    path: "/",
    max_age_secs: LIFETIME_SECS,
};

pub(crate) const LOGIN: Cookie = Cookie {
    name: "crabbage_login",
    path: "/auth/",
    max_age_secs: LOGIN_LIFETIME_SECS,
};

impl Cookie {
    pub(crate) fn value(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .filter_map(|pair| pair.trim().split_once('='))
            .find(|(name, value)| *name == self.name && !value.is_empty())
            .map(|(_, value)| value.to_owned())
    }

    pub(crate) fn set(&self, value: &str) -> String {
        format!(
            "{}={value}; HttpOnly; Secure; SameSite=Lax; Path={}; Max-Age={}",
            self.name, self.path, self.max_age_secs
        )
    }

    pub(crate) fn clear(&self) -> String {
        format!(
            "{}=; HttpOnly; Secure; SameSite=Lax; Path={}; Max-Age=0",
            self.name, self.path
        )
    }
}
