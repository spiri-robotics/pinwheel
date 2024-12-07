/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2022
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use crate::{jobs::QueuedJob, utils::Mode};
use uuid::Uuid;

pub fn set_span_parent(_span: &tracing::Span, _rj: &Uuid) {}

pub(crate) type OtelProvider = Option<()>;


#[cfg(not(feature = "otel"))]
pub fn otel_ctx() -> () {}

#[cfg(feature = "otel")]
#[inline(always)]
pub fn otel_ctx() -> opentelemetry::Context {
    opentelemetry::Context::current()
}

#[cfg(not(feature = "otel"))]
impl<T: Sized> FutureExt for T {}

#[cfg(not(feature = "otel"))]
pub trait FutureExt: Sized {
    fn with_context(self, _otel_cx: ()) -> Self {
        self
    }
}

use tracing_subscriber::EnvFilter;

pub(crate) fn init_logs_bridge(_mode: &Mode) -> Option<EnvFilter> {
    None
}


pub(crate) fn init_meter_provider(_mode: &Mode) -> OtelProvider {
    None
}

pub fn add_root_flow_job_to_otlp(_queued_job: &QueuedJob, _success: bool) {}
