/*
 * Author: Windmill Labs, Inc
 * Copyright: Windmill Labs, Inc 2024
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */


use sqlx::{Postgres, Transaction};
use windmill_common::db::DB;
use windmill_common::error;

pub async fn _refresh_token<'c>(
    tx: Transaction<'c, Postgres>,
    path: &str,
    w_id: &str,
    id: i32,
    db: &DB,
) -> error::Result<String> {
    windmill_oauth::refresh_token(
        tx,
        path,
        w_id,
        id,
        db,
        &*windmill_oauth::OAUTH_CLIENTS.load(),
        &windmill_oauth::OAUTH_HTTP_CLIENT,
        include_str!("../../oauth_connect.json"),
    )
    .await
}
