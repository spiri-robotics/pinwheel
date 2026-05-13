use sqlx::postgres::Postgres;
use sqlx::Pool;
use uuid::Uuid;
use windmill_common::jobs::{JobPayload, RawCode};
use windmill_common::scripts::ScriptLang;
use windmill_test_utils::*;

// ============================================================================
// Basic Execution Tests
// ============================================================================

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_simple(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"
export function main() {
    return "hello world";
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!("hello world"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_with_args(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"
export function main(name: string, count: number) {
    return `Hello ${name}, count: ${count}`;
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = RunJob::from(job)
        .arg("name", serde_json::json!("World"))
        .arg("count", serde_json::json!(42))
        .run_until_complete(&db, false, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!("Hello World, count: 42"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_return_types(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Test object return
    {
        let content = r#"
export function main() {
    return { name: "test", value: 123 };
}
"#
        .to_owned();

        let job = JobPayload::Code(RawCode {
            hash: None,
            content,
            path: None,
            language: ScriptLang::Bun,
            lock: None,
            concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default(
            )
            .into(),
            debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
            cache_ttl: None,
            cache_ignore_s3_path: None,
            dedicated_worker: None,
            modules: None,
            tag: None,
        });

        let result = run_job_in_new_worker_until_complete(&db, false, job, port)
            .await
            .json_result()
            .unwrap();

        assert_eq!(result, serde_json::json!({"name": "test", "value": 123}));
    }

    // Test array return
    {
        let content = r#"
export function main() {
    return [1, 2, 3, "four"];
}
"#
        .to_owned();

        let job = JobPayload::Code(RawCode {
            hash: None,
            content,
            path: None,
            language: ScriptLang::Bun,
            lock: None,
            concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default(
            )
            .into(),
            debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
            cache_ttl: None,
            cache_ignore_s3_path: None,
            dedicated_worker: None,
            modules: None,
            tag: None,
        });

        let result = run_job_in_new_worker_until_complete(&db, false, job, port)
            .await
            .json_result()
            .unwrap();

        assert_eq!(result, serde_json::json!([1, 2, 3, "four"]));
    }

    // Test BigInt serialization
    {
        let content = r#"
export function main() {
    // Use BigInt literal notation to avoid JavaScript number precision loss
    return 9007199254740993n;
}
"#
        .to_owned();

        let job = JobPayload::Code(RawCode {
            hash: None,
            content,
            path: None,
            language: ScriptLang::Bun,
            lock: None,
            concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default(
            )
            .into(),
            debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
            cache_ttl: None,
            cache_ignore_s3_path: None,
            dedicated_worker: None,
            modules: None,
            tag: None,
        });

        let result = run_job_in_new_worker_until_complete(&db, false, job, port)
            .await
            .json_result()
            .unwrap();

        // BigInt should be serialized as string
        assert_eq!(result, serde_json::json!("9007199254740993"));
    }

    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_async(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"
export async function main() {
    await new Promise(resolve => setTimeout(resolve, 100));
    return "async completed";
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!("async completed"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_null_undefined_handling(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Test null return
    {
        let content = r#"
export function main() {
    return null;
}
"#
        .to_owned();

        let job = JobPayload::Code(RawCode {
            hash: None,
            content,
            path: None,
            language: ScriptLang::Bun,
            lock: None,
            concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default(
            )
            .into(),
            debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
            cache_ttl: None,
            cache_ignore_s3_path: None,
            dedicated_worker: None,
            modules: None,
            tag: None,
        });

        let result = run_job_in_new_worker_until_complete(&db, false, job, port)
            .await
            .json_result()
            .unwrap();

        assert_eq!(result, serde_json::json!(null));
    }

    // Test undefined return (should be serialized as null)
    {
        let content = r#"
export function main() {
    return undefined;
}
"#
        .to_owned();

        let job = JobPayload::Code(RawCode {
            hash: None,
            content,
            path: None,
            language: ScriptLang::Bun,
            lock: None,
            concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default(
            )
            .into(),
            debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
            cache_ttl: None,
            cache_ignore_s3_path: None,
            dedicated_worker: None,
            modules: None,
            tag: None,
        });

        let result = run_job_in_new_worker_until_complete(&db, false, job, port)
            .await
            .json_result()
            .unwrap();

        assert_eq!(result, serde_json::json!(null));
    }

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_runtime_error(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"
export function main() {
    throw new Error("intentional error");
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let completed = run_job_in_new_worker_until_complete(&db, false, job, port).await;

    assert!(!completed.success);
    let result = completed.json_result().unwrap();
    // Error is wrapped: {"error": {"message": "...", "name": "...", "stack": "..."}}
    let error = &result["error"];
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("intentional error"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_missing_main_function(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"
export function notMain() {
    return "hello";
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let completed = run_job_in_new_worker_until_complete(&db, false, job, port).await;

    assert!(!completed.success);
    let result = completed.json_result().unwrap();
    // Error is wrapped: {"error": {"message": "...", "name": "...", "stack": "..."}}
    let error = &result["error"];
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("main function is missing"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_syntax_error(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"
export function main() {
    return "unclosed string
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let completed = run_job_in_new_worker_until_complete(&db, false, job, port).await;

    assert!(!completed.success);
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_syntax_error_unclosed_bracket(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Reproduces the "Unexpected end of file at main.ts:0" error reported
    // when a TS file has a missing closing bracket — Bun's bundler gives no
    // useful location info.
    let content = r#"
export async function main() {
    if (true) {
        return "hello";
    // missing closing bracket for the function
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let completed = run_job_in_new_worker_until_complete(&db, false, job, port).await;

    assert!(!completed.success);
    let result = completed
        .result
        .as_ref()
        .and_then(|v| v.get("error"))
        .and_then(|v| v.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        result.contains("Unexpected end of file"),
        "should contain bun parser error, got: {result}"
    );
    assert!(
        result.contains("syntax error"),
        "should contain helpful hint about syntax errors, got: {result}"
    );
    Ok(())
}

// ============================================================================
// Annotation Mode Tests
// ============================================================================

#[sqlx::test(fixtures("base"))]
async fn test_bun_nodejs_mode(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"//nodejs

export function main() {
    // Node.js specific API
    return process.version.startsWith("v");
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!(true));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_bun_nobundling_mode(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"//nobundling

export function main() {
    return "nobundling works";
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!("nobundling works"));
    Ok(())
}

/// Regression test: a `//nobundling` script that pulls a package whose CJS
/// internals do bare-specifier `require()` of a sibling dependency.
///
/// Before the `--preserve-symlinks` fix, Bun 1.2/1.3+ would follow the
/// directory symlink in `node_modules/@langchain/core` to its global cache
/// entry, walk parent dirs from the cache realpath, and fail to find
/// `node_modules/zod` — producing:
///   ENOENT while resolving package 'zod/v3' from
///   '.../cache_nomount/bun/@langchain/core@<ver>@@@1/dist/runnables/base.js'
///
/// The fix passes `--preserve-symlinks` so Bun resolves from the
/// symlink path under `<job_dir>/node_modules/`, where `zod` is a sibling.
#[sqlx::test(fixtures("base"))]
async fn test_bun_nobundling_transitive_require(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"//nobundling
import { ChatPromptTemplate } from "@langchain/core/prompts";

export async function main() {
    const tpl = ChatPromptTemplate.fromMessages([
        ["system", "you are a {role}"],
        ["human", "{input}"],
    ]);
    const out = await tpl.formatMessages({ role: "tester", input: "ping" });
    return out.length;
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!(2));
    Ok(())
}

// ============================================================================
// Native Mode Tests (requires deno_core feature)
// ============================================================================

#[cfg(feature = "deno_core")]
#[sqlx::test(fixtures("base"))]
async fn test_bun_native_mode(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"//native

export function main() {
    return "native execution";
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!("native execution"));
    Ok(())
}

// ============================================================================
// Relative Import Tests
// ============================================================================

#[sqlx::test(fixtures("base", "relative_bun"))]
async fn test_bun_relative_imports(db: Pool<Postgres>) -> anyhow::Result<()> {
    let content = r#"
import { main as test1 } from "/f/system/same_folder_script.ts";
import { main as test2 } from "./same_folder_script.ts";
import { main as test3 } from "/f/system_relative/different_folder_script.ts";
import { main as test4 } from "../system_relative/different_folder_script.ts";

export function main() {
    return [test1(), test2(), test3(), test4()];
}
"#
    .to_string();

    run_deployed_relative_imports(&db, content.clone(), ScriptLang::Bun).await?;
    run_preview_relative_imports(&db, content, ScriptLang::Bun).await?;
    Ok(())
}

#[sqlx::test(fixtures("base", "relative_bun"))]
async fn test_bun_nested_imports(db: Pool<Postgres>) -> anyhow::Result<()> {
    // Test with absolute path (/f/...)
    let content_absolute = r#"
import { main as test } from "/f/system_relative/nested_script.ts";

export function main() {
    return test();
}
"#
    .to_string();

    run_deployed_relative_imports(&db, content_absolute.clone(), ScriptLang::Bun).await?;
    run_preview_relative_imports(&db, content_absolute, ScriptLang::Bun).await?;

    // Test with relative path (../...)
    let content_relative = r#"
import { main as test } from "../system_relative/nested_script.ts";

export function main() {
    return test();
}
"#
    .to_string();

    run_preview_relative_imports(&db, content_relative, ScriptLang::Bun).await?;
    Ok(())
}

// ============================================================================
// Deeply Nested Import Tests
// ============================================================================

#[sqlx::test(fixtures("base", "bun_edge_cases"))]
async fn test_bun_deeply_nested_imports(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Test with absolute path import (/f/nested/level1.ts)
    // Note: The fixture uses relative imports internally (level1 -> ./level2 -> ./level3)
    {
        let content = r#"
import { main as level1 } from "/f/nested/level1.ts";

export function main() {
    return level1();
}
"#
        .to_owned();

        let job = JobPayload::Code(RawCode {
            hash: None,
            content,
            path: Some("f/nested/test_deep".to_string()),
            language: ScriptLang::Bun,
            lock: None,
            concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default(
            )
            .into(),
            debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
            cache_ttl: None,
            cache_ignore_s3_path: None,
            dedicated_worker: None,
            modules: None,
            tag: None,
        });

        let result = run_job_in_new_worker_until_complete(&db, false, job, port)
            .await
            .json_result()
            .unwrap();

        // level1 -> level2 -> level3, each adds to the chain
        assert_eq!(result, serde_json::json!("level1 -> level2 -> level3"));
    }

    // Test with relative path import (./level1.ts)
    {
        let content = r#"
import { main as level1 } from "./level1.ts";

export function main() {
    return level1();
}
"#
        .to_owned();

        let job = JobPayload::Code(RawCode {
            hash: None,
            content,
            path: Some("f/nested/test_deep_relative".to_string()),
            language: ScriptLang::Bun,
            lock: None,
            concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default(
            )
            .into(),
            debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
            cache_ttl: None,
            cache_ignore_s3_path: None,
            dedicated_worker: None,
            modules: None,
            tag: None,
        });

        let result = run_job_in_new_worker_until_complete(&db, false, job, port)
            .await
            .json_result()
            .unwrap();

        // Same result: level1 -> level2 -> level3
        assert_eq!(result, serde_json::json!("level1 -> level2 -> level3"));
    }

    Ok(())
}

#[sqlx::test(fixtures("base", "bun_edge_cases"))]
async fn test_bun_shared_imports_both_styles(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Test importing modules that use different import styles internally:
    // - module_a uses relative import: ./shared.ts
    // - module_b uses absolute import: /f/circular/shared.ts
    // Both should work correctly
    let content = r#"
import { getValue as getA } from "/f/circular/module_a.ts";
import { getValue as getB } from "./module_b.ts";

export function main() {
    return [getA(), getB()];
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: Some("f/circular/test_both".to_string()),
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    // Both modules should correctly import SHARED_VALUE
    assert_eq!(
        result,
        serde_json::json!(["from_a_shared", "from_b_shared"])
    );
    Ok(())
}

// ============================================================================
// Preprocessor Tests
// ============================================================================

#[sqlx::test(fixtures("base"))]
async fn test_bun_preprocessor_execution(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Test that the main function works correctly
    // Note: Preprocessor execution requires specific job configuration
    // (flow_step_id != "preprocessor" and preprocessed == Some(false))
    // which is not set by default in RawCode jobs
    let content = r#"
export function main(x: number) {
    return x + 10;
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    // x=5, main adds 10 = 15
    let result = RunJob::from(job)
        .arg("x", serde_json::json!(5))
        .run_until_complete(&db, false, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!(15));
    Ok(())
}

// ============================================================================
// Wmill SDK Tests
// ============================================================================

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_with_wmill_env_vars(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Test accessing Windmill environment variables (which the SDK uses internally)
    // This validates that the execution environment is properly configured
    let content = r#"
export function main() {
    // WM_WORKSPACE and WM_TOKEN are injected by Windmill worker
    return {
        workspace: process.env.WM_WORKSPACE,
        hasToken: process.env.WM_TOKEN !== undefined && process.env.WM_TOKEN.length > 0,
        baseUrl: process.env.BASE_URL !== undefined
    };
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result["workspace"], serde_json::json!("test-workspace"));
    assert_eq!(result["hasToken"], serde_json::json!(true));
    assert_eq!(result["baseUrl"], serde_json::json!(true));
    Ok(())
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

#[sqlx::test(fixtures("base"))]
async fn test_bun_job_env_vars(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let content = r#"
export function main() {
    return {
        workspace: process.env.WM_WORKSPACE,
        hasToken: !!process.env.WM_TOKEN,
    };
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = run_job_in_new_worker_until_complete(&db, false, job, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result["workspace"], serde_json::json!("test-workspace"));
    assert_eq!(result["hasToken"], serde_json::json!(true));
    Ok(())
}

// ============================================================================
// Bundle Wrapper Safety Tests
// ============================================================================

/// Regression test for the "TS source ends up in the bun bundle cache" bug.
///
/// The wrapper-side hardening: `node_builder.ts` discarded `Bun.build`'s
/// return value, so any silent-failure mode (`success: false` without
/// throwing — `throw: false`, or a future Bun where defaults change) made
/// the wrapper exit 0 even though no `main.js` was written. Pair that with
/// a pre-existing `main.js` containing raw TypeScript and `save_cache`
/// happily copied that TS into the bundle cache; the worker later choked
/// on `type GpgKey = {`.
///
/// This test patches `node_builder.ts` to force the silent-failure shape
/// and asserts that our wrapper now refuses to silently succeed — bun must
/// exit non-zero so prebundling fails loudly instead of writing TypeScript
/// into the bundle cache.
#[test]
fn test_bun_bundle_wrapper_catches_silent_failure() {
    use std::process::Command;
    use windmill_worker::{build_loader, LoaderMode, BUN_PATH};

    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let dir_str = dir.to_str().unwrap();

    // Script imports a package that won't exist in node_modules.
    std::fs::write(
        dir.join("main.ts"),
        r#"
import x from "definitely-not-a-real-pkg-windmill-test";
export function main() { return x; }
"#,
    )
    .unwrap();

    // Generate the real node_builder.ts via the production code path.
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(build_loader(
            dir_str,
            "http://localhost:8000",
            "test_token",
            "test-workspace",
            "f/test/script",
            LoaderMode::BunBundle,
            &None,
        ))
        .expect("build_loader failed");

    // Force the silent-failure shape by injecting `throw: false`. The
    // wrapper's pre-fix `try/catch` would have swallowed this; the fixed
    // wrapper inspects `result.success` and `result.outputs` and exits 1.
    let path = dir.join("node_builder.ts");
    let original = std::fs::read_to_string(&path).unwrap();
    let patched = original.replace(
        "external: [\"electron\"],",
        "external: [\"electron\"], throw: false,",
    );
    assert_ne!(
        original, patched,
        "expected to find Bun.build options block to patch; node_builder.ts template changed?"
    );
    std::fs::write(&path, patched).unwrap();

    // Pre-seed main.js with raw TypeScript (mimics the historical
    // pre-write that originally seeded the bug).
    std::fs::write(
        dir.join("main.js"),
        "type GpgKey = { email: string };\nexport const main = (): GpgKey => ({ email: \"\" });\n",
    )
    .unwrap();

    let output = Command::new(BUN_PATH.as_str())
        .args(["run", path.to_str().unwrap()])
        .current_dir(dir)
        .output()
        .expect("Failed to run bun");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "node_builder.ts must exit non-zero when Bun.build silently fails to write a bundle.\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Failed to build node bundle"),
        "expected diagnostic in stdout, got:\n{stdout}"
    );
}

/// Regression test for the actual root cause of the "TS source in bundle
/// cache" bug: `generate_bun_bundle` was awaiting `child_process.wait()`
/// without checking the exit code on the no-DB path (used by Docker-build
/// `windmill cache hubPaths.json`). bun would exit 1 after Bun.build threw,
/// `wait().await?` propagated only IO errors, and `generate_bun_bundle`
/// returned `Ok(())`. `save_cache` then copied a stale `main.js` (raw TS
/// source) straight into the bundle cache.
///
/// This test runs `generate_bun_bundle` with `db: None` against a `node_builder.ts`
/// that calls `process.exit(1)`, and asserts the function now returns an error.
#[test]
fn test_generate_bun_bundle_propagates_exit_status() {
    use windmill_worker::{generate_bun_bundle, get_common_bun_proc_envs};

    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let dir_str = dir.to_str().unwrap();

    // node_builder.ts that exits 1, mimicking what bun does when Bun.build throws.
    std::fs::write(
        dir.join("node_builder.ts"),
        "console.log('simulated bun build failure');\nprocess.exit(1);\n",
    )
    .unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let envs = runtime.block_on(get_common_bun_proc_envs(None));

    let result = runtime.block_on(generate_bun_bundle(
        dir_str,
        "test-workspace",
        &uuid::Uuid::new_v4(),
        "test-worker",
        None, // db: None — this is the cache_hub_scripts path that had the bug
        None,
        &mut 0,
        &mut None,
        &envs,
        &mut None,
    ));

    assert!(
        result.is_err(),
        "generate_bun_bundle must surface bun's non-zero exit on the no-DB path. \
         If it returns Ok(()) when bun exited 1, save_cache will silently cache stale main.js content."
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("non-zero status"),
        "expected exit-status error, got: {err_msg}"
    );
}

/// Regression test for the install_bun_lockfile no-DB path: same code shape as
/// `generate_bun_bundle` (site 3 of the original bug) — `wait().await?` ignored
/// non-zero bun exits. A `bun install` failure (e.g. malformed package.json)
/// must now surface as an error so callers don't proceed with a half-installed
/// node_modules.
#[test]
fn test_install_bun_lockfile_propagates_exit_status() {
    use windmill_worker::{get_common_bun_proc_envs, install_bun_lockfile};
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let dir_str = dir.to_str().unwrap();
    // Malformed package.json -> bun install fails with exit 1.
    std::fs::write(dir.join("package.json"), "this is not valid json").unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let envs = runtime.block_on(get_common_bun_proc_envs(None));
    let result = runtime.block_on(install_bun_lockfile(
        &mut 0,
        &mut None,
        &uuid::Uuid::new_v4(),
        "test-workspace",
        None, // db: None — no-DB path that had the bug
        dir_str,
        "test-worker",
        envs,
        false, // npm_mode
        &mut None,
        true, // quiet
    ));
    assert!(
        result.is_err(),
        "install_bun_lockfile must surface bun's non-zero exit on the no-DB path"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("non-zero status"),
        "expected exit-status error, got: {err_msg}"
    );
}

/// Regression test for the post-bundle existence check in `prebundle_bun_script`
/// and `handle_bun_job`. Both call sites guard against the case where
/// `generate_bun_bundle` returns `Ok(())` but `main.js` was never written —
/// the upstream wait-status fix is the primary defense, this is the catch-all
/// for any other silent-failure mode (Bun output-naming change, custom plugin
/// swallowing the build, etc.). Without this check, `save_cache` would
/// happily copy whatever's at the bundle path (often raw TypeScript that some
/// other code path left there).
#[test]
fn test_ensure_bundle_output_exists_rejects_missing_file() {
    use windmill_worker::ensure_bundle_output_exists;
    let temp_dir = tempfile::tempdir().unwrap();
    let dir = temp_dir.path();
    let missing = dir.join("main.js").to_str().unwrap().to_string();

    let result = ensure_bundle_output_exists(&missing);
    assert!(
        result.is_err(),
        "ensure_bundle_output_exists must reject when the bundle file is missing"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("bun bundle output missing"),
        "expected 'bun bundle output missing' in error, got: {err_msg}"
    );

    // Sanity: when the file does exist, it returns Ok.
    std::fs::write(&missing, "// @bun\n").unwrap();
    assert!(ensure_bundle_output_exists(&missing).is_ok());
}

// ============================================================================
// Dedicated Worker Protocol Tests
// ============================================================================


// ============================================================================
// Deno Dedicated Worker Protocol Tests
// ============================================================================


// ============================================================================
// Private Registry Tests
// ============================================================================

/// Test that bun can install packages from a private npm registry with authentication.
/// The registry requires auth tokens for accessing @windmill-test/* packages.
/// Requires:
/// - `private_registry_test` feature enabled
/// - `TEST_NPM_REGISTRY` environment variable set to registry URL with auth token
///   Format: `http://registry-url/:_authToken=TOKEN`
#[cfg(feature = "private_registry_test")]
#[sqlx::test(fixtures("base"))]
async fn test_bun_job_private_npm_registry(db: Pool<Postgres>) -> anyhow::Result<()> {
    use windmill_worker::NPM_CONFIG_REGISTRY;

    let registry_url = std::env::var("TEST_NPM_REGISTRY")
        .expect("TEST_NPM_REGISTRY must be set when running private_registry_test");

    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    // Set the private registry configuration
    {
        let mut registry = NPM_CONFIG_REGISTRY.write().await;
        *registry = Some(registry_url.clone());
    }

    let content = r#"
import { greet } from "@windmill-test/private-pkg";

export function main(name: string) {
    return greet(name);
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = RunJob::from(job)
        .arg("name", serde_json::json!("World"))
        .run_until_complete(&db, false, port)
        .await
        .json_result()
        .unwrap();

    // Clean up
    {
        let mut registry = NPM_CONFIG_REGISTRY.write().await;
        *registry = None;
    }

    assert_eq!(
        result,
        serde_json::json!("Hello from private package, World!")
    );
    Ok(())
}

/// Test that full .npmrc content works for bun jobs with private registries.
/// Requires:
/// - `TEST_NPMRC` environment variable set to the full .npmrc content
#[cfg(feature = "private_registry_test")]
#[sqlx::test(fixtures("base"))]
async fn test_bun_job_private_npmrc(db: Pool<Postgres>) -> anyhow::Result<()> {
    use windmill_worker::NPMRC;

    let npmrc_content = std::env::var("TEST_NPMRC")
        .expect("TEST_NPMRC must be set when running private_registry_test");

    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    {
        let mut npmrc = NPMRC.write().await;
        *npmrc = Some(npmrc_content.clone());
    }

    let content = r#"
import { greet } from "@windmill-test/private-pkg";

export function main(name: string) {
    return greet(name);
}
"#
    .to_owned();

    let job = JobPayload::Code(RawCode {
        hash: None,
        content,
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: windmill_common::runnable_settings::ConcurrencySettings::default()
            .into(),
        debouncing_settings: windmill_common::runnable_settings::DebouncingSettings::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = RunJob::from(job)
        .arg("name", serde_json::json!("World"))
        .run_until_complete(&db, false, port)
        .await
        .json_result()
        .unwrap();

    {
        let mut npmrc = NPMRC.write().await;
        *npmrc = None;
    }

    assert_eq!(
        result,
        serde_json::json!("Hello from private package, World!")
    );
    Ok(())
}

/// Tests for RELATIVE_BUN_BUILDER (loader_builder.bun.js)
/// These tests verify Bun's behavior for import scanning and package.json generation.
/// Purpose: Catch regressions when upgrading Bun versions.
mod bun_builder_tests {
    use std::process::{Command, Stdio};
    use windmill_worker::{BUN_PATH, RELATIVE_BUN_BUILDER, RELATIVE_BUN_LOADER};

    /// Run the builder and return the generated package.json content
    fn run_builder(main_ts_content: &str) -> serde_json::Value {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();

        // Write main.ts
        std::fs::write(dir.join("main.ts"), main_ts_content).unwrap();

        // Write build.js using the loader and builder constants directly
        // Parameters are dummy values since tests don't use Windmill relative imports
        let loader = RELATIVE_BUN_LOADER
            .replace("TEMP_SCRIPT_REFS_PLACEHOLDER", "{}")
            .replace("W_ID", "test-workspace")
            .replace("BASE_INTERNAL_URL", "http://localhost:8000")
            .replace("TOKEN", "test-token")
            .replace("CURRENT_PATH", "f/test/script")
            .replace("RAW_GET_ENDPOINT", "raw");

        let build_script = format!(
            r#"
{loader}

{RELATIVE_BUN_BUILDER}
"#
        );
        std::fs::write(dir.join("build.js"), build_script).unwrap();

        // Run bun build.js
        let output = Command::new(BUN_PATH.as_str())
            .args(["run", "build.js"])
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to run bun");

        if !output.status.success() {
            panic!(
                "Builder failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Read generated package.json
        let package_json =
            std::fs::read_to_string(dir.join("package.json")).expect("package.json not generated");

        serde_json::from_str(&package_json).expect("Invalid JSON in package.json")
    }

    /// Test: scanImports() detects basic imports
    #[test]
    fn test_builder_simple_import() {
        let main_ts = r#"
import lodash from "lodash";
export function main() { return lodash; }
"#;
        let pkg = run_builder(main_ts);
        let deps = pkg["dependencies"].as_object().unwrap();

        assert!(
            deps.contains_key("lodash"),
            "lodash should be in dependencies"
        );
        assert_eq!(deps["lodash"], "latest");
    }

    /// Test: scanImports() preserves version info from versioned imports
    #[test]
    fn test_builder_versioned_import() {
        let main_ts = r#"
import _ from "lodash@4.17.21";
export function main() { return _; }
"#;
        let pkg = run_builder(main_ts);
        let deps = pkg["dependencies"].as_object().unwrap();

        assert!(
            deps.contains_key("lodash"),
            "lodash should be in dependencies"
        );
        assert_eq!(deps["lodash"], "4.17.21");
    }

    /// Test: scanImports() handles @scope/package correctly
    #[test]
    fn test_builder_scoped_package() {
        let main_ts = r#"
import babel from "@babel/core";
export function main() { return babel; }
"#;
        let pkg = run_builder(main_ts);
        let deps = pkg["dependencies"].as_object().unwrap();

        assert!(
            deps.contains_key("@babel/core"),
            "@babel/core should be in dependencies"
        );
        assert_eq!(deps["@babel/core"], "latest");
    }

    /// Test: scanImports() handles multiple imports
    #[test]
    fn test_builder_multiple_packages() {
        let main_ts = r#"
import lodash from "lodash";
import axios from "axios";
import dayjs from "dayjs";
export function main() { return { lodash, axios, dayjs }; }
"#;
        let pkg = run_builder(main_ts);
        let deps = pkg["dependencies"].as_object().unwrap();

        assert!(
            deps.contains_key("lodash"),
            "lodash should be in dependencies"
        );
        assert!(
            deps.contains_key("axios"),
            "axios should be in dependencies"
        );
        assert!(
            deps.contains_key("dayjs"),
            "dayjs should be in dependencies"
        );
        assert_eq!(deps.len(), 3, "Should have exactly 3 dependencies");
    }

    /// Test: isBuiltin() filters out Node.js builtins
    #[test]
    fn test_builder_builtin_skipped() {
        let main_ts = r#"
import fs from "fs";
import path from "path";
import lodash from "lodash";
export function main() { return { fs, path, lodash }; }
"#;
        let pkg = run_builder(main_ts);
        let deps = pkg["dependencies"].as_object().unwrap();

        assert!(
            !deps.contains_key("fs"),
            "fs (builtin) should NOT be in dependencies"
        );
        assert!(
            !deps.contains_key("path"),
            "path (builtin) should NOT be in dependencies"
        );
        assert!(
            deps.contains_key("lodash"),
            "lodash should be in dependencies"
        );
        assert_eq!(
            deps.len(),
            1,
            "Should have exactly 1 dependency (lodash only)"
        );
    }

    /// Test: semver.order() resolves version conflicts (picks lowest version)
    #[test]
    fn test_builder_version_conflict() {
        // This test simulates what happens when the same package is imported with different versions
        // The builder should use semver.order() to pick the lowest version
        let main_ts = r#"
import a from "lodash@4.17.21";
import b from "lodash@4.17.10";
export function main() { return { a, b }; }
"#;
        let pkg = run_builder(main_ts);
        let deps = pkg["dependencies"].as_object().unwrap();

        assert!(
            deps.contains_key("lodash"),
            "lodash should be in dependencies"
        );
        // The builder sorts by semver and picks the first (lowest) version
        assert_eq!(
            deps["lodash"], "4.17.10",
            "Should resolve to lower version 4.17.10"
        );
    }
}

// ============================================================================
// Codebase Mode Tests
// ============================================================================

/// Create a TAR archive in memory containing a single `main.js` file.
fn create_codebase_tar(main_js_content: &str) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    let content = main_js_content.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_path("main.js").unwrap();
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, content).unwrap();
    builder.into_inner().unwrap()
}

/// Place a TAR codebase at the expected cache path for the given job ID and hash.
fn place_codebase_in_cache(job_id: &Uuid, tar_bytes: &[u8], is_esm: bool) {
    let codebase_id = if is_esm {
        format!("{}.esm.tar", job_id)
    } else {
        format!("{}.tar", job_id)
    };
    let bundle_path = format!("script_bundle/test-workspace/{}", codebase_id);
    let cache_path = format!(
        "{}/{}.tar",
        *windmill_common::worker::ROOT_CACHE_NOMOUNT_DIR,
        bundle_path,
    );
    let parent = std::path::Path::new(&cache_path).parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    std::fs::write(&cache_path, tar_bytes).unwrap();
}

#[sqlx::test(fixtures("base"))]
async fn test_cjs_codebase_tar(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let main_js = r#"
module.exports.main = function() {
    return "cjs codebase ok";
};
"#;
    let inner_content = r#"export function main() { return "cjs codebase ok"; }"#;

    let job_id = Uuid::new_v4();
    let tar_bytes = create_codebase_tar(main_js);
    place_codebase_in_cache(&job_id, &tar_bytes, false);

    let job = JobPayload::Code(RawCode {
        hash: Some(-43), // PREVIEW_IS_TAR_CODEBASE_HASH
        content: inner_content.to_string(),
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: Default::default(),
        debouncing_settings: Default::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = RunJob::from(job)
        .job_id(job_id)
        .run_until_complete(&db, false, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!("cjs codebase ok"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_esm_codebase_tar(db: Pool<Postgres>) -> anyhow::Result<()> {
    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let main_js = r#"
export function main() {
    return "esm codebase ok";
}
"#;
    let inner_content = r#"export function main() { return "esm codebase ok"; }"#;

    let job_id = Uuid::new_v4();
    let tar_bytes = create_codebase_tar(main_js);
    place_codebase_in_cache(&job_id, &tar_bytes, true);

    let job = JobPayload::Code(RawCode {
        hash: Some(-45), // PREVIEW_IS_TAR_ESM_CODEBASE_HASH
        content: inner_content.to_string(),
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: Default::default(),
        debouncing_settings: Default::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    let result = RunJob::from(job)
        .job_id(job_id)
        .run_until_complete(&db, false, port)
        .await
        .json_result()
        .unwrap();

    assert_eq!(result, serde_json::json!("esm codebase ok"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_cjs_codebase_tar_nsjail(db: Pool<Postgres>) -> anyhow::Result<()> {
    if std::process::Command::new("nsjail")
        .arg("--help")
        .output()
        .is_err()
    {
        eprintln!("nsjail not found, skipping test");
        return Ok(());
    }

    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let main_js = r#"
module.exports.main = function() {
    return "cjs nsjail ok";
};
"#;
    let inner_content = r#"export function main() { return "cjs nsjail ok"; }"#;

    let job_id = Uuid::new_v4();
    let tar_bytes = create_codebase_tar(main_js);
    place_codebase_in_cache(&job_id, &tar_bytes, false);

    let job = JobPayload::Code(RawCode {
        hash: Some(-43),
        content: inner_content.to_string(),
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: Default::default(),
        debouncing_settings: Default::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    use std::sync::atomic::Ordering;
    windmill_worker::JOB_ISOLATION.store(
        windmill_worker::JobIsolationLevel::NsjailSandboxing as u8,
        Ordering::Relaxed,
    );

    let result = RunJob::from(job)
        .job_id(job_id)
        .run_until_complete(&db, false, port)
        .await;

    windmill_worker::JOB_ISOLATION.store(
        windmill_worker::JobIsolationLevel::Undefined as u8,
        Ordering::Relaxed,
    );

    let json = result.json_result().unwrap();
    assert_eq!(json, serde_json::json!("cjs nsjail ok"));
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn test_esm_codebase_tar_nsjail(db: Pool<Postgres>) -> anyhow::Result<()> {
    if std::process::Command::new("nsjail")
        .arg("--help")
        .output()
        .is_err()
    {
        eprintln!("nsjail not found, skipping test");
        return Ok(());
    }

    initialize_tracing().await;
    let server = ApiServer::start(db.clone()).await?;
    let port = server.addr.port();

    let main_js = r#"
export function main() {
    return "esm nsjail ok";
}
"#;
    let inner_content = r#"export function main() { return "esm nsjail ok"; }"#;

    let job_id = Uuid::new_v4();
    let tar_bytes = create_codebase_tar(main_js);
    place_codebase_in_cache(&job_id, &tar_bytes, true);

    let job = JobPayload::Code(RawCode {
        hash: Some(-45),
        content: inner_content.to_string(),
        path: None,
        language: ScriptLang::Bun,
        lock: None,
        concurrency_settings: Default::default(),
        debouncing_settings: Default::default(),
        cache_ttl: None,
        cache_ignore_s3_path: None,
        dedicated_worker: None,
        modules: None,
        tag: None,
    });

    use std::sync::atomic::Ordering;
    windmill_worker::JOB_ISOLATION.store(
        windmill_worker::JobIsolationLevel::NsjailSandboxing as u8,
        Ordering::Relaxed,
    );

    let result = RunJob::from(job)
        .job_id(job_id)
        .run_until_complete(&db, false, port)
        .await;

    windmill_worker::JOB_ISOLATION.store(
        windmill_worker::JobIsolationLevel::Undefined as u8,
        Ordering::Relaxed,
    );

    let json = result.json_result().unwrap();
    assert_eq!(json, serde_json::json!("esm nsjail ok"));
    Ok(())
}
