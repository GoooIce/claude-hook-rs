# Claude Hook Advisor

一个使用 **三重钩子架构** 与 Claude Code 集成的 Rust CLI 工具，提供智能命令建议和语义目录别名功能。通过自动命令映射和自然语言目录引用来增强您的开发工作流程。

## 🎬 您将体验到的功能

安装后，claude-hook-advisor 会在您的 Claude Code 对话中无形地工作：

### 目录别名魔法 ✨
**您输入：** *"我的文档目录里有什么文件？"*
**Claude 回复：** *"我来检查您文档目录 /Users/you/Documents/Documentation 中的文件。"*

在幕后，您会看到：
```
<user-prompt-submit-hook>目录引用 'docs' 已解析为：/Users/you/Documents/Documentation</user-prompt-submit-hook>
```

**您输入：** *"检查 project_docs 中的 API 文档"*
**Claude 自动知道：** *使用 `/Users/you/Documents/Documentation/my-project/` 而无需您输入完整路径*

### 命令智能的实际应用 🚀
**Claude 尝试运行：** `npm install`
**工具干预：** *根据您的配置建议使用 `bun install`*
**Claude 自动运行：** `bun install` *无需手动干预*

**您看到的是：** Claude 无缝地使用您偏好的工具，无需每次手动纠正。

### 魔法是无形的
- 无需记住额外命令
- 不会中断您的工作流程
- 自然语言目录引用直接有效
- 您偏好的工具被自动使用
- 所有操作都在 Claude Code 对话中透明进行

## 功能特性

### 🎯 命令智能
- **智能命令映射**：将任何命令映射到首选替代方案，支持正则表达式
- **每项目配置**：每个项目都可以有自己的 `.claude-hook-advisor.toml` 文件
- **三重钩子集成**：PreToolUse、UserPromptSubmit 和 PostToolUse 钩子

### 📁 语义目录别名
- **自然语言目录引用**：在对话中使用 "文档"、"central_docs"、"project_docs" 等
- **简单路径映射**：直接别名到路径的映射，支持波浪号扩展
- **自动解析**：Claude Code 自动将语义引用解析为规范路径
- **TOML 配置**：基于简单配置文件的设置

### 🚀 性能与安全
- **快速轻量**：使用 Rust 构建以获得最佳性能
- **路径规范化**：防止目录遍历攻击的安全性
- **优雅错误处理**：强大的回退机制

## 安装

### 从 crates.io 安装（推荐）

使用 cargo 直接从 crates.io 安装：

```bash
cargo install claude-hook-advisor
```

这会将二进制文件安装到 `~/.cargo/bin/claude-hook-advisor`（确保 `~/.cargo/bin` 在您的 PATH 中）。

### 从源码安装

```bash
git clone https://github.com/sirmews/claude-hook-advisor.git
cd claude-hook-advisor
make install
```

## 快速开始

### 1. 安装和配置钩子
```bash
# 安装二进制文件
cargo install claude-hook-advisor

# 自动安装钩子到 Claude Code（创建备份）
claude-hook-advisor --install-hooks

# 如需移除钩子（带备份）
claude-hook-advisor --uninstall-hooks
```

### 2. 配置目录别名
编辑您的 `.claude-hook-advisor.toml` 文件来设置目录别名：

```toml
# 语义目录别名 - 使用自然语言！
[semantic_directories]
"项目文档" = "~/Documents/Documentation/my-project"
"中央文档" = "~/Documents/Documentation"
"claude 文档" = "~/Documents/Documentation/claude"
"测试数据" = "~/Documents/test-data"
```

**专业提示：** 使用带引号的、空格分隔的别名进行自然对话：
- *"检查项目文档文件夹"* → 匹配 `"项目文档"`
- *"查看测试数据目录"* → 匹配 `"测试数据"`

### 3. 配置命令映射
在项目根目录创建 `.claude-hook-advisor.toml` 文件：

```toml
# 命令映射
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
curl = "wget --verbose"

# 语义目录别名 - 自然语言
[semantic_directories]
"项目文档" = "~/Documents/Documentation/my-project"
"中央文档" = "~/Documents/Documentation"
"claude 文档" = "~/Documents/Documentation/claude"
```

