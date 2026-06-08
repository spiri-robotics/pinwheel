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

use std::sync::Arc;

use windmill_common::{
    db::DB,
    error::{Error, Result},
    secret_backend::{database::DatabaseBackend, SecretBackend},
    variables::{build_crypt, decrypt, encrypt},
};









/// Get the current secret backend based on global settings
///
/// OSS: Always returns DatabaseBackend
/// EE: Returns configured backend (Database or Vault)
pub async fn get_secret_backend(db: &DB) -> Result<Arc<dyn SecretBackend>> {
    Ok(Arc::new(DatabaseBackend::new(db.clone())))
}





/// Check if a Vault backend is currently configured
///
/// OSS: Always returns false
/// EE: Checks global settings
pub async fn is_vault_backend_configured(_db: &DB) -> Result<bool> {
    Ok(false)
}


/// Get a secret value using the configured backend
///
/// For database backend: decrypts using workspace key
/// For vault backend (EE only): fetches from Vault directly
pub async fn get_secret_value(
    db: &DB,
    workspace_id: &str,
    path: &str,
    encrypted_value: &str,
) -> Result<String> {
    let backend = get_secret_backend(db).await?;

    match backend.backend_name() {
        "database" => {
            // Use existing database decryption
            let mc = build_crypt(db, workspace_id).await?;
            decrypt(&mc, encrypted_value.to_string()).map_err(|e| {
                Error::internal_err(format!("Error decrypting variable {}: {}", path, e))
            })
        }
        "hashicorp_vault" => {
            // Fetch from Vault directly
            backend.get_secret(workspace_id, path).await
        }
        "azure_key_vault" => backend.get_secret(workspace_id, path).await,
        "aws_secrets_manager" => backend.get_secret(workspace_id, path).await,
        _ => Err(Error::internal_err(format!(
            "Unknown backend: {}",
            backend.backend_name()
        ))),
    }
}

/// Store a secret value using the configured backend
///
/// For database backend: encrypts using workspace key and returns encrypted value
/// For vault backend (EE only): stores in Vault and returns a placeholder for DB storage
pub async fn store_secret_value(
    db: &DB,
    workspace_id: &str,
    path: &str,
    plain_value: &str,
) -> Result<String> {
    let backend = get_secret_backend(db).await?;

    match backend.backend_name() {
        "database" => {
            // Use existing database encryption
            let mc = build_crypt(db, workspace_id).await?;
            Ok(encrypt(&mc, plain_value))
        }
        "hashicorp_vault" => {
            // Store in Vault and return a marker for DB
            backend.set_secret(workspace_id, path, plain_value).await?;
            Ok(format!("$vault:{}", path))
        }
        "azure_key_vault" => {
            backend.set_secret(workspace_id, path, plain_value).await?;
            Ok(format!("$azure_kv:{}", path))
        }
        "aws_secrets_manager" => {
            backend.set_secret(workspace_id, path, plain_value).await?;
            Ok(format!("$aws_sm:{}", path))
        }
        _ => Err(Error::internal_err(format!(
            "Unknown backend: {}",
            backend.backend_name()
        ))),
    }
}

