#![cfg(feature = "github")]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub language: Option<String>,
    pub open_issues_count: u32,
    pub watchers_count: u32,
    pub subscribers_count: u32,
    pub owner: GitHubOwner,
    pub topics: Vec<String>,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
    pub avatar_url: String,
}

pub fn is_github_url(url: &str) -> bool {
    url.contains("github.com")
}

#[derive(Debug, Clone)]
pub struct GitHubBasicPreview {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GitHubDetailedInfo {
    pub full_name: String,
    pub description: String,
    pub stars_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub language: Option<String>,
    pub default_branch: String,
    pub topics: Vec<String>,
    pub html_url: String,
    pub homepage: Option<String>,
}