### 配置示例

**Node.js 项目（优先使用 bun）：**
```toml
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
```

**Python 项目（优先使用 uv）：**
```toml
[commands]
pip = "uv pip"
"pip install" = "uv add"
```

**通用偏好设置：**
```toml
[commands]
curl = "wget --verbose"
cat = "bat"
ls = "eza"
```

## Claude Code 集成

### 自动安装（推荐）
```bash
claude-hook-advisor --install-hooks
```

这会自动配置所有三个钩子：
- **PreToolUse**：命令建议和阻止
- **UserPromptSubmit**：目录引用检测
- **PostToolUse**：分析和执行跟踪

### 手动配置

如果您更喜欢手动设置，请添加到您的 `.claude/settings.json`：

```json
{
  "hooks": {
    "PreToolUse": { "Bash": "claude-hook-advisor --hook" },
    "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" },
    "PostToolUse": { "Bash": "claude-hook-advisor --hook" }
  }
}
```

**注意**：这假设 `claude-hook-advisor` 在您的 PATH 中。在 `cargo install` 后，二进制文件通常位于 `~/.cargo/bin/claude-hook-advisor`。

## 工作原理

### 命令智能（PreToolUse 钩子）🚦

**流程：**
1. **命令检测**：当 Claude Code 尝试运行 Bash 命令时，钩子接收 JSON 输入
2. **配置加载**：工具从当前目录加载 `.claude-hook-advisor.toml`
3. **模式匹配**：使用词边界正则表达式匹配命令（例如，`npm` 匹配 `npm install` 但不匹配 `npm-check`）
4. **建议生成**：如果找到匹配，返回带有建议替换的阻止响应
5. **Claude 集成**：Claude 接收建议并自动使用正确命令重试

**幕后工作：**
```rust
// 简化的代码流程
let config = load_config(".claude-hook-advisor.toml")?;
let command = parse_bash_command(&hook_input.tool_input.command);

if let Some(replacement) = config.commands.get(&command.base_command) {
    return Ok(HookResponse::Block {
        reason: format!("命令 '{}' 已映射到 '{}'", command.base_command, replacement),
        suggested_command: command.replace_base_with(replacement),
    });
}
```

**智能之处：**
- 词边界匹配防止误报（`npm` 不会匹配 `npm-check`）
- 保留命令参数（`npm install --save` → `bun install --save`）
- 快速的基于正则表达式的模式匹配（~1ms 响应时间）

---

### 目录别名（UserPromptSubmit 钩子）📁

**流程：**
1. **文本分析**：扫描用户提示中的语义目录引用（例如，"文档"、"project_docs"）
2. **模式识别**：使用正则表达式在自然语言中检测目录别名
3. **路径扩展**：将波浪号 (~) 扩展为用户主目录
4. **路径解析**：将语义引用转换为规范文件系统路径
5. **安全验证**：执行路径规范化以防止遍历攻击

**幕后工作：**
```rust
// 模式检测
let patterns = [
    r"\b(文档|documentation)\b",
    r"\b项目[_\s]文档?\b",
    r"\b中央[_\s]文档?\b"
];

// 波浪号扩展
let resolved = expand_tilde(path_template)?;

// 安全规范化
let canonical = fs::canonicalize(&resolved)?;
```

**安全之处：**
- 路径规范化防止 `../../../etc/passwd` 攻击
- 只解析到配置的目录
- 在解析前验证路径存在

---

### 分析（PostToolUse 钩子）📊

**流程：**
1. **执行跟踪**：接收带有成功/失败数据的命令结果
2. **性能监控**：跟踪命令成功率和执行模式
3. **分析日志**：为优化和监控提供洞察

**幕后工作：**
```rust
// 成功/失败跟踪
match hook_data.tool_response.exit_code {
    0 => log::info!("命令 '{}' 成功", command),
    code => log::warn!("命令 '{}' 失败 (退出: {})", command, code),
}
```

**未来可能性：**
- 命令成功率分析
- 性能优化建议
- 使用模式洞察

## 示例输出

