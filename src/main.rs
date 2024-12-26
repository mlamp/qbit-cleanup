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

    /// Age threshold (in days)
    #[arg(long = "age", default_value = "100")]
    age: u64,

    /// Remove torrents if predicted ratio in a year is < this value
    #[arg(long = "ratio", default_value = "10")]
    ratio: f64,

    /// qBittorrent WebUI endpoint (e.g. http://127.0.0.1:8080)
    #[arg(long = "endpoint", default_value = "http://127.0.0.1:8080")]
    endpoint: String,

    /// qBittorrent username
    #[arg(long = "username", default_value = "admin")]
    username: String,

    /// qBittorrent password
    #[arg(long = "password", default_value = "adminadmin")]
    password: String,

    /// Run in dry-run mode (no torrents will be deleted)
    #[arg(long = "dry-run", default_value_t = false)]
    dry_run: bool,

    /// Run in debug mode
    #[arg(long)]
    debug: bool,

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // 1. Parse CLI arguments
    let cli = Cli::parse();

    // Configure env_logger at runtime based on the --debug flag
    // 1) Create a builder (this will parse RUST_LOG by default)
    let mut builder = Builder::from_env(Env::default());

    // 2) If --debug is set, override environment variables
    if cli.debug {
        builder.filter_level(LevelFilter::Debug);
    }

    // Initialize the logger
    builder.init();

    // 2. Convert your endpoint string to a proper Url (qbit-rs wants a TryInto<Url>)
    let endpoint_url: Url = cli.endpoint.parse()?;

    // 3. Build Qbit with the Url and your credentials
    let cred = Credential::new(cli.username, cli.password);
    let qbit = QbitBuilder::new()
        .endpoint(endpoint_url)
        .credential(cred)
        .build();  // build returns Result<Qbit, Error>, hence the ?

    // 4. Log in (force = true to always re-auth)
    qbit.login(true).await?;

    // 5. Fetch the list of all torrents
    let torrents = qbit.get_torrent_list(GetTorrentListArg::default()).await?;

    // Current Unix time in seconds
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let one_year_secs = 365 * 24 * 3600;

    // 7. Iterate over each torrent, check ratio + age, remove if needed
    for torrent in torrents {
        let added_on_value = torrent.added_on.clone().unwrap_or(0); // typically an epoch (u64) in seconds
        let name = torrent.name.clone().unwrap_or_default();
        let hash = torrent.hash.clone().unwrap_or_default();

        let torrent_age_secs = now_secs.saturating_sub(added_on_value as u64);
        let age_threshold_secs = cli.age * 24 * 3600;
        debug!(
            "Check for torrent: {} (hash={}), age_days={}, current_ratio={:.2}",
            name,
            hash,
            torrent_age_secs / 86400,
            torrent.ratio.clone().unwrap_or_default(),
            );
        if torrent_age_secs > age_threshold_secs {
            if let Some(ratio_value) = torrent.ratio {
                let predicted_ratio = ratio_value * (one_year_secs as f64 / torrent_age_secs as f64);
                if predicted_ratio < cli.ratio {
                    let hashes = vec![hash.clone()];
                    if cli.dry_run {
                        info!(
                            "Dry run - removing torrent: {} (hash={}), predicted_ratio={:.2}, age_days={}, current_ratio={:.2}",
                            name,
                            hash,
                            predicted_ratio,
                            torrent_age_secs / 86400,
                            ratio_value
                            );
                    } else {
                        info!(
                            "Removing torrent with files: {} (hash={}), predicted_ratio={:.2}, age_days={}, current_ratio={:.2}",
                            name,
                            hash,
                            predicted_ratio,
                            torrent_age_secs / 86400,
                            ratio_value
                            );
                        qbit.delete_torrents(hashes, Some(true)).await?;
                    }
                }
            }
        } else {
            debug!("Torrent too new: {} (hash={}), age_days={}", name, hash, torrent_age_secs / 86400);
        }
    }

    Ok(())
}
