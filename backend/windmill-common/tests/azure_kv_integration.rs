/*
 * Author: Windmill Labs, Inc
 * Copyright (C) Windmill Labs, Inc - All Rights Reserved
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 */

//! Integration tests for Azure Key Vault secret backend
//!
//! These tests require a running Azure Key Vault emulator.
//!
//! ## Setup
//!
//! 1. Run the emulator (james-gould/azure-keyvault-emulator):
//!
//!    ```bash
//!    docker run -d -p 4997:4997 \
//!      -e Persist=true \
//!      --name azure-kv-emulator \
//!      jamesgoulddev/azure-keyvault-emulator:latest
//!    ```
//!
//! 2. The emulator uses HTTPS with a self-signed cert. You may need to either:
//!    - Trust the emulator's certificate, or
//!    - Set `AZURE_KV_ALLOW_INSECURE=true` to skip TLS verification (test only)
//!
//! 3. Run the tests:
//!
//!    ```bash
//!    RUN_AZURE_KV_TESTS=1 cargo test -p windmill-common --features private,enterprise \
//!      azure_kv_integration -- --nocapture
//!    ```
//!
//! ## Emulator authentication
//!
//! The emulator accepts any well-formed JWT token without verifying signatures.
//! A static dummy token is used for testing.

