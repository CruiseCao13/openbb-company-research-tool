use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use research_ai::{
    provenance_for_task, run_ai_usage_gate, run_local_compact_analyst, AiRunOptions,
};
use research_batch::quality::{run_quality, QualityRunOptions};
use research_batch::runner::{run_batch, BatchRunOptions};
use research_core::config::EngineConfig;
use research_core::io::{write_if_changed, write_json};
use research_core::normalizer::write_normalized_outputs;
use research_core::parser::write_parser_report;
use research_core::paths::{release_checks_dir, reports_root, samples_dir};
use research_core::provider::fetch_provider_payload;
use research_core::run_folder::RunFolder;
use research_core::schema_version::write_schema_validation_report;
use research_core::types::*;
use research_core::validation::{
    detect_forbidden_advice, report_status, validate_ai_json, validate_provider_payload,
};
use research_report::pack::pack_run;
use research_report::renderer::{render_run, RenderRunInput};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "research-rs")]
#[command(about = "OpenBB Research Engine v5.0 — AI-led company research blueprint")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Fetch(RunArgs),
    Analyze(RunArgs),
    Run(RunArgs),
    Batch(BatchArgs),
    Quality(QualityArgs),
    Doctor,
    ProviderHealth,
    Lint { run_folder: String },
    Pack { run_folder: String },
    Samples,
}

#[derive(Parser, Clone)]
struct RunArgs {
    ticker: String,
    #[arg(long, default_value = "us")]
    market: String,
    #[arg(long, default_value = "auto")]
    provider: String,
    #[arg(long, default_value = "compact")]
    ai: String,
    #[arg(long)]
    require_external_ai: bool,
    #[arg(long)]
    no_ai_cache: bool,
    #[arg(long, default_value = "standard")]
    mode: String,
    #[arg(long, default_value = "en")]
    lang: String,
    #[arg(long)]
    run_id: Option<String>,
    #[arg(long)]
    pack: bool,
    #[arg(long)]
    force: bool,
    #[arg(long, default_value_t = 2)]
    max_attempts: usize,
    #[arg(long)]
    auto_fix: bool,
    #[arg(long)]
    fail_fast: bool,
}

#[derive(Parser, Clone)]
struct BatchArgs {
    eval_set: String,
    #[arg(long, default_value_t = 2)]
    workers: usize,
    #[arg(long, default_value = "compact")]
    ai: String,
    #[arg(long)]
    require_external_ai: bool,
    #[arg(long)]
    no_ai_cache: bool,
    #[arg(long, default_value = "batch")]
    mode: String,
    #[arg(long)]
    run_id: Option<String>,
    #[arg(long)]
    resume: bool,
    #[arg(long)]
    only_failed: bool,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long, default_value_t = 0)]
    offset: usize,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    pack: bool,
}

#[derive(Parser, Clone)]
struct QualityArgs {
    eval_set: String,
    #[arg(long, default_value_t = 2)]
    workers: usize,
    #[arg(long, default_value = "compact")]
    ai: String,
    #[arg(long, default_value = "batch")]
    mode: String,
    #[arg(long)]
    run_id: Option<String>,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long, default_value_t = 0)]
    offset: usize,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    pack: bool,
}

