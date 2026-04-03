use resume_vizor::{app, config::Settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::from_env()?;
    app::run(settings).await
}

