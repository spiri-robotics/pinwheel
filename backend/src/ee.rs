use anyhow::anyhow;

pub async fn set_license_key(_license_key: String) -> anyhow::Result<()> {
    // Implementation is not open source
    Err(anyhow!("License cannot be set in Windmill CE"))
}


