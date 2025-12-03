# wheel-rs

[English](README.md)

一个提供各种实用工具的 Rust 库，用于处理常见任务。

## 功能特性

- **文件工具**: 提供文件扩展名提取和 SHA256 哈希值计算功能
- **时间工具**: 提供时间戳和时间测量相关工具
- **DNS 工具**: 提供 DNS 解析功能
- **命令行工具**: 提供执行外部命令的功能
- **序列化工具**: 为多种类型提供自定义序列化和反序列化支持
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

### `cmd` 命令行工具模块
命令执行和进程管理：
- `exec`: 执行外部命令
- `is_process_alive`: 检查进程是否仍在运行
- `kill_process`: 终止进程

### `serde` 序列化模块
自定义序列化/反序列化实现：
- `duration_option_serde`: 用于 `Option<Duration>`
- `log_filter_option_serde`: 用于 `Option<LevelFilter>`
- `vec_option_serde`: 用于 `Option<Vec<String>>`

### `urn_utils` URN 工具模块
URN 解析工具：
- `Urn`: 解析和表示 URN
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