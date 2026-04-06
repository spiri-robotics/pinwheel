//! Integration tests for AWS Secrets Manager secret backend.
//!
//! These tests require a running LocalStack instance with the `secretsmanager` service.
//!
//! ## Setup
//!
//! 1. Start LocalStack:
//!
//!    ```bash
//!    docker run -d --name localstack -p 4566:4566 \
//!      -e SERVICES=secretsmanager \
//!      localstack/localstack:3.8
//!    ```
//!
//! 2. Run the tests:
//!
//!    ```bash
//!    RUN_AWS_SM_TESTS=1 cargo test -p windmill-common --features private,enterprise \
//!      aws_sm_integration -- --nocapture
//!    ```
//!
//! ## Environment variables
//!
//! - `RUN_AWS_SM_TESTS=1`     - Required to run the tests
//! - `AWS_SM_ENDPOINT`        - LocalStack endpoint (default: http://localhost:4566)
//! - `AWS_SM_REGION`          - AWS region (default: us-east-1)


mod tests {
    #[test]
    fn test_aws_sm_requires_enterprise() {
        println!("AWS Secrets Manager integration tests require Enterprise Edition features");
        println!("Run with: cargo test -p windmill-common --features private,enterprise");
    }
}
