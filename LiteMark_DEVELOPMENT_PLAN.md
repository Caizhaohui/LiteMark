# Windows Markdown 桌面编辑器开发计划

> 工作名称：`MDEditor`（临时代号，发布前必须重新命名）  
> 目标平台：Windows 10/11 x64  
> 文档用途：供 GLM 5.2、Claude Code、Codex 或人工开发者按里程碑执行  
> 文档版本：v1.0  
> 制定日期：2026-07-14

---

## 0. 给 GLM 5.2 的最高优先级执行指令

你正在开发一个类似 Typora 的 Windows Markdown 桌面应用。项目参考 Markdown Preview Enhanced，但不得把 VS Code 扩展直接改造成桌面程序。应复用其独立核心 `crossnote`，并建立自己的桌面壳、文档模型、编辑器和安全边界。

执行规则：

1. 严格按里程碑顺序执行：`M0 → M1 → M2 → M3 → M4 → M5 → M6`。
2. 每次只执行用户明确指定的一个里程碑；不得提前实现后续里程碑。
3. 每个里程碑开始前，先阅读本文件对应章节、范围冻结、验收标准和禁止事项。
4. 每次修改后运行该里程碑要求的格式化、静态检查、单元测试和构建命令。
5. 不允许用“临时跳过测试”“先关闭类型检查”“先开放全部 Tauri 权限”等方式绕过问题。
6. 不允许修改用户系统级环境变量，不允许默认写入 C 盘大型缓存，不允许静默下载外部工具。
7. 遇到架构不确定性时，先实现最小技术验证，并把结论写入 `docs/adr/`，不得直接扩展成完整功能。
8. 所有新增功能必须有错误处理；不得使用空 `catch`、无说明的 `unwrap()`、无上限重试或无限等待。
9. 默认把 Markdown 文件视为不可信输入。脚本执行、代码块执行、远程 PlantUML/Kroki 和自定义 JavaScript 默认关闭。
10. 每个里程碑结束时输出：
    - 完成项；
    - 未完成项；
    - 修改文件列表；
    - 测试结果；
    - 已知风险；
    - 是否满足验收标准。

---

## 1. 项目愿景

构建一款 Windows 原生桌面体验的 Markdown 阅读和编辑软件，核心体验接近 Typora：

- 双击 `.md` 文件即可打开；
- 支持源码模式、预览模式和所见即所得/所见即所得式编辑模式；
- Markdown 始终是唯一可信源格式，不以私有富文本格式替代；
- 支持 Mermaid、数学公式、目录、脚注、任务列表、代码高亮等技术文档能力；
- 支持导出 PDF、HTML，后续通过 Pandoc 扩展 DOCX、EPUB 等格式；
- 默认本地运行、无遥测、无账号依赖；
- 对恶意 Markdown、HTML、脚本和外部命令保持安全默认值。

本项目不是 Markdown Preview Enhanced 的 VS Code 外壳复制品，而是：

```text
Windows 桌面应用
├── Tauri/Rust：窗口、文件、系统集成、安全边界、生命周期
├── React/TypeScript：界面、状态管理、编辑器交互
├── Milkdown/ProseMirror：Typora 风格的结构化 Markdown 编辑
├── Monaco：完整源码模式
└── Crossnote sidecar：增强渲染、图表、数学公式和文档导出
```

---

## 2. 参考项目调研结论

### 2.1 Markdown Preview Enhanced 的可复用部分

Markdown Preview Enhanced 的 VS Code 扩展主要负责：

- VS Code 扩展入口和命令注册；
- WebView 面板；
- Workspace/Notebook 管理；
- 配置桥接；
- 编辑器与预览之间的交互。

真正负责 Markdown 解析、转换、增强渲染和导出的核心是独立项目 `crossnote`。Crossnote 提供：

- Markdown → HTML 渲染；
- Mermaid、KaTeX/MathJax、Graphviz、TikZ、WaveDrom 等增强；
- HTML 导出；
- Chromium/Puppeteer PDF、PNG、JPEG 导出；
- Pandoc 导出；
- EPUB/PDF/MOBI 等电子书导出；
- GFM Markdown 导出；
- 预览主题和代码主题；
- Wiki Link、标签、导入文件、目录等扩展语法。

因此本项目应该依赖或轻量封装 `crossnote`，而不是复制 `vscode-markdown-preview-enhanced/src/extension*.ts`。

### 2.2 许可证

`vscode-markdown-preview-enhanced` 和 `crossnote` 使用 University of Illinois/NCSA Open Source License。该许可证允许使用、复制、修改、发布、分发、再许可和销售，但要求：

- 源码再分发保留版权声明、许可条件和免责声明；
- 二进制分发在文档或其他材料中包含相同声明；
- 未经书面许可，不得使用原作者或贡献者名称为产品背书。

项目必须创建：

```text
LICENSE
THIRD_PARTY_NOTICES.md
licenses/crossnote-LICENSE.md
licenses/markdown-preview-enhanced-LICENSE.md   # 仅在实际复制该仓库代码时需要
```

不要使用 “Markdown Preview Enhanced”“Typora” 作为产品名称、图标或营销背书。可以描述为“受其工作流启发”或“兼容部分语法”，但不得暗示官方关系。

### 2.3 为什么使用 Tauri 2

Tauri 2 允许使用 Web 前端和 Rust 后端构建 Windows 应用，在 Windows 上通过 WebView2 渲染界面，并可生成 NSIS `setup.exe` 或 WiX/MSI 安装包。它适合本项目的原因：

