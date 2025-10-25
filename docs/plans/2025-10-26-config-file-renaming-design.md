# 配置文件重命名设计文档

**项目**: Claude Hook Advisor
**日期**: 2025-10-26
**版本**: v0.2.0+
**作者**: Claude Code Assistant

## 概述

本文档描述了将 `.claude-hook-advisor.toml` 配置文件名重命名为更简短形式的完整设计方案。重命名旨在提高用户友好性，同时保持向后兼容性和系统稳定性。

## 设计目标

### 主要目标
- 提供更简洁、易记的配置文件名
- 保持完全的向后兼容性
- 提供平滑的用户迁移路径
- 最小化性能影响

### 次要目标
- 减少用户输入错误
- 提高配置文件的可发现性
- 简化文档中的配置文件引用

## 技术方案

### 选择的文件名

**新的默认配置文件名**: `.claude.toml`

**选择理由**:
- 简洁性：从 21 字符减少到 11 字符（减少 48%）
- 明确性：保留 "claude" 关键词，维持与 Claude Code 的关联
- 可读性：易于理解和记忆
- 独特性：避免与其他常见配置文件冲突

### 文件查找优先级

```rust
// src/config.rs 中的查找逻辑
const CONFIG_FILE_NAMES: &[&str] = &[
    ".claude.toml",                    // 新的默认文件名
    ".claude-hook-advisor.toml",       // 原文件名（向后兼容）
];
```

**查找顺序**:
1. 查找 `.claude.toml`
2. 如果未找到，查找 `.claude-hook-advisor.toml`
3. 如果都未找到，返回 `ConfigError::NotFound`

## 架构设计

### 核心组件修改

#### 1. 配置模块 (`src/config.rs`)

**新增函数**:

```rust
/// 查找可用的配置文件
pub fn find_config_file() -> Result<PathBuf, ConfigError> {
    for filename in CONFIG_FILE_NAMES {
        let path = PathBuf::from(filename);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(ConfigError::NotFound("No configuration file found".to_string()))
}

/// 检测是否需要迁移配置文件
pub fn needs_migration() -> Option<PathBuf> {
    let old_config = PathBuf::from(".claude-hook-advisor.toml");
    let new_config = PathBuf::from(".claude.toml");

    if old_config.exists() && !new_config.exists() {
        Some(old_config)
    } else {
        None
    }
}

/// 迁移配置文件到新名称
pub fn migrate_config() -> Result<(), ConfigError> {
    let old_path = PathBuf::from(".claude-hook-advisor.toml");
    let new_path = PathBuf::from(".claude.toml");
    let backup_path = PathBuf::from(".claude-hook-advisor.toml.backup");

    // 创建备份
    fs::copy(&old_path, &backup_path)?;

    // 重命名文件
    fs::rename(&old_path, &new_path)?;

    // 验证新配置
    load_config_from_path(&new_path)?;

    Ok(())
}
```

**修改现有函数**:

```rust
/// 更新 load_config 函数以支持新的查找逻辑
pub fn load_config() -> Result<Config, ConfigError> {
    let config_path = find_config_file()?;
    load_config_from_path(&config_path)
}
```

#### 2. CLI 模块 (`src/cli.rs`)

**新增命令行选项**:

```rust
#[derive(Parser)]
pub struct Cli {
    // 现有选项...

    /// 配置文件路径（可选）
    #[arg(short = 'c', long = "config")]
    pub config_file: Option<PathBuf>,

    /// 检查配置文件状态
    #[arg(long = "check-config")]
    pub check_config: bool,

    /// 迁移配置文件到新格式
    #[arg(long = "migrate-config")]
    pub migrate_config: bool,

    /// 创建示例配置文件
    #[arg(long = "init-config")]
    pub init_config: bool,
}
```

**更新帮助信息**:

```rust
const HELP_CONFIG: &str = r#"
配置文件:
  .claude.toml                   新的默认配置文件名
  .claude-hook-advisor.toml      原配置文件名（向后兼容）

查找顺序:
  1. .claude.toml
  2. .claude-hook-advisor.toml

迁移命令:
  claude-hook-advisor --migrate-config    自动迁移到新文件名
  claude-hook-advisor --check-config      检查配置状态
  claude-hook-advisor --init-config       创建示例配置
"#;
```

