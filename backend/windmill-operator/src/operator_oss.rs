

pub async fn run(_db: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    anyhow::bail!("K8s operator is not available in this build")
}

pub fn print_crd_yaml() {
    eprintln!("K8s operator CRD generation is not available in this build");
}