- Rust 负责文件和系统边界；
- Web 前端可直接集成 Monaco、Milkdown、React；
- 可通过 sidecar 运行独立 Node 程序；
- Tauri 2 的 capability/permission 模型可限制每个窗口的权限；
- Windows 10/11 普遍具备或可由安装器补充 WebView2。

### 2.4 为什么需要 Node sidecar

Crossnote 不是纯浏览器库。其完整功能会使用 Node.js 文件系统、子进程、Puppeteer、Sharp，以及 Pandoc、Java、LaTeX 等可选外部程序。不能把完整 Crossnote 直接作为 WebView 前端依赖。

因此采用：

```text
React WebView
    │ Tauri invoke/events
    ▼
Rust application core
    │ stdin/stdout JSON Lines
    ▼
Node sidecar（打包为独立 exe）
    └── Crossnote
```

sidecar 最终必须打包为自包含可执行文件，普通用户不需要单独安装 Node.js。

### 2.5 Typora 风格编辑器的实现策略

Crossnote 的 in-preview editor 曾被标记为 beta，并依赖源码映射。它适合作为参考，但不应成为本项目唯一编辑器核心。

本项目采用双编辑器策略：

- **源码模式**：Monaco Editor，保证所有 Markdown 和扩展语法都可无损编辑；
- **混合模式**：Milkdown/ProseMirror，提供接近 Typora 的单页结构化编辑；
- **预览模式**：Crossnote 渲染最终效果。

Markdown 文件始终是规范数据。Milkdown 无法识别的 Crossnote 扩展必须保存为“原始块”，不得删除、重排或悄悄改写。

---

## 3. 产品范围

### 3.1 v1.0 必须支持

- 新建、打开、保存、另存为 Markdown；
- 拖放打开 `.md`、`.markdown`；
- UTF-8、UTF-8 BOM 检测和保存；
- CRLF/LF 保留或明确转换；
- 多标签页；
- 未保存状态和关闭确认；
- 崩溃恢复；
- 源码编辑模式；
- Typora 风格混合编辑模式；
- Crossnote 增强预览；
- 标题、段落、粗体、斜体、删除线、列表、引用、代码、链接、图片、表格、任务列表、水平线；
- YAML front matter；
- Mermaid；
- KaTeX 数学公式；
- 目录导航；
- 查找与替换；
- 图片粘贴到相对资源目录；
- 导出 HTML；
- 导出 PDF；
- 明暗主题；
- Windows 文件关联；
- NSIS 安装包；
- 自动保存可配置，默认关闭；
- 无遥测、无账户、无云端依赖。

### 3.2 v1.0 明确不做

- 在线协同编辑；
- 云同步；
- 移动端；
- macOS/Linux 正式支持；
- 插件市场；
- 浏览器版；
- AI 写作功能；
- Git 客户端；
- 完整 Obsidian 知识库；
- 所有 MPE 图表类型一次性全支持；
- 默认执行代码块；
- 默认加载远程 JavaScript；
- 内置 Chromium；
- 自动安装 Pandoc、Java、LaTeX、Graphviz；
- 为不受支持语法进行有损“自动修复”。

---

## 4. 目标系统和工具链

### 4.1 开发平台

- Windows 10 22H2 或 Windows 11；
- x86_64-pc-windows-msvc；
- Visual Studio 2022 Build Tools；
- Windows 10/11 SDK；
- WebView2 Runtime；
- Rust stable；
- Node.js 22 LTS；
- pnpm 11；
- Git。

不得依赖 WSL 才能构建 Windows 安装包。

### 4.2 推荐初始化命令

```powershell
corepack enable
corepack prepare pnpm@11 --activate

rustup default stable-x86_64-pc-windows-msvc
rustc --version
cargo --version
node --version
pnpm --version

pnpm create tauri-app@latest mdeditor
# TypeScript / React / pnpm
```

### 4.3 版本锁定

仓库必须提交：

```text
rust-toolchain.toml
Cargo.lock
pnpm-lock.yaml
package.json#packageManager
.nvmrc
```

建议：

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
profile = "minimal"
targets = ["x86_64-pc-windows-msvc"]
components = ["rustfmt", "clippy"]
```

```text
# .nvmrc
22
```

不得使用 `latest` 作为生产依赖版本。初始化阶段可以由脚手架选择版本，M0 结束前必须固化 lockfile。

---

## 5. 总体架构

### 5.1 分层

```text
┌─────────────────────────────────────────────────────────────┐
│ React UI                                                     │
│ AppShell / Tabs / Sidebar / StatusBar / Settings             │
├─────────────────────────────────────────────────────────────┤
│ Editor Layer                                                 │
│ MonacoSourceEditor │ MilkdownHybridEditor │ PreviewFrame     │
├─────────────────────────────────────────────────────────────┤
│ Frontend Application Services                                │
│ DocumentStore │ CommandBus │ RenderClient │ ExportClient     │
├─────────────────────────────────────────────────────────────┤
│ Tauri Commands / Events                                      │
├─────────────────────────────────────────────────────────────┤
│ Rust Core                                                    │
│ FileService │ SessionService │ Recovery │ Settings │ Sidecar │
├─────────────────────────────────────────────────────────────┤
│ Node Sidecar                                                 │
│ CrossnoteAdapter │ RenderService │ ExportService             │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 权责边界

#### React 前端

负责：

- UI；
- 标签页；
- 编辑状态；
- 编辑器适配；
- 快捷键；
- 请求渲染和导出；
- 展示进度与错误。

不得：

- 任意访问整个文件系统；
- 直接启动系统命令；
- 直接调用 Pandoc/Java/Chrome；
- 直接信任渲染 HTML；
- 保存权限令牌到 Markdown 目录。

#### Rust Core

负责：

