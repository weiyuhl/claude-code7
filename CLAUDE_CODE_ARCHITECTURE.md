# Claude Code 完整架构文档

## 项目概述

**项目名称**: Claude Code Source (v2.1.88)
**项目类型**: AI 编程助手 CLI 工具
**源码来源**: 从 `@anthropic-ai/claude-code` npm 包的 source map 还原
**技术栈**: TypeScript + React (Ink) + Node.js/Bun

---

## 一、整体架构

### 1.1 目录结构

```
src/
├── entrypoints/           # 程序入口点
│   └── cli.tsx           # CLI 主入口
├── main.tsx              # 主逻辑入口
├── Tool.ts               # 工具类型系统定义
├── Task.ts               # 任务管理核心
├── QueryEngine.ts        # API 查询引擎
├── commands.ts           # 命令注册系统
├── tools.ts              # 工具池管理
├── assistant/            # 会话管理
├── bridge/               # 远程控制桥接
├── buddy/                # 子代理系统
├── cli/                  # CLI 处理器
├── commands/             # 斜杠命令实现 (60+)
├── components/           # UI 组件
├── constants/            # 常量配置
├── context/              # 上下文管理
├── daemon/               # 守护进程
├── hooks/                # React Hooks
├── ink/                  # 终端渲染引擎
├── keybindings/          # 快捷键系统
├── plugins/              # 插件系统
├── services/             # 核心服务
├── skills/               # 技能系统
├── state/                # 状态管理
├── tools/                # 工具实现 (40+)
├── utils/                # 工具函数
└── vim/                  # Vim 模式
```

### 1.2 核心模块关系

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

---

## 二、CLI 工具设计

### 2.1 入口点设计 (cli.tsx)

**快速路径优化**:
- `--version/-v`: 零模块加载，直接输出版本
- `--dump-system-prompt`: 输出系统提示词
- `--daemon-worker`: 守护进程工作器
- `remote-control/rc`: 远程控制模式
- `daemon`: 守护进程模式
- `ps/logs/attach/kill`: 会话管理
- `new/list/reply`: 模板任务

**正常路径**:
- 加载完整 CLI 模块
- 执行 main.tsx 中的主逻辑

### 2.2 主逻辑流程 (main.tsx)

```
1. 初始化阶段
   ├── 预取 MDM 设置
   ├── 预取 Keychain
   ├── 解析命令行参数
   ├── 加载设置
   └── 初始化 GrowthBook

2. 认证阶段
   ├── 检查 OAuth Token
   ├── 验证 API Key
   └── 加载用户上下文

3. 会话阶段
   ├── 创建 App State
   ├── 加载工具池
   ├── 加载命令列表
   └── 初始化 MCP 服务器

4. 运行阶段
   ├── 交互模式 (REPL)
   ├── 非交互模式 (-p)
   └── 远程控制模式
```

### 2.3 命令行参数

| 参数 | 说明 |
|------|------|
| `-p/--print` | 非交互模式，输出后退出 |
| `-c/--continue` | 继续最近的对话 |
| `-r/--resume` | 恢复指定会话 |
| `--model` | 指定模型 |
| `--mcp-config` | MCP 服务器配置 |
| `--permission-mode` | 权限模式 |
| `--bare` | 最小化模式 |
| `--debug` | 调试模式 |
| `--verbose` | 详细输出 |

---

## 三、工具系统架构

### 3.1 工具基类定义 (Tool.ts)

```typescript
interface Tool<Def extends ToolDef> {
  // 核心方法
  call(input, context): Promise<Output>
  description(): string
  prompt(): string
  
  // 权限控制
  checkPermissions(input, context): Promise<PermissionResult>
  isReadOnly(input): boolean
  isDestructive(input): boolean
  
  // UI 渲染
  userFacingName(): string
  renderToolUseMessage(input): ReactNode
  renderToolResultMessage(output): ReactNode
  
  // 生命周期
  isEnabled(): boolean
  interruptBehavior(): 'cancel' | 'block'
}
```

### 3.1.1 BashTool 实现详解

