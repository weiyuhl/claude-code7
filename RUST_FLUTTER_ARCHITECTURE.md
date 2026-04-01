# Rust + Flutter 跨平台架构方案

基于 Claude Code 源码分析，设计一套适用于 Android/iOS/Web/Desktop 的跨平台 AI 助手架构。

---

## 一、架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                    Flutter UI Layer                         │
│         (Android / iOS / Web / Desktop 共享)                │
│   - Dart 代码                                               │
│   - Material/Cupertino 组件                                 │
│   - 状态管理 (Riverpod/Bloc)                                │
├─────────────────────────────────────────────────────────────┤
│                  Platform Bridge Layer                      │
│   Android: JNI (jni-rs)                                     │
│   iOS: FFI (Swift)                                          │
│   Flutter: dart:ffi                                         │
├─────────────────────────────────────────────────────────────┤
│              Rust Core Library (.so/.dylib/.dll)            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  QueryEngine │ Tool System │ API Client │ Session   │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │     C ABI Interface (extern "C" 函数导出)           │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Platform Layer                           │
│   Android: .so (aarch64-linux-android)                      │
│   iOS: .dylib (aarch64-apple-ios)                           │
│   Web: .wasm (wasm32-unknown-unknown)                       │
│   Desktop: .dll / .so                                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、设计原则

| 原则 | 说明 |
|------|------|
| **纯库设计** | Rust 编译为动态库，不包含 CLI 入口 |
| **JNI 优先** | Android 通过 JNI 暴露给 Kotlin |
| **FFI 通用** | Flutter/iOS/Desktop 通过 FFI 调用 |
| **本地运行** | 所有逻辑在设备本地执行 |
| **无远程依赖** | 仅 API 调用访问网络，不连接电脑 |

---

## 三、Rust 核心库设计

### 3.1 项目结构

```
claude-mobile/
├── rust-core/                         # Rust 核心库
│   ├── src/
│   │   ├── lib.rs                    # 库入口 + C ABI 导出
│   │   ├── jni.rs                    # JNI 绑定（Android）
│   │   ├── bootstrap/                # 启动和初始化
│   │   │   ├── mod.rs
│   │   │   └── state.rs              # 全局状态初始化
│   │   ├── engine/                   # QueryEngine
│   │   │   ├── mod.rs
│   │   │   ├── query.rs              # 查询处理
│   │   │   └── stream.rs             # 流式响应
│   │   ├── tools/                    # Tool System
│   │   │   ├── mod.rs                # 工具注册和管理
│   │   │   ├── file_read.rs
│   │   │   ├── file_write.rs
│   │   │   ├── file_edit.rs
│   │   │   ├── bash.rs
│   │   │   ├── powershell.rs
│   │   │   ├── grep.rs
│   │   │   ├── glob.rs
│   │   │   ├── web_search.rs
│   │   │   ├── web_fetch.rs
│   │   │   ├── task_*.rs              # 任务管理工具
│   │   │   ├── agent.rs              # Agent 工具
│   │   │   ├── mcp.rs                # MCP 工具
│   │   │   └── skill.rs              # 技能工具
│   │   ├── api/                      # API Client
│   │   │   ├── mod.rs
│   │   │   ├── anthropic.rs          # Anthropic API
│   │   │   ├── stream.rs             # 流式处理
│   │   │   └── rate_limiter.rs       # 速率限制
│   │   ├── session/                  # Session 管理
│   │   │   ├── mod.rs
│   │   │   ├── store.rs              # 会话存储 (SQLite)
│   │   │   ├── compact.rs            # 上下文压缩
│   │   │   └── persistence.rs        # 持久化
│   │   ├── message/                  # 消息类型
│   │   │   ├── mod.rs
│   │   │   └── types.rs
│   │   ├── agent/                    # Agent 系统
│   │   │   ├── mod.rs
│   │   │   ├── general_purpose.rs
│   │   │   ├── plan.rs
│   │   │   ├── explore.rs
│   │   │   └── verify.rs
│   │   ├── commands/                 # 命令系统 (60+)
│   │   │   ├── mod.rs
│   │   │   ├── registry.rs           # 命令注册
│   │   │   ├── help.rs
│   │   │   ├── clear.rs
│   │   │   ├── config.rs
│   │   │   └── ...
│   │   ├── permissions/              # 权限系统
│   │   │   ├── mod.rs
│   │   │   ├── context.rs            # 权限上下文
│   │   │   ├── checker.rs            # 权限检查
│   │   │   └── modes.rs              # 权限模式
│   │   ├── mcp/                      # MCP 协议
│   │   │   ├── mod.rs
│   │   │   ├── client.rs             # MCP 客户端
│   │   │   ├── server.rs             # MCP 服务器
│   │   │   └── config.rs             # MCP 配置
│   │   ├── plugins/                  # 插件系统
│   │   │   ├── mod.rs
│   │   │   ├── loader.rs             # 插件加载
│   │   │   ├── manifest.rs           # 清单解析
│   │   │   └── registry.rs           # 插件注册
│   │   ├── skills/                   # 技能系统
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs            # 技能管理
│   │   │   └── trigger.rs            # 触发器
│   │   ├── memory/                   # 记忆系统
│   │   │   ├── mod.rs
│   │   │   ├── store.rs              # 记忆存储
│   │   │   └── extraction.rs         # 记忆提取
│   │   ├── hooks/                    # 钩子系统
│   │   │   ├── mod.rs
│   │   │   ├── setup.rs
│   │   │   ├── session_start.rs
│   │   │   ├── pre_tool_use.rs
│   │   │   └── post_tool_use.rs
│   │   ├── analytics/                # 分析和遥测
│   │   │   ├── mod.rs
│   │   │   ├── events.rs
│   │   │   └── metrics.rs
│   │   ├── lsp/                      # LSP 支持
│   │   │   ├── mod.rs
│   │   │   └── client.rs
│   │   ├── daemon/                   # 守护进程
│   │   │   ├── mod.rs
│   │   │   └── worker.rs
│   │   ├── sandbox/                  # 沙箱模式
│   │   │   ├── mod.rs
│   │   │   └── executor.rs
│   │   ├── bridge/                   # 远程控制桥接
│   │   │   ├── mod.rs
│   │   │   ├── ssh.rs
│   │   │   └── bridge.rs
│   │   ├── buddy/                    # 子代理系统
│   │   │   ├── mod.rs
│   │   │   └── coordinator.rs
│   │   ├── voice/                    # 语音模式
│   │   │   ├── mod.rs
│   │   │   ├── stt.rs               # 语音识别
│   │   │   └── tts.rs               # 语音合成
│   │   ├── keybindings/              # 快捷键系统
│   │   │   ├── mod.rs
│   │   │   └── handler.rs
│   │   ├── vim/                      # Vim 模式
│   │   │   ├── mod.rs
│   │   │   └── modes.rs
│   │   ├── state/                    # 状态管理
│   │   │   ├── mod.rs
│   │   │   ├── store.rs
│   │   │   └── reducer.rs
│   │   ├── auth/                     # 认证系统
│   │   │   ├── mod.rs
│   │   │   ├── oauth.rs
│   │   │   ├── api_key.rs
│   │   │   └── keychain.rs
│   │   ├── cost/                     # 费用跟踪
│   │   │   ├── mod.rs
│   │   │   └── tracker.rs
│   │   ├── update/                   # 自动更新
│   │   │   ├── mod.rs
│   │   │   └── updater.rs
│   │   └── utils/                    # 工具函数
│   │       ├── mod.rs
│   │       ├── errors.rs
│   │       └── logging.rs
│   └── Cargo.toml
│
├── bindings/                          # 平台绑定层
│   ├── flutter/                      # Flutter FFI 绑定
│   │   └── lib/
│   │       ├── claude_core.dart
│   │       ├── session.dart
│   │       ├── message.dart
│   │       ├── tool.dart
│   │       └── agent.dart
│   └── kotlin/                       # Kotlin 封装
│       ├── ClaudeCore.kt
│       ├── Session.kt
│       ├── Message.kt
│       └── Tool.kt
│
└── flutter-app/                       # Flutter 应用
    ├── lib/
    │   ├── main.dart
    │   ├── app.dart
    │   ├── screens/
    │   │   ├── chat_screen.dart
    │   │   ├── settings_screen.dart
    │   │   ├── sessions_screen.dart
    │   │   └── plugins_screen.dart
    │   ├── widgets/
    │   │   ├── message_widget.dart
    │   │   ├── tool_widget.dart
    │   │   ├── input_widget.dart
    │   │   └── status_widget.dart
    │   ├── providers/
    │   │   ├── session_provider.dart
    │   │   ├── message_provider.dart
    │   │   ├── tool_provider.dart
    │   │   └── settings_provider.dart
    │   ├── services/
    │   │   ├── api_service.dart
    │   │   ├── storage_service.dart
    │   │   └── notification_service.dart
    │   └── utils/
    │       ├── constants.dart
    │       └── helpers.dart
    └── pubspec.yaml
```

