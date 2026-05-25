use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EvalSet {
    pub name: String,
    pub tickers: Vec<String>,
    pub expected_family: BTreeMap<String, String>,
}

fn group_family(group: &str) -> String {
    match group {
        "mature_compounder" | "us_mature_compounder" => "Mature Compounder",
        "mega_cap_tech" | "us_mega_cap_tech" => "Platform Internet / Mega-cap Tech",
        "semiconductor_ai" | "us_ai_semiconductor" => {
            "AI Semiconductor / Data Center Growth Compounder"
        }
        "semiconductor_turnaround" | "us_semiconductor_turnaround" | "cn_semiconductor" => {
            "Capital-Intensive Semiconductor Turnaround"
        }
        "speculative_growth" | "us_speculative_unknown" => "Speculative Growth",
        "biotech_like" | "pharma" | "us_biotech" | "us_healthcare_pharma" | "cn_pharma_biotech" => {
            "Biotech / Pharma Research Frame"
        }
        "medical_devices" | "us_medical_devices" | "cn_medical_devices" => "Medical Devices",
        "banks"
        | "brokers_exchanges"
        | "us_financials"
        | "us_brokers_exchanges"
        | "cn_banks"
        | "cn_brokers_insurance" => "Financials / Bank-like Screening",
        "insurance" | "us_insurance" => "Insurance-like Screening",
        "reit" | "us_reit" => "REIT-like Screening",
        "energy"
        | "materials_mining"
        | "industrials"
        | "us_energy_materials"
        | "us_industrial_machinery"
        | "cn_coal_metals"
        | "cn_building_chemicals"
        | "cn_machinery_industrial" => "Cyclical / Industrial Cycle",
        "aerospace_defense" | "us_aerospace_defense" | "cn_defense_aerospace" => {
            "Aerospace / Defense"
        }
        "us_space_aerospace_smallcap" => "Speculative Aerospace / Space Systems",
        "airlines_transport"
        | "shipping_logistics"
        | "us_transport_shipping"
        | "us_transport_shipping_airlines" => "Shipping / Airlines / Transport Cycle",
        "consumer_retail" | "restaurants" | "luxury_apparel" | "us_consumer_retail"
        | "us_restaurants" | "cn_baijiu_consumer" => "Consumer / Retail",
        "utilities" | "us_telecom_utilities" | "cn_power_utilities" => "Utilities / Infrastructure",
        "telecom_media" | "us_telecom" | "cn_media_internet" => {
            "Telecom / Infrastructure Cash Flow"
        }
        "payments_fintech" => "Payments / Fintech",
        "cn_new_energy_auto" | "cn_solar_power_equipment" => "Cyclical / Industrial Cycle",
        _ => "Unknown / Data-Limited Screening",
    }
    .to_string()
}

fn ticker_family_override(ticker: &str) -> Option<&'static str> {
    match ticker {
        "AAPL" => Some("Mature Consumer Technology Compounder / Hardware + Services Ecosystem"),
        "ISRG" => Some("Medical Devices / Surgical Robotics"),
        "LLY" => Some("Large Pharma / Drug Portfolio / Regulatory and Patent Risk"),
        "RKLB" => Some("Space Launch / Space Systems / Speculative Aerospace"),
        "LUNR" => Some("Space / Lunar Infrastructure or Data-Limited Aerospace"),
        "JPM" => Some("Financials / Bank-like Screening"),
        "CAT" => Some("Cyclical / Industrial Machinery"),
        "ZIM" => Some("Shipping / Transport Cycle"),
        "600519.SH" => Some("A-share Consumer / Baijiu / Premium Liquor"),
        "000001.SZ" => Some("Financials / Bank-like Screening"),
        _ => None,
    }
}

pub fn is_valid_ticker_symbol(raw: &str) -> bool {
    let ticker = raw.trim();
    if ticker.len() == 9
        && ticker[..6].chars().all(|c| c.is_ascii_digit())
        && matches!(&ticker[6..], ".SH" | ".SZ")
    {
        return true;
    }

    if ticker.is_empty() || ticker.len() > 6 {
        return false;
    }
    ticker
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == '-' || c == '.')
        && ticker.chars().any(|c| c.is_ascii_uppercase())
}

pub fn load_eval_set(path: &Path) -> Result<EvalSet> {
    let raw = fs::read_to_string(path)?;
    let mut name = path.file_stem().unwrap().to_string_lossy().to_string();
    let mut tickers = Vec::new();
    let mut expected_family = BTreeMap::new();
    let mut current_group: Option<String> = None;
    let mut in_groups = false;
    for line in raw.lines() {
        let stripped = line.split('#').next().unwrap_or("").trim();
        if stripped.is_empty() {
            continue;
        }
        if stripped.starts_with("name:") {
            name = stripped.split_once(':').unwrap().1.trim().to_string();
        } else if stripped == "groups:" {
            in_groups = true;
            current_group = None;
        } else if !line.starts_with(" ") && stripped.ends_with(':') {
            in_groups = stripped == "groups:";
            current_group = None;
        } else if line.starts_with("  ") && !line.starts_with("    ") && stripped.ends_with(':') {
            current_group = if in_groups {
                Some(stripped.trim_end_matches(':').to_string())
            } else {
                None
            };
        } else if in_groups && stripped.starts_with("- ") {
            let ticker = stripped.trim_start_matches("- ").trim().to_uppercase();
            if !is_valid_ticker_symbol(&ticker) {
                continue;
            }
            if !tickers.contains(&ticker) {
                tickers.push(ticker.clone());
            }
            if let Some(group) = &current_group {
                let family = ticker_family_override(&ticker)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| group_family(group));
                expected_family.insert(ticker, family);
            }
        }
    }
    Ok(EvalSet {
        name,
        tickers,
        expected_family,
    })
}
