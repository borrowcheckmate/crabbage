//! Serves the crabbage web app.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--healthcheck") {
        std::process::exit(i32::from(!crabbage::healthcheck()));
    }
    let config = crabbage::Config::from_env()?;
    let pool = crabbage_store::open(&config.data_dir).await?;
    let options = leptos::config::get_configuration(None)?.leptos_options;
    let site_addr = options.site_addr;
    let listener = tokio::net::TcpListener::bind(site_addr).await?;
    let router = crabbage::router(options, &config, pool)?;
    axum::serve(listener, router).await?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