- 安全的文件打开/保存；
- 原子写入；
- 路径规范化；
- 文件关联和系统窗口；
- sidecar 生命周期；
- IPC 请求校验、超时和取消；
- 崩溃恢复；
- 配置持久化；
- 外部链接通过系统默认浏览器打开。

#### Node Sidecar

负责：

- 初始化 Crossnote；
- Markdown 增强渲染；
- 导出 HTML/PDF/图片/Pandoc；
- 检测可选外部工具；
- 返回结构化诊断。

不得：

- 接收任意 shell 命令；
- 允许前端传入任意可执行文件和参数；
- 默认联网；
- 默认执行 Markdown 代码块；
- 监听公网端口；
- 写入未经 Rust Core 授权的任意路径。

### 5.3 IPC

sidecar 使用长驻进程和 `stdin/stdout` JSON Lines。每行一个 JSON 对象。禁止把日志写入 stdout；日志写 stderr。

请求：

```json
{"id":"uuid","method":"render","params":{"sessionId":"uuid","markdown":"# Hello","logicalPath":"D:\\docs\\a.md","options":{"theme":"github-light","trusted":false}}}
```

成功响应：

```json
{"id":"uuid","ok":true,"result":{"html":"<h1>Hello</h1>","toc":[],"diagnostics":[],"renderMs":12}}
```

失败响应：

```json
{"id":"uuid","ok":false,"error":{"code":"RENDER_FAILED","message":"...","details":null}}
```

事件：

```json
{"event":"exportProgress","payload":{"jobId":"uuid","stage":"rendering","progress":0.45}}
```

允许的方法必须使用静态枚举：

```text
ping
getCapabilities
createSession
closeSession
render
exportHtml
exportPdf
cancelJob
probeExternalTools
shutdown
```

不得提供 `exec`、`shell`、`runCommand` 等通用入口。

### 5.4 渲染未保存内容

M2 必须解决“未保存内容实时渲染”问题。禁止为了预览而覆盖原文件。

优先方案：为 Crossnote 增加或封装一个稳定的内存渲染入口：

```ts
renderMarkdownText({
  markdown,
  logicalFilePath,
  notebookPath,
  config,
})
```

其中：

- `logicalFilePath` 只用于相对资源、导入文件和链接解析；
- `markdown` 来自编辑器内存；
- 渲染过程不得写入用户源文件；
- 所有相对路径必须限制在文档目录或显式授权工作区内；
- 如果 Crossnote 公共 API 不足，优先提交一个最小 adapter/fork patch；不得从大量未导出的内部模块建立脆弱依赖。

备选方案仅用于技术验证：将 shadow file 写入应用缓存，并实现资源 URL 映射。不得在用户文档目录创建隐藏预览文件。

必须创建 ADR：

```text
docs/adr/0002-crossnote-in-memory-rendering.md
```

---

## 6. 数据模型

### 6.1 DocumentSession

```ts
export interface DocumentSession {
  id: string;
  filePath: string | null;
  displayName: string;
  content: string;
  savedContentHash: string;
  encoding: 'utf-8' | 'utf-8-bom';
  lineEnding: 'lf' | 'crlf';
  dirty: boolean;
  readOnly: boolean;
  mode: 'source' | 'hybrid' | 'preview';
  revision: number;
  lastSavedRevision: number;
  externalMtimeMs: number | null;
  recoveryKey: string;
}
```

规则：

- `dirty` 由内容哈希或 revision 比较得到，不由 UI 手工猜测；
- 保存成功后更新 `savedContentHash`、`lastSavedRevision` 和 `externalMtimeMs`；
- 文件被外部修改时，不得静默覆盖；
- 新建文件 `filePath = null`；
- 所有编辑器模式共享同一个 DocumentSession；
- 模式切换前必须完成 Markdown 序列化和一致性检查。

### 6.2 原子保存

Rust 保存流程：

1. 在目标目录创建唯一临时文件；
2. 写入完整内容；
3. flush；
4. 可选 `sync_all`；
5. 保留文件权限；
6. 使用原子替换；
7. 失败时清理临时文件；
8. 返回新的 mtime 和内容哈希。

不得先 truncate 原文件再写入。

### 6.3 恢复文件

恢复目录：

```text
%LOCALAPPDATA%\<Vendor>\<App>\recovery\
```

每个 dirty 文档最多保留最近若干快照。快照内容：

```json
{
  "sessionId": "...",
  "originalPath": "D:\\docs\\a.md",
  "capturedAt": "2026-07-14T12:00:00Z",
  "revision": 18,
  "content": "..."
}
```

成功保存并安全关闭后删除相应恢复文件。

---

## 7. 编辑器设计

### 7.1 源码模式

使用 Monaco Editor，必须支持：

- Markdown 语法高亮；
- 行号开关；
- 自动换行；
- 查找/替换；
- 多光标；
- 撤销/重做；
- Tab/空格配置；
- 粘贴图片命令；
- 文档大纲同步；
- `Ctrl+S` 保存；
- `Ctrl+Shift+S` 另存为；
- `Ctrl+P` 快速打开后续预留，不在 v1.0 必须范围。

### 7.2 混合模式

使用 Milkdown/ProseMirror。目标不是隐藏所有 Markdown 字符，而是提供稳定的 WYSIWYM 体验。

M4 首批节点：

```text
paragraph
heading 1-6
blockquote
bullet_list
ordered_list
list_item
task_list
task_item
code_block
inline_code
horizontal_rule
image
link
emphasis
strong
strike
hard_break
table/table_row/table_cell
math_inline
math_block
yaml_front_matter
raw_markdown_block
```

核心规则：