/// Persist a freshly minted OAuth access token to the secret variable backing
/// a resource, routing through the configured secret backend.
///
/// This is the write counterpart of the lazy on-fetch OAuth refresh: it stores
/// the token via [`store_secret_value`] (which writes to the external backend —
/// AWS Secrets Manager / Azure Key Vault / Vault — when one is configured, or
/// encrypts for the database backend) and updates `variable.value` with the
/// returned value (the encrypted blob for the DB backend, or a `$...:` marker
/// for an external backend). Using a raw `UPDATE variable SET value = <encrypted>`
/// here instead would leave the external store frozen at its connect-time token
/// while reads (which resolve through the backend) keep serving the stale value.
///
/// The caller has already committed the `account` row as fresh (advanced
/// `expires_at`) by the time we get here. If persisting the token fails — most
/// likely a transient error talking to an external backend — that would leave
/// the account marked fresh while the served secret is stale, so the on-fetch
/// refresh gate (`now() > expires_at`) would skip refresh and keep serving the
/// stale token for the whole token lifetime. To avoid that we reset `expires_at`
/// to the past (and record `refresh_error`) on failure — looking the account up
/// via `variable.account` — so the very next fetch retries the refresh instead.
///
/// Authorization contract: this performs NO access control. It writes the
/// caller-supplied token into the secret variable at `path` and may mutate the
/// linked `account` row, so callers MUST have already authorized the operation
/// against `workspace_id`/`path` (the OAuth refresh adapters only run after the
/// read path has resolved and gated the variable). It is therefore kept
/// `pub(crate)` and intended solely for the in-crate refresh adapters.
#[cfg(feature = "oauth2")]
pub(crate) async fn store_oauth_token_value(
    db: &DB,
    workspace_id: &str,
    path: &str,
    token: &str,
) -> Result<()> {
    let persist = async {
        let value = store_secret_value(db, workspace_id, path, token).await?;
        sqlx::query("UPDATE variable SET value = $1 WHERE workspace_id = $2 AND path = $3")
            .bind(value)
            .bind(workspace_id)
            .bind(path)
            .execute(db)
            .await?;
        Ok::<(), Error>(())
    }
    .await;

    if let Err(e) = persist {
        // Mark the account expired again so the next fetch re-runs the refresh
        // instead of serving the now-stale token until it naturally expires. The
        // account id is the one linked from the variable being refreshed.
        let account_id: Option<i32> = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT account FROM variable WHERE workspace_id = $1 AND path = $2",
        )
        .bind(workspace_id)
        .bind(path)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .flatten();

        if let Some(account_id) = account_id {
            if let Err(reset_err) = sqlx::query(
                "UPDATE account SET expires_at = now() - interval '1 minute', refresh_error = $1 \
                 WHERE workspace_id = $2 AND id = $3",
            )
            .bind(format!(
                "OAuth token was refreshed but persisting it to the secret backend failed: {e}"
            ))
            .bind(workspace_id)
            .bind(account_id)
            .execute(db)
            .await
            {
                tracing::error!(
                    workspace_id = %workspace_id,
                    account_id = %account_id,
                    "failed to reset account expiry after token persistence error: {reset_err}"
                );
            }
        }
        return Err(e);
    }

    Ok(())
}

/// Delete a secret from the configured backend (if using Vault)
///
/// For database backend: no-op (DB delete is handled separately)
/// For vault backend (EE only): deletes from Vault
pub async fn delete_secret_from_backend(db: &DB, workspace_id: &str, path: &str) -> Result<()> {
    if is_vault_backend_configured(db).await? {
        let backend = get_secret_backend(db).await?;
        // Ignore NotFound errors during deletion (secret might not exist in Vault)
        match backend.delete_secret(workspace_id, path).await {
            Ok(()) => Ok(()),
            Err(Error::NotFound(_)) => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

/// Check if a value is stored in Vault (indicated by the $vault: prefix)
pub fn is_vault_stored_value(value: &str) -> bool {
    value.starts_with("$vault:")
}

/// Check if a value is stored in Azure Key Vault (indicated by the $azure_kv: prefix)
pub fn is_azure_kv_stored_value(value: &str) -> bool {
    value.starts_with("$azure_kv:")
}

/// Check if a value is stored in AWS Secrets Manager (indicated by the $aws_sm: prefix)
pub fn is_aws_sm_stored_value(value: &str) -> bool {
    value.starts_with("$aws_sm:")
}

/// Check if a value is stored in any external secret backend
pub fn is_external_stored_value(value: &str) -> bool {
    is_vault_stored_value(value) || is_azure_kv_stored_value(value) || is_aws_sm_stored_value(value)
}

/// Rename a secret in Vault when a variable path changes (EE only)
pub async fn rename_vault_secret(
    _db: &DB,
    _workspace_id: &str,
    _old_path: &str,
    new_path: &str,
    current_value: &str,
) -> Result<Option<String>> {
    if is_vault_stored_value(current_value) {
        tracing::warn!(
            "Variable has $vault: prefix but Vault requires Enterprise Edition. \
             Updating DB reference to {}",
            new_path
        );
        return Ok(Some(format!("$vault:{}", new_path)));
    }
    if is_azure_kv_stored_value(current_value) {
        tracing::warn!(
            "Variable has $azure_kv: prefix but Azure Key Vault requires Enterprise Edition. \
             Updating DB reference to {}",
            new_path
        );
        return Ok(Some(format!("$azure_kv:{}", new_path)));
    }
    if is_aws_sm_stored_value(current_value) {
        tracing::warn!(
            "Variable has $aws_sm: prefix but AWS Secrets Manager requires Enterprise Edition. \
             Updating DB reference to {}",
            new_path
        );
        return Ok(Some(format!("$aws_sm:{}", new_path)));
    }
    Ok(None)
}


/// Bulk rename secrets in Vault when a path prefix changes (e.g., user rename)
pub async fn rename_vault_secrets_with_prefix(
    _db: &DB,
    _workspace_id: &str,
    _old_prefix: &str,
    _new_prefix: &str,
    _variables: Vec<(String, String)>,
) -> Result<Vec<(String, String)>> {
    Ok(vec![])
}