### 实际 Claude Code 对话

以下是 claude-hook-advisor 工作时的实际对话示例：

**🗣️ 您：** "我的文档里有什么文件？"

**🤖 Claude：** "⏺ 我来检查您文档目录 /Users/you/Documents/Documentation 中的文件。"

**幕后工作：**
```
[DEBUG] UserPromptSubmit 钩子被触发
[DEBUG] 模式匹配：'文档' -> '~/Documents/Documentation'
[DEBUG] 路径已解析：/Users/you/Documents/Documentation
```

**Claude 中的钩子消息：**
```
<user-prompt-submit-hook>目录引用 '文档' 已解析为：/Users/you/Documents/Documentation</user-prompt-submit-hook>
```

---

**🗣️ 您：** "安装这个项目的依赖"

**🤖 Claude：** "我将使用 npm install 安装依赖。"
*(Claude 尝试：`npm install`)*

**钩子拦截：**
```json
{
  "decision": "block",
  "reason": "命令 'npm' 已映射到 'bun'",
  "suggested_command": "bun install"
}
```

**🤖 Claude：** "根据您的项目偏好，我将使用 bun install 代替。"
*(Claude 运行：`bun install`)*

**结果：** 您偏好的包管理器被自动使用，无需手动纠正！

---

### 命令行测试

**目录解析：**
```bash
# 通过钩子测试目录解析
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档目录"}' | claude-hook-advisor --hook

# 预期输出：
# 目录引用 '文档' 已解析为：/Users/you/Documents/Documentation

*注意：目录解析要求路径在您的文件系统上存在。*
```

**钩子模拟：**
```bash
$ echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档目录"}' | claude-hook-advisor --hook
<user-prompt-submit-hook>目录引用 '文档' 已解析为：/Users/you/Documents/Documentation</user-prompt-submit-hook>

$ echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
{
  "decision": "block",
  "reason": "命令 'npm' 已映射到 'bun'",
  "suggested_command": "bun install"
}
```

## 开发

### 可用的 Make 目标

```bash
make build         # 调试模式构建
make release       # 发布模式构建
make test          # 运行测试
make lint          # 运行 clippy 检查
make fmt           # 格式化代码
make clean         # 清理构建产物
make example-config# 创建示例配置
make run-example   # 使用示例输入测试
make help          # 显示所有目标
```

### 测试

```bash
# 运行单元测试
make test

# 测试示例 npm 命令
make run-example

# 手动测试 - 命令映射（PreToolUse）
echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"yarn start"}}' | ./target/debug/claude-hook-advisor --hook

# 手动测试 - 目录检测（UserPromptSubmit）
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档目录"}' | ./target/debug/claude-hook-advisor --hook

# 手动测试 - 分析（PostToolUse）
echo '{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"bun install"},"tool_response":{"exit_code":0}}' | ./target/debug/claude-hook-advisor --hook

# 测试现有配置的目录解析
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档目录"}' | ./target/debug/claude-hook-advisor --hook
```

## 🔧 故障排除与调试

### 理解钩子消息

当 claude-hook-advisor 正常工作时，您会在 Claude Code 中看到这些消息：

**目录解析：**
```
<user-prompt-submit-hook>目录引用 '文档' 已解析为：/Users/you/Documents/Documentation</user-prompt-submit-hook>
```

**命令建议：**
```
<pre-tool-use-hook>命令 'npm' 已映射到 'bun'。建议：bun install</pre-tool-use-hook>
```

**执行跟踪：**
```
<post-tool-use-hook>命令 'bun install' 成功完成（退出代码：0）</post-tool-use-hook>
```

### 调试模式

启用详细日志记录以查看幕后发生的情况：

```bash
# 将 RUST_LOG=debug 添加到您的 Claude Code 设置
{
  "hooks": {
    "UserPromptSubmit": { ".*": "RUST_LOG=debug claude-hook-advisor --hook" },
    "PreToolUse": { "Bash": "RUST_LOG=debug claude-hook-advisor --hook" },
    "PostToolUse": { "Bash": "RUST_LOG=debug claude-hook-advisor --hook" }
  }
}
```

