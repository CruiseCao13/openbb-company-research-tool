# Company Profile Routing

The asset router chooses a research frame from evidence, not from ticker names.

## Hard Rule

Production logic must not do this:

```python
if ticker == "AAPL":
    use_mature_compounder_template()
```

Ticker examples are allowed in tests and documentation only.

## Routing Signals

The router uses:

- sector and industry
- business summary text
- revenue growth
- operating margin
- free cash flow margin
- positive net income years
- positive FCF years
- valuation multiple availability
- cash runway
- financial / cyclical / biotech-like / aerospace-like clues
- semiconductor / foundry / manufacturing / processor / data-center clues
- data availability

## Profile-Specific Frames

Mature Compounder:
margin durability, FCF stability, business mix, capital returns, premium valuation risk.

Speculative / Unprofitable Growth:
growth quality, gross-margin improvement, operating-loss narrowing, FCF burn, runway, dilution, path to profitability.

Capital-Intensive Semiconductor Turnaround:
foundry execution, capex intensity, gross-margin recovery, process roadmap, data-center competitiveness, manufacturing utilization, inventory pressure, and free cash flow bridge.

Financials:
ROE, P/B, NIM, credit losses, asset quality, capital adequacy.

Cyclical:
normalized earnings, cycle position, through-cycle margin, commodity or demand sensitivity.

Unknown / Data-Limited:
screening-only output, framework gap analysis, manual verification.

Unsupported or partially covered industries must produce a specific framework gap, not a generic "technology" or "company-specific framework" note. Biotech-like, REIT-like, insurance/financial, and semiconductor-turnaround cases each need explicit missing metrics and manual verification actions.