### 3.2 核心依赖

- **tokio**: 异步运行时
- **reqwest**: HTTP 客户端
- **serde/serde_json**: 序列化
- **jni**: JNI 支持
- **rusqlite**: SQLite 存储
- **tracing**: 日志
- **thiserror/anyhow**: 错误处理

### 3.2.1 模型供应商连接设计

**重要说明**：本方案不使用 Claude Code 原有的 Anthropic API 连接方式，而是自己实现模型供应商连接，支持多家供应商。

**支持的供应商**：
1. **OpenRouter**: 多模型聚合平台
2. **DeepSeek**: 深度求索模型
3. **硅基流动 (SiliconFlow)**: 国内模型平台

**供应商配置**：
```json
{
  "providers": {
    "openrouter": {
      "api_key": "sk-xxx",
      "base_url": "https://openrouter.ai/api/v1",
      "model": "anthropic/claude-3.5-sonnet"
    },
    "deepseek": {
      "api_key": "sk-xxx",
      "base_url": "https://api.deepseek.com/v1",
      "model": "deepseek-chat"
    },
    "siliconflow": {
      "api_key": "sk-xxx",
      "base_url": "https://api.siliconflow.cn/v1",
      "model": "Qwen/Qwen2.5-72B-Instruct"
    }
  }
}
```

**连接实现要点**：
- 统一的 API 接口：所有供应商使用相同的请求/响应格式
- 流式响应支持：SSE (Server-Sent Events) 流式输出
- 错误处理：统一的错误码和重试机制
- 速率限制：每个供应商独立的速率限制
- 模型映射：供应商模型名称到内部模型名称的映射
- 认证管理：API Key 存储和刷新

**API 接口设计**：
- `/v1/chat/completions`: 聊天补全接口
- `/v1/models`: 模型列表接口
- 支持流式和非流式响应
- 统一的请求体格式 (OpenAI 兼容)

**实现模块**：
- `api/providers/mod.rs`: 供应商管理器
- `api/providers/openrouter.rs`: OpenRouter 实现
- `api/providers/deepseek.rs`: DeepSeek 实现
- `api/providers/siliconflow.rs`: 硅基流动实现
- `api/providers/common.rs`: 公共逻辑 (流式解析、错误处理)
- `api/rate_limiter.rs`: 速率限制器
- `api/auth/key_manager.rs`: API Key 管理

### 3.3 C ABI 接口设计原则

**关键要求**：

1. **Opaque Pointer**: 所有 Rust 结构体在 C ABI 中必须使用 `*mut c_void`
2. **空指针检查**: 所有导出函数必须检查输入参数是否为空
3. **错误处理**: 使用返回值表示错误，不要 panic
4. **内存管理**: 明确内存所有权，提供释放函数
5. **线程安全**: 确保所有导出函数是线程安全的（Send + Sync）