#### 3. 类型定义 (`src/types.rs`)

**新增常量**:

```rust
/// 支持的配置文件名
pub const CONFIG_FILE_NAMES: &[&str] = &[
    ".claude.toml",
    ".claude-hook-advisor.toml",
];

/// 默认配置文件名
pub const DEFAULT_CONFIG_FILE: &str = ".claude.toml";

/// 备份文件后缀
pub const BACKUP_SUFFIX: &str = ".backup";
```

**错误类型扩展**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    // 现有错误类型...

    #[error("Configuration file not found")]
    NotFound(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Backup creation failed: {0}")]
    BackupFailed(String),

    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
}
```

### 错误处理策略

#### 分层错误处理

1. **用户友好错误信息**:
   ```rust
   match error {
       ConfigError::NotFound(_) => {
           eprintln!("❌ 未找到配置文件");
           eprintln!("💡 使用 --init-config 创建示例配置");
           eprintln!("📖 查看文档了解更多：https://github.com/sirmews/claude-hook-advisor");
       }
       ConfigError::MigrationFailed(msg) => {
           eprintln!("❌ 配置迁移失败: {}", msg);
           eprintln!("🔄 请手动重命名配置文件");
           eprintln!("📋 备份文件已创建");
       }
       // 其他错误类型...
   }
   ```

2. **详细诊断信息**:
   ```rust
   fn diagnose_config_issues() -> Vec<String> {
       let mut issues = Vec::new();

       // 检查文件权限
       if let Ok(metadata) = fs::metadata(".claude.toml") {
           if metadata.permissions().readonly() {
               issues.push("配置文件为只读".to_string());
           }
       }

       // 检查文件格式
       if let Err(e) = load_config() {
           issues.push(format!("配置文件格式错误: {}", e));
       }

       issues
   }
   ```

### 迁移流程设计

#### 自动迁移流程

```bash
# 1. 检测迁移需求
$ claude-hook-advisor --check-config

# 输出示例：
# ℹ️  发现旧配置文件：.claude-hook-advisor.toml
# 💡 建议迁移到：.claude.toml
# 🔄 运行 'claude-hook-advisor --migrate-config' 进行自动迁移

# 2. 执行迁移
$ claude-hook-advisor --migrate-config

# 执行步骤：
# ✅ 正在读取原配置文件...
# ✅ 创建备份：.claude-hook-advisor.toml.backup
# ✅ 写入新配置文件：.claude.toml
# ✅ 验证新配置...
# ✅ 迁移完成！
# 🗑️  如需删除备份文件，请手动删除 .claude-hook-advisor.toml.backup
```

#### 迁移实现细节

```rust
pub fn migrate_config_with_progress() -> Result<(), ConfigError> {
    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new(4);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")
        .unwrap());

    // 步骤 1: 读取原配置
    pb.set_message("正在读取原配置文件...");
    let old_config = load_config_from_path(&PathBuf::from(".claude-hook-advisor.toml"))?;
    pb.inc(1);

    // 步骤 2: 创建备份
    pb.set_message("创建备份文件...");
    let backup_path = PathBuf::from(".claude-hook-advisor.toml.backup");
    fs::copy(".claude-hook-advisor.toml", &backup_path)?;
    pb.inc(1);

    // 步骤 3: 写入新配置
    pb.set_message("写入新配置文件...");
    save_config_to_path(&old_config, &PathBuf::from(".claude.toml"))?;
    pb.inc(1);

    // 步骤 4: 验证配置
    pb.set_message("验证新配置...");
    let _ = load_config_from_path(&PathBuf::from(".claude.toml"))?;
    pb.inc(1);

    pb.finish_with_message("✅ 迁移完成！");

    // 清理原文件
    fs::remove_file(".claude-hook-advisor.toml")?;

    Ok(())
}
```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_file_priority() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join(".claude-hook-advisor.toml");
        let new_path = temp_dir.path().join(".claude.toml");

        // 创建新配置文件
        fs::write(&new_path, "[commands]\nnpm = \"bun\"").unwrap();

        // 应该找到新配置文件
        let found = find_config_file_in_dir(temp_dir.path()).unwrap();
        assert_eq!(found, new_path);
    }

    #[test]
    fn test_fallback_to_old_config() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join(".claude-hook-advisor.toml");

        // 只创建旧配置文件
        fs::write(&old_path, "[commands]\nnpm = \"bun\"").unwrap();

        // 应该回退到旧配置文件
        let found = find_config_file_in_dir(temp_dir.path()).unwrap();
        assert_eq!(found, old_path);
    }

    #[test]
    fn test_migration_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join(".claude-hook-advisor.toml");
        let new_path = temp_dir.path().join(".claude.toml");

        // 创建旧配置文件
        fs::write(&old_path, "[commands]\nnpm = \"bun\"").unwrap();

        // 执行迁移
        migrate_config_in_dir(temp_dir.path()).unwrap();

        // 验证结果
        assert!(!old_path.exists());
        assert!(new_path.exists());

        // 验证备份文件
        let backup_path = temp_dir.path().join(".claude-hook-advisor.toml.backup");
        assert!(backup_path.exists());
    }
}
```

