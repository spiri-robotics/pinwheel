
pub async fn set_license_key(license_key: String) -> anyhow::Result<()> {
    use windmill_api::ee::validate_license_key;
    use windmill_common::ee::{LICENSE_KEY, LICENSE_KEY_ID, LICENSE_KEY_VALID};

    let id = validate_license_key(license_key.clone()).await?;
    {
        let mut l = LICENSE_KEY_ID.write().await;
        *l = id.to_string()
    }

    {
        let mut l = LICENSE_KEY.write().await;
        *l = license_key
    }
    {
        let mut l = LICENSE_KEY_VALID.write().await;
        *l = true
    }

    Ok(())
}