BashTool 是最复杂的工具之一，实现了完整的 shell 命令执行系统：

**核心功能**:
- 命令执行与超时控制
- 后台任务管理
- 沙箱安全执行
- 实时进度反馈
- 输出流处理
- 权限验证

**输入参数**:
```typescript
{
  command: string           // 要执行的命令
  timeout?: number         // 超时时间（毫秒）
  description?: string     // 命令描述
  run_in_background?: boolean  // 是否后台运行
  dangerouslyDisableSandbox?: boolean  // 禁用沙箱
}
```

**输出结果**:
```typescript
{
  stdout: string           // 标准输出
  stderr: string           // 标准错误
  interrupted: boolean     // 是否被中断
  isImage?: boolean        // 是否为图片输出
  backgroundTaskId?: string  // 后台任务ID
  returnCodeInterpretation?: string  // 退出码解释
  noOutputExpected?: boolean  // 是否预期无输出
}
```

**安全机制**:
- 沙箱执行检测 (`shouldUseSandbox`)
- 危险命令警告
- 权限规则匹配
- sed 编辑预览
- 输出大小限制

**后台任务**:
- 自动后台化（超时或 Ctrl+B）
- 手动后台化（用户请求）
- 代理模式自动后台化（15秒预算）
- 任务状态跟踪和通知

**性能优化**:
- 快速路径检测（搜索/只读命令）
- 输出压缩（图片尺寸限制）
- 大文件持久化（>64MB）
- 流式进度更新

### 3.2 工具分类

#### 文件操作工具
- **FileReadTool**: 读取文件内容
- **FileWriteTool**: 写入文件
- **FileEditTool**: 编辑文件（精确替换）
- **GlobTool**: 文件模式匹配
- **GrepTool**: 内容搜索
- **NotebookEditTool**: Jupyter Notebook 编辑

#### 系统交互工具
- **BashTool**: Bash 命令执行
- **PowerShellTool**: PowerShell 命令执行
- **WebFetchTool**: HTTP 请求
- **WebSearchTool**: 网页搜索
- **LSPTool**: 语言服务器协议

#### 任务管理工具
- **TaskCreateTool**: 创建任务
- **TaskListTool**: 列出任务
- **TaskUpdateTool**: 更新任务
- **TaskStopTool**: 停止任务
- **TodoWriteTool**: 待办事项管理

#### 代理协作工具
- **AgentTool**: 子代理管理
- **TeamCreateTool**: 团队创建
- **ScheduleCronTool**: 定时任务
- **SendMessageTool**: 代理间通信
- **SleepTool**: 主动模式休眠

#### MCP 相关工具
- **MCPTool**: MCP 服务器交互
- **McpAuthTool**: MCP 认证
- **ListMcpResourcesTool**: 列出资源
- **ReadMcpResourceTool**: 读取资源

#### 其他工具
- **EnterPlanModeTool**: 进入计划模式
- **ExitPlanModeTool**: 退出计划模式
- **BriefTool**: 简报通信
- **SkillTool**: 技能调用
- **WorkflowTool**: 工作流执行

### 3.3 工具池管理 (tools.ts)

```typescript
// 工具预设
type ToolPreset = 'default' | 'all' | 'none' | string[]

// 工具组装
function assembleToolPool(
  baseTools: Tool[],
  allowedTools: string[],
  disallowedTools: string[],
  mcpTools: Tool[],
  permissionContext: ToolPermissionContext
): Tool[]

// 工具过滤
function filterToolsByDenyRules(
  tools: Tool[],
  denyRules: string[]
): Tool[]
```

---

## 四、命令系统

### 4.1 命令类型

- **slash**: 斜杠命令 (`/command`)
- **prompt**: 提示命令
- **local**: 本地命令

### 4.2 核心命令列表

