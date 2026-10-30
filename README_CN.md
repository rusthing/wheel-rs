# wheel-rs

[English](README.md)

一个提供各种常用工具能力的 Rust 库。

## 功能特性

- **文件工具**：扩展名提取、SHA256 哈希、跨设备错误判断、带去抖动的文件监听
- **时间工具**：时间戳、时长字符串互转、定时器构建、外部时间戳规范化
- **DNS 工具**：主机名 / `host:port` 解析（IPv4、带方括号与裸 IPv6）
- **地址工具**：`host[:port]` 的解析与格式化
- **命令行工具**：同步（`std::process`）与异步（`tokio::process`）两种外部命令执行方式
- **进程管理**：PID 文件、进程存活检查、终止进程、Unix 信号发送与监听
- **配置工具**：配置差异比较与变更判断
- **网络工具**：IP 网段排序、本机 IP 获取
- **前缀树**：压缩前缀树（Patricia Trie）
- **字符串工具**：驼峰拆分、`snake_case` 转 `PascalCase`
- **序列化工具**：为 `Duration`、`IpNet`、`PathBuf`、`LevelFilter`、`Rotation`、`u64`、`Addr`、`Urn`、`Regex` 及其 `Option` / `Vec` 变体提供自定义 serde 实现

## 模块说明

### `file_utils` 文件工具模块
- `get_file_ext`：从文件名中提取（小写化的）扩展名
- `calc_hash_of_file`：计算文件的 SHA256 哈希值
- `is_cross_device_error`：判断 IO 错误是否为跨设备错误
- `watch_file_changed`：监听文件变更，去抖动后触发回调

### `time_utils` 时间工具模块
- `now_ts` / `now_ms` / `now_ns`：当前 Unix 时间戳（秒 / 毫秒 / 纳秒）
- `duration_to_string` / `string_to_duration`：`Duration` 与 humantime 字符串互转
- `build_ticker`：构建 `MissedTickBehavior::Skip` 的 `tokio::time::Interval`
- `is_plausible_ms` / `normalize_ms`：外部毫秒时间戳的校验与规范化

### `dns_utils` DNS 工具模块
- `parse_host`：将主机名解析为 IP 地址
- `parse_host_port`：将 `host[:port]` 解析为 `(IpAddr, u16)`

### `addr_utils` 地址工具模块
- `Addr`：`host` + 可选 `port`，支持 `Display`、`FromStr`、`ToSocketAddrs` 与 serde

### `cmd` 命令行工具模块
- `cmd::std::execute`：同步执行命令并返回 stdout
- `cmd::spawn::execute`：异步启动命令，通过 `broadcast` 通道流式转发 stdout
- `is_process_alive` / `kill_process`：进程存活检查与终止（同步 / 异步各一套）

### `process` 进程管理模块
- `read_pid` / `write_pid` / `delete_pid_file`：PID 文件读写与删除
- `PidFileGuard`：drop 时自动清理 PID 文件的 RAII 守卫
- `check_process` / `terminate_process`：进程存在性检查与优雅终止
- `send_signal_by_instruction` / `watch_signal`：Unix 信号发送与监听

### `config_utils` 配置工具模块
- `diff_config`：递归比较两个 `config::Config`
- `has_config_changed`：判断某个 key 前缀下是否有变更（按 `.` 分段边界匹配）

### `ipnet_utils` IP 网络工具模块
- `com_ip` / `is_exact`：排序时让精确地址排在网段之前
- `get_local_ip`：获取本机首个非回环 IPv4 地址

### `trie` 前缀树模块
- `PatriciaTrie`：压缩前缀树，提供 `put` / `get` / `list`（前缀扫描）/ `keys`

### `str_utils` 字符串工具模块
- `split_camel_case`：拆分 `helloWorld`、`XMLParser` 等驼峰字符串
- `snake_to_pascal`：`deleted_at` → `DeletedAt`

### `serde` 序列化模块
通过 `#[serde(with = "...")]` 使用，例如：
- `duration_serde` / `duration_option_serde` / `duration_option_option_serde`
- `ipnet_serde` / `ipnet_option_serde`
- `path_buf_serde` / `path_buf_option_serde`
- `log_filter_serde` / `log_filter_option_serde`
- `rotation_serde`、`resolve_hosts_serde` / `resolve_hosts_option_serde`
- `u64_serde` / `u64_option_serde`
- `vec_serde`、`vec_option_serde`、`vec_addr_serde`、`vec_ipnet_serde`、`vec_regex_serde`、`vec_urn_serde`
- `option_option_serde`（三态，用于部分更新语义）

### `urn_utils` URN 工具模块
- `Urn`：解析与表示 `METHOD:url`，`matches` 支持前缀匹配
- `Method`：HTTP 方法枚举

## 使用方法

将以下内容添加到您的 `Cargo.toml` 文件中：

```toml
[dependencies]
wheel-rs = "1.13"
```

然后在您的代码中使用：

```rust
use wheel_rs::file_utils::get_file_ext;
use wheel_rs::time_utils::now_ms;
use wheel_rs::dns_utils::parse_host_port;

assert_eq!(get_file_ext("archive.TAR.GZ").as_deref(), Some("gz"));
println!("now = {} ms", now_ms());
```

## 平台支持

本库依赖 `nix`，目前**仅支持 Unix**。

## 许可证

本项目采用 MIT 许可证，详情请参见 [LICENSE](LICENSE) 文件。
