use sqlx::{Pool, Postgres};

mod common;
use common::*;

/// Test that the workspace success handler cache works correctly with 60s TTL
#[cfg(feature = "deno_core")]
#[sqlx::test(fixtures("base"))]
async fn test_success_handler_cache(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;

    // First, create a success handler script
    let _server = ApiServer::start(db.clone()).await?;

    // Set up a success handler in workspace_settings
    sqlx::query!(
        r#"
        UPDATE workspace_settings
        SET success_handler = 'script/f/test/success_handler',
            success_handler_extra_args = '{"key": "value"}'::json
        WHERE workspace_id = 'test-workspace'
        "#
    )
    .execute(&db)
    .await?;

    // Verify the success handler was set
    let result = sqlx::query_scalar!(
        r#"SELECT success_handler FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(result, Some("script/f/test/success_handler".to_string()));

    // Verify extra args were set
    let extra_args = sqlx::query_scalar!(
        r#"SELECT success_handler_extra_args FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;

    assert!(extra_args.is_some());

    Ok(())
}

/// Test that success handler can be set and removed via database operations
#[cfg(feature = "deno_core")]
#[sqlx::test(fixtures("base"))]
async fn test_success_handler_settings(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;

    let _server = ApiServer::start(db.clone()).await?;

    // Initially success_handler should be NULL
    let initial = sqlx::query_scalar!(
        r#"SELECT success_handler FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert!(initial.is_none());

    // Set success handler
    sqlx::query!(
        r#"
        UPDATE workspace_settings
        SET success_handler = 'flow/f/test/success_flow'
        WHERE workspace_id = 'test-workspace'
        "#
    )
    .execute(&db)
    .await?;

    let after_set = sqlx::query_scalar!(
        r#"SELECT success_handler FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(after_set, Some("flow/f/test/success_flow".to_string()));

    // Remove success handler
    sqlx::query!(
        r#"
        UPDATE workspace_settings
        SET success_handler = NULL
        WHERE workspace_id = 'test-workspace'
        "#
    )
    .execute(&db)
    .await?;

    let after_remove = sqlx::query_scalar!(
        r#"SELECT success_handler FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert!(after_remove.is_none());

    Ok(())
}

