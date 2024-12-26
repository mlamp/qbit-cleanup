# qbit-cleanup

A Rust CLI tool that cleans up qBittorrent torrents by predicting their ratio growth and removing those that fail to meet a target ratio by the end of one year, given their current ratio and age. It uses the qBittorrent Web API (via [qbit-rs](https://crates.io/crates/qbit-rs)).

## Features

- **Age threshold**: Removes torrents older than a certain number of days (`--age`).  
- **Predicted ratio**: Calculates a one-year ratio projection to decide which torrents to remove (`--ratio`).  
- **Dry-run mode**: Lets you see which torrents _would_ be removed without actually deleting them (`--dry-run`).  
- **Logging**:  
  - Respects `RUST_LOG` environment variable by default.  
  - Can override logs to **debug** level with `--debug`.

## Installation

1. **Install Rust** (if you haven’t already) using [rustup](https://rustup.rs/).  
2. **Clone or download** this repository.  
3. **Build** using Cargo:

   ```bash
   cargo build --release
   ```

4. The compiled binary can be found in `target/release/qbit-cleanup`.

*(Alternatively, you can run `cargo run -- [OPTIONS]` during development.)*

## Usage

Run the binary and provide the necessary arguments. For example:

```bash
./qbit-cleanup --age 100 --ratio 10
```

### Command-Line Arguments

- **`--age <DAYS>`** (default: `100`)  
  Minimum age (in days) for a torrent to be considered for removal.  
- **`--ratio <FLOAT>`** (default: `10`)  
  If the _predicted_ ratio over one year is below this value, the torrent is removed.  
- **`--endpoint <URL>`** (default: `http://127.0.0.1:8080`)  
  qBittorrent WebUI endpoint.  
- **`--username <STRING>`** (default: `admin`)  
  qBittorrent username.  
- **`--password <STRING>`** (default: `adminadmin`)  
  qBittorrent password.  
- **`--dry-run`**  
  If set, torrents will **not** actually be deleted—only logged.  
- **`--debug`**  
  Overrides the default or environment-based log level to **debug**.

### Examples

1. **Default usage** (age = 100 days, ratio = 10, endpoint = `localhost`):
   ```bash
   ./qbit-cleanup
   ```
   Prints info logs for each torrent removed (or considered).

2. **Custom options**: Check torrents older than 30 days and remove them if their predicted ratio is < 5 by the end of the year:
   ```bash
   ./qbit-cleanup --age 30 --ratio 5
   ```

3. **Dry run mode**: Check what would happen, but don’t actually remove torrents:
   ```bash
   ./qbit-cleanup --dry-run
   ```

4. **Debug logs**: Show detailed debug logs:
   ```bash
   ./qbit-cleanup --debug
   ```

   Or combine it with `--dry-run`:
   ```bash
   ./qbit-cleanup --dry-run --debug
   ```

5. **Custom endpoint**: If qBittorrent is running elsewhere or on a different port:
   ```bash
   ./qbit-cleanup --endpoint http://192.168.1.10:8085 --username bob --password s3cr3t
   ```

## Logging

- By default, this tool uses [env_logger](https://crates.io/crates/env_logger), which respects the `RUST_LOG` environment variable.
- Use `--debug` to override all log levels to **debug**.
- Examples:
  - **`RUST_LOG=warn ./qbit-cleanup`**: Show only warnings and errors.
  - **`RUST_LOG=trace ./qbit-cleanup --debug`**: `--debug` will override `RUST_LOG`, so logs will be at debug level.

## How It Works

1. **Connect** to the qBittorrent Web API at the specified `--endpoint` using the provided credentials.  
2. **Fetch** all torrents (via `get_torrent_list`).  
3. **Calculate** each torrent’s age and ratio.  
4. **Predict** how the ratio might evolve after a year, based on current ratio and the time since it was added.  
5. **Compare** that predicted ratio to the threshold (`--ratio`).  
6. **Optionally** remove torrents older than `--age` days whose predicted ratio is too low.  
7. If in `--dry-run` mode, just log which torrents **would** be removed, but don’t actually remove them.

## Contributing

1. **Fork** this repo.  
2. **Create** your feature branch: `git checkout -b my-new-feature`  
3. **Commit** your changes: `git commit -am 'Add some feature'`  
4. **Push** to the branch: `git push origin my-new-feature`  
5. **Open** a pull request.

## License

This project is licensed under the [MIT License](LICENSE).  

---

Happy torrent-cleaning! Feel free to open issues or pull requests if you find a bug or have a suggestion.