- 解析和序列化必须可测试；
- 不支持的 fenced block、MPE directive、HTML 或扩展语法作为 `raw_markdown_block` 保存；
- `raw_markdown_block` 默认显示为可编辑源码卡片；
- 不得因为进入混合模式而改变用户原始文档；
- 序列化后若语义不等价，阻止模式切换并提示用户返回源码模式；
- 表格、公式和 Mermaid 可以采用 NodeView；
- Mermaid 编辑弹窗操作原始代码，预览采用受清洗 SVG；
- 粘贴 HTML 必须先净化并转换为支持的 Markdown 节点。

### 7.3 模式切换一致性

每次 `source → hybrid`：

1. 解析 Markdown；
2. 收集 unsupported constructs；
3. 转为 ProseMirror 文档；
4. 立即序列化回 Markdown；
5. 执行规范化比较；
6. 若存在可能数据损失，展示警告并保持源码模式。

每次 `hybrid → source`：

1. 序列化；
2. 更新 DocumentSession；
3. 增加 revision；
4. 通知 Monaco 和预览；
5. 保持撤销边界清晰。

不要尝试让 Monaco 和 Milkdown 同时写同一份状态。

---

## 8. 渲染和预览

### 8.1 默认配置

```ts
const safeCrossnoteDefaults = {
  markdownParser: 'markdown-it',
  mathRenderingOption: 'KaTeX',
  previewTheme: 'github-light.css',
  codeBlockTheme: 'auto.css',
  enableScriptExecution: false,
  enableHTML5Embed: false,
  printBackground: true,
  protocolsWhiteList: 'http://, https://, mailto:, tel:',
};
```

注意：`file://` 不应直接向渲染页面开放。由 Rust/Tauri 自定义协议或受控资源映射提供本地图片。

### 8.2 防抖和取消

- 输入防抖默认 250 ms；
- 每个 render 请求携带 revision；
- 新 revision 发出后取消旧请求或丢弃旧结果；
- 单次普通渲染超时 10 秒；
- 大文件进入降级模式；
- 预览结果必须携带原 revision，前端只接收最新 revision。

### 8.3 大文件策略

默认阈值：

```text
< 1 MiB：完整实时预览
1–5 MiB：降低防抖频率，关闭昂贵图表自动渲染
> 5 MiB：默认仅源码模式，用户可手动请求预览
```

阈值必须可配置，但需要合理上限，防止内存耗尽。

### 8.4 HTML 安全

- Crossnote 服务端清洗不能被绕过；
- 前端插入 HTML 前再次使用 DOMPurify；
- 禁止 `dangerouslySetInnerHTML` 接收未经清洗的字符串；
- 禁止 `<script>`、事件属性、`javascript:`、不受控 iframe；
- Mermaid/Graphviz/TikZ 产生的 SVG 同样清洗；
- 远程资源默认不加载；
- 点击链接由前端发送给 Rust，由 opener 打开；
- 本地文件链接打开前检查路径和用户授权。

---

## 9. 导出设计

### 9.1 HTML 导出

M3 支持：

- 单文件 HTML；
- 可选离线资源内嵌；
- 预览主题和代码主题；
- 图片路径重写；
- Mermaid、公式等已渲染结果；
- 导出前选择目标路径；
- 不覆盖原 Markdown。

### 9.2 PDF 导出

M3 首选 Crossnote `chromeExport`/Puppeteer 路径，使用系统 Edge 或 Chrome。检测顺序：

1. 用户设置的浏览器路径；
2. Microsoft Edge 稳定版；
3. Google Chrome 稳定版；
4. 返回 `BROWSER_NOT_FOUND`，展示安装/选择路径提示。

不要内置完整 Chromium 到首个版本。不要静默联网下载浏览器。

PDF 参数：

```ts
interface PdfExportOptions {
  pageSize: 'A4' | 'Letter' | 'Legal';
  landscape: boolean;
  marginTopMm: number;
  marginRightMm: number;
  marginBottomMm: number;
  marginLeftMm: number;
  printBackground: boolean;
  displayHeaderFooter: boolean;
}
```

### 9.3 Pandoc 导出

M5 再实现：

- DOCX；
- EPUB；
- LaTeX；
- 其他 Pandoc 格式。

Pandoc 作为可选外部工具：

- 自动探测；
- 用户可指定路径；
- 参数使用白名单；
- 不允许用户通过 GUI 传任意 shell 字符串；
- 高级自定义参数只能作为参数数组保存，仍需阻止危险输出路径和命令替换。

---

## 10. 仓库结构

```text
mdeditor/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release-windows.yml
├── docs/
│   ├── adr/
│   │   ├── 0001-tauri-and-node-sidecar.md
│   │   └── 0002-crossnote-in-memory-rendering.md
│   ├── architecture.md
│   ├── security.md
│   └── testing.md
├── licenses/
├── packages/
│   ├── app-ui/
│   │   └── src/
│   │       ├── app/
│   │       ├── components/
│   │       ├── editors/
│   │       │   ├── monaco/
│   │       │   └── milkdown/
│   │       ├── features/
│   │       ├── services/
│   │       ├── state/
│   │       └── types/
│   ├── shared-protocol/
│   │   └── src/
│   └── render-sidecar/
│       └── src/
│           ├── crossnote-adapter/
│           ├── handlers/
│           ├── protocol/
│           └── security/
├── src-tauri/
│   ├── capabilities/
│   ├── src/
│   │   ├── commands/
│   │   ├── document/
│   │   ├── recovery/
│   │   ├── sidecar/
│   │   ├── settings/
│   │   └── lib.rs
│   ├── binaries/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── testdata/
│   ├── markdown/
│   ├── malicious/
│   ├── roundtrip/
│   └── export-golden/
├── DEVELOPMENT_PLAN.md
├── LICENSE
├── THIRD_PARTY_NOTICES.md
├── package.json
├── pnpm-workspace.yaml
└── rust-toolchain.toml
```