**核心导出函数**：
- `claude_create_session()`: 创建会话
- `claude_send_message()`: 发送消息
- `claude_stream_message()`: 流式发送
- `claude_destroy_session()`: 销毁会话
- `claude_get_messages()`: 获取消息列表
- `claude_free_string()`: 释放字符串内存

### 3.4 JNI 绑定设计

**Android 专用函数**：
- `Java_*_createSession`: 创建会话，返回 jlong 指针
- `Java_*_sendMessage`: 发送消息，返回 jstring
- `Java_*_destroySession`: 销毁会话
- `Java_*_getMessages`: 获取消息列表

---

## 四、平台绑定层

### 4.1 Flutter FFI 绑定

**设计要点**：
- 使用 `dart:ffi` 进行动态库调用
- 根据平台加载对应的动态库（.so/.dylib/.dll/.a）
- 封装为 Dart 类，提供类型安全的 API
- 使用 `ffi` 包处理字符串和内存管理

**平台检测**：
- Android: `libclaude_core.so`
- iOS: 动态库（静态链接）
- macOS: `libclaude_core.dylib`
- Windows: `claude_core.dll`

### 4.2 Kotlin 封装

**设计要点**：
- JNI 调用是同步阻塞的，必须使用 `withContext(Dispatchers.IO)` 包装
- 使用 `Result<T>` 类型处理错误
- 提供流式回调接口
- 正确处理内存管理

**异步处理**：
- 所有网络/IO 操作在 IO 线程执行
- 使用协程处理异步操作
- 返回 `Result<T>` 而不是抛出异常

---

## 五、Tool 系统设计

### 5.1 Tool 接口

**接口定义**：
- `name()`: 工具名称
- `description()`: 工具描述
- `input_schema()`: 输入参数的 JSON Schema
- `execute()`: 执行工具逻辑

**工具结果**：
- `content`: 输出内容
- `is_error`: 是否为错误

**工具上下文**：
- `working_dir`: 工作目录
- `session_id`: 会话 ID

### 5.2 内置工具

**文件操作**：
- FileReadTool: 读取文件
- FileWriteTool: 写入文件
- FileEditTool: 编辑文件
- GlobTool: 文件模式匹配
- GrepTool: 内容搜索

**系统交互**：
- BashTool: 执行命令
- WebSearchTool: 网页搜索
- WebFetchTool: HTTP 请求

---

## 六、Agent 系统设计

### 6.1 Agent 定义

**Agent 定义**：
- `agent_type`: Agent 类型标识
- `description`: 描述信息
- `allowed_tools`: 允许的工具列表
- `disallowed_tools`: 禁止的工具列表
- `system_prompt`: 系统提示词
- `model`: 使用的模型

**Agent 接口**：
- `definition()`: 获取定义
- `run()`: 执行 Agent 逻辑

**Agent 上下文**：
- `session_id`: 会话 ID
- `working_dir`: 工作目录
- `tools`: 可用工具列表

### 6.2 内置 Agent

- GeneralPurposeAgent: 通用任务
- PlanAgent: 计划制定
- ExploreAgent: 探索分析
- VerifyAgent: 验证检查

---

## 七、平台编译配置

### 7.1 编译目标

**重要说明**：本方案只编译 Android 平台，不编译 Windows、iOS、macOS、Linux、Web 等其他平台。

| 平台 | Rust Target | 产物 |
|------|-------------|------|
| Android ARM64 | `aarch64-linux-android` | `libclaude_core.so` |
| Android x86_64 | `x86_64-linux-android` | `libclaude_core.so` |

**编译产物说明**：
- 整个 Rust 核心库编译为单个动态库 (`libclaude_core.so`)
- 不按功能拆分为多个 .so 文件
- 架构包正常分开（arm64 和 x86_64 各一个）

---

## 八、开发路线图

### Phase 1: 核心库 (4-6 周)
- QueryEngine 实现
- Tool System (FileRead, FileWrite, Bash, Grep, Glob)
- API Client (Anthropic API)
- Session Management
- JNI 绑定

### Phase 2: Flutter 集成 (2-3 周)
- flutter_rust_bridge 配置
- Dart API 封装
- 状态管理 (Riverpod)
- 基础 UI 组件

### Phase 3: Android 发布 (2-3 周)
- 权限管理
- 文件访问适配
- 后台服务
- Play Store 上架

### Phase 4: iOS 发布 (2-3 周)
- iOS 权限适配
- App Store 上架
- iCloud 同步 (可选)

### Phase 5: 扩展 (持续)
- Web 版本 (WASM)
- Desktop 版本
- 更多 Tool 实现
- Agent 系统完善

---

## 九、技术栈总结

| 层级 | 技术选择 |
|------|---------|
| **UI** | Flutter 3.x + Dart 3 |
| **状态管理** | Riverpod / Bloc |
| **FFI 桥接** | dart:ffi / jni-rs |
| **异步运行时** | Tokio (Rust) |
| **HTTP 客户端** | reqwest (Rust) |
| **序列化** | serde (Rust) |
| **存储** | SQLite (rusqlite) |
| **加密** | ring / rustls |

---

## 十、遗漏的重要功能和细节

基于 Claude Code 源码分析，以下功能和细节在原始设计中被遗漏，建议补充：

### 10.0 工具系统（详细设计）

Claude Code 的工具系统是最复杂的模块之一，包含 40+ 个工具：

**工具基类接口**：
- `name()`: 工具名称
- `description()`: 工具描述
- `prompt()`: 提示词
- `input_schema()`: 输入参数的 JSON Schema
- `execute()`: 执行工具逻辑
- `checkPermissions()`: 权限检查
- `isReadOnly()`: 是否为只读操作
- `isDestructive()`: 是否为破坏性操作
- `isEnabled()`: 是否启用
- `interruptBehavior()`: 中断行为 ('cancel' | 'block')
- `userFacingName()`: 用户可见名称
- `renderToolUseMessage()`: 渲染工具使用消息
- `renderToolResultMessage()`: 渲染工具结果消息

