//! Serves the crabbage web app.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = crabbage::Config::from_env()?;
    let _pool = crabbage_store::open(&config.data_dir).await?;
    let options = leptos::config::get_configuration(None)?.leptos_options;
    let listener = tokio::net::TcpListener::bind(options.site_addr).await?;
    axum::serve(listener, crabbage::router(options)).await?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
