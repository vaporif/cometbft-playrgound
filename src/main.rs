use std::env;

use cometbft_playrgound::app::App;

fn main() -> eyre::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    env::set_var("RUST_LOG", "trace");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    tracing::info!("starting");

    let app = App::new();

    let app = tendermint_abci::ServerBuilder::default().bind("127.0.0.1:26658", app)?;
    tracing::info!("listenening for abci events");
    _ = app.listen();
    Ok(())
}
