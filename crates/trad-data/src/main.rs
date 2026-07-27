mod format;
mod fundamentals;
mod http;
mod indicators;
mod market;
mod news;
mod sentiment;

use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "trad-data", about = "TradingAgents data fetcher")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stock {
            symbol,
            start,
            end,
            tail,
            indicators,
            no_indicators,
            stats,
            no_stats,
            raw,
        } => {
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
                match market::fetch_ohlcv(&symbol, &start_date, &end_date).await {
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
                match market::fetch_ohlcv(&symbol, &start_date, &end_date).await {
                    Ok(data) => {
                        let report = format::build_compact_report(
                            &symbol,
                            &start_date,
                            &end_date,
                            &data,
                            tail,
                            use_indicators,
                            use_stats,
                            false,
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
        Commands::Fundamentals { symbol } => {
            let result = fundamentals::fetch_fundamentals(&symbol).await;
            if result.starts_with("错误") {
                eprintln!("{}", result);
                std::process::exit(1);
            }
            println!("{}", result);
        }
        Commands::News {
            symbol,
            days,
            limit,
        } => {
            let result = news::fetch_news(&symbol, days, limit).await;
            if result.starts_with("错误") {
                eprintln!("{}", result);
                std::process::exit(1);
            }
            println!("{}", result);
        }
        Commands::Sentiment { symbol, limit } => {
            let result = sentiment::fetch_sentiment(&symbol, limit).await;
            if result.starts_with("错误") {
                eprintln!("{}", result);
                std::process::exit(1);
            }
            println!("{}", result);
        }
    }

    Ok(())
}