| 命令 | 功能 |
|------|------|
| `/help` | 显示帮助 |
| `/clear` | 清除上下文 |
| `/compact` | 压缩上下文 |
| `/model` | 切换模型 |
| `/config` | 配置管理 |
| `/login` | 登录认证 |
| `/logout` | 退出登录 |
| `/mcp` | MCP 管理 |
| `/plugin` | 插件管理 |
| `/resume` | 恢复会话 |
| `/memory` | 记忆管理 |
| `/permissions` | 权限管理 |
| `/doctor` | 健康检查 |
| `/feedback` | 反馈提交 |
| `/cost` | 费用查看 |
| `/status` | 状态查看 |
| `/theme` | 主题切换 |
| `/vim` | Vim 模式 |
| `/voice` | 语音模式 |

### 4.3 命令实现示例

```typescript
// src/commands/help.ts
export const helpCommand: Command = {
  type: 'slash',
  name: 'help',
  description: '显示帮助信息',
  async execute(context) {
    // 实现逻辑
  }
}
```

---

## 五、权限系统

### 5.1 权限模式

- **default**: 默认模式（询问用户）
- **acceptEdits**: 自动接受编辑
- **bypassPermissions**: 绕过所有权限
- **dontAsk**: 不询问
- **plan**: 计划模式

### 5.2 权限上下文

```typescript
interface ToolPermissionContext {
  mode: PermissionMode
  allowedTools: string[]
  disallowedTools: string[]
  workingDirectory: string[]
}
```

### 5.3 权限检查流程

```
1. 工具调用请求
   ↓
2. 检查工具是否启用
   ↓
3. 检查权限模式
   ↓
4. 检查允许/禁止规则
   ↓
5. 执行或拒绝
```

---

## 六、状态管理

### 6.1 App State 结构

```typescript
interface AppState {
  // 会话状态
  messages: Message[]
  sessionId: string
  
  // 模型配置
  mainLoopModel: string
  thinkingEnabled: boolean
  
  // 工具状态
  toolPermissionContext: ToolPermissionContext
  mcp: MCPState
  
  // UI 状态
  verbose: boolean
  isBriefOnly: boolean
  
  // 代理状态
  agent?: string
  teamContext?: TeamContext
  
  // 通知
  notifications: Notification[]
}
```

### 6.2 状态管理流程

```
User Action
    ↓
Dispatch Action
    ↓
Reducer
    ↓
New State
    ↓
Re-render UI
```

---

## 七、MCP 协议集成

### 7.1 MCP 服务器配置

```json
{
  "mcpServers": {
    "server-name": {
      "command": "node",
      "args": ["server.js"],
      "env": {}
    }
  }
}
```

### 7.2 MCP 功能

- **工具调用**: 调用远程工具
- **资源访问**: 访问远程资源
- **提示模板**: 使用远程提示
- **实时通知**: 接收服务器推送

### 7.3 MCP 客户端

```typescript
// src/services/mcp/client.ts
async function getMcpToolsCommandsAndResources(
  callback: (result: MCPResult) => void,
  configs: Record<string, McpServerConfig>
): Promise<void>
```

---

## 八、插件系统

### 8.1 插件结构

```
plugin/
├── manifest.json        # 插件清单
├── commands/           # 命令定义
├── tools/              # 工具定义
├── skills/             # 技能定义
└── resources/          # 资源文件
```

### 8.2 插件清单

```json
{
  "name": "plugin-name",
  "version": "1.0.0",
  "description": "插件描述",
  "commands": [...],
  "tools": [...],
  "skills": [...]
}
```

### 8.3 插件管理

- **marketplace**: 插件市场
- **install**: 安装插件
- **uninstall**: 卸载插件
- **enable/disable**: 启用/禁用
- **update**: 更新插件

---

## 九、远程控制

### 9.1 架构

```
Local CLI ←→ Bridge Server ←→ Remote Session
```

### 9.2 功能

- **SSH 远程**: 通过 SSH 连接远程主机
- **Claude.ai**: 通过云端代理
- **Direct Connect**: 直接连接

### 9.3 实现

```typescript
// src/bridge/bridgeMain.ts
async function bridgeMain(args: string[]): Promise<void> {
  // 1. 验证认证
  // 2. 检查权限
  // 3. 创建桥接会话
  // 4. 处理远程调用
}
```

