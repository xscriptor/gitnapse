#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub published_at: Option<String>,
    pub prerelease: bool,
}