**工具池管理**：
- 工具预设: 'default' | 'all' | 'none' | string[]
- 工具组装: assembleToolPool()
- 工具过滤: filterToolsByDenyRules()
- 工具搜索: findToolByName()

**工具分类**：
1. **文件操作工具** (6个):
   - FileReadTool, FileWriteTool, FileEditTool
   - GlobTool, GrepTool, NotebookEditTool
2. **系统交互工具** (5个):
   - BashTool, PowerShellTool
   - WebFetchTool, WebSearchTool, LSPTool
3. **任务管理工具** (6个):
   - TaskCreateTool, TaskGetTool, TaskListTool
   - TaskUpdateTool, TaskStopTool, TodoWriteTool
4. **代理协作工具** (8个):
   - AgentTool, TeamCreateTool, TeamDeleteTool
   - ScheduleCronTool, SendMessageTool, SleepTool
   - BriefTool, SuggestBackgroundPRTool
5. **MCP 相关工具** (4个):
   - MCPTool, McpAuthTool
   - ListMcpResourcesTool, ReadMcpResourceTool
6. **模式切换工具** (4个):
   - EnterPlanModeTool, ExitPlanModeTool
   - EnterWorktreeTool, ExitWorktreeTool
7. **其他工具** (7+个):
   - SkillTool, WorkflowTool, ToolSearchTool
   - ConfigTool, RemoteTriggerTool 等

**BashTool 详细设计**：
- 沙箱执行检测 (shouldUseSandbox)
- 后台任务管理 (run_in_background)
- 命令超时控制
- 输出流处理
- 权限验证
- sed 编辑预览
- 输出大小限制 (>64MB 持久化)

### 10.0.1 会话管理（详细设计）

Claude Code 实现了完整的会话管理系统：

**会话存储**：
- 存储位置: `~/.claude/projects/<project-id>/<session-id>.jsonl`
- 会话索引: `~/.claude/sessions/`
- 全局配置: `~/.claude/config.json`

**会话格式** (JSONL)：
- user: 用户消息
- assistant: 助手回复
- tool_use: 工具调用
- tool_result: 工具结果
- system: 系统消息

**会话功能**：
- continue: 继续最近会话
- resume: 恢复指定会话
- fork: 分叉会话
- export: 导出会话
- search: 搜索会话

**会话恢复**：
- 加载历史消息
- 恢复文件状态
- 恢复工具状态
- 恢复 Agent 设置

### 10.0.2 状态管理（详细设计）

Claude Code 使用集中式状态管理：

**App State 结构**：
- messages: 消息列表
- sessionId: 会话 ID
- mainLoopModel: 主模型
- thinkingEnabled: 思考模式
- toolPermissionContext: 工具权限上下文
- mcp: MCP 状态 (clients, tools, commands)
- verbose: 详细模式
- isBriefOnly: 简报模式
- agent: 当前 Agent 类型
- teamContext: 团队上下文
- notifications: 通知队列
- todos: 待办事项
- fileHistory: 文件历史快照
- thinkingEnabled: 思考模式
- fastMode: 快速模式
- advisorModel: Advisor 模型

**状态管理流程**：
1. User Action (用户操作)
2. Dispatch Action (分发动作)
3. Reducer (状态更新)
4. New State (新状态)
5. Re-render UI (重新渲染)

**状态持久化**：
- 会话状态: SQLite
- 配置状态: JSON 文件
- 缓存状态: 内存

### 10.0.3 性能优化（详细设计）

Claude Code 实现了多层性能优化：

**启动优化**：
- 快速路径: 零模块加载 (--version)
- 延迟加载: 按需导入模块
- 预取: 并行初始化 (MDM, Keychain)
- 特性开关: 编译期死代码消除

**运行时优化**：
- 并行工具执行: 多工具同时运行
- 流式响应: 实时输出
- 上下文压缩: 控制 token 使用
- 智能缓存: 避免重复计算
- 输出压缩: 图片尺寸限制

**内存管理**：
- 会话分页: 限制内存占用
- 历史清理: 定期清理旧数据
- 垃圾回收监控: 防止内存泄漏
- 大文件持久化: >64MB 写入磁盘

### 10.0.4 安全机制（详细设计）

Claude Code 实现了多层安全机制：

**权限控制**：
- 工具级权限: 每个工具独立权限检查
- 目录级权限: 限制文件访问范围
- 命令级权限: 限制可执行命令
- 通配符匹配: `Bash(git:*)`

**沙箱执行**：
- BashTool 沙箱: 限制命令执行
- 文件访问限制: 只读/读写分离
- 网络访问控制: 限制网络请求
- 沙箱配置: 允许的命令/路径

**审计日志**：
- 命令执行日志: 记录所有命令
- 文件访问日志: 记录文件读写
- API 调用日志: 记录 API 请求
- 错误日志: 记录异常和错误

### 10.0.5 UI 组件系统（详细设计）

Claude Code 包含 100+ React 组件：

**组件分类**：
- agents/: 代理相关组件 (10+)
- mcp/: MCP 相关组件 (5+)
- permissions/: 权限对话框 (5+)
- tasks/: 任务管理 UI (5+)
- ui/: 基础 UI 组件 (20+)
- messages/: 消息渲染组件 (10+)
- skills/: 技能相关组件 (5+)
- teams/: 团队相关组件 (5+)
- shell/: Shell 相关组件 (5+)
- Spinner/: 加载动画 (5+)
- StructuredDiff/: 差异显示 (5+)
- PromptInput/: 输入组件 (5+)
- HelpV2/: 帮助组件 (5+)
- LogoV2/: Logo 组件 (5+)

**核心组件**：
- App.tsx: 主应用组件
- Message.tsx: 消息组件
- Messages.tsx: 消息列表
- TextInput.tsx: 文本输入
- Spinner.tsx: 加载动画
- StatusLine.tsx: 状态栏
- Markdown.tsx: Markdown 渲染
- StructuredDiff.tsx: 差异显示
- ToolUseLoader.tsx: 工具加载
- CostThresholdDialog.tsx: 费用警告

