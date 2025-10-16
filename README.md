# wheel-rs

[中文](README_CN.md)

A Rust utility library providing file operations, time utilities and Duration serialization support.

## Features

- **File Utilities**: Functions for file extension extraction and SHA256 hash calculation
- **Time Utilities**: Tools for working with timestamps and time measurements
- **Duration Serialization**: Custom serialization and deserialization for `std::time::Duration` types

## Modules

### `file_utils`
Provides utility functions for file operations:
- `get_file_ext`: Extract file extension from filename
- `calc_hash`: Calculate SHA256 hash of a file
- `is_cross_device_error`: Check if an IO error is a cross-device error

### `time_utils`
Time-related utilities:
- `get_current_timestamp`: Get current timestamp in milliseconds

### `duration_serde`
Custom serialization/deserialization for `Option<Duration>`:
- `serialize`: Serialize Duration as string (e.g., "5s" for 5 seconds)
- `deserialize`: Deserialize string to Duration

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wheel-rs = "0.1.0"
```

Then use it in your code:

```rust
use wheel_rs::file_utils;
use wheel_rs::time_utils;
use wheel_rs::duration_serde;
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
- **Duration 序列化**: 为 `std::time::Duration` 类型提供自定义序列化和反序列化支持

## 模块说明

### `file_utils` 文件工具模块
提供文件操作相关的实用工具函数：
- `get_file_ext`: 从文件名中提取扩展名
- `calc_hash`: 计算文件的 SHA256 哈希值
- `is_cross_device_error`: 检查 IO 错误是否为跨设备错误

### `time_utils` 时间工具模块
时间相关的工具函数：
- `get_current_timestamp`: 获取当前时间戳（毫秒）

### `duration_serde` Duration 序列化模块
为 `Option<Duration>` 提供自定义序列化/反序列化：
- `serialize`: 将 Duration 序列化为字符串（例如，"5s" 表示 5 秒）
- `deserialize`: 将字符串反序列化为 Duration

## 使用方法

将以下内容添加到您的 `Cargo.toml` 文件中：

```toml
[dependencies]
wheel-rs = "0.1.0"
```

然后在您的代码中使用：

```rust
use wheel_rs::file_utils;
use wheel_rs::time_utils;
use wheel_rs::duration_serde;
```

## 许可证

本项目采用 MIT 许可证，详情请参见 [LICENSE](LICENSE) 文件。