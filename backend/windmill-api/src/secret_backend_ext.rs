/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2024
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

//! Secret backend extension for the API layer
//!
//! This module provides helper functions for integrating the SecretBackend
//! trait with variable operations in the API.
//!
//! Note: HashiCorp Vault integration requires Enterprise Edition.
//! The OSS version only supports the database backend.


use windmill_common::{
    db::DB,
    error::Result,
};











/// Bulk rename secrets in Vault when a path prefix changes (e.g., user rename)
/// EE only feature.
///
/// This is used when renaming users where many secrets need their paths updated.
/// Returns a list of (old_path, new_value) pairs for updating the database.
pub async fn rename_vault_secrets_with_prefix(
    _db: &DB,
    _workspace_id: &str,
    _old_prefix: &str,
    _new_prefix: &str,
    _variables: Vec<(String, String)>,
) -> Result<Vec<(String, String)>> {
    // OSS: No Vault support, return empty
    Ok(vec![])
}