**调试输出显示：**
- 配置文件加载
- 模式匹配详情
- 路径解析步骤
- 变量替换
- 安全验证

### 常见问题与解决方案

#### 🚫 钩子未触发
**问题：** Claude Code 对话中没有出现钩子消息

**解决方案：**
1. 通过检查 Claude Code 设置文件验证钩子安装
2. 检查 `.claude/settings.json` 或 `.claude/settings.local.json`：
   ```json
   {
     "hooks": {
       "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" }
     }
   }
   ```
3. 确保 `claude-hook-advisor` 在您的 PATH 中：`which claude-hook-advisor`
4. 手动测试：`echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档"}' | claude-hook-advisor --hook`

#### 📁 目录未解析
**问题：** "文档" 没有解析到预期路径

**解决方案：**
1. 检查配置文件存在：`ls .claude-hook-advisor.toml`
2. 验证别名配置：
   ```toml
   [semantic_directories]
   文档 = "~/Documents/Documentation"
   ```
3. 通过钩子测试解析：`echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档"}' | claude-hook-advisor --hook`
4. 检查文件权限：`ls -la .claude-hook-advisor.toml`

#### ⚙️ 命令未被映射
**问题：** `npm` 仍然运行而不是 `bun`

**解决方案：**
1. 验证配置中的命令映射：
   ```toml
   [commands]
   npm = "bun"
   ```
2. 测试映射：`echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook`
3. 检查词边界：`npm-check` 不会匹配 `npm = "bun"`（设计如此）
4. 添加调试日志以查看模式匹配

#### 🔒 权限问题
**问题：** 钩子因权限错误失败

**解决方案：**
1. 使二进制文件可执行：`chmod +x ~/.cargo/bin/claude-hook-advisor`
2. 检查文件所有权：`ls -la ~/.cargo/bin/claude-hook-advisor`
3. 验证 PATH 包含 `~/.cargo/bin`：`echo $PATH`

#### 🐛 调试您的配置

**单独测试每个组件：**

```bash
# 通过钩子测试目录解析
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档"}' | claude-hook-advisor --hook

# 测试命令映射
echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook

# 测试用户提示分析
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档目录"}' | claude-hook-advisor --hook

# 通过测试解析检查配置
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"检查文档"}' | claude-hook-advisor --hook
```

### 性能说明

- **启动时间：** 每次钩子调用约 1-5ms
- **内存使用：** 每个进程约 2-3MB
- **文件监控：** 配置在每次钩子调用时加载（无缓存）
- **路径解析：** 使用文件系统规范化确保安全

## 配置文件查找

工具按以下顺序查找配置文件：

1. 使用 `-c/--config` 标志指定的自定义路径
2. 当前目录中的 `.claude-hook-advisor.toml`
3. 如果未找到配置，允许所有命令（无映射）

## 使用场景

### 命令智能
- **包管理器一致性**：强制使用 `bun` 而不是 `npm`/`yarn`
- **工具偏好**：用 `wget` 替换 `curl`，用 `bat` 替换 `cat` 等
- **项目标准**：确保团队成员使用一致的工具
- **遗留迁移**：逐步从旧工具迁移到新工具
- **安全策略**：阻止危险命令或重定向到更安全的替代方案

### 目录别名
- **文档管理**：使用 "文档" 而不是输入完整路径
- **项目组织**：自然引用 "project_docs"、"central_docs"
- **跨平台路径**：抽象化平台特定的目录结构
- **团队协作**：团队成员间共享语义目录引用
- **工作流自动化**：Claude 对话中的自然语言目录引用

## 类似工具

这个工具的灵感来源于并类似于：
- Shell 别名（但在 Claude Code 级别工作）
- Git 钩子（但用于命令执行）
- 包管理器配置文件

## 支持

如果您觉得这个工具有用，请考虑支持其开发：

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A01HT0RG)

---

## 许可证

本项目采用 MIT 或 Apache-2.0 双重许可证。详细信息请参见 [LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE) 文件。

## 贡献

欢迎贡献！请随时提交 issue 和 pull request。

---

*让您的 Claude Code 体验更智能、更高效。*