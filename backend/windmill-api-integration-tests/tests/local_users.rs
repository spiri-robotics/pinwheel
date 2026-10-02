use serde_json::json;
use sqlx::{Pool, Postgres};

use windmill_test_utils::*;

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

#[sqlx::test(migrations = "../migrations", fixtures("base"))]
async fn test_create_user_and_set_password(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();
    let users = format!("http://localhost:{port}/api/users");
    let auth = format!("http://localhost:{port}/api/auth");

    let login = |email: &'static str, password: &'static str| {
        client()
            .post(format!("{auth}/login"))
            .json(&json!({ "email": email, "password": password }))
            .send()
    };
    let create = |token: &'static str, body: serde_json::Value| {
        client()
            .post(format!("{users}/create"))
            .bearer_auth(token)
            .json(&body)
            .send()
    };

    // Only a superadmin creates users.
    let resp = create(
        "SECRET_TOKEN_2",
        json!({"email": "new@windmill.dev", "password": "pw", "super_admin": false}),
    )
    .await?;
    assert_ne!(resp.status(), 201);

    let resp = create(
        "SECRET_TOKEN",
        json!({"email": "New@Windmill.dev", "password": "first-pw", "super_admin": false,
               "name": "New User"}),
    )
    .await?;
    assert_eq!(resp.status(), 201, "create: {}", resp.text().await?);

    let (username, verified): (Option<String>, bool) =
        sqlx::query_as("SELECT username, verified FROM password WHERE email = 'new@windmill.dev'")
            .fetch_one(&db)
            .await?;
    assert_eq!(username.as_deref(), Some("new"));
    assert!(verified);

    assert_eq!(login("new@windmill.dev", "first-pw").await?.status(), 200);
    assert_eq!(login("new@windmill.dev", "wrong").await?.status(), 400);

    // Duplicates, missing passwords and LDAP accounts are refused.
    let resp = create(
        "SECRET_TOKEN",
        json!({"email": "new@windmill.dev", "password": "x", "super_admin": false}),
    )
    .await?;
    assert_eq!(resp.status(), 400);
    let resp = create(
        "SECRET_TOKEN",
        json!({"email": "nopw@windmill.dev", "super_admin": false}),
    )
    .await?;
    assert_eq!(resp.status(), 400);
    let resp = create(
        "SECRET_TOKEN",
        json!({"email": "dir@windmill.dev", "super_admin": false, "login_type": "ldap"}),
    )
    .await?;
    assert_eq!(resp.status(), 400);

    // A pending_oauth account has no credential until a password is set.
    let resp = create(
        "SECRET_TOKEN",
        json!({"email": "pending@windmill.dev", "super_admin": false,
               "login_type": "pending_oauth"}),
    )
    .await?;
    assert_eq!(resp.status(), 201, "create pending: {}", resp.text().await?);

    let resp = client()
        .post(format!("{users}/set_password_of/pending@windmill.dev"))
        .bearer_auth("SECRET_TOKEN")
        .json(&json!({"password": "now-set"}))
        .send()
        .await?;
    assert_eq!(resp.status(), 200, "set_password_of: {}", resp.text().await?);
    assert_eq!(login("pending@windmill.dev", "now-set").await?.status(), 200);

    // A user changes their own password.
    let resp = client()
        .post(format!("{users}/setpassword"))
        .bearer_auth("SECRET_TOKEN_2")
        .json(&json!({"password": "mine"}))
        .send()
        .await?;
    assert_eq!(resp.status(), 200, "setpassword: {}", resp.text().await?);
    assert_eq!(login("test2@windmill.dev", "mine").await?.status(), 200);

    // LDAP-backed passwords live in the directory.
    sqlx::query("UPDATE password SET login_type = 'ldap' WHERE email = 'test3@windmill.dev'")
        .execute(&db)
        .await?;
    let resp = client()
        .post(format!("{users}/set_password_of/test3@windmill.dev"))
        .bearer_auth("SECRET_TOKEN")
        .json(&json!({"password": "nope"}))
        .send()
        .await?;
    assert_eq!(resp.status(), 400);

    Ok(())
}