### 集成测试

```rust
#[test]
fn test_cli_with_new_config() {
    // 测试 CLI 命令使用新配置文件
    let output = Command::new("./target/debug/claude-hook-advisor")
        .arg("--config")
        .arg(".claude.toml")
        .arg("--check-config")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("✅ 配置文件正常"));
}
```

### 端到端测试

```rust
#[test]
fn test_full_migration_workflow() {
    // 1. 创建旧配置文件
    fs::write(".claude-hook-advisor.toml", TEST_CONFIG_CONTENT).unwrap();

    // 2. 检查迁移需求
    let output = Command::new("./target/debug/claude-hook-advisor")
        .arg("--check-config")
        .output()
        .unwrap();

    assert!(String::from_utf8(output.stdout).unwrap().contains("发现旧配置文件"));

    // 3. 执行迁移
    let output = Command::new("./target/debug/claude-hook-advisor")
        .arg("--migrate-config")
        .output()
        .unwrap();

    assert!(output.status.success());

    // 4. 验证新配置可用
    assert!(Path::new(".claude.toml").exists());
    assert!(!Path::new(".claude-hook-advisor.toml").exists());

    // 清理
    fs::remove_file(".claude.toml").unwrap();
    fs::remove_file(".claude-hook-advisor.toml.backup").unwrap();
}
```

## 文档更新计划

### 需要更新的文件

1. **README.md**
   - 更新所有配置文件引用
   - 添加迁移指南
   - 更新安装和配置示例

2. **README.zh.md**
   - 中文版同步更新
   - 保持与英文版一致性

3. **CLAUDE.md**
   - 更新开发者指南中的配置文件引用
   - 添加测试相关的配置信息

4. **docs/configuration.md**
   - 重写配置文件章节
   - 添加迁移流程说明
   - 更新故障排除部分

5. **docs/installation.md**
   - 更新安装后的配置步骤
   - 添加自动迁移说明

### 示例配置更新

**新的示例配置** (`example.claude-hook-advisor.toml` → `example.claude.toml`):

```toml
# Claude Hook Advisor 配置示例
# 文件名：.claude.toml

[commands]
# Node.js / JavaScript 开发
npm = "bun"
yarn = "bun"
npx = "bunx"

# 现代工具替代
cat = "bat"
ls = "eza"
find = "fd"
grep = "rg"

[semantic_directories]
"项目文档" = "~/Documents/Documentation/my-project"
"中央文档" = "~/Documents/Documentation"
"测试数据" = "~/Documents/test-data"
```

## 实施计划

### 第一阶段：核心功能（预计 2-3 天）

**任务清单**：
1. ✅ 设计配置查找逻辑
2. ⏳ 修改 `src/config.rs` 中的核心函数
3. ⏳ 更新 `src/types.rs` 中的常量和错误类型
4. ⏳ 实现基础的向后兼容性
5. ⏳ 添加单元测试

