/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2023
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use std::process::Command;

use anyhow;



use crate::db::DB;
use axum::extract::Path;
use axum::routing::get;
use axum::Extension;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

pub fn global_service() -> Router {
    Router::new()
}

pub fn workspaced_service() -> Router {
    Router::new()
}


#[derive(Debug, Clone, serde::Serialize)]
struct Keys {
    private_key: String,
}

async fn gen_pems(db: &DB) -> anyhow::Result<Keys> {
    let private_key_cmd = Command::new("openssl")
        .arg("genrsa")
        .arg("--traditional")
        .arg("2048")
        .output()
        .expect("failed to execute process");

    let private_key = String::from_utf8(private_key_cmd.stdout).unwrap();

    tracing::debug!("Generated private key: {}", private_key);
    let keys = Keys { private_key };

    sqlx::query!(
        "INSERT INTO global_settings (name, value) VALUES ('rsa_keys', $1)",
        serde_json::to_value(&keys).unwrap()
    )
    .execute(db)
    .await?;

    Ok(keys)
}





#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
struct JobClaim {
    job_id: String,
    path: Option<String>,
    flow_path: Option<String>,
    groups: Vec<String>,
    username: String,
    email: String,
    workspace: String,
}

use crate::db::ApiAuthed;
use crate::users::Tokened;