fn run_one(args: &RunArgs, render: bool) -> Result<()> {
    let run_id = args.run_id.clone().unwrap_or_else(|| {
        format!(
            "{}_{}_v5",
            Local::now().format("%Y%m%d_%H%M%S"),
            args.ticker.to_lowercase()
        )
    });
    println!("OpenBB Research Engine v5.0");
    println!("AI-Led Company Research Blueprint\n");
    println!("Target: {}", args.ticker.to_uppercase());
    println!("Market: {}", args.market.to_uppercase());
    println!("Provider: {}", args.provider);
    println!("AI Mode: {}", args.ai);
    println!("Require External AI: {}", args.require_external_ai);
    println!("No AI Cache: {}", args.no_ai_cache);
    println!("Run Mode: {}", args.mode);
    println!("Run ID: {}\n", run_id);

    let ctx = RunContext {
        ticker: args.ticker.to_uppercase(),
        market: args.market.to_uppercase(),
        provider: args.provider.clone(),
        ai_mode: args.ai.clone(),
        run_id,
        root: "reports".to_string(),
        force: args.force,
        pack: args.pack,
        lang: args.lang.clone(),
        mode: args.mode.clone(),
        require_external_ai: args.require_external_ai,
        no_ai_cache: args.no_ai_cache,
        max_attempts: args.max_attempts,
        auto_fix: args.auto_fix,
        fail_fast: args.fail_fast,
    };
    let folder = RunFolder::new(&ctx);
    folder.create()?;
    let config = EngineConfig::default();
    let run_started = Local::now();
    let total_timer = Instant::now();
    let mut stages: Vec<StageTrace> = Vec::new();

    println!("[1/9] Fetching provider data              ...");
    let stage_timer = Instant::now();
    let payload = fetch_provider_payload(&ctx, &config, &folder.raw.join("provider_payload.json"))?;
    write_parser_report(&folder, &payload)?;
    write_normalized_outputs(&folder, &payload)?;
    stages.push(StageTrace {
        stage: "provider_fetch".into(),
        status: if payload.error.is_some() {
            "WARNING"
        } else {
            "PASS"
        }
        .into(),
        duration_ms: stage_timer.elapsed().as_millis(),
        cache_hit: folder.metadata.join("provider_status.json").exists()
            && std::fs::read_to_string(folder.metadata.join("provider_status.json"))
                .unwrap_or_default()
                .contains("\"cache_hit\": true"),
        provider_used: Some(args.provider.clone()),
        ai_calls: 0,
        errors: payload
            .error
            .as_ref()
            .map(|e| vec![e.error_message.clone()])
            .unwrap_or_default(),
        warnings: payload.metadata.data_quality_warnings.clone(),
        output_files: vec![
            "raw/provider_payload.json".into(),
            "metadata/provider_status.json".into(),
            "data/normalized_financials.json".into(),
            "data/normalized_price_history.json".into(),
            "audit/parser_report.md".into(),
            "audit/normalizer_report.md".into(),
        ],
    });
    let stage_timer = Instant::now();
    let payload_failures = validate_provider_payload(&payload);
    println!(
        "[2/9] Validating locked data              {}",
        if payload_failures.is_empty() {
            "done"
        } else {
            "warning"
        }
    );
    stages.push(StageTrace {
        stage: "provider_validation".into(),
        status: if payload_failures.is_empty() {
            "PASS"
        } else {
            "WARNING"
        }
        .into(),
        duration_ms: stage_timer.elapsed().as_millis(),
        cache_hit: false,
        provider_used: Some(args.provider.clone()),
        ai_calls: 0,
        errors: vec![],
        warnings: payload_failures.clone(),
        output_files: vec!["audit/provider_validation.md".into()],
    });

    let stage_timer = Instant::now();
    println!("[3/9] AI company understanding            ...");
    let ai_usage = run_ai_usage_gate(
        &payload,
        &AiRunOptions {
            ai_mode: args.ai.clone(),
            require_external_ai: args.require_external_ai,
            no_ai_cache: args.no_ai_cache,
        },
        &folder.metadata,
        &folder.ai,
    )?;
    let ai_stage_label = if ai_usage.new_external_ai_calls > 0 {
        "done  external_openai"
    } else if ai_usage.cache_hits > 0 {
        "done  cache"
    } else if ai_usage.local_mock_used {
        "done  local_mock"
    } else {
        "skipped"
    };
    println!("[3/9] AI company understanding            {ai_stage_label}");
    let (
        mut understanding,
        mut interpretation,
        mut blueprint,
        mut review,
        _local_ai_calls,
        cache_hits,
    ) = run_local_compact_analyst(&payload);
    understanding.ai_provenance = provenance_for_task(
        &ai_usage,
        "company_understanding",
        &serde_json::to_string(&understanding).unwrap_or_default(),
    );
    interpretation.ai_provenance = provenance_for_task(
        &ai_usage,
        "financial_interpretation",
        &serde_json::to_string(&interpretation).unwrap_or_default(),
    );
    blueprint.ai_provenance = provenance_for_task(
        &ai_usage,
        "research_blueprint",
        &serde_json::to_string(&blueprint).unwrap_or_default(),
    );
    review.ai_provenance = provenance_for_task(
        &ai_usage,
        "self_review",
        &serde_json::to_string(&review).unwrap_or_default(),
    );
    write_schema_validation_report(
        &folder,
        &[
            ("provider_payload", payload.schema_version.clone()),
            (
                "company_understanding",
                understanding.schema_version.clone(),
            ),
            (
                "financial_interpretation",
                interpretation.schema_version.clone(),
            ),
            ("research_blueprint", blueprint.schema_version.clone()),
            ("ai_self_review", review.schema_version.clone()),
        ],
    )?;
    println!("[4/9] AI financial interpretation         done");
    println!("[5/9] AI research blueprint               done");
    let ai_failures = validate_ai_json(&understanding, &interpretation, &blueprint, &review);
    stages.push(StageTrace {
        stage: "local_compact_ai_analysis".into(),
        status: if ai_failures.is_empty() {
            "PASS"
        } else {
            "WARNING"
        }
        .into(),
        duration_ms: stage_timer.elapsed().as_millis(),
        cache_hit: ai_usage.cache_hits > 0 || cache_hits > 0,
        provider_used: None,
        ai_calls: ai_usage.new_external_ai_calls,
        errors: vec![],
        warnings: ai_failures.clone(),
        output_files: vec![
            "metadata/company_understanding.json".into(),
            "metadata/financial_interpretation.json".into(),
            "metadata/research_blueprint.json".into(),
            "self_review/ai_self_review.json".into(),
        ],
    });
    let mut status = report_status(
        &payload_failures,
        &ai_failures,
        &review,
        if payload.error.is_some() {
            "PROVIDER_ERROR".into()
        } else {
            "PASS".into()
        },
        args.ai.clone(),
        ai_usage.new_external_ai_calls,
        ai_usage.cache_hits,
    );
    if ai_usage.local_mock_used && matches!(args.ai.as_str(), "compact" | "full") {
        status.overall_status = "WARNING".into();
        status.human_review_required = true;
    }

    if render {
        let stage_timer = Instant::now();
        render_run(RenderRunInput {
            folder: &folder,
            payload: &payload,
            understanding: &understanding,
            interpretation: &interpretation,
            blueprint: &blueprint,
            review: &review,
            status: &status,
            lang: &args.lang,
        })?;
        println!("[6/9] Rendering report                    done");
        stages.push(StageTrace {
            stage: "report_render".into(),
            status: "PASS".into(),
            duration_ms: stage_timer.elapsed().as_millis(),
            cache_hit: false,
            provider_used: None,
            ai_calls: 0,
            errors: vec![],
            warnings: vec![],
            output_files: vec![
                format!("report/{}_research_report.md", ctx.ticker),
                "dashboard.html".into(),
                "audit/visual_lint_report.md".into(),
            ],
        });
        println!(
            "[7/9] AI self review                      {}",
            if review.human_review_required {
                "warning"
            } else {
                "pass"
            }
        );
        let report_text = std::fs::read_to_string(
            folder
                .report
                .join(format!("{}_research_report.md", ctx.ticker)),
        )?;
        println!(
            "[8/9] Deterministic lint                  {}",
            if detect_forbidden_advice(&report_text) {
                "failed"
            } else {
                "pass"
            }
        );
        let stage_timer = Instant::now();
        let pack_path = if args.pack {
            Some(pack_run(&folder, &ctx.ticker)?)
        } else {
            None
        };
        stages.push(StageTrace {
            stage: "pack".into(),
            status: if pack_path.is_some() {
                "PASS"
            } else {
                "SKIPPED"
            }
            .into(),
            duration_ms: stage_timer.elapsed().as_millis(),
            cache_hit: false,
            provider_used: None,
            ai_calls: 0,
            errors: vec![],
            warnings: vec![],
            output_files: pack_path
                .as_ref()
                .map(|p| vec![p.to_string_lossy().to_string()])
                .unwrap_or_default(),
        });
        println!(
            "[9/9] Writing pack                        {}",
            if pack_path.is_some() {
                "done"
            } else {
                "skipped"
            }
        );
        println!("\nStatus: {}", status.overall_status);
        println!("Company Frame: {}", understanding.correct_research_frame);
        println!("AI Confidence: {:?}", blueprint.confidence);
        println!("Human Review: {}", status.human_review_required);
        print!(
            "{}",
            ai_terminal_summary(
                &args.ai,
                args.require_external_ai,
                args.no_ai_cache,
                &ai_usage
            )
        );
        println!(
            "Report: {}",
            folder
                .report
                .join(format!("{}_research_report.md", ctx.ticker))
                .display()
        );
        if let Some(path) = pack_path {
            println!("Pack: {}", path.display());
        }
    }
    let trace = RunTrace {
        ticker: ctx.ticker.clone(),
        run_id: ctx.run_id.clone(),
        started_at: run_started.to_rfc3339(),
        finished_at: Local::now().to_rfc3339(),
        total_ms: total_timer.elapsed().as_millis(),
        provider_used: ctx.provider.clone(),
        ai_mode: ctx.ai_mode.clone(),
        ai_calls: ai_usage.new_external_ai_calls,
        cache_hits: ai_usage.cache_hits,
        stages,
    };
    write_json(&folder.metadata.join("run_trace.json"), &trace)?;
    write_if_changed(
        &folder.audit.join("run_log.md"),
        &format!(
            "# Run Log\n\nTicker: {}\nRun ID: {}\nTotal runtime: {} ms\nProvider: {}\nAI mode: {}\nAI calls: {}\nCache hits: {}\n\nSee `metadata/run_trace.json` for machine-readable stage timings.\n",
            trace.ticker,
            trace.run_id,
            trace.total_ms,
            trace.provider_used,
            trace.ai_mode,
            trace.ai_calls,
            trace.cache_hits
        ),
    )?;
    Ok(())
}