---

## 十、构建系统

### 10.1 构建配置

```typescript
// build.ts
const result = await Bun.build({
  entrypoints: ['./src/entrypoints/cli.tsx'],
  outdir: './dist',
  target: 'node',
  format: 'esm',
  sourcemap: 'linked',
  minify: false,
  define: {
    'MACRO.VERSION': JSON.stringify(VERSION),
    'MACRO.BUILD_TIME': JSON.stringify(BUILD_TIME),
  },
  plugins: [
    bunBundleFeatureShimPlugin,
    textFileLoaderPlugin,
  ],
})
```

### 10.2 特性开关

90+ 个编译期特性标志，用于：
- 死代码消除
- 功能模块开关
- 平台特定代码

### 10.3 构建产物

- `dist/cli.js`: 主程序 (~21MB)
- `dist/cli.js.map`: Source Map

---

## 十一、认证系统

### 11.1 认证方式

- **API Key**: `ANTHROPIC_API_KEY` 环境变量
- **OAuth**: 浏览器授权
- **Keychain**: 系统密钥链
- **Bedrock/Vertex**: 第三方提供商

### 11.2 认证流程

```
1. 检查环境变量
   ↓
2. 检查 Keychain
   ↓
3. 检查 OAuth Token
   ↓
4. 显示登录提示
   ↓
5. 执行认证流程
```

---

## 十二、会话管理

### 12.1 会话存储

```
~/.claude/
├── projects/           # 项目会话
│   └── <project-id>/
│       └── <session-id>.jsonl
├── sessions/           # 会话索引
└── config.json         # 全局配置
```

### 12.2 会话格式

```jsonl
{"type":"user","content":"消息内容"}
{"type":"assistant","content":"回复内容"}
{"type":"tool_use","tool":"工具名","input":{}}
{"type":"tool_result","output":"结果"}
```

### 12.3 会话功能

- **continue**: 继续最近会话
- **resume**: 恢复指定会话
- **fork**: 分叉会话
- **export**: 导出会话
- **search**: 搜索会话

---

## 十三、性能优化

### 13.1 启动优化

- 快速路径：零模块加载
- 延迟加载：按需导入
- 预取：并行初始化

### 13.2 运行时优化

- 并行工具执行
- 流式响应
- 上下文压缩
- 智能缓存

### 13.3 内存管理

- 会话分页
- 历史清理
- 垃圾回收监控

---

## 十四、扩展性设计

### 14.1 扩展点

- **工具**: 实现 Tool 接口
- **命令**: 注册 Command
- **插件**: 创建 Plugin
- **技能**: 定义 Skill
- **MCP**: 连接服务器

### 14.2 钩子系统

- **Setup**: 初始化钩子
- **SessionStart**: 会话开始钩子
- **PreToolUse**: 工具使用前钩子
- **PostToolUse**: 工具使用后钩子
- **Notification**: 通知钩子

---

## 十五、安全机制

### 15.1 权限控制

- 工具级权限
- 目录级权限
- 命令级权限

### 15.2 沙箱执行

- BashTool 沙箱
- 文件访问限制
- 网络访问控制

### 15.3 审计日志

- 命令执行日志
- 文件访问日志
- API 调用日志

---

## 十六、统计信息

| 指标 | 数值 |
|------|------|
| 源文件总数 | 4,756 |
| 核心源码文件 | 1,906 |
| 工具数量 | 40+ |
| 命令数量 | 60+ |
| 特性开关 | 90+ |
| 构建产物大小 | 21.2MB |
| 版本号 | 2.1.88 |

---

## 十七、运行方式

### 17.1 开发模式

```bash
# 安装依赖
npm install

# 构建
npx bun run build.ts

# 运行
node dist/cli.js
```

### 17.2 使用示例

```bash
# 查看版本
node dist/cli.js --version

# 交互模式
node dist/cli.js

# 非交互模式
node dist/cli.js -p "解释这段代码"

# 继续会话
node dist/cli.js -c

# 指定模型
node dist/cli.js --model claude-sonnet-4-6

# 调试模式
node dist/cli.js --debug
```

