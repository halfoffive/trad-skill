mod format;
mod fundamentals;
mod http;
mod indicators;
mod install;
mod market;
mod news;
mod sentiment;
mod yahoo;

use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "trad-skill",
    about = "TradingAgents skill: installer + data fetcher",
    version
)]
struct Cli {
    /// 目标 agent：claude | agents | opencode（默认 agents）。仅 install 模式生效。
    #[arg(long, value_enum, global = true)]
    agent: Option<install::AgentTarget>,

    /// 自定义目标 skills 父目录（与 --agent 互斥）。仅 install 模式生效。
    #[arg(long, global = true)]
    dir: Option<String>,

    /// 源技能目录。仅 install 模式生效。
    #[arg(long = "skills-dir", global = true)]
    skills_dir: Option<String>,

    /// 要复制的平台二进制路径。默认 self-copy。仅 install 模式生效。
    #[arg(long = "bin-path", global = true)]
    bin_path: Option<String>,

    /// 跳过复制平台二进制。仅 install 模式生效。
    #[arg(long = "no-bin", global = true)]
    no_bin: bool,

    /// 只打印安装计划，不写入。仅 install 模式生效。
    #[arg(long = "dry-run", global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 获取行情数据 (兼容 fetch_stock_data.py)
    Stock {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
        #[arg(long, default_value_t = 30)]
        tail: u32,
        #[arg(long, default_value_t = true)]
        indicators: bool,
        #[arg(long = "no-indicators")]
        no_indicators: bool,
        #[arg(long, default_value_t = false)]
        stats: bool,
        #[arg(long = "no-stats")]
        no_stats: bool,
        #[arg(long, default_value_t = false)]
        raw: bool,
        /// 数据渠道：yahoo | eastmoney（默认按市场自动选择）
        #[arg(long, value_enum)]
        source: Option<market::Source>,
    },
    /// 获取基本面数据 (兼容 fetch_fundamentals.py)
    Fundamentals {
        #[arg(long)]
        symbol: String,
    },
    /// 获取新闻数据 (兼容 fetch_news.py)
    News {
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value_t = 7)]
        days: u32,
        #[arg(long, default_value_t = 8)]
        limit: u32,
    },
    /// 获取情绪数据 (兼容 fetch_sentiment.py)
    Sentiment {
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value_t = 15)]
        limit: u32,
    },
    /// 安装 tradingagents-analysis 技能（无子命令时的默认行为）
    Install {
        #[command(flatten)]
        args: install::InstallArgs,
    },
}

impl Cli {
    /// 把顶层平铺的 install flags 收集为 InstallArgs（显式 `install` 子命令优先）。
    fn into_install_args(self) -> install::InstallArgs {
        match self.command {
            Some(Commands::Install { args }) => args,
            _ => install::InstallArgs {
                agent: self.agent,
                dir: self.dir,
                skills_dir: self.skills_dir,
                wrapper: None,
                bin_path: self.bin_path,
                no_bin: self.no_bin,
                dry_run: self.dry_run,
            },
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 无子命令，或显式 `install` → 走安装器（无需 HTTP 客户端）
    let is_install = matches!(cli.command, None | Some(Commands::Install { .. }));
    if is_install {
        let args = cli.into_install_args();
        install::run(args)?;
        return Ok(());
    }

    let client = http::build_client()?;

    match cli.command {
        Some(Commands::Stock {
            symbol,
            start,
            end,
            tail,
            indicators,
            no_indicators,
            stats,
            no_stats,
            raw,
            source,
        }) => {
            let use_indicators = indicators && !no_indicators;
            let use_stats = stats && !no_stats;

            // 默认日期：end=今天, start=今天往前365天
            let today = Utc::now().format("%Y-%m-%d").to_string();
            let end_date = end.unwrap_or_else(|| today.clone());
            let start_date = start.unwrap_or_else(|| {
                (Utc::now() - Duration::days(365))
                    .format("%Y-%m-%d")
                    .to_string()
            });

            if raw {
                // --raw 模式：纯 CSV 输出
                match market::fetch_ohlcv(&client, &symbol, &start_date, &end_date, source).await {
                    Ok(data) => {
                        if data.is_empty() {
                            eprintln!("错误: 未获取到 {} 的数据", symbol);
                        } else {
                            print!("{}", format::ohlcv_to_csv(&data));
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // 默认模式：精简报告（指标 + 尾部 OHLCV）
                match market::fetch_ohlcv(&client, &symbol, &start_date, &end_date, source).await {
                    Ok(data) => {
                        let opts = format::ReportOptions {
                            tail,
                            indicators: use_indicators,
                            stats: use_stats,
                            raw: false,
                        };
                        let report = format::build_compact_report(
                            &symbol,
                            &start_date,
                            &end_date,
                            &data,
                            &opts,
                        );
                        print!("{}", report);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Some(Commands::Fundamentals { symbol }) => {
            let result = fundamentals::fetch_fundamentals(&client, &symbol).await;
            if result.starts_with("错误") {
                eprintln!("{}", result);
                std::process::exit(1);
            }
            println!("{}", result);
        }
        Some(Commands::News {
            symbol,
            days,
            limit,
        }) => {
            let result = news::fetch_news(&client, &symbol, days, limit).await;
            if result.starts_with("错误") {
                eprintln!("{}", result);
                std::process::exit(1);
            }
            println!("{}", result);
        }
        Some(Commands::Sentiment { symbol, limit }) => {
            let result = sentiment::fetch_sentiment(&client, &symbol, limit).await;
            if result.starts_with("错误") {
                eprintln!("{}", result);
                std::process::exit(1);
            }
            println!("{}", result);
        }
        Some(Commands::Install { .. }) | None => unreachable!(),
    }

    Ok(())
}