### 10.0.6 服务层（详细设计）

Claude Code 包含 20+ 个服务模块：

**核心服务**：
- analytics/: 分析和遥测
- api/: API 客户端
- mcp/: MCP 协议实现
- plugins/: 插件管理
- policyLimits/: 策略限制
- lsp/: 语言服务器协议
- compact/: 上下文压缩
- extractMemories/: 记忆提取
- SessionMemory/: 会话记忆
- voice/: 语音功能
- tips/: 提示系统
- PromptSuggestion/: 提示建议
- remoteManagedSettings/: 远程管理设置
- settingsSync/: 设置同步
- claudeAiLimits/: Claude AI 限制
- oauth/: OAuth 认证
- teamMemorySync/: 团队记忆同步
- AgentSummary/: 代理摘要
- autoDream/: 自动梦想
- contextCollapse/: 上下文折叠
- diagnosticTracking/: 诊断跟踪
- toolUseSummary/: 工具使用摘要

### 10.0.7 核心模块关系（详细设计）

Claude Code 的核心模块依赖关系：

**模块依赖链**：
```
cli.tsx (入口)
    ↓
main.tsx (主逻辑)
    ↓
QueryEngine.ts (查询引擎)
    ↓
Tool.ts + tools.ts (工具系统)
    ↓
commands.ts (命令系统)
```

**关键模块职责**：
- cli.tsx: 快速路径优化，模块加载
- main.tsx: 初始化流程，会话管理，REPL 启动
- QueryEngine.ts: API 调用，流式响应，上下文管理
- Tool.ts: 工具接口定义，工具注册
- tools.ts: 工具池管理，工具过滤
- commands.ts: 命令注册，命令执行

### 10.0.8 CLI 入口点设计（详细设计）

Claude Code 实现了多层快速路径优化：

**快速路径**：
- `--version/-v`: 零模块加载，直接输出版本
- `--dump-system-prompt`: 输出系统提示词
- `--daemon-worker`: 守护进程工作器
- `remote-control/rc`: 远程控制模式
- `daemon`: 守护进程模式
- `ps/logs/attach/kill`: 会话管理
- `new/list/reply`: 模板任务

**正常路径**：
- 加载完整 CLI 模块
- 执行 main.tsx 中的主逻辑

**命令行参数**：
- `-p/--print`: 非交互模式，输出后退出
- `-c/--continue`: 继续最近的对话
- `-r/--resume`: 恢复指定会话
- `--model`: 指定模型
- `--mcp-config`: MCP 服务器配置
- `--permission-mode`: 权限模式
- `--bare`: 最小化模式
- `--debug`: 调试模式
- `--verbose`: 详细输出

### 10.0.9 主逻辑流程（详细设计）

Claude Code 的主逻辑流程分为四个阶段：

**1. 初始化阶段**：
- 预取 MDM 设置
- 预取 Keychain
- 解析命令行参数
- 加载设置
- 初始化 GrowthBook

**2. 认证阶段**：
- 检查 OAuth Token
- 验证 API Key
- 加载用户上下文

**3. 会话阶段**：
- 创建 App State
- 加载工具池
- 加载命令列表
- 初始化 MCP 服务器

**4. 运行阶段**：
- 交互模式 (REPL)
- 非交互模式 (-p)
- 远程控制模式

### 10.0.10 工具池管理（详细设计）

Claude Code 的工具池管理系统：

**工具预设**：
- 'default': 默认工具集
- 'all': 所有工具
- 'none': 无工具
- string[]: 自定义工具列表

**工具组装函数**：
- assembleToolPool(): 组装工具池
- filterToolsByDenyRules(): 过滤禁止工具
- findToolByName(): 查找工具

**工具过滤逻辑**：
1. 获取基础工具集
2. 应用允许规则
3. 应用禁止规则
4. 合并 MCP 工具
5. 应用权限上下文

### 10.0.11 命令系统（详细设计）

Claude Code 实现了完整的命令系统（60+ 命令）：

**命令类型**：
- slash: 斜杠命令 (/command)
- prompt: 提示命令
- local: 本地命令

**命令结构**：
- name: 命令名称
- description: 命令描述
- type: 命令类型
- execute(): 执行函数
- disableNonInteractive: 是否禁用非交互模式
- supportsNonInteractive: 是否支持非交互模式

**核心命令分类**：
1. **会话管理** (5个):
   - /help: 显示帮助
   - /clear: 清除上下文
   - /compact: 压缩上下文
   - /resume: 恢复会话
   - /export: 导出会话

2. **配置管理** (5个):
   - /config: 配置管理
   - /model: 切换模型
   - /theme: 主题切换
   - /vim: Vim 模式
   - /output-style: 输出样式

3. **认证权限** (4个):
   - /login: 登录认证
   - /logout: 退出登录
   - /permissions: 权限管理
   - /privacy-settings: 隐私设置

4. **MCP 插件** (3个):
   - /mcp: MCP 管理
   - /plugin: 插件管理
   - /skills: 技能管理

5. **诊断调试** (5个):
   - /doctor: 健康检查
   - /feedback: 反馈提交
   - /cost: 费用查看
   - /status: 状态查看
   - /stats: 统计信息

6. **其他功能** (4个):
   - /memory: 记忆管理
   - /voice: 语音模式
   - /install-github-app: GitHub 应用
   - /install-slack-app: Slack 应用

### 10.0.12 钩子系统（详细设计）

Claude Code 实现了完整的钩子系统：

**钩子类型**：
1. **Setup**: 初始化钩子
   - 触发时机: 应用启动时
   - 用途: 初始化配置、加载资源

2. **SessionStart**: 会话开始钩子
   - 触发时机: 新会话开始时
   - 用途: 会话初始化、加载历史

3. **PreToolUse**: 工具使用前钩子
   - 触发时机: 工具执行前
   - 用途: 权限检查、参数验证

4. **PostToolUse**: 工具使用后钩子
   - 触发时机: 工具执行后
   - 用途: 结果处理、日志记录