---

## 十八、架构特点

### 18.1 设计原则

1. **模块化**: 清晰的模块边界
2. **可扩展**: 插件和技能系统
3. **高性能**: 并行执行和流式处理
4. **安全性**: 多层权限控制
5. **用户体验**: 丰富的 UI 反馈

### 18.2 技术亮点

- **TypeScript**: 类型安全
- **React (Ink)**: 声明式 UI
- **Bun**: 快速构建
- **MCP**: 标准化协议
- **Feature Flags**: 灵活配置

### 18.3 创新功能

- **主动模式**: 自主探索和执行
- **子代理**: 并行任务处理
- **远程控制**: 跨设备协作
- **记忆系统**: 持久化上下文
- **技能系统**: 可复用能力

---

## 附录 A：关键文件索引

### 核心入口
| 文件 | 功能 |
|------|------|
| `src/entrypoints/cli.tsx` | CLI 入口，快速路径优化 |
| `src/main.tsx` | 主逻辑，会话初始化 |
| `src/QueryEngine.ts` | API 查询引擎 |

### 工具系统
| 文件 | 功能 |
|------|------|
| `src/Tool.ts` | 工具接口定义 |
| `src/tools.ts` | 工具池管理 |
| `src/tools/BashTool/BashTool.tsx` | Bash 命令执行 |
| `src/tools/FileReadTool/` | 文件读取 |
| `src/tools/FileEditTool/` | 文件编辑 |
| `src/tools/AgentTool/` | 子代理管理 |

### 命令系统
| 文件 | 功能 |
|------|------|
| `src/commands.ts` | 命令注册系统 |
| `src/commands/` | 60+ 斜杠命令实现 |

### 状态管理
| 文件 | 功能 |
|------|------|
| `src/state/AppStateStore.ts` | 应用状态存储 |
| `src/state/onChangeAppState.ts` | 状态变更处理 |
| `src/state/store.ts` | 状态存储创建 |

### 服务层
| 目录 | 功能 |
|------|------|
| `src/services/analytics/` | 分析和遥测 |
| `src/services/api/` | API 客户端 |
| `src/services/mcp/` | MCP 协议实现 |
| `src/services/plugins/` | 插件管理 |
| `src/services/policyLimits/` | 策略限制 |
| `src/services/lsp/` | 语言服务器协议 |
| `src/services/compact/` | 上下文压缩 |
| `src/services/extractMemories/` | 记忆提取 |
| `src/services/SessionMemory/` | 会话记忆 |
| `src/services/voice/` | 语音功能 |

### UI 组件
| 目录 | 功能 |
|------|------|
| `src/components/` | 100+ React 组件 |
| `src/components/agents/` | 代理相关组件 |
| `src/components/mcp/` | MCP 相关组件 |
| `src/components/permissions/` | 权限对话框 |
| `src/components/tasks/` | 任务管理 UI |
| `src/components/ui/` | 基础 UI 组件 |

### 类型定义
| 文件 | 功能 |
|------|------|
| `src/types/command.ts` | 命令类型 |
| `src/types/ids.ts` | ID 类型 |
| `src/types/logs.ts` | 日志类型 |
| `src/types/permissions.ts` | 权限类型 |
| `src/types/generated/` | 生成的类型 |

---

## 附录 B：完整工具列表

### 文件操作工具 (6个)
1. **FileReadTool** - 读取文件内容
2. **FileWriteTool** - 写入文件
3. **FileEditTool** - 精确编辑文件
4. **GlobTool** - 文件模式匹配
5. **GrepTool** - 内容搜索
6. **NotebookEditTool** - Jupyter Notebook 编辑

### 系统交互工具 (5个)
7. **BashTool** - Bash 命令执行
8. **PowerShellTool** - PowerShell 命令执行
9. **WebFetchTool** - HTTP 请求
10. **WebSearchTool** - 网页搜索
11. **LSPTool** - 语言服务器协议

