#![allow(dead_code)]

#[path = "../src/github.rs"]
mod github;
#[path = "../src/models.rs"]
mod models;

use github::GitHubClient;
use mockito::{Matcher, Server};
use serial_test::serial;

fn with_api_base<T>(base: &str, test: impl FnOnce() -> T) -> T {
    let prev = std::env::var("GITNAPSE_GITHUB_API").ok();
    unsafe { std::env::set_var("GITNAPSE_GITHUB_API", base) };
    let out = test();
    if let Some(value) = prev {
        unsafe { std::env::set_var("GITNAPSE_GITHUB_API", value) };
    } else {
        unsafe { std::env::remove_var("GITNAPSE_GITHUB_API") };
    }
    out
}

#[test]
#[serial]
fn search_general_uses_search_endpoint() {
    let mut server = Server::new();
    let _m = server
        .mock("GET", "/search/repositories")
        .match_query(Matcher::Regex(
            r"q=rust\+language:rust.*per_page=30.*page=1".to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
              "items": [
                {
                  "name": "repo-one",
                  "full_name": "x/repo-one",
                  "description": "General search result",
                  "stargazers_count": 10,
                  "language": "Rust",
                  "clone_url": "https://github.com/x/repo-one.git",
                  "owner": { "login": "x" },
                  "default_branch": "main"
                }
              ]
            }"#,
        )
        .create();

    with_api_base(&server.url(), || {
        let client = GitHubClient::new(None).expect("client");
        let repos = client
            .search_repositories_page("rust language:rust", 1, 30)
            .expect("search");
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].full_name, "x/repo-one");
    });
}

#[test]
#[serial]
fn me_query_lists_and_filters_authenticated_repos() {
    let mut server = Server::new();
    let _m = server
        .mock("GET", "/user/repos")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("visibility".into(), "all".into()),
            Matcher::UrlEncoded(
                "affiliation".into(),
                "owner,collaborator,organization_member".into(),
            ),
            Matcher::UrlEncoded("per_page".into(), "30".into()),
            Matcher::UrlEncoded("page".into(), "1".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[
              {
                "name": "alpha-rust",
                "full_name": "me/alpha-rust",
                "description": "Rust private project",
                "stargazers_count": 1,
                "language": "Rust",
                "clone_url": "https://github.com/me/alpha-rust.git",
                "owner": { "login": "me" },
                "default_branch": "main"
              },
              {
                "name": "beta-js",
                "full_name": "me/beta-js",
                "description": "JavaScript project",
                "stargazers_count": 2,
                "language": "JavaScript",
                "clone_url": "https://github.com/me/beta-js.git",
                "owner": { "login": "me" },
                "default_branch": "main"
              }
            ]"#,
        )
        .create();

    with_api_base(&server.url(), || {
        let client = GitHubClient::new(Some("token")).expect("client");
        let repos = client
            .search_repositories_page("@me language:rust private", 1, 30)
            .expect("search");
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].full_name, "me/alpha-rust");
    });
}

#[test]
#[serial]
fn me_query_returns_error_on_unauthorized() {
    let mut server = Server::new();
    let _m = server
        .mock("GET", "/user/repos")
        .match_query(Matcher::Any)
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Bad credentials"}"#)
        .create();

    with_api_base(&server.url(), || {
        let client = GitHubClient::new(None).expect("client");
        let err = client
            .search_repositories_page("@me", 1, 30)
            .expect_err("must fail");
        assert!(
            err.to_string().contains("requires a valid token/session"),
            "unexpected error: {err}"
        );
    });
}
