use std::fmt;

use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

const REDACTED: &str = "[REDACTED]";

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionCookie {
    pub name: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub value: SecretString,
    pub domain: Option<String>,
    pub path: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub secure: bool,
    pub http_only: bool,
}

impl SessionCookie {
    pub fn new(name: impl Into<String>, value: SecretString) -> Self {
        Self {
            name: name.into(),
            value,
            domain: None,
            path: "/".to_string(),
            expires_at: None,
            secure: false,
            http_only: true,
        }
    }
}

impl fmt::Debug for SessionCookie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionCookie")
            .field("name", &self.name)
            .field("value", &REDACTED)
            .field("domain", &self.domain)
            .field("path", &self.path)
            .field("expires_at", &self.expires_at)
            .field("secure", &self.secure)
            .field("http_only", &self.http_only)
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionMaterial {
    pub cookies: Vec<SessionCookie>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl fmt::Debug for SessionMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionMaterial")
            .field("cookies", &self.cookies)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Clone)]
pub struct SessionSnapshot {
    pub cookies: Vec<SessionCookie>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_authenticated_at: DateTime<Utc>,
}

impl SessionSnapshot {
    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at <= now)
    }
}

impl fmt::Debug for SessionSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionSnapshot")
            .field("cookies", &self.cookies)
            .field("expires_at", &self.expires_at)
            .field("last_authenticated_at", &self.last_authenticated_at)
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SrtRuntimeSession {
    active: Option<SessionSnapshot>,
}

impl SrtRuntimeSession {
    pub fn activate(&mut self, material: SessionMaterial, now: DateTime<Utc>) {
        self.active = Some(SessionSnapshot {
            cookies: material.cookies,
            expires_at: material.expires_at,
            last_authenticated_at: now,
        });
    }

    pub fn apply_update(&mut self, material: SessionMaterial, now: DateTime<Utc>) {
        self.activate(material, now);
    }

    pub fn clear(&mut self) {
        self.active = None;
    }

    pub fn snapshot(&self) -> Option<SessionSnapshot> {
        self.active.clone()
    }

    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        self.active
            .as_ref()
            .is_none_or(|snapshot| snapshot.is_expired_at(now))
    }
}
