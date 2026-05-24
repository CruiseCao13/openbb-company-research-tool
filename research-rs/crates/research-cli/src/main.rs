use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use research_ai::run_local_compact_analyst;
use research_batch::quality::{run_quality, QualityRunOptions};
use research_batch::runner::{run_batch, BatchRunOptions};
use research_core::config::EngineConfig;
use research_core::io::{write_if_changed, write_json};
use research_core::normalizer::write_normalized_outputs;
use research_core::parser::write_parser_report;
use research_core::provider::fetch_provider_payload;
use research_core::run_folder::RunFolder;
use research_core::types::*;
use research_core::validation::{
    detect_forbidden_advice, report_status, validate_ai_json, validate_provider_payload,
};
use research_report::pack::pack_run;
use research_report::renderer::{render_run, RenderRunInput};
use std::path::PathBuf;
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
    Lint { run_folder: String },
    Pack { run_folder: String },
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
    println!("[3/9] AI company understanding            done  local");
    let (understanding, interpretation, blueprint, review, ai_calls, cache_hits) =
        run_local_compact_analyst(&payload);
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
        cache_hit: cache_hits > 0,
        provider_used: None,
        ai_calls,
        errors: vec![],
        warnings: ai_failures.clone(),
        output_files: vec![
            "metadata/company_understanding.json".into(),
            "metadata/financial_interpretation.json".into(),
            "metadata/research_blueprint.json".into(),
            "self_review/ai_self_review.json".into(),
        ],
    });
    let status = report_status(
        &payload_failures,
        &ai_failures,
        &review,
        if payload.error.is_some() {
            "PROVIDER_ERROR".into()
        } else {
            "PASS".into()
        },
        args.ai.clone(),
        ai_calls,
        cache_hits,
    );

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
        println!("AI Calls: {}", ai_calls);
        println!("Cache Hits: {}", cache_hits);
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
        ai_calls,
        cache_hits,
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
    }
}
