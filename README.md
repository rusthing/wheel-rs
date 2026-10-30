# wheel-rs

[中文](README_CN.md)

A Rust utility library providing various helpful utilities for common tasks.

## Features

- **File Utilities**: File extension extraction, SHA256 hash calculation, cross-device error detection, debounced file watching
- **Time Utilities**: Timestamps, duration string conversion, ticker building, external timestamp normalization
- **DNS Utilities**: Host / host:port resolution (IPv4, bracketed and bare IPv6)
- **Address Utilities**: `host[:port]` parsing and formatting
- **Command Execution**: Sync (`std::process`) and async (`tokio::process`) external command execution
- **Process Management**: PID files, process existence checks, termination, Unix signal sending and watching
- **Configuration Utilities**: Config diffing and change detection
- **Network Utilities**: IP network ordering and local IP discovery
- **Trie**: Compressed prefix tree (Patricia trie)
- **String Utilities**: Camel case splitting, `snake_case` → `PascalCase`
- **Serialization**: Custom serde implementations for `Duration`, `IpNet`, `PathBuf`, `LevelFilter`, `Rotation`, `u64`, `Addr`, `Urn`, `Regex` and their `Option` / `Vec` variants

## Modules

### `file_utils`
- `get_file_ext`: Extract the (lowercased) file extension from a filename
- `calc_hash_of_file`: Calculate the SHA256 hash of a file
- `is_cross_device_error`: Check whether an IO error is a cross-device error
- `watch_file_changed`: Watch files and run a callback after debouncing changes

### `time_utils`
- `now_ts` / `now_ms` / `now_ns`: Current Unix timestamp in seconds / milliseconds / nanoseconds
- `duration_to_string` / `string_to_duration`: `Duration` ↔ humantime string
- `build_ticker`: Build a `tokio::time::Interval` with `MissedTickBehavior::Skip`
- `is_plausible_ms` / `normalize_ms`: Validate and normalize externally supplied millisecond timestamps

### `dns_utils`
- `parse_host`: Resolve a hostname to an IP address
- `parse_host_port`: Resolve `host[:port]` to `(IpAddr, u16)`

### `addr_utils`
- `Addr`: `host` + optional `port`, with `Display`, `FromStr`, `ToSocketAddrs` and serde support

### `cmd`
- `cmd::std::execute`: Run a command synchronously and return its stdout
- `cmd::spawn::execute`: Spawn a command asynchronously, streaming stdout through a `broadcast` channel
- `is_process_alive` / `kill_process`: Process liveness check and termination (sync and async)

### `process`
- `read_pid` / `write_pid` / `delete_pid_file`: PID file operations
- `PidFileGuard`: RAII guard that removes the PID file on drop
- `check_process` / `terminate_process`: Process existence check and graceful termination
- `send_signal_by_instruction` / `watch_signal`: Send and watch Unix signals

### `config_utils`
- `diff_config`: Recursively diff two `config::Config` values
- `has_config_changed`: Check whether anything under a key prefix changed

### `ipnet_utils`
- `com_ip` / `is_exact`: Order IP networks so exact hosts come first
- `get_local_ip`: Find the first non-loopback IPv4 address of the local host

### `trie`
- `PatriciaTrie`: Compressed prefix tree with `put` / `get` / `list` (prefix scan) / `keys`

### `str_utils`
- `split_camel_case`: Split `helloWorld` / `XMLParser` into words
- `snake_to_pascal`: `deleted_at` → `DeletedAt`

### `serde`
Custom serialization/deserialization, used via `#[serde(with = "...")]`, e.g.:
- `duration_serde` / `duration_option_serde` / `duration_option_option_serde`
- `ipnet_serde` / `ipnet_option_serde`
- `path_buf_serde` / `path_buf_option_serde`
- `log_filter_serde` / `log_filter_option_serde`
- `rotation_serde`, `resolve_hosts_serde` / `resolve_hosts_option_serde`
- `u64_serde` / `u64_option_serde`
- `vec_serde`, `vec_option_serde`, `vec_addr_serde`, `vec_ipnet_serde`, `vec_regex_serde`, `vec_urn_serde`
- `option_option_serde` (three-state partial update semantics)

### `urn_utils`
- `Urn`: Parse and represent `METHOD:url` URNs, with prefix matching via `matches`
- `Method`: HTTP method enumeration

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wheel-rs = "1.13"
```

Then use it in your code:

```rust
use wheel_rs::file_utils::get_file_ext;
use wheel_rs::time_utils::now_ms;
use wheel_rs::dns_utils::parse_host_port;

assert_eq!(get_file_ext("archive.TAR.GZ").as_deref(), Some("gz"));
println!("now = {} ms", now_ms());
```

## Platform support

The crate depends on `nix` and is currently **Unix only**.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