**验收标准**：
- 所有现有测试通过
- 新配置文件可以被正确加载
- 旧配置文件仍然可用
- 向后兼容性测试通过

### 第二阶段：CLI 增强（预计 2 天）

**任务清单**：
1. ⏳ 添加新的命令行选项
2. ⏳ 实现迁移功能
3. ⏳ 改进错误提示信息
4. ⏳ 添加进度指示器
5. ⏳ 集成测试

**验收标准**：
- 迁移命令正常工作
- 错误信息清晰易懂
- 配置检查功能正常
- 进度指示器显示正确

### 第三阶段：文档和测试完善（预计 1-2 天）

**任务清单**：
1. ⏳ 更新所有文档文件
2. ⏳ 更新示例配置文件
3. ⏳ 完善端到端测试
4. ⏳ 性能测试和优化
5. ⏳ 最终验收测试

**验收标准**：
- 所有文档更新完成
- 测试覆盖率达到 90%+
- 性能无明显回归
- 用户验收测试通过

## 风险评估与缓解

### 主要风险

1. **向后兼容性破坏**
   - **风险级别**: 中等
   - **缓解措施**: 保持旧配置文件支持，添加迁移工具
   - **应急计划**: 如果发现问题，快速回退到原实现

2. **用户迁移困难**
   - **风险级别**: 低
   - **缓解措施**: 提供自动迁移工具，详细文档说明
   - **应急计划**: 提供手动迁移指南

3. **性能影响**
   - **风险级别**: 极低
   - **缓解措施**: 优化查找逻辑，添加缓存机制
   - **应急计划**: 性能监控和优化

4. **文档更新遗漏**
   - **风险级别**: 中等
   - **缓解措施**: 系统性检查所有文档，自动化验证
   - **应急计划**: 快速修复发现的遗漏

### 回滚计划

如果新实现出现严重问题，回滚步骤：

1. 保持 `src/config.rs` 中的原始 `load_config()` 函数
2. 添加特性标志 `use_legacy_config` 强制使用旧逻辑
3. 通过环境变量 `CLAUDE_HOOK_LEGACY_CONFIG=1` 启用回滚
4. 发布紧急修复版本

## 性能分析

### 预期性能影响

- **文件查找开销**: +0.5-1ms（额外的文件存在性检查）
- **内存使用**: 无明显变化
- **启动时间**: 几乎无影响（<1%）
- **配置加载**: 无变化（解析逻辑相同）

### 性能优化措施

1. **查找结果缓存**: 在同一进程中缓存配置文件路径
2. **早期退出**: 找到第一个有效配置文件后立即返回
3. **批量检查**: 在需要时批量检查多个配置文件

## 部署策略

### 版本发布计划

**v0.2.1 - 配置文件重命名版本**
- 包含所有重命名功能
- 保持完全向后兼容
- 添加迁移工具
- 更新所有文档

**发布检查清单**：
- [ ] 所有测试通过
- [ ] 文档更新完成
- [ ] 迁移工具测试通过
- [ ] 向后兼容性验证
- [ ] 性能测试通过
- [ ] 安全审查完成
- [ ] 用户验收测试通过

### 用户通知计划

1. **Release Notes**: 详细说明重命名和迁移信息
2. **GitHub Discussions**: 提前讨论和收集反馈
3. **文档预览**: 提供新文档的预览版本
4. **迁移指南**: 详细的分步迁移指南

## 总结

本设计文档详细描述了将 `.claude-hook-advisor.toml` 重命名为 `.claude.toml` 的完整方案。主要特点包括：

1. **用户友好**: 更简短、易记的文件名
2. **向后兼容**: 完全支持现有配置文件
3. **平滑迁移**: 自动化迁移工具和详细指导
4. **风险可控**: 全面的测试和回滚计划
5. **性能优秀**: 最小的性能影响

通过这个设计，用户将获得更好的使用体验，同时现有的用户不会受到任何影响。分阶段的实施计划确保了功能的稳定发布，全面的测试策略保证了代码质量。

---

**文档状态**: 设计完成，等待实施
**下一步**: 进入实施阶段，按照计划进行开发
**负责人**: 开发团队
**审核人**: 项目负责人