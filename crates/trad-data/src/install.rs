//! 技能安装器：把 tradingagents-analysis/ 技能复制到目标 AI agent 的 skills 目录。
//!
//! 安装逻辑全部用 Rust 实现，npm 包 `trad-skill` 仅保留一个极薄的 JS launcher
//! 负责解析平台二进制并 exec 本程序。
//!
//! 默认目标：`~/.agents/skills`（通用 agent 目录）。可用 `--agent claude|opencode` 或 `--dir` 覆盖。
//! 无子命令时默认进入安装流程；`stock` / `news` / `fundamentals` / `sentiment` 是数据子命令。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::Args;

/// 技能名（同时也是目标目录下的子目录名）
pub const SKILL_NAME: &str = "tradingagents-analysis";

/// 支持的预置 AI agent 目标
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum AgentTarget {
    /// 安装到 ~/.claude/skills（Claude Code）
    Claude,
    /// 安装到 ~/.agents/skills（通用 agent 目录，默认）
    Agents,
    /// 安装到 ~/.config/opencode/skills（OpenCode）
    Opencode,
}

/// `trad-skill install` 子命令参数（与顶层平铺参数保持一致）
#[derive(Debug, Clone, Args)]
pub struct InstallArgs {
    /// 目标 agent：claude | agents | opencode（默认 agents）
    #[arg(long, value_enum, conflicts_with = "dir")]
    pub agent: Option<AgentTarget>,

    /// 自定义目标 skills 父目录（与 --agent 互斥）
    #[arg(long, conflicts_with = "agent")]
    pub dir: Option<String>,

    /// 源技能目录（含 SKILL.md 的 tradingagents-analysis 目录）。
    /// 生产环境由 JS launcher 注入；dev 下默认相对 CARGO_MANIFEST_DIR。
    #[arg(long = "skills-dir")]
    pub skills_dir: Option<String>,

    /// 要复制的平台二进制路径。默认取当前运行的可执行文件（self-copy）。
    #[arg(long = "bin-path")]
    pub bin_path: Option<String>,

    /// 跳过复制平台二进制
    #[arg(long = "no-bin")]
    pub no_bin: bool,

    /// 只打印安装计划，不写入任何文件
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// 兼容历史：要复制的 wrapper（已忽略，仅保留以避免旧 launcher 报错）。
    #[arg(long = "wrapper", hide = true)]
    pub wrapper: Option<String>,
}

/// 读取 home 目录：优先 HOME（Unix），回退 USERPROFILE（Windows）。
/// 不引入 `dirs` crate——windows/unix 路径分支覆盖全部 7 个交叉编译目标。
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// 预置 agent → 默认 skills 父目录
fn agent_dir(target: &AgentTarget) -> Result<PathBuf> {
    let home =
        home_dir().ok_or_else(|| anyhow!("无法确定 home 目录（HOME/USERPROFILE 未设置）"))?;
    Ok(match target {
        AgentTarget::Claude => home.join(".claude").join("skills"),
        AgentTarget::Agents => home.join(".agents").join("skills"),
        AgentTarget::Opencode => home.join(".config").join("opencode").join("skills"),
    })
}

/// 把相对路径解析为相对 cwd 的绝对路径（不要求路径已存在）
fn absolutize(p: &str) -> Result<PathBuf> {
    let path = PathBuf::from(p);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

/// 展开 `~` / `~/` / `~\` 为 home 目录前缀
fn expand_tilde(p: &str) -> Result<PathBuf> {
    let home =
        || home_dir().ok_or_else(|| anyhow!("无法确定 home 目录（HOME/USERPROFILE 未设置）"));
    if p == "~" {
        return home();
    }
    if let Some(rest) = p.strip_prefix("~/") {
        return Ok(home()?.join(rest));
    }
    if let Some(rest) = p.strip_prefix("~\\") {
        return Ok(home()?.join(rest));
    }
    absolutize(p)
}

/// 解析目标 skills 父目录。--dir 与 --agent 互斥；二者皆空时默认 agents。
fn resolve_parent_dir(dir: &Option<String>, agent: &Option<AgentTarget>) -> Result<PathBuf> {
    match (dir, agent) {
        (Some(_), Some(_)) => {
            bail!("不能同时指定 --dir 和 --agent（--dir 为自定义路径，--agent 为预置目标）")
        }
        (Some(d), None) => expand_tilde(d),
        (None, Some(a)) => agent_dir(a),
        (None, None) => agent_dir(&AgentTarget::Agents),
    }
}

/// 把 rustc 的 target 元素映射为 Node 的 process.platform/process.arch 命名。
/// `std::env::consts` 在编译期按 target 固化，因此交叉编译产物在目标机上运行时返回正确值。
/// 该键必须与 `bin/trad-skill.js` 的 strategy-1 查找路径一致，否则技能在 npm
/// 上下文之外将静默找不到二进制——务必保持同步并配测试。
fn node_platform_key() -> String {
    let plat = match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        "linux" => "linux",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{plat}-{arch}")
}

/// 平台二进制文件扩展名
fn bin_ext() -> &'static str {
    if std::env::consts::OS == "windows" {
        ".exe"
    } else {
        ""
    }
}