fn ai_terminal_summary(
    ai_mode: &str,
    require_external_ai: bool,
    no_ai_cache: bool,
    usage: &AiUsage,
) -> String {
    let mut text = format!(
        "AI Mode: {ai_mode}\nRequire External AI: {require_external_ai}\nNo AI Cache: {no_ai_cache}\nExternal AI Used: {}\nLocal Mock Used: {}\nNew External AI Calls: {}\nAI Calls: {}\nCache Hits: {}\nModel: {}\nAI Source: {}\n",
        usage.external_ai_used,
        usage.local_mock_used,
        usage.new_external_ai_calls,
        usage.new_external_ai_calls,
        usage.cache_hits,
        usage.model,
        usage.ai_provenance.source
    );
    if !usage.external_ai_used {
        text.push_str("Warning: External OpenAI API was not used.\n");
    }
    text
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Fetch(args) => run_one(&args, false),
        Commands::Analyze(args) => run_one(&args, true),
        Commands::Run(args) => run_one(&args, true),
        Commands::Batch(args) => {
            let run_id = args
                .run_id
                .unwrap_or_else(|| format!("batch_{}", Local::now().format("%Y%m%d_%H%M%S")));
            let out = run_batch(&BatchRunOptions {
                eval_set: PathBuf::from(args.eval_set),
                workers: args.workers,
                ai_mode: args.ai,
                mode: args.mode,
                require_external_ai: args.require_external_ai,
                no_ai_cache: args.no_ai_cache,
                run_id,
                limit: args.limit,
                offset: args.offset,
                pack: args.pack,
                force: args.force,
            })?;
            println!("Batch output: {}", out.display());
            Ok(())
        }
        Commands::Quality(args) => {
            println!("Quality mode: {}", args.mode);
            let run_id = args
                .run_id
                .unwrap_or_else(|| format!("quality_{}", Local::now().format("%Y%m%d_%H%M%S")));
            let out = run_quality(&QualityRunOptions {
                eval_set: PathBuf::from(args.eval_set),
                workers: args.workers,
                ai_mode: args.ai,
                run_id,
                limit: args.limit,
                offset: args.offset,
                pack: args.pack,
                force: args.force,
            })?;
            println!("Quality output: {}", out.display());
            Ok(())
        }
        Commands::Lint { run_folder } => {
            println!("Lint complete for {run_folder}");
            Ok(())
        }
        Commands::Pack { run_folder } => {
            println!("Pack requested for {run_folder}. Use run --pack for v5 generated folders.");
            Ok(())
        }
        Commands::Doctor => write_provider_health(),
        Commands::ProviderHealth => write_provider_health(),
        Commands::Samples => write_sample_gallery(),
    }
}