`shared-protocol` 使用 TypeScript schema 定义请求和响应，并生成 JSON Schema。Rust 端使用对应结构体，至少通过共享测试向量验证兼容性。

---

## 11. 里程碑

# M0 — Windows 仓库初始化和架构验证

## M0 目标

建立可构建、可测试、可启动的 Windows Tauri 2 仓库，并验证 Node sidecar 与 Crossnote 的最小闭环。

## M0 允许范围

- 初始化 pnpm workspace；
- 初始化 React + TypeScript + Vite；
- 初始化 Tauri 2 Rust 项目；
- 创建基础窗口；
- 添加格式化、lint、TypeScript、Rust 检查；
- 创建 sidecar Hello/Ping；
- sidecar 引入固定版本 `crossnote`；
- 完成一次静态 Markdown 字符串到 HTML 的技术验证；
- 完成一次静态测试文件到 PDF 的技术验证；
- 写 ADR；
- 配置 CI；
- 创建许可证文件。

## M0 禁止范围

- 不做文件打开/保存 UI；
- 不做标签页；
- 不做 Monaco；
- 不做 Milkdown；
- 不做正式预览界面；
- 不做设置页；
- 不做自动更新；
- 不做 Pandoc；
- 不做自定义标题栏；
- 不做图标和品牌设计；
- 不做安装器发布流程以外的产品功能。

## M0 必须产出

```text
pnpm dev          # 启动桌面窗口
pnpm check        # 前端和 sidecar 静态检查
pnpm test         # 单元测试
pnpm tauri build  # 生成 Windows bundle
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

sidecar ping：

```json
{"id":"1","method":"ping","params":{}}
{"id":"1","ok":true,"result":{"version":"...","crossnoteVersion":"0.9.31"}}
```

技术验证：

- 输入包含标题、Mermaid、KaTeX 的测试 Markdown；
- 输出非空 HTML；
- PDF 文件存在且大小大于 1 KiB；
- 脚本执行保持关闭；
- sidecar 在主应用退出时结束；
- 主应用能在 sidecar 异常退出后展示结构化错误而不是卡死。

## M0 验收标准

- 全新 Windows 开发环境按 README 可成功构建；
- `pnpm check && pnpm test` 通过；
- `cargo fmt --check` 通过；
- `cargo clippy ... -D warnings` 通过；
- Tauri 窗口启动；
- Rust 能启动 sidecar 并完成 ping；
- Crossnote 渲染/导出 spike 成功；
- `docs/adr/0001-tauri-and-node-sidecar.md` 完成；
- `docs/adr/0002-crossnote-in-memory-rendering.md` 至少记录验证结果和后续决定；
- 仓库不存在硬编码开发者绝对路径。

---

# M1 — 文档生命周期和文件安全

> **状态（2026-07-29）：已完成。** 全部验收标准满足。实现见
> `src-tauri/src/{files,session,recovery,commands}` 与
> `packages/app-ui/src/{hooks,components,services}`；决策记录见
> ADR 0003（MSVC 工具链）、0004（原子保存/编码）、0005（崩溃恢复/外部修改）。
> 注：构建工具链由 GNU 切回 MSVC（ADR 0003，反转 ADR 0001 §D2）。

## M1 目标

实现可靠的 Markdown 新建、打开、保存、另存为、关闭和恢复，但暂不实现正式编辑器。

## M1 允许范围

- DocumentSession；
- 文件选择器；
- 新建文档；
- 打开文件；
- 文本区域临时编辑器；
- 保存、另存为；
- dirty 标记；
- 关闭确认；
- 文件编码和换行检测；
- 原子写入；
- 最近文件；
- 恢复快照；
- 外部文件修改检测；
- 拖放打开；
- 单实例参数转发的最小实现。

## M1 禁止范围

- 不引入 Monaco；
- 不引入 Milkdown；
- 不集成 Crossnote 实时预览；
- 不做 PDF/HTML 导出 UI；
- 不做 Mermaid；
- 不做多窗口；
- 不做目录知识库。

## M1 验收标准

- 新建文档可编辑并另存为；
- 打开 UTF-8、UTF-8 BOM、LF、CRLF 文件后保存不发生意外编码变化；
- 文件保存使用原子替换；
- 未保存关闭时有 Save/Discard/Cancel；
- 应用异常退出后可恢复 dirty 内容；
- 外部修改时提示 Reload/Keep Mine/Compare Later，不静默覆盖；
- 测试覆盖空文件、大文件、只读文件、无权限目录、长路径、中文路径、emoji 文件名；
- 所有文件操作错误为用户可理解的信息。

---

# M2 — Monaco 源码编辑和 Crossnote 实时预览

> **状态（2026-08-01）：已完成。** 验收能力已实现：Monaco 源码编辑、Crossnote
> 内存渲染预览（250 ms 防抖 + revision 丢弃）、Source/Split/Preview 布局、TOC、
> 诊断区、>5 MiB 大文件降级、DOMPurify 二次清洗、外链经 Rust 打开、相对图片经
> `lmlocal://` 协议授权加载、关闭标签释放 sidecar session。实现见
> `packages/app-ui/src/{components,hooks,services}`、`src-tauri/src/commands/render.rs`、
> `src-tauri/src/assets.rs`；协议扩展见 `packages/shared-protocol` M2 段。

## M2 目标

形成第一版可用 Markdown IDE：源码编辑、增强预览和同步导航。