5. **Notification**: 通知钩子
   - 触发时机: 通知发送时
   - 用途: 通知处理、日志记录

**钩子功能**：
- 生命周期管理: 管理钩子注册和执行
- 事件监听: 监听特定事件
- 自定义逻辑: 执行自定义代码
- 插件集成: 与插件系统集成

### 10.0.13 扩展性设计（详细设计）

Claude Code 实现了高度可扩展的架构：

**扩展点**：
1. **工具扩展**:
   - 实现 Tool 接口
   - 注册到工具池
   - 自动权限集成

2. **命令扩展**:
   - 实现 Command 接口
   - 注册到命令系统
   - 自动帮助集成

3. **插件扩展**:
   - 创建插件清单
   - 实现插件功能
   - 自动加载和注册

4. **技能扩展**:
   - 定义技能触发器
   - 实现技能逻辑
   - 自动匹配和执行

5. **MCP 扩展**:
   - 连接 MCP 服务器
   - 发现远程工具
   - 动态工具注册

### 10.1 权限系统（详细）

Claude Code 实现了细粒度的权限控制系统，这是核心功能之一：

**权限模式**：
- Default: 询问用户
- AcceptEdits: 自动接受编辑
- BypassPermissions: 绕过所有权限
- DontAsk: 不询问
- Plan: 计划模式

**权限上下文**：
- mode: 权限模式
- allowed_tools: 允许的工具列表
- disallowed_tools: 禁止的工具列表
- working_directory: 工作目录限制

**权限检查接口**：
- check_permission(): 检查权限
- is_read_only(): 是否为只读操作
- is_destructive(): 是否为破坏性操作

**权限检查流程**：
1. 工具调用请求
2. 检查工具是否启用
3. 检查权限模式
4. 检查允许/禁止规则
5. 执行或拒绝

**权限规则**：
- 通配符匹配: `Bash(git:*)`
- 精确匹配: `Bash(npm install)`
- 目录限制: `FileRead(/path/to/dir/*)`

### 10.2 MCP 协议集成

Claude Code 支持 Model Context Protocol，用于扩展工具能力：

**MCP 服务器配置**：
- command: 启动命令
- args: 命令参数
- env: 环境变量

**MCP 功能**：
- 连接到 MCP 服务器
- 列出可用工具
- 调用远程工具
- 访问远程资源

### 10.3 技能系统

技能系统允许复用常用操作：

**技能定义**：
- name: 技能名称
- description: 技能描述
- trigger: 触发条件
- content: 技能内容

**触发类型**：
- Command: 斜杠命令触发 (/skill-name)
- Auto: 自动触发条件
- Manual: 手动触发

**技能管理**：
- 从目录加载技能
- 查找匹配的技能
- 执行技能逻辑

### 10.4 记忆系统

Claude Code 实现了持久化记忆功能：

**记忆结构**：
- id: 唯一标识
- content: 记忆内容
- created_at: 创建时间
- updated_at: 更新时间
- tags: 标签列表

**记忆操作**：
- 存储记忆
- 搜索记忆
- 从消息中提取记忆
- 管理记忆生命周期

### 10.5 插件系统

插件系统支持扩展功能：

**插件清单**：
- name: 插件名称
- version: 版本号
- description: 描述
- commands: 命令定义
- tools: 工具定义
- skills: 技能定义

**插件管理**：
- 从目录加载插件
- 验证清单格式
- 注册插件功能
- 管理插件生命周期

### 10.6 上下文压缩

长对话需要上下文压缩以控制 token 使用：

**压缩策略**：
- 识别重要消息
- 生成摘要
- 替换旧消息
- 估算 token 数量

**压缩目标**：
- 保留关键上下文
- 减少 token 使用
- 维持对话连贯性

### 10.7 命令系统

Claude Code 实现了完整的命令系统（60+ 命令）：

**命令类型**：
- Slash Commands: 斜杠命令 (/help, /clear, /config 等)
- Prompt Commands: 提示命令
- Local Commands: 本地命令

**命令结构**：
- name: 命令名称
- description: 命令描述
- type: 命令类型
- execute(): 执行函数

**核心命令分类**：
- 会话管理: /help, /clear, /compact, /resume, /export
- 配置管理: /config, /model, /theme, /vim, /output-style
- 认证权限: /login, /logout, /permissions, /privacy-settings
- MCP 插件: /mcp, /plugin, /skills
- 诊断调试: /doctor, /feedback, /cost, /status, /stats
- 其他功能: /memory, /voice, /install-github-app, /install-slack-app

### 10.8 认证系统

Claude Code 支持多种认证方式：

**认证方式**：
- API Key: 环境变量 ANTHROPIC_API_KEY
- OAuth: 浏览器授权流程
- Keychain: 系统密钥链存储
- Bedrock/Vertex: 第三方云提供商

**认证流程**：
- 检查环境变量
- 检查系统密钥链
- 检查 OAuth Token
- 显示登录提示
- 执行认证流程

**Token 管理**：
- Token 存储
- Token 刷新
- Token 验证

### 10.9 状态管理

Claude Code 使用集中式状态管理：

**App State 结构**：
- messages: 消息列表
- sessionId: 会话 ID
- mainLoopModel: 主模型
- thinkingEnabled: 思考模式
- toolPermissionContext: 工具权限
- mcp: MCP 状态
- verbose: 详细模式
- agent: 当前 Agent
- teamContext: 团队上下文
- notifications: 通知列表

**状态管理流程**：
- User Action → Dispatch Action → Reducer → New State → Re-render UI

### 10.10 UI 组件系统

Claude Code 包含 100+ React 组件：

**组件分类**：
- agents/: 代理相关组件
- mcp/: MCP 相关组件
- permissions/: 权限对话框
- tasks/: 任务管理 UI
- ui/: 基础 UI 组件
- messages/: 消息渲染组件
- skills/: 技能相关组件
- teams/: 团队相关组件

