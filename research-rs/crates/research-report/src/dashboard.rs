use research_core::types::*;

pub fn render_company_dashboard(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    status: &ReportStatus,
) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{ticker} Research Dashboard</title>
<style>
body {{ margin: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: #101418; color: #e7edf2; }}
main {{ max-width: 1120px; margin: 0 auto; padding: 32px; }}
.grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; }}
.card {{ border: 1px solid #2b3540; background: #161d24; border-radius: 8px; padding: 16px; }}
.hero {{ border: 1px solid #334155; background: linear-gradient(135deg,#111827,#172033); border-radius: 10px; padding: 22px; margin-bottom: 16px; }}
.badge {{ display:inline-block; border:1px solid #475569; border-radius:999px; padding:4px 9px; margin-right:6px; color:#dbeafe; font-size:12px; }}
.chart-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 14px; }}
.chart-card img {{ max-width: 100%; border: 1px solid #2b3540; border-radius: 6px; background: #fff; }}
.label {{ color: #93a4b5; font-size: 12px; text-transform: uppercase; }}
h1, h2 {{ margin-bottom: 8px; }}
a {{ color: #8fd3ff; }}
@media (min-width: 900px) {{ .chart-grid {{ grid-template-columns: repeat(2, minmax(0, 1fr)); }} }}
</style>
</head>
<body><main>
<section class="hero">
<h1>{ticker} Research Dashboard</h1>
<p>{identity}</p>
<span class="badge"># DATA: {provider}</span><span class="badge">◆ AI: compact/local</span><span class="badge">▣ REPORT: {status_value}</span>
</section>
<div class="grid">
<div class="card"><div class="label">Status</div><strong>{status_value}</strong></div>
<div class="card"><div class="label">Frame</div><strong>{frame}</strong></div>
<div class="card"><div class="label">AI Confidence</div><strong>{confidence:?}</strong></div>
<div class="card"><div class="label">Human Review</div><strong>{human_review}</strong></div>
</div>
<div class="grid">
<div class="card"><h2>Company Identity</h2><p>{identity}</p></div>
<div class="card"><h2>Business Model</h2><p>{business}</p></div>
<div class="card"><h2>Money Flow</h2><p><strong>Comes from:</strong> {money_from}</p><p><strong>Goes to:</strong> {money_goes}</p></div>
<div class="card"><h2>Financial Interpretation</h2><p>{cash_flow}</p></div>
<div class="card"><h2>Research Blueprint</h2><p>{thesis}</p></div>
<div class="card"><h2>Valuation Frame</h2><p>{valuation}</p></div>
<div class="card"><h2>Red Flags</h2><p>{red_flags}</p></div>
<div class="card"><h2>Data Gaps</h2><p>{data_gaps}</p></div>
<div class="card"><h2>AI Self Review</h2><p>Framework fit and unsupported claims are reviewed in <a href="self_review/ai_self_review.md">AI self review</a>.</p></div>
</div>
<h2>Chart Grid</h2>
<div class="chart-grid">
<div class="card chart-card"><h3>Figure 1. Price / Benchmark Performance</h3><img src="charts/Figure_01_price_vs_benchmark.png" alt="Figure 1 price chart"><p><strong>Why it matters:</strong> frames price path and opportunity cost. Source: provider_payload.json</p></div>
<div class="card chart-card"><h3>Figure 2. Drawdown / Risk Path</h3><img src="charts/Figure_02_drawdown.png" alt="Figure 2 drawdown chart"><p><strong>Why it matters:</strong> shows the pain path behind returns. Source: provider_payload.json</p></div>
<div class="card chart-card"><h3>Figure 3. Financial Trend</h3><img src="charts/Figure_03_financial_trend.png" alt="Figure 3 financial trend"><p><strong>Why it matters:</strong> links business progress to reported financials. Source: provider_payload.json</p></div>
<div class="card chart-card"><h3>Figure 4. Money Flow</h3><img src="charts/Figure_04_money_flow.png" alt="Figure 4 money flow"><p><strong>Why it matters:</strong> shows whether growth is self-funded or cash-consuming. Source: provider_payload.json</p></div>
<div class="card chart-card"><h3>Figure 5. Valuation Frame</h3><img src="charts/Figure_05_valuation_frame.png" alt="Figure 5 valuation frame"><p><strong>Why it matters:</strong> frames available valuation evidence without implying a target price. Source: provider_payload.json</p></div>
</div>
<h2>Evidence Strip</h2>
<div class="grid">
<div class="card"><div class="label">Key Claim</div><p>{thesis}</p></div>
<div class="card"><div class="label">Confidence</div><p>{confidence:?}</p></div>
<div class="card"><div class="label">Source Links</div><p><a href="metadata/evidence_map.json">evidence_map.json</a> · <a href="metadata/data_inventory.json">data_inventory.json</a> · <a href="metadata/data_usage_coverage.json">data_usage_coverage.json</a> · <a href="metadata/chart_plan.json">chart_plan.json</a> · <a href="metadata/chart_table_quality.json">chart_table_quality.json</a></p></div>
</div>
<h2>Files</h2>
<ul>
<li><a href="report/{ticker}_research_report.md">Markdown report</a></li>
<li><a href="report/{ticker}_research_report.pdf">PDF report</a></li>
<li><a href="raw/provider_payload.json">Locked provider payload</a></li>
<li><a href="metadata/research_blueprint.json">Research blueprint JSON</a></li>
<li><a href="metadata/evidence_map.json">Evidence map</a></li>
<li><a href="self_review/ai_self_review.md">AI self review</a></li>
<li><a href="audit/validator_report.md">Validator report</a></li>
<li><a href="audit/data_usage_coverage_report.md">Data usage coverage report</a></li>
<li><a href="audit/chart_table_quality_report.md">Chart/table quality report</a></li>
<li><a href="audit/pdf_export_report.md">PDF export report</a></li>
</ul>
</main></body></html>
"#,
        ticker = payload.ticker,
        identity = understanding.company_identity,
        status_value = status.overall_status,
        provider = payload.provider,
        frame = understanding.correct_research_frame,
        confidence = blueprint.confidence,
        human_review = status.human_review_required,
        business = understanding.business_model,
        money_from = interpretation.where_money_comes_from,
        money_goes = interpretation.where_money_goes,
        cash_flow = interpretation.cash_flow_explanation,
        valuation = blueprint.valuation_frame,
        red_flags = blueprint.red_flags.join("; "),
        data_gaps = if blueprint.data_gaps.is_empty() {
            "No major data gaps flagged by current blueprint.".to_string()
        } else {
            blueprint.data_gaps.join("; ")
        },
        thesis = blueprint.core_thesis,
    )
}

pub fn render_batch_dashboard(title: &str, rows: &[(String, String, String)]) -> String {
    let mut html_rows = String::new();
    for (ticker, status, frame) in rows {
        html_rows.push_str(&format!(
            "<tr><td>{ticker}</td><td>{status}</td><td>{frame}</td></tr>"
        ));
    }
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>{title}</title>
<style>body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:#101418;color:#e7edf2;padding:32px}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #2b3540;padding:8px}}th{{background:#18222c}}</style>
</head><body><h1>{title}</h1><table><thead><tr><th>Ticker</th><th>Status</th><th>Frame</th></tr></thead><tbody>{html_rows}</tbody></table></body></html>"#
    )
}
