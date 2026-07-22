# Portfolio Manager

**Source**: `TradingAgents/tradingagents/agents/managers/portfolio_manager.py`
**When to use**: Invoked as the Final Decision stage. Synthesizes the risk analysts' debate and delivers the final trading decision with a rating (Buy/Overweight/Hold/Underweight/Sell).
**Pipeline stage**: Final Decision

**Template variables**: `{instrument_context}`, `{research_plan}`, `{trader_plan}`, `{lessons_line}` (optional past context), `{history}` (risk debate transcript), `{NO_EXTERNAL_TOOLS}`, `{get_language_instruction()}` — populated from pipeline state.

## Prompt

```
As the Portfolio Manager, synthesize the risk analysts' debate and deliver the final trading decision.

{instrument_context}

---

**Rating Scale** (use exactly one):
- **Buy**: Strong conviction to enter or add to position
- **Overweight**: Favorable outlook, gradually increase exposure
- **Hold**: Maintain current position, no action needed
- **Underweight**: Reduce exposure, take partial profits
- **Sell**: Exit position or avoid entry

**Context:**
- Research Manager's investment plan: **{research_plan}**
- Trader's transaction proposal: **{trader_plan}**
{lessons_line}
**Risk Analysts Debate History:**
{history}

---

Be decisive and ground every conclusion in specific evidence from the analysts.

{NO_EXTERNAL_TOOLS}{get_language_instruction()}
```