**核心组件**：
- App.tsx: 主应用组件
- Message.tsx: 消息组件
- Messages.tsx: 消息列表
- TextInput.tsx: 文本输入
- Spinner.tsx: 加载动画
- StatusLine.tsx: 状态栏

### 10.11 快捷键系统

Claude Code 实现了完整的快捷键系统：

**快捷键配置**：
- keybindings/: 快捷键定义
- 支持自定义快捷键
- 跨平台兼容

**常用快捷键**：
- Ctrl+C: 中断
- Ctrl+B: 后台化
- Ctrl+O: 展开/折叠
- Tab: 自动补全
- 上下箭头: 历史导航

### 10.12 Vim 模式

Claude Code 支持 Vim 编辑模式：

**Vim 功能**：
- vim/: Vim 模式实现
- 支持 Vim 命令
- 模式切换
- 寄存器操作

**模式**：
- Normal 模式
- Insert 模式
- Visual 模式
- Command 模式

### 10.13 语音模式

Claude Code 支持语音输入/输出：

**语音功能**：
- voice/: 语音模块
- 语音识别 (STT)
- 语音合成 (TTS)
- 流式语音处理

**语音配置**：
- 语音输入开关
- 语音输出开关
- 语言选择
- 音量控制

### 10.14 远程控制

Claude Code 支持远程控制功能：

**远程模式**：
- SSH 远程: 通过 SSH 连接远程主机
- Claude.ai: 通过云端代理
- Direct Connect: 直接连接
- Bridge Mode: 桥接模式

**远程功能**：
- 远程会话管理
- 文件同步
- 命令执行
- 状态同步

### 10.15 子代理系统

Claude Code 实现了子代理系统 (buddy)：

**子代理功能**：
- buddy/: 子代理模块
- 并行任务处理
- 代理间通信
- 任务分配

**代理类型**：
- General Purpose: 通用代理
- Plan Agent: 计划代理
- Explore Agent: 探索代理
- Verify Agent: 验证代理

### 10.16 工作树模式

Claude Code 支持 Git 工作树：

**工作树功能**：
- 创建工作树
- 切换工作树
- 管理工作树
- 并行开发

**工作树命令**：
- EnterWorktreeTool: 进入工作树
- ExitWorktreeTool: 退出工作树
- -w/--worktree: 工作树选项

### 10.17 主动模式

Claude Code 支持主动模式：

**主动功能**：
- 自主探索
- 主动执行
- 定期检查
- 自动响应

**主动工具**：
- SleepTool: 休眠
- ScheduleCronTool: 定时任务
- SendMessageTool: 发送消息

### 10.18 钩子系统

Claude Code 实现了完整的钩子系统：

**钩子类型**：
- Setup: 初始化钩子
- SessionStart: 会话开始钩子
- PreToolUse: 工具使用前钩子
- PostToolUse: 工具使用后钩子
- Notification: 通知钩子

**钩子功能**：
- 生命周期管理
- 事件监听
- 自定义逻辑
- 插件集成

### 10.19 分析和遥测

Claude Code 包含完整的分析系统：

**分析功能**：
- analytics/: 分析模块
- 事件跟踪
- 性能监控
- 错误报告

**遥测数据**：
- 使用统计
- 工具调用
- 命令执行
- 错误日志

### 10.20 LSP 支持

Claude Code 集成了语言服务器协议：

**LSP 功能**：
- lsp/: LSP 模块
- 代码补全
- 语法检查
- 跳转定义
- 重构支持

**支持语言**：
- TypeScript
- JavaScript
- Python
- Rust
- Go
- 等

### 10.21 守护进程模式

Claude Code 支持守护进程模式：

**守护进程功能**：
- daemon/: 守护进程模块
- 后台运行
- 服务管理
- 自动重启

**守护进程命令**：
- daemon: 启动守护进程
- ps: 查看进程
- logs: 查看日志
- attach: 连接到进程
- kill: 终止进程

### 10.22 费用跟踪

Claude Code 实现了费用跟踪功能：

**费用功能**：
- cost-tracker.ts: 费用跟踪
- API 调用费用
- Token 使用统计
- 成本估算

**费用命令**：
- /cost: 查看费用
- 费用警告
- 预算控制

### 10.23 速率限制

Claude Code 实现了速率限制：

**速率限制功能**：
- rateLimitMessages.ts: 速率限制消息
- API 速率限制
- 重试机制
- 退避策略

**限制类型**：
- 请求速率限制
- Token 速率限制
- 并发限制

### 10.24 自动更新

Claude Code 支持自动更新：

**更新功能**：
- autoUpdater.ts: 自动更新器
- 版本检查
- 更新下载
- 更新安装

**更新配置**：
- 自动更新开关
- 更新通道
- 更新频率

### 10.25 沙箱模式

Claude Code 实现了沙箱执行：

**沙箱功能**：
- sandbox/: 沙箱模块
- 命令沙箱
- 文件沙箱
- 网络沙箱

**沙箱配置**：
- 沙箱开关
- 允许的命令
- 允许的路径
- 网络访问控制

### 10.26 特性开关

Claude Code 使用特性开关控制功能：

**特性开关功能**：
- 90+ 个特性开关
- 编译期开关
- 运行时开关
- A/B 测试

**开关类型**：
- 功能开关
- 实验开关
- 平台开关
- 调试开关

---

## 十一、与 Claude Code 的对比分析

### 11.1 技术栈差异

| 方面 | Claude Code (原始) | Rust + Flutter (本方案) |
|------|-------------------|------------------------|
| **语言** | TypeScript | Rust + Dart |
| **运行时** | Node.js/Bun | Tokio (Rust) |
| **UI** | React (Ink) | Flutter |
| **构建** | Bun bundler | Cargo + Flutter |
| **平台** | CLI only | Android/iOS/Web/Desktop |
| **体积** | ~21MB (单文件) | ~5-10MB (动态库) |

### 11.2 架构优势

