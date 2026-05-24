use anyhow::{anyhow, Result};
use research_core::cache::digest_str;
use research_core::io::{ensure_dir, write_if_changed, write_json};
use research_core::types::{AiProvenance, AiTaskUsage, AiUsage, ProviderPayload, SCHEMA_VERSION};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AiRunOptions {
    pub ai_mode: String,
    pub require_external_ai: bool,
    pub no_ai_cache: bool,
}

const TASKS: [(&str, &str); 4] = [
    ("company_understanding", "company_understanding_v1"),
    ("financial_interpretation", "financial_interpretation_v1"),
    ("research_blueprint", "research_blueprint_v1"),
    ("self_review", "self_review_v1"),
];
const MAX_CALLS_PER_TICKER: usize = 6;

pub fn external_ai_available() -> bool {
    std::env::var("OPENAI_API_KEY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub fn compact_payload_size(payload: &ProviderPayload) -> usize {
    payload.company_profile.description.len()
        + payload.income_statement.len() * 32
        + payload.cash_flow.len() * 32
        + payload.balance_sheet.len() * 32
}

fn model_name() -> String {
    std::env::var("OPENAI_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "gpt-4.1-mini".to_string())
}

fn base_url() -> String {
    std::env::var("OPENAI_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
}

fn timeout_seconds() -> u64 {
    std::env::var("OPENAI_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60)
}

fn compact_prompt(payload: &ProviderPayload, task: &str, prompt_version: &str) -> String {
    let latest_income = payload
        .income_statement
        .iter()
        .take(12)
        .map(|row| format!("{}:{}={:?}", row.period, row.metric, row.value))
        .collect::<Vec<_>>()
        .join("; ");
    let latest_cash = payload
        .cash_flow
        .iter()
        .take(12)
        .map(|row| format!("{}:{}={:?}", row.period, row.metric, row.value))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "prompt_version: {prompt_version}\ntask: {task}\nReturn compact JSON only. Do not provide investment advice, target prices, or unsupported facts.\nTicker: {ticker}\nMarket: {market}\nCompany: {name}\nSector: {sector}\nIndustry: {industry}\nDescription: {description}\nIncome summary: {latest_income}\nCash-flow summary: {latest_cash}\n",
        ticker = payload.ticker,
        market = payload.market,
        name = payload.company_profile.name,
        sector = payload.company_profile.sector,
        industry = payload.company_profile.industry,
        description = payload.company_profile.description
    )
}

fn cache_path(cache_key: &str) -> PathBuf {
    PathBuf::from("reports")
        .join("_cache")
        .join("ai")
        .join(format!("{cache_key}.json"))
}

fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

fn source_for(task: &AiTaskUsage) -> String {
    if task.external_ai_used && task.cache_hit {
        "cache".into()
    } else if task.external_ai_used {
        "external_openai".into()
    } else if task.local_mock_used {
        "local_mock".into()
    } else {
        "skipped".into()
    }
}

struct AggregateProvenanceInput<'a> {
    ai_mode: &'a str,
    source: &'a str,
    external_ai_used: bool,
    local_mock_used: bool,
    cache_hit: bool,
    new_external_ai_call: bool,
    request_attempted: bool,
    request_success: bool,
    model: &'a str,
    output_seed: &'a str,
}

fn aggregate_provenance(input: AggregateProvenanceInput<'_>) -> AiProvenance {
    AiProvenance {
        source: input.source.into(),
        external_ai_used: input.external_ai_used,
        local_mock_used: input.local_mock_used,
        cache_hit: input.cache_hit,
        new_external_ai_call: input.new_external_ai_call,
        model: input.model.into(),
        prompt_version: "aggregate_ai_usage_v1".into(),
        request_attempted: input.request_attempted,
        request_success: input.request_success,
        request_id: String::new(),
        generated_at: chrono::Local::now().to_rfc3339(),
        input_digest: digest_str(&format!(
            "{}:{}:{}",
            input.ai_mode, input.model, input.source
        )),
        output_digest: digest_str(input.output_seed),
    }
}

pub fn provenance_for_task(usage: &AiUsage, task_name: &str, output_text: &str) -> AiProvenance {
    let task = usage
        .tasks
        .iter()
        .find(|task| task.task == task_name)
        .cloned()
        .unwrap_or_default();
    AiProvenance {
        source: if task.source.is_empty() {
            source_for(&task)
        } else {
            task.source.clone()
        },
        external_ai_used: task.external_ai_used,
        local_mock_used: task.local_mock_used,
        cache_hit: task.cache_hit,
        new_external_ai_call: task.new_external_ai_call,
        model: if task.model.is_empty() {
            usage.model.clone()
        } else {
            task.model.clone()
        },
        prompt_version: task.prompt_version.clone(),
        request_attempted: task.request_attempted,
        request_success: task.request_success,
        request_id: task.request_id.clone(),
        generated_at: chrono::Local::now().to_rfc3339(),
        input_digest: digest_str(&format!("{}:{}:{}", usage.ai_mode, usage.model, task_name)),
        output_digest: digest_str(output_text),
    }
}

fn call_openai(task: &str, prompt: &str, model: &str) -> Result<(String, String)> {
    if std::env::var("OPENAI_MOCK_SUCCESS").ok().as_deref() == Some("1") {
        return Ok((
            format!("mock-request-{task}"),
            json!({"task": task, "status": "mock_external_success"}).to_string(),
        ));
    }
    let key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY missing; cannot call external OpenAI API"))?;
    if key.trim().is_empty() {
        return Err(anyhow!(
            "OPENAI_API_KEY missing; cannot call external OpenAI API"
        ));
    }
    let url = format!("{}/chat/completions", base_url().trim_end_matches('/'));
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(timeout_seconds()))
        .build();
    let body = json!({
        "model": model,
        "temperature": 0.1,
        "response_format": {"type": "json_object"},
        "messages": [
            {"role": "system", "content": "You are a bounded company research analyst. Return valid JSON only. Never reveal secrets. Never give buy/sell advice or target prices."},
            {"role": "user", "content": prompt}
        ]
    });
    let response = agent
        .post(&url)
        .set("Authorization", &format!("Bearer {key}"))
        .set("Content-Type", "application/json")
        .send_json(body)?;
    let request_id = response.header("x-request-id").unwrap_or("").to_string();
    let value: serde_json::Value = response.into_json()?;
    let content = value
        .pointer("/choices/0/message/content")
        .and_then(|value| value.as_str())
        .unwrap_or("{}")
        .to_string();
    Ok((request_id, content))
}

fn local_usage(task: &str, prompt_version: &str, model: &str, ai_mode: &str) -> AiTaskUsage {
    AiTaskUsage {
        task: task.to_string(),
        source: if ai_mode == "off" {
            "skipped"
        } else {
            "local_mock"
        }
        .into(),
        prompt_version: prompt_version.to_string(),
        external_ai_used: false,
        external_ai_attempted: false,
        external_ai_used_from_cache: false,
        local_mock_used: ai_mode != "off",
        cache_hit: false,
        new_external_ai_call: false,
        request_attempted: false,
        request_success: false,
        request_id: String::new(),
        model: model.to_string(),
        input_tokens_estimate: 0,
        output_tokens_estimate: 0,
        error: if ai_mode == "off" {
            "AI skipped by --ai off".into()
        } else {
            "External OpenAI API not used; local compact fallback used.".into()
        },
    }
}

pub fn run_ai_usage_gate(
    payload: &ProviderPayload,
    options: &AiRunOptions,
    metadata_dir: &Path,
    ai_dir: &Path,
) -> Result<AiUsage> {
    let ai_mode = options.ai_mode.to_lowercase();
    if !matches!(ai_mode.as_str(), "off" | "local" | "compact" | "full") {
        return Err(anyhow!("invalid --ai mode: {}", options.ai_mode));
    }
    let key_available = external_ai_available();
    let external_candidate = matches!(ai_mode.as_str(), "compact" | "full") && key_available;
    let model = if ai_mode == "local" || ai_mode == "off" || !external_candidate {
        "local-compact-analyst-fallback".to_string()
    } else {
        model_name()
    };
    if options.require_external_ai && !key_available {
        let usage = AiUsage {
            schema_version: SCHEMA_VERSION.to_string(),
            ai_provenance: aggregate_provenance(AggregateProvenanceInput {
                ai_mode: &ai_mode,
                source: "skipped",
                external_ai_used: false,
                local_mock_used: false,
                cache_hit: false,
                new_external_ai_call: false,
                request_attempted: false,
                request_success: false,
                model: &model,
                output_seed: "OPENAI_API_KEY missing",
            }),
            ai_mode: ai_mode.clone(),
            require_external_ai: true,
            no_ai_cache: options.no_ai_cache,
            external_ai_used: false,
            external_ai_attempted: false,
            external_ai_used_from_cache: false,
            local_mock_used: false,
            cache_used: false,
            ai_calls: 0,
            new_external_ai_calls: 0,
            cache_hits: 0,
            model: model.clone(),
            provider: "openai".into(),
            tasks: TASKS
                .iter()
                .map(|(task, version)| AiTaskUsage {
                    error: "OPENAI_API_KEY missing".into(),
                    model: model.clone(),
                    task: (*task).into(),
                    prompt_version: (*version).into(),
                    source: "skipped".into(),
                    ..Default::default()
                })
                .collect(),
        };
        write_json(&metadata_dir.join("ai_usage.json"), &usage)?;
        return Err(anyhow!("OPENAI_API_KEY missing; --require-external-ai forbids local fallback. See docs/error_handbook.md#ai-json-invalid"));
    }
    if options.require_external_ai && options.no_ai_cache && TASKS.len() > MAX_CALLS_PER_TICKER {
        return Err(anyhow!(
            "AI budget exceeded before request: {} planned tasks exceeds max_calls_per_ticker={MAX_CALLS_PER_TICKER}",
            TASKS.len()
        ));
    }

    let mut tasks = Vec::new();
    let mut new_external_ai_calls = 0usize;
    let mut cache_hits = 0usize;
    let mut external_ai_attempted = false;
    let mut external_ai_used = false;
    let mut external_ai_used_from_cache = false;
    let mut local_mock_used = false;

    ensure_dir(&ai_dir.join("responses"))?;
    ensure_dir(&ai_dir.join("prompts"))?;

    for (task, prompt_version) in TASKS {
        let prompt = compact_prompt(payload, task, prompt_version);
        write_if_changed(
            &ai_dir
                .join("prompts")
                .join(format!("{task}_{prompt_version}.md")),
            &prompt,
        )?;
        if ai_mode == "off" || ai_mode == "local" || !key_available {
            local_mock_used |= ai_mode != "off";
            tasks.push(local_usage(task, prompt_version, &model, &ai_mode));
            continue;
        }
        let cache_key = digest_str(&format!(
            "{}:{}:{}:{}:{}",
            payload.ticker,
            digest_str(&serde_json::to_string(payload).unwrap_or_default()),
            task,
            prompt_version,
            model
        ));
        let cache_path = cache_path(&cache_key);
        if !options.no_ai_cache && cache_path.exists() {
            let cached = std::fs::read_to_string(&cache_path).unwrap_or_default();
            write_if_changed(
                &ai_dir.join("responses").join(format!("{task}.json")),
                &cached,
            )?;
            cache_hits += 1;
            external_ai_used = true;
            external_ai_used_from_cache = true;
            tasks.push(AiTaskUsage {
                task: task.to_string(),
                source: "cache".into(),
                prompt_version: prompt_version.to_string(),
                external_ai_used: true,
                external_ai_attempted: false,
                external_ai_used_from_cache: true,
                local_mock_used: false,
                cache_hit: true,
                new_external_ai_call: false,
                request_attempted: false,
                request_success: true,
                request_id: String::new(),
                model: model.clone(),
                input_tokens_estimate: estimate_tokens(&prompt),
                output_tokens_estimate: estimate_tokens(&cached),
                error: String::new(),
            });
            continue;
        }

        external_ai_attempted = true;
        match call_openai(task, &prompt, &model) {
            Ok((request_id, content)) => {
                ensure_dir(
                    cache_path
                        .parent()
                        .unwrap_or_else(|| Path::new("reports/_cache/ai")),
                )?;
                write_if_changed(&cache_path, &content)?;
                write_if_changed(
                    &ai_dir.join("responses").join(format!("{task}.json")),
                    &content,
                )?;
                new_external_ai_calls += 1;
                external_ai_used = true;
                tasks.push(AiTaskUsage {
                    task: task.to_string(),
                    source: "external_openai".into(),
                    prompt_version: prompt_version.to_string(),
                    external_ai_used: true,
                    external_ai_attempted: true,
                    external_ai_used_from_cache: false,
                    local_mock_used: false,
                    cache_hit: false,
                    new_external_ai_call: true,
                    request_attempted: true,
                    request_success: true,
                    request_id,
                    model: model.clone(),
                    input_tokens_estimate: estimate_tokens(&prompt),
                    output_tokens_estimate: estimate_tokens(&content),
                    error: String::new(),
                });
            }
            Err(err) => {
                let err_text = err.to_string();
                tasks.push(AiTaskUsage {
                    task: task.to_string(),
                    source: "external_openai".into(),
                    prompt_version: prompt_version.to_string(),
                    external_ai_used: false,
                    external_ai_attempted: true,
                    external_ai_used_from_cache: false,
                    local_mock_used: false,
                    cache_hit: false,
                    new_external_ai_call: false,
                    request_attempted: true,
                    request_success: false,
                    request_id: String::new(),
                    model: model.clone(),
                    input_tokens_estimate: estimate_tokens(&prompt),
                    output_tokens_estimate: 0,
                    error: err_text.clone(),
                });
                if options.require_external_ai {
                    let usage = AiUsage {
                        schema_version: SCHEMA_VERSION.to_string(),
                        ai_provenance: aggregate_provenance(AggregateProvenanceInput {
                            ai_mode: &ai_mode,
                            source: "external_openai",
                            external_ai_used,
                            local_mock_used: false,
                            cache_hit: cache_hits > 0,
                            new_external_ai_call: new_external_ai_calls > 0,
                            request_attempted: true,
                            request_success: false,
                            model: &model,
                            output_seed: &err_text,
                        }),
                        ai_mode: ai_mode.clone(),
                        require_external_ai: options.require_external_ai,
                        no_ai_cache: options.no_ai_cache,
                        external_ai_used,
                        external_ai_attempted: true,
                        external_ai_used_from_cache,
                        local_mock_used: false,
                        cache_used: cache_hits > 0,
                        ai_calls: new_external_ai_calls,
                        new_external_ai_calls,
                        cache_hits,
                        model: model.clone(),
                        provider: "openai".into(),
                        tasks,
                    };
                    write_json(&metadata_dir.join("ai_usage.json"), &usage)?;
                    return Err(anyhow!("External OpenAI API request failed and --require-external-ai forbids fallback: {err_text}"));
                }
                local_mock_used = true;
                break;
            }
        }
    }

    let ai_calls = new_external_ai_calls;
    let source = if new_external_ai_calls > 0 {
        "external_openai"
    } else if cache_hits > 0 {
        "cache"
    } else if local_mock_used || (ai_mode != "off" && !external_ai_used) {
        "local_mock"
    } else {
        "skipped"
    };
    let request_success = if external_ai_attempted {
        external_ai_used
    } else {
        local_mock_used || ai_mode == "off" || cache_hits > 0
    };
    let usage = AiUsage {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: aggregate_provenance(AggregateProvenanceInput {
            ai_mode: &ai_mode,
            source,
            external_ai_used,
            local_mock_used: local_mock_used || (ai_mode != "off" && !external_ai_used),
            cache_hit: cache_hits > 0,
            new_external_ai_call: new_external_ai_calls > 0,
            request_attempted: external_ai_attempted,
            request_success,
            model: &model,
            output_seed: &format!("{source}:{new_external_ai_calls}:{cache_hits}"),
        }),
        ai_mode: ai_mode.clone(),
        require_external_ai: options.require_external_ai,
        no_ai_cache: options.no_ai_cache,
        external_ai_used,
        external_ai_attempted,
        external_ai_used_from_cache,
        local_mock_used: local_mock_used || (ai_mode != "off" && !external_ai_used),
        cache_used: cache_hits > 0,
        ai_calls,
        new_external_ai_calls,
        cache_hits,
        model,
        provider: if external_ai_used || external_ai_attempted {
            "openai"
        } else {
            "local"
        }
        .into(),
        tasks,
    };
    if options.no_ai_cache && usage.cache_hits > 0 {
        write_json(&metadata_dir.join("ai_usage.json"), &usage)?;
        return Err(anyhow!(
            "--no-ai-cache was set but AI cache was used; refusing ambiguous AI provenance"
        ));
    }
    if options.require_external_ai
        && (!usage.external_ai_used || usage.local_mock_used || usage.new_external_ai_calls == 0)
    {
        write_json(&metadata_dir.join("ai_usage.json"), &usage)?;
        return Err(anyhow!(
            "--require-external-ai requires a successful new external OpenAI API call and forbids local fallback"
        ));
    }
    write_json(&metadata_dir.join("ai_usage.json"), &usage)?;
    write_if_changed(
        &ai_dir.join("cache_info.json"),
        &format!(
            "{{\n  \"schema_version\": \"{}\",\n  \"ai_mode\": \"{}\",\n  \"external_ai_used\": {},\n  \"new_external_ai_calls\": {},\n  \"cache_hits\": {},\n  \"no_ai_cache\": {}\n}}\n",
            SCHEMA_VERSION,
            usage.ai_mode,
            usage.external_ai_used,
            usage.new_external_ai_calls,
            usage.cache_hits,
            usage.no_ai_cache
        ),
    )?;
    Ok(usage)
}
