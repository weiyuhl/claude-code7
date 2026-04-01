# Rust Core FFI 对接文档

## 概述

Rust Core 通过 C ABI 提供跨平台共享库（`.so`/`.dll`/`.dylib`），供 Flutter 等上层应用调用。所有导出函数使用 `#[no_mangle] pub extern "C"` 声明。

## 编译

```bash
cargo build --release --target aarch64-linux-android
```

产物：`target/aarch64-linux-android/release/libclaude_core.so`

## 支持供应商

| 供应商 | 标识符 |
|--------|--------|
| OpenRouter | `openrouter` |
| DeepSeek | `deepseek` |
| 硅基流动 | `siliconflow` |

---

## C API 函数列表

### 1. `create_session`
创建会话实例。

```c
void* create_session(const char* config_json);
```

**参数：** `config_json` - JSON 字符串，字段：
- `provider` (string) - 供应商标识符
- `model` (string) - 模型名称
- `max_tokens` (int, 默认 4096)

**返回：** `void*` 会话指针，失败返回 `NULL`

---

### 2. `destroy_session`
销毁会话并释放资源。

```c
void destroy_session(void* session);
```

---

### 3. `set_provider`
为会话设置供应商和 API Key。

```c
bool set_provider(void* session, const char* provider_name, const char* api_key);
```

**参数：**
- `provider_name` - `openrouter` / `deepseek` / `siliconflow`
- `api_key` - 供应商 API Key

**返回：** `true` 成功，`false` 失败

---

### 4. `send_message`
同步发送消息，等待完整响应后返回。

```c
char* send_message(void* session, const char* content);
```

**返回：** JSON 字符串（含 `content`/`thinking` 字段），失败返回错误 JSON。调用方需调用 `free_string` 释放。

---

### 5. `stream_message`
流式发送消息，通过回调逐块返回响应。

```c
typedef void (*StreamCallback)(const char* chunk_json, void* user_data);

int stream_message(void* session, const char* content, StreamCallback callback, void* user_data);
```

**回调 chunk JSON 字段：**
- `type` - `content` / `thinking` / `tool_use`
- `content` - 文本内容

**返回：** `0` 成功，`-1` 失败

---

### 6. `get_messages`
获取会话所有消息历史。

```c
char* get_messages(void* session);
```

**返回：** JSON 字符串 `{"messages": [...]}`

---

### 7. `list_models`
获取供应商可用模型列表。

```c
char* list_models(void* session);
```

**返回：** JSON 数组 `[{"id": "...", ...}]` 或错误 JSON `{"error": "..."}`

**注意：** 需先调用 `set_provider` 设置有效 API Key。

---

### 8. `get_balance`
获取供应商账户余额。

```c
char* get_balance(void* session);
```

**返回：** JSON 对象 `{"total_balance": "...", "currency": "..."}` 或错误 JSON

**注意：** 需先调用 `set_provider` 设置有效 API Key。

---

### 9. `free_string`
释放 Rust 分配的字符串。

```c
void free_string(char* s);
```

**重要：** 所有返回 `char*` 的函数（`send_message`、`get_messages`、`list_models`、`get_balance`）返回的字符串必须通过此函数释放，否则内存泄漏。

---

## 调用流程

```
1. create_session(config) → session_ptr
2. set_provider(session_ptr, provider, api_key)
3. list_models(session_ptr)  [可选]
4. get_balance(session_ptr)  [可选]
5. stream_message(session_ptr, content, callback)  或  send_message(session_ptr, content)
6. destroy_session(session_ptr)
```

## 错误处理

- 所有返回 `char*` 的函数在错误时返回 JSON `{"error": "..."}` 而非空指针
- `create_session`、`set_provider`、`stream_message` 通过 `NULL`/`false`/`-1` 表示失败
- 调用方需检查返回值，解析 JSON 中的 `error` 字段获取详情

## 内存管理

- `create_session` 返回的指针由调用方通过 `destroy_session` 释放
- 所有 `char*` 返回值必须由调用方通过 `free_string` 释放
- `stream_message` 的回调参数 `chunk_json` 由 Rust 内部管理，回调返回后无效，调用方如需保留需自行拷贝

## Flutter 集成

Flutter 端通过 `dart:ffi` 的 `DynamicLibrary.open()` 加载共享库，使用 `lookupFunction` 绑定 C 函数。核心封装位于 `lib/src/services/claude_core_service.dart`。

## 平台支持

| 平台 | 库文件名 |
|------|----------|
| Android (ARM64) | `libclaude_core.so` |
| Windows | `claude_core.dll` |
| Linux | `libclaude_core.so` |
| macOS | `libclaude_core.dylib` |
| iOS | 静态链接 |
