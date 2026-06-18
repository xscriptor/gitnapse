use gitnapse::auth;
use serial_test::serial;

#[test]
#[serial]
fn env_token_has_precedence_over_stored_sources() {
    temp_env::with_var("GITHUB_TOKEN", Some("env-priority-token"), || {
        let loaded = auth::load_token().expect("load token");
        assert_eq!(loaded.as_deref(), Some("env-priority-token"));
    });
}