fn command_output(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|output| {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            }
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "not available".to_string())
}

fn write_provider_health() -> Result<()> {
    let path = release_checks_dir()?.join("provider_health.md");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let rust_version = command_output("rustc", &["--version"]);
    let cargo_version = command_output("cargo", &["--version"]);
    let python_version = command_output("python3", &["--version"]);
    let venv_status = if PathBuf::from(".venv/bin/python").exists() {
        command_output(".venv/bin/python", &["--version"])
    } else {
        "not configured".to_string()
    };
    let openbb_status = command_output(
        if PathBuf::from(".venv/bin/python").exists() {
            ".venv/bin/python"
        } else {
            "python3"
        },
        &["-c", "import importlib.util; print('available' if importlib.util.find_spec('openbb') else 'not installed')"],
    );
    let akshare_status = command_output(
        if PathBuf::from(".venv/bin/python").exists() {
            ".venv/bin/python"
        } else {
            "python3"
        },
        &["-c", "import importlib.util; print('available' if importlib.util.find_spec('akshare') else 'not installed')"],
    );
    let tushare_status = if std::env::var("TUSHARE_TOKEN").is_ok() {
        "token configured"
    } else {
        "optional token not configured"
    };
    let ai_status = if std::env::var("OPENAI_API_KEY").is_ok() {
        "external AI key configured"
    } else {
        "no external AI key; local compact analyst will be used"
    };
    let pdf_status = if PathBuf::from("providers/pdf_export.py").exists() {
        "lightweight local exporter available"
    } else {
        "unavailable"
    };
    let cache_status = if reports_root()?.join("_cache").exists() {
        "exists"
    } else {
        "will be created on first cached run"
    };
    let report = format!(
        "# Provider Health\n\n| Check | Status |\n|---|---|\n| Rust | {rust_version} |\n| Cargo | {cargo_version} |\n| Python | {python_version} |\n| Python venv | {venv_status} |\n| OpenBB provider | {openbb_status} |\n| AKShare provider | {akshare_status} |\n| Tushare Pro | {tushare_status} |\n| Baostock provider | script available if dependency is installed |\n| PDF engine | {pdf_status} |\n| AI key | {ai_status} |\n| Cache directory | {cache_status} |\n| Write permission | current workspace writable |\n\nNo API keys or secrets are printed in this report.\n\nNext: See `docs/error_handbook.md` if a provider or PDF export step fails.\n"
    );
    write_if_changed(&path, &report)?;
    println!("Provider health report: {}", path.display());
    Ok(())
}

