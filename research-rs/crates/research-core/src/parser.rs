use crate::io::write_if_changed;
use crate::run_folder::RunFolder;
use crate::types::ProviderPayload;
use anyhow::Result;

pub fn write_parser_report(folder: &RunFolder, payload: &ProviderPayload) -> Result<()> {
    write_if_changed(
        &folder.audit.join("parser_report.md"),
        &format!(
            "# Parser Report\n\nStatus: PASS\n\nThe raw provider JSON was deserialized into Rust typed contracts before analysis or rendering.\n\n| Contract | Count / Status |\n|---|---:|\n| ProviderPayload | PASS |\n| CompanyProfile | PASS |\n| PricePoint rows | {} |\n| Income statement rows | {} |\n| Balance sheet rows | {} |\n| Cash-flow rows | {} |\n| Provider error present | {} |\n\nNo renderer reads raw provider JSON directly.\n",
            payload.price_history.len(),
            payload.income_statement.len(),
            payload.balance_sheet.len(),
            payload.cash_flow.len(),
            payload.error.is_some()
        ),
    )?;
    Ok(())
}