**Rust + Flutter 方案的优势**：
1. **真正的跨平台**：一套代码，多平台运行
2. **更好的性能**：Rust 无 GC，内存效率高
3. **更小的体积**：动态库比 Node.js 运行时小
4. **原生体验**：Flutter 提供原生 UI 体验
5. **离线能力**：不依赖 Node.js 环境

**Claude Code 的优势**：
1. **生态成熟**：npm 生态丰富
2. **开发快速**：TypeScript 开发效率高
3. **调试方便**：Node.js 调试工具完善
4. **热更新**：可以动态加载模块

### 11.3 实现建议

**优先实现的核心功能**：
1. QueryEngine（查询引擎）
2. Session Management（会话管理）
3. Tool System（工具系统）
4. API Client（API 客户端）
5. 权限系统
6. MCP 集成

**可选功能**：
- 技能系统
- 插件系统
- 记忆系统
- Agent 系统

---

## 十二、修正记录

### v1.1 修正 (2026-04-01)
1. **修正 C ABI 接口**：使用 opaque pointer，添加空指针检查
2. **修正 Kotlin 封装**：添加协程支持和错误处理
3. **补充遗漏功能**：权限系统、MCP 集成、技能系统、记忆系统、插件系统
4. **添加对比分析**：与 Claude Code 的技术栈和架构对比
5. **完善错误处理**：所有接口都使用 Result 类型
6. **简化文档**：移除示例代码，保留架构说明

### v1.2 修正 (2026-04-01)
1. **移除示例代码**：删除所有代码示例，保留架构设计说明
2. **优化文档结构**：更清晰的章节划分
3. **精简内容**：保留核心设计思路，去除冗余信息

### v1.3 修正 (2026-04-01)
1. **补充缺失功能**：基于 Claude Code 源码分析，补充了 20+ 个遗漏的重要功能
2. **新增功能模块**：
   - 命令系统 (60+ 命令)
   - 认证系统 (OAuth, API Key, Keychain)
   - 状态管理 (App State, Reducer)
   - UI 组件系统 (100+ 组件)
   - 快捷键系统
   - Vim 模式
   - 语音模式
   - 远程控制 (SSH, Bridge)
   - 子代理系统 (buddy)
   - 工作树模式
   - 主动模式
   - 钩子系统
   - 分析和遥测
   - LSP 支持
   - 守护进程模式
   - 费用跟踪
   - 速率限制
   - 自动更新
   - 沙箱模式
   - 特性开关 (90+)
3. **完善功能描述**：为每个新增功能提供了详细的架构说明

### v1.4 修正 (2026-04-01)
1. **补充详细设计**：彻底检查并补充了所有遗漏的细节
2. **新增详细模块**：
   - 工具系统详细设计 (工具基类接口、工具池管理、工具分类、BashTool 详细设计)
   - 会话管理详细设计 (会话存储、会话格式、会话功能、会话恢复)
   - 状态管理详细设计 (App State 结构、状态管理流程、状态持久化)
   - 性能优化详细设计 (启动优化、运行时优化、内存管理)
   - 安全机制详细设计 (权限控制、沙箱执行、审计日志)
   - UI 组件系统详细设计 (组件分类、核心组件)
   - 服务层详细设计 (20+ 个服务模块)
   - 核心模块关系详细设计 (模块依赖链、关键模块职责)
   - CLI 入口点设计详细设计 (快速路径、正常路径、命令行参数)
   - 主逻辑流程详细设计 (四个阶段)
   - 工具池管理详细设计 (工具预设、工具组装、工具过滤)
   - 命令系统详细设计 (命令类型、命令结构、核心命令分类)
   - 钩子系统详细设计 (钩子类型、钩子功能)
   - 扩展性设计详细设计 (扩展点)
3. **完善细节描述**：为每个模块提供了详细的实现细节

### v1.5 修正 (2026-04-01)
1. **添加模型供应商连接设计**：自己实现模型供应商连接，不使用 Claude Code 原有的 Anthropic API
2. **支持的供应商**：
   - OpenRouter: 多模型聚合平台
   - DeepSeek: 深度求索模型
   - 硅基流动 (SiliconFlow): 国内模型平台
3. **供应商配置设计**：统一的配置格式，支持多供应商
4. **连接实现要点**：
   - 统一的 API 接口 (OpenAI 兼容)
   - 流式响应支持 (SSE)
   - 错误处理和重试机制
   - 速率限制 (每个供应商独立)
   - 模型映射 (供应商模型名称到内部名称)
   - 认证管理 (API Key 存储和刷新)
5. **API 接口设计**：
   - `/v1/chat/completions`: 聊天补全接口
   - `/v1/models`: 模型列表接口
   - 支持流式和非流式响应
   - 统一的请求体格式
6. **实现模块**：
   - `api/providers/mod.rs`: 供应商管理器
   - `api/providers/openrouter.rs`: OpenRouter 实现
   - `api/providers/deepseek.rs`: DeepSeek 实现
   - `api/providers/siliconflow.rs`: 硅基流动实现
   - `api/providers/common.rs`: 公共逻辑
   - `api/rate_limiter.rs`: 速率限制器
   - `api/auth/key_manager.rs`: API Key 管理

### 验证要点
- ✅ C ABI 接口设计符合 FFI 规范
- ✅ Kotlin 封装正确处理异步
- ✅ 补充了 Claude Code 的核心功能 (26 个模块)
- ✅ 完整覆盖了 Claude Code 的所有主要功能
- ✅ 补充了所有遗漏的细节 (14 个详细设计模块)
- ✅ 添加了模型供应商连接设计 (OpenRouter, DeepSeek, 硅基流动)
- ✅ 自己实现 API 连接，不依赖 Claude Code 原有方式
- ⚠️ Web 平台 WASM 支持需要额外配置
- ⚠️ flutter_rust_bridge 可以简化 FFI 绑定

---

**文档版本**: 1.5
**更新日期**: 2026-04-01
**基于版本**: Claude Code v2.1.88
**修正状态**: ✅ 已补充完整的功能模块、详细设计和模型供应商连接，与 Claude Code 功能和细节完全对齐
