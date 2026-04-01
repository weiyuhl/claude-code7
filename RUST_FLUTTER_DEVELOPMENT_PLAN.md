# Rust 核心库开发计划

基于 RUST_FLUTTER_ARCHITECTURE.md 架构文档，制定以下开发顺序。

**重要说明**：本计划只关注 Rust 核心库开发，不包含 Flutter/Android/iOS 等前端应用。

---

## 第一阶段：核心基础 + 供应商连接 (并行)

### 1.1 项目初始化
- 创建 Rust 项目结构 (rust-core)
- 配置 Cargo.toml 和依赖
- 定义项目目录结构

### 1.2 基本消息类型
- 定义 Message 结构体
- 定义 Role 枚举 (user, assistant, system, tool_use, tool_result)
- 实现消息序列化/反序列化

### 1.3 C ABI 接口
- 实现 create_session()
- 实现 send_message()
- 实现 stream_message()
- 实现 destroy_session()
- 实现 get_messages()
- 实现 free_string()
- 空指针检查和错误处理

### 1.4 JNI 绑定 (Android)
- 实现创建会话 JNI 函数
- 实现发送消息 JNI 函数
- 实现销毁会话 JNI 函数
- 实现获取消息列表 JNI 函数

### 1.5 绑定层实现
- **JNI 绑定** (Android):
  - 创建会话，返回 jlong 指针
  - 发送消息，返回 jstring
  - 销毁会话
  - 获取消息列表
- **C ABI 接口**:
  - create_session(): 创建会话
  - send_message(): 发送消息
  - stream_message(): 流式发送
  - destroy_session(): 销毁会话
  - get_messages(): 获取消息列表
  - free_string(): 释放字符串内存
- **绑定层设计要点**:
  - 所有 Rust 结构体使用 opaque pointer (*mut c_void)
  - 空指针检查
  - 错误处理使用返回值
  - 明确内存所有权
  - 线程安全 (Send + Sync)

### 1.6 供应商连接 (提前)
- 实现 api/providers/mod.rs
- 定义统一的 Provider 接口
- 实现供应商选择逻辑
- 实现 api/providers/openrouter.rs (OpenRouter 连接)
- 实现 api/providers/deepseek.rs (DeepSeek 连接)
- 实现 api/providers/siliconflow.rs (硅基流动连接)
- 实现 api/providers/common.rs (公共逻辑)
- SSE 流式解析
- 错误处理和重试
- 速率限制器
- 实现 api/auth/key_manager.rs (认证管理)
- API Key 存储
- API Key 刷新

---

## 第二阶段：基本聊天 + 会话管理

### 2.1 QueryEngine
- 实现 engine/query.rs
- 发送消息到供应商
- 接收流式响应
- 上下文管理

### 2.2 Session 管理
- 实现 session/store.rs (SQLite)
- 会话创建和加载
- 会话持久化
- 消息历史管理

### 2.3 流式响应
- 实现 SSE 流式解析
- 实现流式响应处理
- 实现流式输出到 UI

---

## 第三阶段：工具系统 + 权限系统 (合并)

### 3.1 工具基类
- 实现 tools/mod.rs
- 定义 Tool trait
- 定义 ToolResult 和 ToolError
- 定义 ToolContext

### 3.2 文件操作工具
- 实现 FileReadTool
- 实现 FileWriteTool
- 实现 FileEditTool
- 实现 GlobTool
- 实现 GrepTool

### 3.3 权限上下文
- 实现 permissions/context.rs
- 定义权限模式
- 定义权限规则

### 3.4 权限检查
- 实现 permissions/checker.rs
- 工具级权限检查
- 目录级权限检查
- 命令级权限检查

### 3.5 工具池管理
- 实现工具注册
- 实现工具过滤
- 实现工具搜索

---

## 第四阶段：命令系统

### 4.1 命令注册
- 实现 commands/registry.rs
- 定义 Command trait
- 命令发现和加载

