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

---

## 三、Flutter 项目架构

### 3.1 目录结构

```
flutter_app/lib/
├── main.dart                              # 应用入口 (ProviderScope)
└── src/
    ├── core/                              # 核心配置层
    │   ├── app_theme.dart                 # iOS 风格主题定义
    │   ├── app_config.dart                # 应用常量配置
    │   └── core.dart                      # 导出文件
    │
    ├── services/                          # 服务层（外部接口封装）
    │   ├── claude_core_service.dart       # Rust FFI 动态库加载和函数绑定
    │   └── services.dart                  # 导出文件
    │
    ├── repositories/                      # 仓库层（业务数据访问）
    │   ├── session_repository.dart        # 会话管理（创建/销毁/操作）
    │   └── repositories.dart              # 导出文件
    │
    ├── viewmodels/                        # 视图模型层（Riverpod 状态管理）
    │   ├── providers.dart                 # 基础 Provider（ClaudeCore, SessionRepository）
    │   ├── chat_viewmodel.dart            # 聊天页面状态和逻辑
    │   ├── settings_viewmodel.dart        # 设置页面状态和逻辑
    │   └── viewmodels.dart                # 导出文件
    │
    └── views/                             # 视图层（UI 页面和组件）
        ├── chat/                          # 聊天页面
        │   ├── chat_page.dart             # 聊天主页面
        │   ├── chat.dart                  # 导出文件
        │   └── widgets/                   # 聊天组件
        │       ├── message_bubble.dart    # 消息气泡组件
        │       └── widgets.dart           # 导出文件
        │
        ├── settings/                      # 设置页面
        │   ├── settings_page.dart         # 设置入口页（列表式）
        │   ├── provider_config_page.dart  # 供应商配置管理页（双Tab）
        │   └── settings.dart              # 导出文件
        │
        └── views.dart                     # 导出文件
```

### 3.2 架构分层说明

| 层级 | 职责 | 禁止依赖 |
|------|------|----------|
| **Views** | UI 展示和用户交互 | 不能直接调用 Service |
| **ViewModels** | 状态管理和业务逻辑 | 不能直接操作 UI |
| **Repositories** | 数据访问和会话管理 | 不能包含业务逻辑 |
| **Services** | 外部接口封装（FFI） | 不能包含业务逻辑 |
| **Core** | 常量和主题配置 | 无依赖 |

**数据流向：** Views → ViewModels → Repositories → Services

### 3.3 状态管理

使用 **Riverpod** 进行状态管理：

```dart
// 基础 Provider
final claudeCoreProvider = Provider<ClaudeCore>((ref) => ClaudeCore());
final sessionRepositoryProvider = Provider<SessionRepository>((ref) => ...);

// 状态 Notifier
final chatNotifierProvider = StateNotifierProvider<ChatNotifier, ChatState>(...);
final settingsNotifierProvider = StateNotifierProvider<SettingsNotifier, SettingsState>(...);
```

### 3.4 页面结构

#### 设置入口页 (`settings_page.dart`)
- iOS 风格设置列表
- 当前仅一个入口：供应商配置

#### 供应商配置页 (`provider_config_page.dart`)
- **Tab 1: 供应商配置** - 选择供应商、输入 API Key、保存设置
- **Tab 2: 模型列表** - 获取模型列表、获取余额、模型选择列表

---

## 四、Rust C API 对接

### 4.1 函数列表

| 函数名 | 用途 | 返回值 |
|--------|------|--------|
| `create_session` | 创建会话 | `void*` 会话指针 |
| `destroy_session` | 销毁会话 | `void` |
| `set_provider` | 设置供应商和 API Key | `bool` |
| `send_message` | 同步发送消息 | `char*` JSON 响应 |
| `stream_message` | 流式发送消息 | `int` (0成功/-1失败) |
| `get_messages` | 获取消息历史 | `char*` JSON |
| `list_models` | 获取模型列表 | `char*` JSON 数组 |
| `get_balance` | 获取账户余额 | `char*` JSON 对象 |
| `free_string` | 释放字符串 | `void` |

### 4.2 支持供应商

| 供应商标识符 | 说明 |
|-------------|------|
| `openrouter` | OpenRouter |
| `deepseek` | DeepSeek |
| `siliconflow` | 硅基流动 |

### 4.3 调用流程

```
1. create_session(config_json) → session_ptr
2. set_provider(session_ptr, provider, api_key)
3. list_models(session_ptr)     [可选]
4. get_balance(session_ptr)     [可选]
5. stream_message(...) 或 send_message(...)
6. destroy_session(session_ptr)
```

### 4.4 内存管理规则

- `create_session` 返回的指针必须通过 `destroy_session` 释放
- 所有 `char*` 返回值必须通过 `free_string` 释放
- `stream_message` 回调中的 `chunk_json` 由 Rust 内部管理，回调返回后无效
- Dart 端使用 `toNativeUtf8()` 创建的指针必须用 `calloc.free()` 释放

### 4.5 错误处理

- 返回 `char*` 的函数在错误时返回 JSON `{"error": "..."}` 而非空指针
- `create_session` 失败返回 `NULL`
- `set_provider` 失败返回 `false`
- `stream_message` 失败返回 `-1`

---

## 五、构建和测试

### 5.1 构建 Flutter APK

```bash
cd d:/claude-code7/flutter_app

# 构建 Debug 版本
flutter build apk --debug

# 构建 Release 版本
flutter build apk --release
```

### 5.2 运行应用

```bash
# 连接 Android 设备或启动模拟器
flutter run
```

### 5.3 完整开发流程

```
1. 修改 Rust 代码 (rust-core/src/)
2. 编译 Rust 库：cargo build --release --target aarch64-linux-android
3. 复制共享库到 Flutter 项目 jniLibs/arm64-v8a/
4. 修改 Flutter 代码（按架构分层修改）
5. 获取依赖：flutter pub get
6. 测试功能：flutter run 或 flutter build apk
```

---

**最后更新**：2026-04-01  
**项目版本**：v0.1.0  
**架构**：Riverpod + MVVM (View → ViewModel → Repository → Service)
