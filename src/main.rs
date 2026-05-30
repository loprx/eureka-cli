use clap::Parser;
use eureka_cli::cli::Cli;
use eureka_cli::error::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Restore default SIGPIPE handling so `eureka-cli ... | head` exits cleanly
    // instead of panicking when stdout pipes close. Rust ignores SIGPIPE by
    // default on Unix; that turns every short-pipe read into a BrokenPipe
    // panic on the first println! after the consumer exits.
    #[cfg(unix)]
    {
        // SAFETY: signal() with SIG_DFL is a documented async-signal-safe call.
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "eureka_cli=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    cli.execute().await?;

    Ok(())
}