## M2 允许范围

- Monaco Editor；
- Crossnote sidecar 正式 render API；
- 内存 Markdown 渲染；
- 250 ms 防抖；
- revision 丢弃旧结果；
- 预览主题；
- KaTeX；
- Mermaid；
- 代码高亮；
- 目录；
- 源码/预览切换；
- 左右分栏；
- 基础滚动同步；
- 诊断面板；
- 大文件降级；
- 安全 HTML 插入；
- 链接打开策略。

## M2 禁止范围

- 不做 Milkdown；
- 不做 PDF/HTML 导出；
- 不执行代码块；
- 不启用远程 PlantUML/Kroki；
- 不做 Wiki Graph；
- 不做插件系统。

## M2 验收标准

- 修改 Markdown 后预览在正常文档上感知延迟小于约 500 ms；
- 未保存内容可预览，原文件未被覆盖；
- 相对图片正常显示；
- Mermaid 和 KaTeX 正常；
- 恶意 HTML 测试不能执行脚本；
- 快速输入不会导致旧预览覆盖新预览；
- 关闭文档会释放 Crossnote session；
- sidecar 崩溃后可重启并重新建立 session；
- 5 MiB 以上文档触发降级模式；
- 预览链接不会在 WebView 内导航离开应用。

---

# M3 — HTML/PDF 导出和 Windows 可安装版本

> **状态（2026-08-01）：已完成。** HTML/PDF 导出 UI、进度与取消、浏览器探测、
> 最近导出目录、路径授权、NSIS 文件关联与资源打包脚本、第三方许可证页面均已落地。
> 实现见 `src-tauri/src/commands/export.rs`、`packages/render-sidecar` jobs/progress、
> `packages/app-ui` ExportDialog/useExport、`scripts/prepare-release-resources.mjs`。
> 端用户免装 Node 需在发布时设置 `LITEMARK_BUNDLE_NODE=1` 并附带便携 Node（见
> `scripts/SIDECAR-BUNDLE.txt`）；开发模式继续使用系统 Node。

## M3 目标

实现用户最需要的正式文档导出能力，并生成可安装的 Windows beta。

## M3 允许范围

- HTML 导出；
- PDF 导出；
- 导出设置对话框；
- 导出进度；
- 取消；
- 浏览器路径探测；
- 导出错误诊断；
- 最近导出目录；
- NSIS 安装包；
- `.md` 文件关联；
- 应用图标占位正式化；
- 安装/卸载测试；
- 第三方许可证页面。

## M3 禁止范围

- 不做 DOCX；
- 不做 EPUB；
- 不捆绑 Pandoc；
- 不做 Milkdown；
- 不做自动更新；
- 不做 Microsoft Store；
- 不做代码签名购买流程，但要预留配置。

## M3 验收标准

- HTML 导出可离线打开；
- PDF 支持 A4/Letter、页边距、横向和背景；
- Mermaid、公式、图片在 PDF 中正常；
- 无 Edge/Chrome 时给出明确提示；
- 导出不会修改源 Markdown；
- 取消任务后无残留浏览器进程；
- 导出路径不允许逃逸到用户未选择的位置；
- NSIS 安装、文件关联、卸载正常；
- 普通用户无需 Node.js 即可运行；
- 应用退出后无 sidecar/Chromium 僵尸进程。

---

# M4 — Typora 风格混合编辑模式

> **状态（2026-08-01）：已完成。** Milkdown hybrid 编辑器、Source/Hybrid 切换、
> remark 往返守卫（潜在损失时阻止切换）、工具栏、`@litemark/markdown-core` 与
> ≥100 golden roundtrip 测试。见 `packages/app-ui/src/editors/HybridEditor.tsx`、
> `packages/markdown-core/`。

## M4 目标

提供单页结构化 Markdown 编辑体验，并保证不支持语法不丢失。

## M4 允许范围

- Milkdown/ProseMirror；
- 基础 Markdown 节点；
- 工具栏；
- Slash command；
- Markdown 快捷输入；
- 表格；
- 图片；
- 公式 NodeView；
- Mermaid 原始块 + 预览；
- YAML front matter；
- raw_markdown_block；
- source/hybrid 模式切换；
- roundtrip 测试；
- 混合模式撤销/重做；
- 光标与文档大纲联动。

## M4 禁止范围

- 不做实时协作；
- 不做复杂分页排版；
- 不追求 MPE 全部私有语法原生 NodeView；
- 不删除不认识的语法；
- 不把 ProseMirror JSON 保存为主文件；
- 不默认格式化整个文档。

## M4 验收标准

- 基础 Markdown 在 source → hybrid → source 后语义一致；
- roundtrip golden tests 覆盖至少 100 个样例；
- 不支持语法保存在 raw block 中且字节内容可恢复；
- 进入混合模式不会直接改变磁盘文件；
- 表格、任务列表、公式和 Mermaid 可编辑；
- 中文输入法组合输入稳定；
- 粘贴富文本不会插入脚本；
- 10,000 行普通文档仍可编辑；
- 模式切换出现潜在数据损失时阻止并说明原因。

---

# M5 — 高级导出和增强语法

> **状态（2026-08-01）：已完成。** Pandoc 探测与 DOCX/EPUB/LaTeX 导出（argv 数组）、
> 可选工具探测（Graphviz/PlantUML 不阻塞）、可信工作区（可撤销）、自定义 CSS 路径
> 消毒、Settings UI、wiki-link 开关、实验性代码执行仅 UI 开关。见
> `src-tauri/src/commands/{pandoc,settings}.rs`、`SettingsDialog.tsx`。

## M5 目标

