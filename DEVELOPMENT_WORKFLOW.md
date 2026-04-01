# Rust-Flutter 项目编译和对接指南

本文档指导如何在当前项目中编译 Rust 核心库，并与 Flutter 应用对接使用。

---

## 一、Rust 核心库编译

### 1.1 编译命令

#### 编译 Android ARM64 版本
```bash
cd d:/claude-code7/rust-core

# 编译 Release 版本（推荐）
cargo build --release --target aarch64-linux-android

# 或编译 Debug 版本（开发调试用）
cargo build --target aarch64-linux-android
```

#### 编译说明
- **目标架构**：仅编译 `aarch64-linux-android`（ARM64）
- **不编译其他平台**：不需要编译 x86、x86_64、armv7 等架构
- **Release 版本**：优化编译，体积小，性能高
- **Debug 版本**：包含调试信息，体积大，适合开发

### 1.2 编译输出

编译成功后，输出文件位于：

```
# Release 版本
rust-core/target/aarch64-linux-android/release/libclaude_core.so

# Debug 版本
rust-core/target/aarch64-linux-android/debug/libclaude_core.so
```

### 1.3 编译配置

#### Cargo.toml 配置
```toml
[lib]
name = "claude_core"
crate-type = ["cdylib", "rlib"]  # cdylib 用于生成共享库

[profile.release]
opt-level = "z"        # 最高压缩优化
lto = true             # 链接时优化
codegen-units = 1      # 减少代码生成单元
panic = "abort"        # 终止 panic
strip = true           # 剥离调试信息

[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
# ... 其他依赖
```

#### .cargo/config.toml 配置
```toml
[target.aarch64-linux-android]
linker = "aarch64-linux-android21-clang"
ar = "aarch64-linux-android-ar"

[env]
ANDROID_NDK_HOME = "D:\\Android\\Ndk\\android-ndk-r27c"
PATH = "${env:ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/windows-x86_64/bin;${env:PATH}"
```

---

## 二、将共享库复制到 Flutter 项目

### 2.1 复制命令

#### Windows PowerShell
```powershell
# Release 版本
Copy-Item "d:\claude-code7\rust-core\target\aarch64-linux-android\release\libclaude_core.so" `
          "d:\claude-code7\flutter_app\android\app\src\main\jniLibs\arm64-v8a\libclaude_core.so"

# Debug 版本
Copy-Item "d:\claude-code7\rust-core\target\aarch64-linux-android\debug\libclaude_core.so" `
          "d:\claude-code7\flutter_app\android\app\src\main\jniLibs\arm64-v8a\libclaude_core.so"
```

### 2.2 目标目录说明

Flutter Android 项目的共享库目录：
```
flutter_app/android/app/src/main/jniLibs/arm64-v8a/
```

- **arm64-v8a**：仅支持 ARM64 架构
- **不需要其他架构目录**：不需要 x86、x86_64、armv7 等目录

## 三、构建和测试

### 3.1 构建 Flutter APK

```bash
cd d:/claude-code7/flutter_app

# 构建 Debug 版本
flutter build apk --debug

# 构建 Release 版本
flutter build apk --release
```

### 3.2 运行应用

```bash
# 连接 Android 设备或启动模拟器
flutter run
```

### 3.3 测试功能

1. **初始化测试**：验证 Rust 库是否正确加载
2. **消息发送测试**：验证能否发送消息并接收响应
3. **供应商切换测试**：验证能否切换不同的供应商
4. **错误处理测试**：验证错误情况下的行为

---

## 四、常见问题

### 问题 1：找不到 NDK 工具链
**错误**：`failed to find tool 'aarch64-linux-android21-clang'`

**解决方案**：
1. 检查 `.cargo/config.toml` 中的 NDK 路径配置
2. 确保 NDK 已安装到 `D:\Android\Ndk\android-ndk-r27c`
3. 重启终端使环境变量生效

### 问题 2：共享库找不到
**错误**：`DynamicLibrary.open failed: dlopen failed: library "libclaude_core.so" not found`

**解决方案**：
1. 确认 `.so` 文件已复制到正确的目录
2. 检查文件名是否为 `libclaude_core.so`
3. 运行 `flutter clean && flutter build apk` 重新构建

### 问题 3：内存泄漏
**解决方案**：
- 使用 `toNativeUtf8()` 创建的指针必须用 `calloc.free()` 释放
- Rust 返回的字符串必须调用 `free_string()` 释放
- 会话必须调用 `destroy_session()` 销毁

---

## 四、开发流程总结

```
1. 修改 Rust 代码
2. 编译 Rust 库：cargo build --release --target aarch64-linux-android
3. 复制共享库到 Flutter 项目
4. 修改 Flutter 代码（如果需要）
5. 构建并运行 Flutter 应用
6. 测试功能
```

---

**最后更新**：2026-04-01  
**项目版本**：v0.1.0
