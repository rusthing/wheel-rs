# wheel-rs

[中文](README_CN.md)

A Rust utility library providing various helpful utilities for common tasks.

## Features

- **File Utilities**: Functions for file extension extraction and SHA256 hash calculation
- **Time Utilities**: Tools for working with timestamps and time measurements
- **DNS Utilities**: Tools for DNS resolution
- **Command Execution**: Execute external commands and manage processes
- **Serialization**: Custom serialization and deserialization for various types
- **URN Parsing**: Parse and handle Uniform Resource Names

## Modules

### `file_utils`
Provides utility functions for file operations:
- `get_file_ext`: Extract file extension from filename
- `calc_hash`: Calculate SHA256 hash of a file
- `is_cross_device_error`: Check if an IO error is a cross-device error

### `time_utils`
Time-related utilities:
- `get_current_timestamp`: Get current timestamp in milliseconds

### `dns_utils`
DNS resolution utilities:
- `parse_host`: Resolve hostname to IP address
- `parse_host_port`: Resolve hostname and port to IP address and port

### `cmd`
Command execution and process management:
- `exec`: Execute external commands
- `is_process_alive`: Check if a process is still running
- `kill_process`: Kill a process

### `serde`
Custom serialization/deserialization implementations:
- `duration_option_serde`: For `Option<Duration>`
- `log_filter_option_serde`: For `Option<LevelFilter>`
- `vec_option_serde`: For `Option<Vec<String>>`

### `urn_utils`
URN parsing utilities:
- `Urn`: Parse and represent URNs
- `Method`: HTTP method enumeration

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wheel-rs = "0.4.0"
```

Then use it in your code:

```rust
use wheel_rs::file_utils;
use wheel_rs::time_utils;
use wheel_rs::dns_utils;
use wheel_rs::cmd::cmd_utils;
use wheel_rs::serde::duration_option_serde;
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<span id="中文"></span>
# wheel-rs

一个提供文件操作、时间工具和 Duration 序列化支持的 Rust 工具库。

## 功能特性

- **文件工具**: 提供文件扩展名提取和 SHA256 哈希值计算功能
- **时间工具**: 提供时间戳和时间测量相关工具
- **DNS 工具**: 提供 DNS 解析工具
- **命令执行**: 执行外部命令和进程管理
- **序列化**: 提供各种类型的自定义序列化和反序列化支持
- **URN 解析**: 解析和处理统一资源名称

## 模块说明

### `file_utils` 文件工具模块
提供文件操作相关的实用工具函数：
- `get_file_ext`: 从文件名中提取扩展名
- `calc_hash`: 计算文件的 SHA256 哈希值
- `is_cross_device_error`: 检查 IO 错误是否为跨设备错误

### `time_utils` 时间工具模块
时间相关的工具函数：
- `get_current_timestamp`: 获取当前时间戳（毫秒）

### `dns_utils` DNS 工具模块
DNS 解析工具函数：
- `parse_host`: 将主机名解析为 IP 地址
- `parse_host_port`: 将主机名和端口解析为 IP 地址和端口

### `cmd` 命令执行模块
命令执行和进程管理工具函数：
- `exec`: 执行外部命令
- `is_process_alive`: 检查进程是否仍在运行
- `kill_process`: 杀死进程

### `serde` 序列化模块
自定义序列化/反序列化实现：
- `duration_option_serde`: 用于 `Option<Duration>`
- `log_filter_option_serde`: 用于 `Option<LevelFilter>`
- `vec_option_serde`: 用于 `Option<Vec<String>>`

### `urn_utils` URN 解析模块
URN 解析工具函数：
- `Urn`: 解析和表示 URNs
- `Method`: HTTP 方法枚举

## 使用方法

将以下内容添加到您的 `Cargo.toml` 文件中：

```toml
[dependencies]
wheel-rs = "0.4.0"
```

然后在您的代码中使用：

```rust
use wheel_rs::file_utils;
use wheel_rs::time_utils;
use wheel_rs::dns_utils;
use wheel_rs::cmd::cmd_utils;
use wheel_rs::serde::duration_option_serde;
```

## 许可证

本项目采用 MIT 许可证，详情请参见 [LICENSE](LICENSE) 文件。