扩展 Markdown Preview Enhanced 的高价值能力，但保持可选依赖和安全边界。

## M5 允许范围

- Pandoc 探测和配置；
- DOCX/EPUB/LaTeX 导出；
- Graphviz 本地渲染；
- PlantUML 本地 jar；
- Reveal.js 演示导出；
- Wiki Link；
- 文件导入；
- 自定义 CSS；
- 文档级 front matter 导出设置；
- 受控的可信工作区；
- 可选代码块执行设计和实验功能。

## M5 安全规则

- 新文档和下载文档默认不可信；
- `enableScriptExecution` 默认永远为 false；
- 启用可信工作区必须有持久化用户确认；
- 代码块执行必须显示命令、工作目录和风险；
- 可执行程序路径必须经 Rust 验证；
- 导出参数必须为数组，不拼接 shell 字符串；
- 远程图表服务默认关闭。

## M5 验收标准

- 缺少可选工具时不影响核心编辑器；
- 工具探测不阻塞启动；
- DOCX/EPUB 的错误信息可诊断；
- 自定义 CSS 不能绕过脚本安全策略；
- 可信状态可撤销；
- 未经确认不能执行任何 Markdown 内嵌命令。

---

# M6 — 发布质量、安全和维护

> **状态（2026-08-01）：已完成（工程基线）。** CI 工作流、README/用户文档/隐私说明、
> 性能基线文档、SBOM/审计脚本、崩溃报告导出、更新端点占位（默认关闭）、无障碍
> skip-link/aria、恶意样例保留。完整签名发布流水线与硬件性能实测可在正式发版时补测。

## M6 目标

达到 Windows 1.0 发布条件。

## M6 允许范围

- 完整 E2E；
- 性能优化；
- 内存泄漏修复；
- 可访问性；
- 自动更新；
- Windows 代码签名配置；
- 崩溃日志本地导出；
- 隐私说明；
- SBOM；
- 依赖审计；
- 发布流水线；
- 用户文档；
- 迁移和回滚测试。

## M6 验收标准

- 冷启动目标小于 2 秒（常见开发机，排除首次 WebView2 安装）；
- 空闲内存和打开 10 个普通文档后的内存有基准记录；
- 连续打开/关闭 100 个文档无明显泄漏；
- 恶意 Markdown 测试集全部通过；
- `cargo audit`、npm/pnpm audit 结果已评审；
- 生成 CycloneDX 或 SPDX SBOM；
- 安装包升级和卸载不删除用户文档；
- 自动更新具备签名验证；
- 第三方许可证完整；
- 屏幕阅读器可访问主要菜单和对话框；
- 关键快捷键有菜单入口；
- 发布构建可复现到合理程度。

---

## 12. 测试策略

### 12.1 TypeScript 单元测试

- protocol schema；
- DocumentSession reducer/store；
- dirty 计算；
- mode switch；
- Markdown roundtrip；
- export option validation；
- diagnostics mapping；
- path/display name formatting。

### 12.2 Rust 单元和集成测试

- 路径 canonicalization；
- 原子保存；
- BOM/换行；
- 恢复文件；
- sidecar 生命周期；
- IPC 超时和取消；
- 非法 method 拒绝；
- 未授权路径拒绝；
- 外部修改检测。

### 12.3 Sidecar 测试

- JSONL framing；
- 并发请求；
- 旧 revision；
- render timeout；
- Crossnote 初始化失败；
- HTML sanitization；
- 导出取消；
- 浏览器不存在；
- 相对资源；
- Unicode 路径；
- Windows 长路径。

### 12.4 Golden tests

```text
testdata/markdown/input.md
testdata/markdown/expected.html
testdata/roundtrip/input.md
testdata/roundtrip/expected.md
testdata/export-golden/*.pdf.sha256-or-visual-baseline
```

HTML golden 需要去除动态 ID、时间戳等不稳定字段。PDF 不建议只比较字节；使用页数、文本提取、关键图像和人工视觉基线组合。

### 12.5 恶意输入

至少包含：

- `<script>`；
- `onerror=`；
- `javascript:` 链接；
- SVG script/event；
- iframe；
- 超深嵌套；
- 超长单行；
- 巨大 data URI；
- 路径穿越 `../../`；
- Windows UNC path；
- `file://`；
- 恶意 Mermaid；
- front matter 注入；
- Crossnote `.crossnote/parser.js`；
- 代码块命令注入；
- Pandoc 参数注入。

默认不读取或执行文档目录中的 `.crossnote/parser.js`。如后续支持，必须仅限可信工作区。

---

## 13. 质量门禁

根目录脚本建议：

```json
{
  "scripts": {
    "dev": "...",
    "build": "...",
    "check": "pnpm lint && pnpm typecheck && pnpm check:rust",
    "lint": "...",
    "typecheck": "...",
    "test": "pnpm -r test",
    "check:rust": "cargo fmt --check --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings",
    "test:rust": "cargo test --manifest-path src-tauri/Cargo.toml",
    "tauri": "tauri"
  }
}
```

合并前必须：

```powershell
pnpm install --frozen-lockfile
pnpm check
pnpm test
pnpm test:rust
pnpm build
pnpm tauri build
```

CI 至少包括：

- Windows build；
- TypeScript lint/typecheck/test；
- Rust fmt/clippy/test；
- sidecar packaging；
- Tauri bundle；
- 许可证清单生成；
- 产物上传。

---

## 14. 错误模型

统一错误码示例：