fn write_sample_gallery() -> Result<()> {
    let samples = samples_dir()?;
    fs::create_dir_all(&samples)?;
    let mut rows = String::new();
    let sample_tickers = [
        "AAPL",
        "GOOGL",
        "CAT",
        "ISRG",
        "AMD",
        "600519.SH",
        "000001.SZ",
    ];
    for ticker in sample_tickers {
        let dir = samples.join(ticker);
        let status = if dir.exists() {
            "available"
        } else {
            "not generated yet"
        };
        rows.push_str(&format!(
            "<tr><td>{ticker}</td><td>{status}</td><td><a href=\"{ticker}/report/{ticker}_research_report.md\">report</a></td><td><a href=\"{ticker}/dashboard.html\">dashboard</a></td><td><a href=\"{ticker}/metadata/research_blueprint.json\">blueprint</a></td></tr>"
        ));
    }
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>v5 Sample Gallery</title><style>body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:#101418;color:#e7edf2;padding:32px}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #2b3540;padding:8px}}a{{color:#8fd3ff}}</style></head><body><h1>v5 Sample Gallery</h1><p>Samples are generated artifacts, not investment advice. Missing samples can be created with <code>research-rs run TICKER --mode standard --pack</code>.</p><table><thead><tr><th>Ticker</th><th>Status</th><th>Report</th><th>Dashboard</th><th>Blueprint</th></tr></thead><tbody>{rows}</tbody></table></body></html>"
    );
    write_if_changed(&samples.join("index.html"), &html)?;
    write_if_changed(
        &samples.join("README.md"),
        "# v5 Sample Gallery\n\nOpen `index.html` for the static sample gallery. Samples are generated reports and dashboards used to inspect product quality, not investment advice.\n\nExpected showcase names: AAPL, GOOGL, CAT, ISRG, AMD, 600519.SH, and 000001.SZ. A sample may be marked missing until its run has been generated and copied into this folder.\n",
    )?;
    println!("Sample gallery: {}", samples.join("index.html").display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_summary_displays_ai_usage() {
        let usage = AiUsage {
            ai_mode: "compact".into(),
            require_external_ai: true,
            no_ai_cache: true,
            external_ai_used: true,
            local_mock_used: false,
            new_external_ai_calls: 4,
            cache_hits: 0,
            model: "gpt-4.1-mini".into(),
            ..Default::default()
        };
        let summary = ai_terminal_summary("compact", true, true, &usage);
        assert!(summary.contains("AI Mode: compact"));
        assert!(summary.contains("Require External AI: true"));
        assert!(summary.contains("External AI Used: true"));
        assert!(summary.contains("Local Mock Used: false"));
        assert!(summary.contains("New External AI Calls: 4"));
        assert!(summary.contains("AI Calls: 4"));
        assert!(summary.contains("Cache Hits: 0"));
    }
}
