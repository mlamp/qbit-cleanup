use clap::Parser;
use qbit_rs::QbitBuilder;
use qbit_rs::model::Credential;
use qbit_rs::model::GetTorrentListArg;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use env_logger::{Builder, Env};
use log::{info, debug, LevelFilter};

/// Simple CLI to clean up qBittorrent torrents by ratio and age (in days).
#[derive(Parser, Debug)]
#[command(name = "qbit-cleanup")]
#[command(version = "0.1.0")]
struct Cli {
    /// Age threshold in days. Torrents older than this will be considered for removal.
    /// Default: 100 days
    #[arg(long = "age", default_value = "100")]
    age: u64,

    /// Target ratio threshold. Torrents whose predicted one-year ratio is below this value
    /// will be removed. The prediction is based on current ratio and age.
    /// Default: 10.0
    #[arg(long = "ratio", default_value = "10")]
    ratio: f64,

    /// qBittorrent WebUI endpoint URL.
    /// Example: http://127.0.0.1:8080
    #[arg(long = "endpoint", default_value = "http://127.0.0.1:8080")]
    endpoint: String,

    /// qBittorrent WebUI username for authentication.
    /// Default: admin
    #[arg(long = "username", default_value = "admin")]
    username: String,

    /// qBittorrent WebUI password for authentication.
    /// Default: adminadmin
    #[arg(long = "password", default_value = "adminadmin")]
    password: String,

    /// When enabled, performs a simulation run without actually deleting any torrents.
    /// Useful for testing and previewing which torrents would be removed.
    #[arg(long = "dry-run", default_value_t = false)]
    dry_run: bool,

    /// Enables debug-level logging for more detailed output.
    /// This overrides the RUST_LOG environment variable.
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments using clap
    let cli = Cli::parse();

    // Configure logging based on --debug flag and RUST_LOG env var
    let mut builder = Builder::from_env(Env::default());
    if cli.debug {
        // Override with debug level if --debug flag is set
        builder.filter_level(LevelFilter::Debug);
    }
    builder.init();

    debug!("Starting qbit-cleanup with age threshold {} days and ratio threshold {}", cli.age, cli.ratio);

    // Parse and validate qBittorrent WebUI endpoint URL
    let endpoint_url: Url = cli.endpoint.parse()?;
    debug!("Connecting to qBittorrent at {}", endpoint_url);

    // Initialize qBittorrent client with credentials
    let cred = Credential::new(cli.username, cli.password);
    let qbit = QbitBuilder::new()
        .endpoint(endpoint_url)
        .credential(cred)
        .build();

    // Authenticate with qBittorrent WebUI
    info!("Authenticating with qBittorrent WebUI...");
    qbit.login(true).await?;

    // Fetch complete torrent list
    info!("Fetching torrent list...");
    let torrents = qbit.get_torrent_list(GetTorrentListArg::default()).await?;
    debug!("Found {} torrents to analyze", torrents.len());

    // Calculate current time and constants
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_secs();
    let one_year_secs = 365 * 24 * 3600;
    let age_threshold_secs = cli.age * 24 * 3600;

    // Process each torrent
    for torrent in torrents {
        let added_on_value = torrent.added_on.clone().unwrap_or(0);
        let name = torrent.name.clone().unwrap_or_default();
        let hash = torrent.hash.clone().unwrap_or_default();
        let torrent_age_secs = now_secs.saturating_sub(added_on_value as u64);
        let age_days = torrent_age_secs / 86400;

        debug!(
            "Analyzing torrent: {} (hash={})\n\tAge: {} days\n\tCurrent ratio: {:.2}",
            name,
            hash,
            age_days,
            torrent.ratio.clone().unwrap_or_default(),
        );

        // Skip torrents younger than threshold
        if torrent_age_secs <= age_threshold_secs {
            debug!("Skipping {} - too young ({} days < {} days threshold)", 
                name, age_days, cli.age);
            continue;
        }

        // Check ratio and predict future ratio
        if let Some(ratio_value) = torrent.ratio {
            let predicted_ratio = ratio_value * (one_year_secs as f64 / torrent_age_secs as f64);
            
            if predicted_ratio < cli.ratio {
                let hashes = vec![hash.clone()];
                
                if cli.dry_run {
                    info!(
                        "DRY RUN: Would remove torrent: {}\n\tHash: {}\n\tPredicted ratio: {:.2}\n\tAge: {} days\n\tCurrent ratio: {:.2}",
                        name, hash, predicted_ratio, age_days, ratio_value
                    );
                } else {
                    info!(
                        "Removing torrent: {}\n\tHash: {}\n\tPredicted ratio: {:.2}\n\tAge: {} days\n\tCurrent ratio: {:.2}",
                        name, hash, predicted_ratio, age_days, ratio_value
                    );
                    qbit.delete_torrents(hashes, Some(true)).await?;
                }
            } else {
                debug!(
                    "Keeping torrent {} - predicted ratio {:.2} >= threshold {}", 
                    name, predicted_ratio, cli.ratio
                );
            }
        }
    }

    info!("Cleanup complete");
    Ok(())
}
