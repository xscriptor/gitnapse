#![allow(dead_code)]

#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/oauth_session.rs"]
mod oauth_session;
#[path = "../src/secure_store.rs"]
mod secure_store;

use serial_test::serial;

#[test]
#[serial]
fn env_token_has_precedence_over_stored_sources() {
    let prev = std::env::var("GITHUB_TOKEN").ok();
    unsafe { std::env::set_var("GITHUB_TOKEN", "env-priority-token") };

    let loaded = auth::load_token().expect("load token");
    assert_eq!(loaded.as_deref(), Some("env-priority-token"));

    if let Some(value) = prev {
        unsafe { std::env::set_var("GITHUB_TOKEN", value) };
    } else {
        unsafe { std::env::remove_var("GITHUB_TOKEN") };
    }
}
