//! OSS stubs for OTEL tracing proxy (EE feature)


/// Start the OTEL tracing proxy (no-op in OSS)
pub async fn start_otel_tracing_proxy(
    _db: windmill_common::DB,
    _killpill_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}