### 任务管理工具 (6个)
12. **TaskCreateTool** - 创建任务
13. **TaskGetTool** - 获取任务
14. **TaskListTool** - 列出任务
15. **TaskUpdateTool** - 更新任务
16. **TaskStopTool** - 停止任务
17. **TodoWriteTool** - 待办事项管理

### 代理协作工具 (8个)
18. **AgentTool** - 子代理管理
19. **TeamCreateTool** - 团队创建
20. **TeamDeleteTool** - 团队删除
21. **ScheduleCronTool** - 定时任务
22. **SendMessageTool** - 代理间通信
23. **SleepTool** - 主动模式休眠
24. **BriefTool** - 简报通信
25. **SuggestBackgroundPRTool** - 后台 PR 建议

### MCP 相关工具 (4个)
26. **MCPTool** - MCP 服务器交互
27. **McpAuthTool** - MCP 认证
28. **ListMcpResourcesTool** - 列出资源
29. **ReadMcpResourceTool** - 读取资源

### 模式切换工具 (4个)
30. **EnterPlanModeTool** - 进入计划模式
31. **ExitPlanModeTool** - 退出计划模式
32. **EnterWorktreeTool** - 进入工作树
33. **ExitWorktreeTool** - 退出工作树

### 其他工具 (7+个)
34. **SkillTool** - 技能调用
35. **WorkflowTool** - 工作流执行
36. **ToolSearchTool** - 工具搜索
37. **WebSearchTool** - 网页搜索
38. **ConfigTool** - 配置管理
39. **LSPTool** - 语言服务器
40. **RemoteTriggerTool** - 远程触发

**总计**: 40+ 个工具

---

## 附录 C：完整命令列表

### 会话管理
| 命令 | 功能 |
|------|------|
| `/help` | 显示帮助 |
| `/clear` | 清除上下文 |
| `/compact` | 压缩上下文 |
| `/resume` | 恢复会话 |
| `/export` | 导出会话 |

### 配置管理
| 命令 | 功能 |
|------|------|
| `/config` | 配置管理 |
| `/model` | 切换模型 |
| `/theme` | 主题切换 |
| `/vim` | Vim 模式 |
| `/output-style` | 输出样式 |

### 认证和权限
| 命令 | 功能 |
|------|------|
| `/login` | 登录认证 |
| `/logout` | 退出登录 |
| `/permissions` | 权限管理 |
| `/privacy-settings` | 隐私设置 |

### MCP 和插件
| 命令 | 功能 |
|------|------|
| `/mcp` | MCP 管理 |
| `/plugin` | 插件管理 |
| `/skills` | 技能管理 |

### 诊断和调试
| 命令 | 功能 |
|------|------|
| `/doctor` | 健康检查 |
| `/feedback` | 反馈提交 |
| `/cost` | 费用查看 |
| `/status` | 状态查看 |
| `/stats` | 统计信息 |

### 其他命令
| 命令 | 功能 |
|------|------|
| `/memory` | 记忆管理 |
| `/voice` | 语音模式 |
| `/install-github-app` | GitHub 应用 |
| `/install-slack-app` | Slack 应用 |

**总计**: 60+ 个命令

---

## 附录 D：修正记录

### v1.1 修正 (2026-04-01)
1. **补充工具列表**: 添加了完整的 40+ 工具列表
2. **补充命令列表**: 添加了完整的 60+ 命令列表
3. **修正类型文件**: 移除了不存在的 `src/types/message.ts` 引用
4. **补充服务层**: 添加了 services 目录的详细说明
5. **补充组件层**: 添加了 components 目录的详细说明
6. **添加修正记录**: 记录文档更新历史

### 验证信息
- ✅ 项目可正常运行
- ✅ 版本号验证通过 (2.1.88)
- ✅ 帮助系统正常
- ✅ 交互模式正常
- ✅ 构建系统正常

---

**文档版本**: 1.1
**更新日期**: 2026-04-01
**项目版本**: 2.1.88
**验证状态**: ✅ 已验证