### 4.2 核心命令
- /help: 显示帮助
- /clear: 清除上下文
- /config: 配置管理
- /model: 切换模型
- /login: 登录认证

### 4.3 命令解析
- 斜杠命令解析
- 命令参数处理
- 命令自动补全

---

## 第五阶段：MCP 集成

### 5.1 MCP 客户端
- 实现 mcp/client.rs
- 连接到 MCP 服务器
- 发现远程工具

### 5.2 MCP 工具
- 实现 MCPTool
- 实现 McpAuthTool
- 实现 ListMcpResourcesTool
- 实现 ReadMcpResourceTool

### 5.3 MCP 配置
- 实现 mcp/config.rs
- 服务器配置解析
- 服务器管理

---

## 第六阶段：高级功能

### 6.1 上下文压缩
- 实现 session/compact.rs
- 识别重要消息
- 生成摘要
- 替换旧消息

### 6.2 记忆系统
- 实现 memory/store.rs
- 记忆存储和搜索
- 记忆提取
- 记忆生命周期管理

### 6.3 技能系统
- 实现 skills/manager.rs
- 技能加载
- 技能匹配
- 技能执行

### 6.4 插件系统
- 实现 plugins/loader.rs
- 插件清单解析
- 插件注册
- 插件生命周期管理

---

## 第七阶段：Agent 系统

### 7.1 Agent 基础
- 实现 agent/mod.rs
- 定义 Agent trait
- 定义 AgentDefinition
- 定义 AgentContext

### 7.2 内置 Agent
- 实现 GeneralPurposeAgent
- 实现 PlanAgent
- 实现 ExploreAgent
- 实现 VerifyAgent

### 7.3 子代理系统
- 实现 buddy/coordinator.rs
- 并行任务处理
- 代理间通信
- 任务分配

---

## 第八阶段：状态和配置

### 8.1 状态管理
- 实现 state/store.rs
- 实现 state/reducer.rs
- App State 结构
- 状态持久化

### 8.2 配置管理
- 实现配置加载
- 实现配置保存
- 配置验证

### 8.3 Vim 模式
- 实现 vim/modes.rs
- Normal 模式
- Insert 模式
- Visual 模式

---

## 第九阶段：扩展功能

### 9.1 语音模式
- 实现 voice/stt.rs (语音识别)
- 实现 voice/tts.rs (语音合成)
- 流式语音处理

### 9.2 远程控制
- 实现 bridge/ssh.rs
- 实现 bridge/bridge.rs
- SSH 远程连接
- Bridge 模式

### 9.3 守护进程
- 实现 daemon/worker.rs
- 后台运行
- 服务管理
- 自动重启

### 9.4 LSP 支持
- 实现 lsp/client.rs
- 代码补全
- 语法检查
- 跳转定义

---

## 第十阶段：优化和发布

### 10.1 性能优化
- 启动优化
- 内存优化
- 流式响应优化
- 缓存优化

### 10.2 平台适配
- Android 适配 (ARM64)
- 只编译 Android 平台
- 不编译 Windows、iOS、macOS、Linux、Web

### 10.3 测试和发布
- 单元测试
- 集成测试
- 端到端测试
- 应用商店发布

---

## 风险提示

| 风险 | 建议 |
|------|------|
| SSE 流式解析复杂 | 使用成熟的 SSE 库 (futures-stream) |
| JNI 内存泄漏 | 使用 GlobalRef 管理 Java 对象引用 |
| SQLite 并发问题 | 使用 rusqlite 的连接池 |
| 权限系统复杂度 | 先实现基础权限，后续迭代增强 |

---

## 开发原则

1. **先核心后扩展**：先实现基本聊天和供应商，再实现其他功能
2. **先简单后复杂**：先实现简单工具，再实现复杂工具
3. **先本地后远程**：先实现本地功能，再实现远程功能
4. **先基础后高级**：先实现基础功能，再实现高级功能

---

**文档版本**: 1.0
**创建日期**: 2026-04-01
**基于文档**: RUST_FLUTTER_ARCHITECTURE.md v1.5