```text
FILE_NOT_FOUND
FILE_PERMISSION_DENIED
FILE_CHANGED_EXTERNALLY
FILE_ENCODING_UNSUPPORTED
SAVE_ATOMIC_REPLACE_FAILED
SIDECAR_START_FAILED
SIDECAR_CRASHED
SIDECAR_TIMEOUT
PROTOCOL_INVALID
RENDER_FAILED
RENDER_CANCELLED
EXPORT_FAILED
EXPORT_CANCELLED
BROWSER_NOT_FOUND
PANDOC_NOT_FOUND
UNTRUSTED_OPERATION_BLOCKED
PATH_NOT_AUTHORIZED
ROUNDTRIP_DATA_LOSS_RISK
```

用户界面展示：

- 一句清楚的主错误；
- 可执行的下一步；
- 可展开的技术详情；
- 可复制诊断信息；
- 不展示隐私敏感的完整环境变量。

---

## 15. 设置设计

首版设置：

```ts
interface AppSettings {
  appearance: 'system' | 'light' | 'dark';
  defaultEditorMode: 'source' | 'hybrid' | 'preview';
  autoSave: 'off' | 'afterDelay' | 'onFocusChange';
  autoSaveDelayMs: number;
  wordWrap: boolean;
  fontFamily: string;
  fontSize: number;
  previewTheme: string;
  codeBlockTheme: string;
  mathRenderer: 'KaTeX' | 'MathJax' | 'None';
  imageFolder: string;
  chromePath: string | null;
  pandocPath: string | null;
  restorePreviousSession: boolean;
  confirmBeforeOpeningExternalLinks: boolean;
}
```

敏感项和可信工作区列表由 Rust 存储。设置迁移必须带 schema version。

---

## 16. Windows 集成

- `.md`、`.markdown` 文件关联；
- `Open with MDEditor`；
- 单实例；
- 第二次启动时把文件路径发送给已有实例；
- Jump List 可后续实现；
- 系统主题；
- 标准菜单和快捷键；
- Windows 缩放 100%–250%；
- 高对比度；
- 中文、英文长路径；
- 安装范围首版优先 per-user，避免管理员权限。

文件关联不得强制抢占默认应用。安装器提供可选项。

---

## 17. 性能预算

建议目标：

```text
主窗口可交互：< 2 s
打开 100 KiB 文件：< 300 ms
普通实时预览：< 500 ms 感知延迟
保存 1 MiB 文件：< 300 ms
HTML 导出 1 MiB 技术文档：< 5 s
PDF 导出：按图表复杂度记录，不设不现实硬门槛
空闲内存：建立基准并持续监控
```

禁止为了追求启动速度而绕过 HTML 清洗、安全校验或崩溃恢复。

---

## 18. 关键 ADR

至少创建：

```text
0001-tauri-and-node-sidecar.md
0002-crossnote-in-memory-rendering.md
0003-markdown-canonical-document-model.md
0004-monaco-and-milkdown-dual-editor.md
0005-local-resource-protocol.md
0006-export-browser-discovery.md
0007-trusted-workspace-security.md
```

每个 ADR 包含：Context、Decision、Alternatives、Consequences、Status。

---

## 19. 首个执行提示词

将以下内容单独发送给 GLM 5.2：

```text
执行 DEVELOPMENT_PLAN.md 中的 M0 — Windows 仓库初始化和架构验证。

严格遵守 M0 范围冻结：只完成仓库、Tauri 2、React/TypeScript、Rust 检查、Node sidecar、Crossnote 最小渲染/PDF spike、CI、ADR 和许可证工作。不要实现 M1+ 的文件编辑、Monaco、Milkdown、正式预览、导出界面或设置页。

开始前：
1. 阅读 DEVELOPMENT_PLAN.md 第 0、2、4、5、10、11/M0、12、13、18 节。
2. 检查 Windows、Rust、Node、pnpm、MSVC、SDK、WebView2 环境。
3. 列出准备执行的 M0 文件和命令。

执行中：
- 使用 pnpm workspace；
- 使用 Tauri 2 + React + TypeScript；
- 使用 Rust stable MSVC；
- Node sidecar 通过 stdin/stdout JSON Lines 通信；
- stdout 只输出协议，日志写 stderr；
- 固定 crossnote 版本；
- 默认 enableScriptExecution=false；
- 不开放任意 shell 命令；
- 不使用硬编码绝对路径。

结束前必须运行：
pnpm check
pnpm test
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm build
pnpm tauri build

最终报告完成项、未完成项、文件列表、测试输出摘要、Crossnote 渲染/PDF spike 结果、sidecar 打包结果、已知风险，以及是否满足 M0 验收标准。
```

---

## 20. 参考资料

```text
Markdown Preview Enhanced:
https://github.com/shd101wyy/vscode-markdown-preview-enhanced

Crossnote:
https://github.com/shd101wyy/crossnote
https://shd101wyy.github.io/crossnote/

Tauri 2:
https://v2.tauri.app/
https://v2.tauri.app/learn/sidecar-nodejs/
https://v2.tauri.app/develop/sidecar/
https://v2.tauri.app/security/capabilities/
https://v2.tauri.app/distribute/windows-installer/

Milkdown:
https://milkdown.dev/

ProseMirror:
https://prosemirror.net/docs/guide/
```

---

## 21. 最终原则

1. Markdown 是唯一规范数据。
2. 不认识的语法必须保留，而不是猜测转换。
3. Crossnote 是渲染和导出引擎，不是整个应用架构。
4. Rust 是系统权限边界，Node sidecar 是受限服务，不是通用 shell。
5. 源码模式保证完整能力，混合模式改善体验，两者不可互相破坏数据。
6. 所有文档默认不可信。
7. PDF/HTML 先做稳，再扩展 Pandoc 和代码执行。
8. 每个里程碑范围冻结，先通过验收再进入下一阶段。
