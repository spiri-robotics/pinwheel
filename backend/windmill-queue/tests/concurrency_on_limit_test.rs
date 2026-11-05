//! The concurrency limiter's fail mode on a full key (`apply_concurrency_limit`,
//! windmill-queue/jobs_ee.rs), driven through `pull` without a worker.
//!
//! Run with:
//!   cargo test -p windmill-queue --test concurrency_on_limit_test --features private,enterprise

