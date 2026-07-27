use crate::indicators;
use crate::market::OhlcvRow;

/// 构建精简报告（对齐 Python build_compact_report）
///
/// 默认模式：指标快照表 + 尾部 OHLCV（默认30行）
/// `stats` 模式：附加区间统计
/// `raw` 模式：纯 CSV 输出
#[allow(clippy::too_many_arguments)]
pub fn build_compact_report(
    symbol: &str,
    start: &str,
    end: &str,
    data: &[OhlcvRow],
    tail: u32,
    use_indicators: bool,
    use_stats: bool,
    raw: bool,
) -> String {
    if data.is_empty() {
        return format!(
            "错误: 未获取到 {} 在 {} 至 {} 的数据，请检查代码和日期范围。",
            symbol, start, end
        );
    }

    // --raw 模式：纯 CSV 输出
    if raw {
        return ohlcv_to_csv(data);
    }

    let mut sections: Vec<String> = Vec::new();
    sections.push(format!("# {} 行情（{} 至 {}）\n", symbol, start, end));

    // 区间统计
    if use_stats {
        sections.push(indicators::compute_stats(data));
    }

    // 技术指标快照
    if use_indicators {
        sections.push(indicators::compute_indicators(data));
    }

    // 尾部 OHLCV
    let tail_n = tail as usize;
    let tail_data = if data.len() > tail_n {
        &data[data.len() - tail_n..]
    } else {
        data
    };

    sections.push(format!("## 最近 {} 行 OHLCV\n", tail_data.len()));
    sections.push("```csv".to_string());
    sections.push(ohlcv_to_csv(tail_data));
    sections.push("```\n".to_string());

    sections.join("\n")
}

/// 将 OhlcvRow 切片转为 CSV 字符串
///
/// 格式：Date,Open,High,Low,Close,Volume
pub fn ohlcv_to_csv(data: &[OhlcvRow]) -> String {
    let mut lines = Vec::new();
    lines.push("Date,Open,High,Low,Close,Volume".to_string());
    for row in data {
        lines.push(format!(
            "{},{},{},{},{},{}",
            row.date, row.open, row.high, row.low, row.close, row.volume
        ));
    }
    lines.join("\n")
}