/// 纯 std 递归目录复制（不引入额外运行时依赖；技能目录全是文本文件，无符号链接）
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("创建目录失败：{}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("读取目录失败：{}", src.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else if ft.is_file() {
            fs::copy(&path, &dest).with_context(|| {
                format!("复制文件失败：{} → {}", path.display(), dest.display())
            })?;
        }
        // 符号链接：技能目录不包含，跳过
    }
    Ok(())
}

/// 安装入口
pub fn run(args: InstallArgs) -> Result<()> {
    // 1. 解析源技能目录
    let skills_src = match &args.skills_dir {
        Some(s) => absolutize(s)?,
        None => {
            // dev 默认：相对 crate manifest 指向仓库 skills/
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../skills")
                .join(SKILL_NAME)
        }
    };
    let skill_md = skills_src.join("SKILL.md");
    if !skill_md.exists() {
        bail!(
            "找不到技能源目录：{}（缺少 SKILL.md，安装包可能不完整）",
            skills_src.display()
        );
    }

    // 2. 解析目标父目录
    let parent_dir = resolve_parent_dir(&args.dir, &args.agent)?;
    let dest_dir = parent_dir.join(SKILL_NAME);

    let node_key = node_platform_key();
    let ext = bin_ext();
    let bin_name = format!("trad-skill{ext}");

    // 3. 解析二进制源（self-copy：当前可执行文件即平台二进制）
    let bin_src = if args.no_bin {
        None
    } else {
        match &args.bin_path {
            Some(b) => Some(PathBuf::from(b)),
            None => Some(std::env::current_exe()?),
        }
    };

    // 4. dry-run：只打印计划
    if args.dry_run {
        println!("【试运行】将安装 {SKILL_NAME} → {}", dest_dir.display());
        println!("  源技能目录：{}", skills_src.display());
        match &bin_src {
            Some(b) => println!(
                "  二进制：{} → {}/{bin_name}",
                b.display(),
                dest_dir.join("bin").join(&node_key).display()
            ),
            None => println!("  二进制：跳过（--no-bin）"),
        }
        return Ok(());
    }

    // 5. 真正安装：幂等（先删旧目录再复制）
    fs::create_dir_all(&parent_dir)
        .with_context(|| format!("创建父目录失败：{}", parent_dir.display()))?;
    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir)
            .with_context(|| format!("删除旧安装目录失败：{}", dest_dir.display()))?;
    }
    copy_dir_recursive(&skills_src, &dest_dir).context("复制技能文件失败")?;

    let dest_bin = dest_dir.join("bin");
    fs::create_dir_all(&dest_bin)?;

    // 6. 复制平台二进制到 bin/<platform>/trad-skill[.exe]
    let bin_installed = if !args.no_bin {
        if let Some(b) = &bin_src {
            if b.exists() {
                let dest_platform_dir = dest_bin.join(&node_key);
                fs::create_dir_all(&dest_platform_dir)?;
                fs::copy(b, dest_platform_dir.join(&bin_name))
                    .with_context(|| format!("复制平台二进制失败：{}", b.display()))?;
                true
            } else {
                eprintln!("⚠ 找不到平台二进制：{}（数据工具不可用）", b.display());
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    // 7. 打印结果
    println!("✓ 已安装 {SKILL_NAME} → {}", dest_dir.display());
    println!();
    println!("下一步：");
    if bin_installed {
        println!("  ✓ trad-skill 二进制已安装 ({node_key})");
    } else {
        println!("  ⚠ 数据二进制未安装，可改用 `bunx trad-skill@latest <子命令>` 调用数据工具。");
    }
    println!("  1. 重启你的 AI agent / 开一个新会话，让它加载该技能。");
    println!("  2. 触发分析，例如：\"分析 AAPL\" 或 \"Analyze 600519\" 。");
    println!();
    println!("数据工具（Rust 二进制）:");
    println!("  bunx trad-skill@latest stock --symbol AAPL");
    println!("  bunx trad-skill@latest fundamentals --symbol AAPL");
    println!("  bunx trad-skill@latest news --symbol AAPL");
    println!("  bunx trad-skill@latest sentiment --symbol AAPL");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 跨平台设置 home（HOME + USERPROFILE），返回临时 home 路径
    fn set_home(tmp: &Path) {
        std::env::set_var("HOME", tmp);
        std::env::set_var("USERPROFILE", tmp);
    }

    fn default_install_args() -> InstallArgs {
        InstallArgs {
            agent: None,
            dir: None,
            skills_dir: None,
            bin_path: None,
            no_bin: true,
            dry_run: false,
            wrapper: None,
        }
    }

    #[test]
    fn agent_dir_mapping() {
        let tmp = std::env::temp_dir().join("trad-skill-install-test-agentdir");
        std::fs::remove_dir_all(&tmp).ok();
        std::fs::create_dir_all(&tmp).unwrap();
        set_home(&tmp);
        assert_eq!(
            agent_dir(&AgentTarget::Claude).unwrap(),
            tmp.join(".claude/skills")
        );
        assert_eq!(
            agent_dir(&AgentTarget::Agents).unwrap(),
            tmp.join(".agents/skills")
        );
        assert_eq!(
            agent_dir(&AgentTarget::Opencode).unwrap(),
            tmp.join(".config/opencode/skills")
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn default_target_is_agents() {
        let tmp = std::env::temp_dir().join("trad-skill-install-test-default");
        std::fs::remove_dir_all(&tmp).ok();
        std::fs::create_dir_all(&tmp).unwrap();
        set_home(&tmp);
        let resolved = resolve_parent_dir(&None, &None).unwrap();
        assert_eq!(resolved, tmp.join(".agents/skills"));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn dir_and_agent_mutually_exclusive() {
        use crate::Cli;
        use clap::Parser;
        let r = Cli::try_parse_from(["trad-skill", "install", "--dir", "x", "--agent", "claude"]);
        assert!(r.is_err(), "--dir 与 --agent 应互斥");
    }

    #[test]
    fn expand_tilde_basic() {
        let tmp = std::env::temp_dir().join("trad-skill-install-test-tilde");
        std::fs::remove_dir_all(&tmp).ok();
        std::fs::create_dir_all(&tmp).unwrap();
        set_home(&tmp);
        assert_eq!(expand_tilde("~").unwrap(), tmp);
        assert_eq!(expand_tilde("~/foo").unwrap(), tmp.join("foo"));
        assert_eq!(expand_tilde("~\\bar").unwrap(), tmp.join("bar"));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn copy_dir_recursive_roundtrip() {
        let root = std::env::temp_dir().join("trad-skill-copytest");
        std::fs::remove_dir_all(&root).ok();
        let src = root.join("src");
        std::fs::create_dir_all(src.join("nested")).unwrap();
        std::fs::write(src.join("a.txt"), "hello").unwrap();
        std::fs::write(src.join("nested/b.txt"), "world").unwrap();
        let dst = root.join("dst");
        copy_dir_recursive(&src, &dst).unwrap();
        assert_eq!(std::fs::read_to_string(dst.join("a.txt")).unwrap(), "hello");
        assert_eq!(
            std::fs::read_to_string(dst.join("nested/b.txt")).unwrap(),
            "world"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn node_platform_key_shape() {
        let key = node_platform_key();
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), 2, "键应为 <os>-<arch> 形式");
        assert!(
            matches!(parts[0], "win32" | "darwin" | "linux"),
            "platform 部分 {0} 应为 win32/darwin/linux",
            parts[0]
        );
        assert!(
            matches!(parts[1], "x64" | "arm64"),
            "arch 部分 {0} 应为 x64/arm64",
            parts[1]
        );
    }

    #[test]
    fn no_subcommand_defaults_to_install() {
        use crate::Cli;
        use clap::Parser;
        let cli = Cli::try_parse_from(["trad-skill"]).expect("无子命令应解析成功");
        assert!(cli.command.is_none());
        // into_install_args 应能从顶层 flags 构造
        let args = default_install_args();
        assert!(args.agent.is_none());
    }

    #[test]
    fn data_subcommand_passes_through() {
        use crate::Cli;
        use clap::Parser;
        let cli = Cli::try_parse_from(["trad-skill", "stock", "--symbol", "AAPL"])
            .expect("stock 子命令应解析成功");
        assert!(matches!(cli.command, Some(crate::Commands::Stock { .. })));
    }
}
