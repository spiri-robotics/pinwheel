use sqlx::{Pool, Postgres};

mod common;
use common::*;

/// Test that workspace error handler can be set and removed via database operations
#[cfg(feature = "deno_core")]
#[sqlx::test(fixtures("base"))]
async fn test_error_handler_settings(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;

    let _server = ApiServer::start(db.clone()).await?;

    // Initially error_handler should be NULL
    let initial = sqlx::query_scalar!(
        r#"SELECT error_handler->>'path' FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert!(initial.is_none());

    // Set error handler with all options
    sqlx::query!(
        r#"
        UPDATE workspace_settings
        SET error_handler = '{"path": "script/f/test/error_handler", "extra_args": {"notify": true}, "muted_on_cancel": true, "muted_on_user_path": false}'::jsonb
        WHERE workspace_id = 'test-workspace'
        "#
    )
    .execute(&db)
    .await?;

    let after_set = sqlx::query_scalar!(
        r#"SELECT error_handler->>'path' FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(
        after_set,
        Some("script/f/test/error_handler".to_string())
    );

    // Verify extra_args
    let extra_args = sqlx::query_scalar!(
        r#"SELECT error_handler->'extra_args' FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert!(extra_args.is_some());

    // Verify muted_on_cancel
    let muted_on_cancel = sqlx::query_scalar!(
        r#"SELECT (error_handler->>'muted_on_cancel')::boolean FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(muted_on_cancel, Some(true));

    // Verify muted_on_user_path
    let muted_on_user_path = sqlx::query_scalar!(
        r#"SELECT (error_handler->>'muted_on_user_path')::boolean FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(muted_on_user_path, Some(false));

    // Remove error handler
    sqlx::query!(
        r#"
        UPDATE workspace_settings
        SET error_handler = NULL
        WHERE workspace_id = 'test-workspace'
        "#
    )
    .execute(&db)
    .await?;

    let after_remove = sqlx::query_scalar!(
        r#"SELECT error_handler->>'path' FROM workspace_settings WHERE workspace_id = 'test-workspace'"#
    )
    .fetch_one(&db)
    .await?;
    assert!(after_remove.is_none());

    Ok(())
}



