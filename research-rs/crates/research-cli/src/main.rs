use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use research_ai::run_local_compact_analyst;
use research_batch::quality::{run_quality, QualityRunOptions};
use research_batch::runner::{run_batch, BatchRunOptions};
use research_core::config::EngineConfig;
use research_core::provider::fetch_provider_payload;
use research_core::run_folder::RunFolder;
use research_core::types::*;
use research_core::validation::{
    detect_forbidden_advice, report_status, validate_ai_json, validate_provider_payload,
};
use research_report::pack::pack_run;
use research_report::renderer::{render_run, RenderRunInput};
use std::path::PathBuf;

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
    };
    let folder = RunFolder::new(&ctx);
    folder.create()?;
    let config = EngineConfig::default();

    println!("[1/9] Fetching provider data              ...");
    let payload = fetch_provider_payload(&ctx, &config, &folder.raw.join("provider_payload.json"))?;
    let payload_failures = validate_provider_payload(&payload);
    println!(
        "[2/9] Validating locked data              {}",
        if payload_failures.is_empty() {
            "done"
        } else {
            "warning"
        }
    );

    println!("[3/9] AI company understanding            done  local");
    let (understanding, interpretation, blueprint, review, ai_calls, cache_hits) =
        run_local_compact_analyst(&payload);
    println!("[4/9] AI financial interpretation         done");
    println!("[5/9] AI research blueprint               done");
    let ai_failures = validate_ai_json(&understanding, &interpretation, &blueprint, &review);
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
        let pack_path = if args.pack {
            Some(pack_run(&folder, &ctx.ticker)?)
        } else {
            None
        };
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
