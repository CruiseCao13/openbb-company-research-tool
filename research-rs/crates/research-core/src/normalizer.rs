use crate::io::{write_if_changed, write_json};
use crate::run_folder::RunFolder;
use crate::types::{
    NormalizedFinancials, NormalizedPriceHistory, ProviderPayload, StatementRow, ValuationSnapshot,
    SCHEMA_VERSION,
};
use anyhow::Result;

fn clean_rows(rows: &[StatementRow]) -> Vec<StatementRow> {
    rows.iter()
        .filter(|row| {
            row.value.map(|value| value.is_finite()).unwrap_or(false)
                && !row.metric.trim().is_empty()
        })
        .cloned()
        .collect()
}

fn num_at(payload: &ProviderPayload, keys: &[&str]) -> Option<f64> {
    let object = payload.valuation_snapshot.as_object()?;
    for key in keys {
        if let Some(value) = object.get(*key).and_then(|v| v.as_f64()) {
            if value.is_finite() {
                return Some(value);
            }
        }
    }
    None
}

pub fn normalized_valuation(payload: &ProviderPayload) -> ValuationSnapshot {
    let raw_keys = payload
        .valuation_snapshot
        .as_object()
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default();
    ValuationSnapshot {
        pe: num_at(payload, &["trailingPE", "pe", "price_earnings"]),
        ps: num_at(
            payload,
            &["priceToSalesTrailing12Months", "ps", "price_sales"],
        ),
        pb: num_at(payload, &["priceToBook", "pb", "price_book"]),
        ev_revenue: num_at(payload, &["enterpriseToRevenue", "ev_revenue"]),
        ev_ebitda: num_at(payload, &["enterpriseToEbitda", "ev_ebitda"]),
        market_cap: num_at(payload, &["marketCap", "market_cap"]),
        raw_keys,
    }
}

pub fn write_normalized_outputs(folder: &RunFolder, payload: &ProviderPayload) -> Result<()> {
    let financials = NormalizedFinancials {
        schema_version: SCHEMA_VERSION.to_string(),
        ticker: payload.ticker.clone(),
        reporting_currency: payload.company_profile.currency.clone(),
        income_statement: clean_rows(&payload.income_statement),
        balance_sheet: clean_rows(&payload.balance_sheet),
        cash_flow: clean_rows(&payload.cash_flow),
        valuation_snapshot: normalized_valuation(payload),
    };
    let prices = NormalizedPriceHistory {
        schema_version: SCHEMA_VERSION.to_string(),
        ticker: payload.ticker.clone(),
        price_currency: payload.company_profile.currency.clone(),
        points: payload
            .price_history
            .iter()
            .filter(|point| point.close.map(|value| value.is_finite()).unwrap_or(false))
            .cloned()
            .collect(),
    };
    write_json(&folder.data.join("normalized_financials.json"), &financials)?;
    write_json(
        &folder.data.join("valuation_snapshot.json"),
        &financials.valuation_snapshot,
    )?;
    write_json(&folder.data.join("normalized_price_history.json"), &prices)?;
    write_if_changed(
        &folder.audit.join("normalizer_report.md"),
        &format!(
            "# Normalizer Report\n\nStatus: PASS\n\nProvider data was normalized before report rendering. Non-finite numeric values and empty metric rows are removed from normalized outputs; raw provider data remains locked in `raw/provider_payload.json`.\n\n| Output | Rows / Fields |\n|---|---:|\n| normalized income rows | {} |\n| normalized balance rows | {} |\n| normalized cash-flow rows | {} |\n| normalized price points | {} |\n| valuation raw keys | {} |\n\nRenderer and dashboard should use typed normalized data or typed AI artifacts, not ad hoc raw JSON field access.\n",
            financials.income_statement.len(),
            financials.balance_sheet.len(),
            financials.cash_flow.len(),
            prices.points.len(),
            financials.valuation_snapshot.raw_keys.len()
        ),
    )?;
    Ok(())
}
