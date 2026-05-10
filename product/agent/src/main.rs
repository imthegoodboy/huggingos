use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::OnceLock;
use uuid::Uuid;

const MAX_TEXT_BYTES: u64 = 64 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProductConfig {
    name: String,
    version: String,
    track: String,
    phase: String,
    base_strategy: String,
}

impl Default for ProductConfig {
    fn default() -> Self {
        Self {
            name: "huggingOS".to_string(),
            version: "unknown".to_string(),
            track: "product".to_string(),
            phase: "Product Phase 3".to_string(),
            base_strategy: "Ubuntu LTS hosted prototype".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct RuntimeConfig {
    #[serde(default = "default_app_id")]
    app_id: String,
    #[serde(default = "default_config_env")]
    config_env: String,
    #[serde(default = "default_state_dir_env")]
    state_dir_env: String,
    #[serde(default = "default_workspace_dir_env")]
    workspace_dir_env: String,
    #[serde(default)]
    workspace_dir: Option<String>,
    #[serde(default)]
    state_dir: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AiConfig {
    #[serde(default = "default_ai_provider")]
    default_provider: String,
    #[serde(default = "default_ai_provider_env")]
    provider_env: String,
    #[serde(default = "default_ai_offline_env")]
    offline_env: String,
    #[serde(default)]
    offline_mode: bool,
    #[serde(default = "default_openai_api_key_env")]
    openai_api_key_env: String,
    #[serde(default = "default_anthropic_api_key_env")]
    anthropic_api_key_env: String,
    #[serde(default = "default_local_model_endpoint_env")]
    local_model_endpoint_env: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            default_provider: default_ai_provider(),
            provider_env: default_ai_provider_env(),
            offline_env: default_ai_offline_env(),
            offline_mode: true,
            openai_api_key_env: default_openai_api_key_env(),
            anthropic_api_key_env: default_anthropic_api_key_env(),
            local_model_endpoint_env: default_local_model_endpoint_env(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PolicyConfig {
    audit_log_name: String,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            audit_log_name: "audit.log".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct Config {
    product: ProductConfig,
    runtime: RuntimeConfig,
    ai: AiConfig,
    features: BTreeMap<String, bool>,
    policy: PolicyConfig,
    config_path: PathBuf,
    product_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct FileConfig {
    #[serde(default)]
    product: ProductConfig,
    #[serde(default)]
    runtime: RuntimeConfig,
    #[serde(default)]
    ai: AiConfig,
    #[serde(default)]
    features: BTreeMap<String, bool>,
    #[serde(default)]
    policy: PolicyConfig,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
enum RiskLevel {
    Read,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum PolicyDecision {
    Allow,
    Deny,
    Confirm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ActionStatus {
    Succeeded,
    Failed,
    Denied,
    ConfirmationRequired,
    DryRun,
}

#[derive(Clone, Debug, Serialize)]
struct CapabilityMetadata {
    name: String,
    version: String,
    owner: String,
    description: String,
    risk: RiskLevel,
    permissions: Vec<String>,
    input_schema: Value,
    result_schema: Value,
    reversible: bool,
}

#[derive(Clone)]
struct Capability {
    metadata: CapabilityMetadata,
    execute: fn(&ActionRequest, &Config) -> Result<Value, String>,
    verify: fn(&ActionRequest, &Config, &Value) -> Verification,
}

#[derive(Clone, Debug, Serialize)]
struct ActionRequest {
    action_id: String,
    capability: String,
    params: Map<String, Value>,
    actor: String,
    reason: String,
    dry_run: bool,
    confirmed: bool,
    requested_at: String,
}

#[derive(Clone, Debug, Serialize)]
struct Verification {
    ok: bool,
    message: String,
    data: Value,
}

#[derive(Clone, Debug, Serialize)]
struct ActionResult {
    action_id: String,
    capability: String,
    status: ActionStatus,
    started_at: String,
    finished_at: String,
    summary: String,
    data: Value,
    error: Option<String>,
    verification: Verification,
    audit_ref: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct PolicyOutcome {
    decision: PolicyDecision,
    reason: String,
}

#[derive(Default)]
struct RunOptions {
    params: Vec<String>,
    params_json: Option<String>,
    actor: String,
    reason: String,
    dry_run: bool,
    confirmed: bool,
    json: bool,
}

#[derive(Default)]
struct AiOptions {
    provider: Option<String>,
    actor: String,
    reason: String,
    dry_run: bool,
    confirmed: bool,
    json: bool,
    prompt: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct SecretStatus {
    name: String,
    env_var: String,
    required_for: Vec<String>,
    present: bool,
    source: Option<String>,
    redacted: bool,
}

#[derive(Clone, Debug)]
struct SecretSpec {
    name: String,
    env_var: String,
    required_for: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct AiProviderStatus {
    id: String,
    provider_type: String,
    configured: bool,
    available: bool,
    offline_capable: bool,
    uses_network: bool,
    secret_names: Vec<String>,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
struct AiRuntimeStatus {
    default_provider: String,
    selected_provider: String,
    offline_mode: bool,
    providers: Vec<AiProviderStatus>,
    secrets: Vec<SecretStatus>,
}

#[derive(Clone, Debug, Serialize)]
struct AiPlanStep {
    step_id: String,
    capability: String,
    params: Map<String, Value>,
    reason: String,
    risk: Option<RiskLevel>,
    requires_confirmation: bool,
}

#[derive(Clone, Debug, Serialize)]
struct AiPlan {
    plan_id: String,
    provider: String,
    prompt: String,
    created_at: String,
    executable: bool,
    steps: Vec<AiPlanStep>,
    warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct AiRunReport {
    run_id: String,
    status: String,
    summary: String,
    plan: AiPlan,
    results: Vec<ActionResult>,
}

fn main() -> ExitCode {
    match run(
        env::args().skip(1).collect(),
        &mut io::stdout(),
        &mut io::stderr(),
    ) {
        0 => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn run(args: Vec<String>, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let args = if args.is_empty() {
        vec!["status".to_string()]
    } else {
        args
    };

    let config = match load_config() {
        Ok(config) => config,
        Err(err) => {
            let _ = writeln!(stderr, "huggingos-agent: {err}");
            return 2;
        }
    };

    match args[0].as_str() {
        "status" => {
            let as_json = args.iter().any(|arg| arg == "--json");
            emit_status(stdout, &config, as_json);
            0
        }
        "capabilities" => {
            let as_json = args.iter().any(|arg| arg == "--json");
            emit_capabilities(stdout, as_json);
            0
        }
        "secrets" => run_secrets_command(&args[1..], stdout, stderr, &config),
        "ai" => run_ai_command(&args[1..], stdout, stderr, &config),
        "run" => run_capability_command(&args[1..], stdout, stderr, &config),
        command => {
            let _ = writeln!(stderr, "huggingos-agent: unknown command: {command}");
            2
        }
    }
}

fn run_capability_command(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    config: &Config,
) -> i32 {
    if args.is_empty() {
        let _ = writeln!(stderr, "huggingos-agent: run requires a capability name");
        return 2;
    }
    let capability = args[0].clone();
    let options = match parse_run_options(&args[1..]) {
        Ok(options) => options,
        Err(err) => {
            let _ = writeln!(stderr, "huggingos-agent: {err}");
            return 2;
        }
    };
    let params = match parse_params(&options.params, options.params_json.as_deref()) {
        Ok(params) => params,
        Err(err) => {
            let _ = writeln!(stderr, "huggingos-agent: {err}");
            return 2;
        }
    };
    let request = ActionRequest {
        action_id: Uuid::new_v4().to_string(),
        capability,
        params,
        actor: if options.actor.is_empty() {
            "user".to_string()
        } else {
            options.actor
        },
        reason: options.reason,
        dry_run: options.dry_run,
        confirmed: options.confirmed,
        requested_at: utc_now(),
    };
    let result = execute_capability(config, &build_registry(), request);
    emit_result(stdout, &result, options.json);
    if result.error.is_some() {
        1
    } else {
        0
    }
}

fn parse_run_options(args: &[String]) -> Result<RunOptions, String> {
    let mut options = RunOptions {
        actor: "user".to_string(),
        ..RunOptions::default()
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--param" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--param requires key=value".to_string())?;
                options.params.push(value.clone());
            }
            "--params-json" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--params-json requires a value".to_string())?;
                options.params_json = Some(value.clone());
            }
            "--actor" => {
                index += 1;
                options.actor = args
                    .get(index)
                    .ok_or_else(|| "--actor requires a value".to_string())?
                    .clone();
            }
            "--reason" => {
                index += 1;
                options.reason = args
                    .get(index)
                    .ok_or_else(|| "--reason requires a value".to_string())?
                    .clone();
            }
            "--dry-run" => options.dry_run = true,
            "--confirm" => options.confirmed = true,
            "--json" => options.json = true,
            other => return Err(format!("unknown run option: {other}")),
        }
        index += 1;
    }
    Ok(options)
}

fn emit_status(stdout: &mut dyn Write, config: &Config, as_json: bool) {
    let status = product_status(config);
    if as_json {
        emit_json(stdout, &status);
    } else {
        let _ = writeln!(
            stdout,
            "{} {}\ntrack: {}\nphase: {}",
            config.product.name, config.product.version, config.product.track, config.product.phase
        );
    }
}

fn emit_capabilities(stdout: &mut dyn Write, as_json: bool) {
    let capabilities: Vec<CapabilityMetadata> = build_registry()
        .values()
        .map(|capability| capability.metadata.clone())
        .collect();
    if as_json {
        emit_json(stdout, &json!({ "capabilities": capabilities }));
    } else {
        for capability in capabilities {
            let _ = writeln!(
                stdout,
                "{} v{} [{:?}] {}",
                capability.name, capability.version, capability.risk, capability.description
            );
        }
    }
}

fn emit_result(stdout: &mut dyn Write, result: &ActionResult, as_json: bool) {
    if as_json {
        emit_json(stdout, result);
        return;
    }
    let _ = writeln!(
        stdout,
        "{}: {:?}\n{}",
        result.capability, result.status, result.summary
    );
    if let Some(error) = &result.error {
        let _ = writeln!(stdout, "error: {error}");
    }
    if let Some(audit_ref) = &result.audit_ref {
        let _ = writeln!(stdout, "audit: {audit_ref}");
    }
}

fn run_secrets_command(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    config: &Config,
) -> i32 {
    let (command, rest) = if let Some(first) = args.first().filter(|arg| !arg.starts_with("--")) {
        (first.as_str(), &args[1..])
    } else {
        ("status", args)
    };
    match command {
        "status" => {
            let as_json = rest.iter().any(|arg| arg == "--json");
            emit_secret_status(stdout, config, as_json);
            0
        }
        other => {
            let _ = writeln!(stderr, "huggingos-agent: unknown secrets command: {other}");
            2
        }
    }
}

fn run_ai_command(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    config: &Config,
) -> i32 {
    let (command, rest) = if let Some(first) = args.first().filter(|arg| !arg.starts_with("--")) {
        (first.as_str(), &args[1..])
    } else {
        ("status", args)
    };
    match command {
        "status" => {
            let options = match parse_ai_options(rest) {
                Ok(options) => options,
                Err(err) => {
                    let _ = writeln!(stderr, "huggingos-agent: {err}");
                    return 2;
                }
            };
            emit_ai_status(stdout, config, options.provider.as_deref(), options.json);
            0
        }
        "plan" => run_ai_plan_command(rest, stdout, stderr, config),
        "run" => run_ai_run_command(rest, stdout, stderr, config),
        other => {
            let _ = writeln!(stderr, "huggingos-agent: unknown ai command: {other}");
            2
        }
    }
}

fn run_ai_plan_command(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    config: &Config,
) -> i32 {
    let options = match parse_ai_options(args) {
        Ok(options) => options,
        Err(err) => {
            let _ = writeln!(stderr, "huggingos-agent: {err}");
            return 2;
        }
    };
    let prompt = options.prompt.join(" ").trim().to_string();
    if prompt.is_empty() {
        let _ = writeln!(stderr, "huggingos-agent: ai plan requires a prompt");
        return 2;
    }

    match build_ai_plan(
        config,
        &build_registry(),
        &prompt,
        options.provider.as_deref(),
    ) {
        Ok(plan) => {
            emit_ai_plan(stdout, &plan, options.json);
            if plan.executable {
                0
            } else {
                1
            }
        }
        Err(error) => {
            if options.json {
                emit_json(
                    stdout,
                    &json!({
                        "error": error,
                        "status": ai_runtime_status(config, options.provider.as_deref()),
                    }),
                );
            } else {
                let _ = writeln!(stderr, "huggingos-agent: {error}");
            }
            1
        }
    }
}

fn run_ai_run_command(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    config: &Config,
) -> i32 {
    let options = match parse_ai_options(args) {
        Ok(options) => options,
        Err(err) => {
            let _ = writeln!(stderr, "huggingos-agent: {err}");
            return 2;
        }
    };
    let prompt = options.prompt.join(" ").trim().to_string();
    if prompt.is_empty() {
        let _ = writeln!(stderr, "huggingos-agent: ai run requires a prompt");
        return 2;
    }

    let registry = build_registry();
    let plan = match build_ai_plan(config, &registry, &prompt, options.provider.as_deref()) {
        Ok(plan) => plan,
        Err(error) => {
            if options.json {
                emit_json(
                    stdout,
                    &json!({
                        "error": error,
                        "status": ai_runtime_status(config, options.provider.as_deref()),
                    }),
                );
            } else {
                let _ = writeln!(stderr, "huggingos-agent: {error}");
            }
            return 1;
        }
    };

    let report = execute_ai_plan(config, &registry, plan, &options);
    emit_ai_run_report(stdout, &report, options.json);
    if report.status == "succeeded" || report.status == "dry_run" {
        0
    } else {
        1
    }
}

fn parse_ai_options(args: &[String]) -> Result<AiOptions, String> {
    let mut options = AiOptions {
        actor: "ai.cli".to_string(),
        ..AiOptions::default()
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--provider" => {
                index += 1;
                options.provider = Some(
                    args.get(index)
                        .ok_or_else(|| "--provider requires a value".to_string())?
                        .clone(),
                );
            }
            "--actor" => {
                index += 1;
                options.actor = args
                    .get(index)
                    .ok_or_else(|| "--actor requires a value".to_string())?
                    .clone();
            }
            "--reason" => {
                index += 1;
                options.reason = args
                    .get(index)
                    .ok_or_else(|| "--reason requires a value".to_string())?
                    .clone();
            }
            "--dry-run" => options.dry_run = true,
            "--confirm" => options.confirmed = true,
            "--json" => options.json = true,
            "--" => {
                options.prompt.extend(args[index + 1..].iter().cloned());
                break;
            }
            other if other.starts_with("--") => return Err(format!("unknown ai option: {other}")),
            other => options.prompt.push(other.to_string()),
        }
        index += 1;
    }
    Ok(options)
}

fn emit_secret_status(stdout: &mut dyn Write, config: &Config, as_json: bool) {
    let secrets = secret_statuses(config);
    if as_json {
        emit_json(stdout, &json!({ "secrets": secrets }));
        return;
    }
    for secret in secrets {
        let state = if secret.present {
            "present (redacted)"
        } else {
            "missing"
        };
        let _ = writeln!(stdout, "{} [{}]: {state}", secret.name, secret.env_var);
    }
}

fn emit_ai_status(
    stdout: &mut dyn Write,
    config: &Config,
    provider_override: Option<&str>,
    as_json: bool,
) {
    let status = ai_runtime_status(config, provider_override);
    if as_json {
        emit_json(stdout, &status);
        return;
    }
    let _ = writeln!(
        stdout,
        "AI runtime provider: {}\noffline mode: {}",
        status.selected_provider, status.offline_mode
    );
    for provider in status.providers {
        let state = if provider.available {
            "available"
        } else if provider.configured {
            "configured"
        } else {
            "not configured"
        };
        let _ = writeln!(stdout, "{}: {state} - {}", provider.id, provider.message);
    }
}

fn emit_ai_plan(stdout: &mut dyn Write, plan: &AiPlan, as_json: bool) {
    if as_json {
        emit_json(stdout, plan);
        return;
    }
    let _ = writeln!(
        stdout,
        "plan {} via {} (executable: {})",
        plan.plan_id, plan.provider, plan.executable
    );
    for step in &plan.steps {
        let _ = writeln!(
            stdout,
            "{} -> {} {:?}",
            step.step_id, step.capability, step.params
        );
    }
    for warning in &plan.warnings {
        let _ = writeln!(stdout, "warning: {warning}");
    }
}

fn emit_ai_run_report(stdout: &mut dyn Write, report: &AiRunReport, as_json: bool) {
    if as_json {
        emit_json(stdout, report);
        return;
    }
    let _ = writeln!(
        stdout,
        "ai run {}: {}\n{}",
        report.run_id, report.status, report.summary
    );
    for result in &report.results {
        let _ = writeln!(
            stdout,
            "{}: {:?} - {}",
            result.capability, result.status, result.summary
        );
        if let Some(error) = &result.error {
            let _ = writeln!(stdout, "error: {error}");
        }
    }
}

fn emit_json<T: Serialize>(stdout: &mut dyn Write, payload: &T) {
    match serde_json::to_string_pretty(payload) {
        Ok(text) => {
            let _ = writeln!(stdout, "{text}");
        }
        Err(err) => {
            let _ = writeln!(stdout, "{{\"error\":\"{err}\"}}");
        }
    }
}

fn load_config() -> Result<Config, String> {
    let repo_root = find_repo_root()?;
    let default_config = repo_root
        .join("product")
        .join("config")
        .join("defaults.toml");
    let config_path = env::var("HUGGINGOS_CONFIG_FILE")
        .map(PathBuf::from)
        .unwrap_or(default_config);
    let config_path = absolute_path(config_path)?;
    let raw = fs::read_to_string(&config_path)
        .map_err(|err| format!("config file not found: {} ({err})", config_path.display()))?;
    let file_config: FileConfig = toml::from_str(&raw)
        .map_err(|err| format!("invalid TOML config in {}: {err}", config_path.display()))?;

    Ok(Config {
        product: file_config.product,
        runtime: file_config.runtime,
        ai: file_config.ai,
        features: file_config.features,
        policy: file_config.policy,
        config_path,
        product_root: repo_root.join("product"),
    })
}

fn find_repo_root() -> Result<PathBuf, String> {
    let mut current = env::current_dir().map_err(|err| err.to_string())?;
    loop {
        if current
            .join("product")
            .join("config")
            .join("defaults.toml")
            .exists()
        {
            return Ok(current);
        }
        if !current.pop() {
            return Err("could not find product/config/defaults.toml".to_string());
        }
    }
}

fn product_status(config: &Config) -> Value {
    json!({
        "product": config.product.name,
        "version": config.product.version,
        "track": config.product.track,
        "phase": config.product.phase,
        "base_strategy": config.product.base_strategy,
        "host": {
            "system": env::consts::OS,
            "machine": env::consts::ARCH,
            "is_linux": cfg!(target_os = "linux"),
        },
        "agent": {
            "name": "huggingos-agent",
            "version": env!("CARGO_PKG_VERSION"),
            "language": "rust",
        },
        "ai": {
            "default_provider": config.ai.default_provider,
            "selected_provider": selected_ai_provider(config, None),
            "offline_mode": ai_offline_mode(config),
            "cloud_ai_enabled": feature_enabled(config, "cloud_ai_enabled"),
        },
        "paths": {
            "product_root": config.product_root,
            "config_file": config.config_path,
            "state_dir": state_dir(config),
            "workspace_dir": workspace_dir(config),
            "audit_log": audit_log_path(config),
        },
        "features": config.features,
    })
}

fn state_dir(config: &Config) -> PathBuf {
    if let Ok(explicit) = env::var(&config.runtime.state_dir_env) {
        return absolute_path_lossy(explicit);
    }
    if let Some(configured) = &config.runtime.state_dir {
        return absolute_path_lossy(configured);
    }
    if let Ok(xdg_state) = env::var("XDG_STATE_HOME") {
        return absolute_path_lossy(PathBuf::from(xdg_state).join(&config.runtime.app_id));
    }
    home_dir()
        .join(".local")
        .join("state")
        .join(&config.runtime.app_id)
}

fn workspace_dir(config: &Config) -> PathBuf {
    if let Ok(explicit) = env::var(&config.runtime.workspace_dir_env) {
        return absolute_path_lossy(explicit);
    }
    if let Some(configured) = &config.runtime.workspace_dir {
        let path = expand_home(configured);
        if path.is_absolute() {
            return absolute_path_lossy(path);
        }
        return absolute_path_lossy(state_dir(config).join(path));
    }
    absolute_path_lossy(state_dir(config).join("workspace"))
}

fn audit_log_path(config: &Config) -> PathBuf {
    let name = Path::new(&config.policy.audit_log_name)
        .file_name()
        .and_then(OsStr::to_str)
        .filter(|name| !name.is_empty() && *name != "." && *name != "..")
        .unwrap_or("audit.log");
    absolute_path_lossy(state_dir(config).join(name))
}

fn default_app_id() -> String {
    "huggingos".to_string()
}

fn default_config_env() -> String {
    "HUGGINGOS_CONFIG_FILE".to_string()
}

fn default_state_dir_env() -> String {
    "HUGGINGOS_STATE_DIR".to_string()
}

fn default_workspace_dir_env() -> String {
    "HUGGINGOS_WORKSPACE_DIR".to_string()
}

fn default_ai_provider() -> String {
    "local.rules".to_string()
}

fn default_ai_provider_env() -> String {
    "HUGGINGOS_AI_PROVIDER".to_string()
}

fn default_ai_offline_env() -> String {
    "HUGGINGOS_AI_OFFLINE".to_string()
}

fn default_openai_api_key_env() -> String {
    "HUGGINGOS_OPENAI_API_KEY".to_string()
}

fn default_anthropic_api_key_env() -> String {
    "HUGGINGOS_ANTHROPIC_API_KEY".to_string()
}

fn default_local_model_endpoint_env() -> String {
    "HUGGINGOS_LOCAL_MODEL_ENDPOINT".to_string()
}

fn feature_enabled(config: &Config, name: &str) -> bool {
    config.features.get(name).copied().unwrap_or(false)
}

fn selected_ai_provider(config: &Config, provider_override: Option<&str>) -> String {
    provider_override
        .map(str::to_string)
        .or_else(|| env::var(&config.ai.provider_env).ok())
        .unwrap_or_else(|| config.ai.default_provider.clone())
        .trim()
        .to_ascii_lowercase()
}

fn ai_offline_mode(config: &Config) -> bool {
    env::var(&config.ai.offline_env)
        .ok()
        .and_then(|value| parse_bool(&value))
        .unwrap_or(config.ai.offline_mode)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn secret_specs(config: &Config) -> Vec<SecretSpec> {
    vec![
        SecretSpec {
            name: "openai_api_key".to_string(),
            env_var: config.ai.openai_api_key_env.clone(),
            required_for: vec!["cloud.openai".to_string()],
        },
        SecretSpec {
            name: "anthropic_api_key".to_string(),
            env_var: config.ai.anthropic_api_key_env.clone(),
            required_for: vec!["cloud.anthropic".to_string()],
        },
        SecretSpec {
            name: "local_model_endpoint".to_string(),
            env_var: config.ai.local_model_endpoint_env.clone(),
            required_for: vec!["local.model".to_string()],
        },
    ]
}

fn secret_statuses(config: &Config) -> Vec<SecretStatus> {
    secret_specs(config)
        .into_iter()
        .map(|spec| {
            let value = env::var(&spec.env_var).ok();
            secret_status_from_value(spec, value.as_deref())
        })
        .collect()
}

fn secret_status_from_value(spec: SecretSpec, value: Option<&str>) -> SecretStatus {
    let present = value.is_some_and(|value| !value.trim().is_empty());
    SecretStatus {
        name: spec.name,
        env_var: spec.env_var,
        required_for: spec.required_for,
        present,
        source: present.then(|| "environment".to_string()),
        redacted: present,
    }
}

fn ai_runtime_status(config: &Config, provider_override: Option<&str>) -> AiRuntimeStatus {
    let secrets = secret_statuses(config);
    AiRuntimeStatus {
        default_provider: config.ai.default_provider.clone(),
        selected_provider: selected_ai_provider(config, provider_override),
        offline_mode: ai_offline_mode(config),
        providers: ai_provider_statuses(config, &secrets),
        secrets,
    }
}

fn ai_provider_statuses(config: &Config, secrets: &[SecretStatus]) -> Vec<AiProviderStatus> {
    let local_model_configured = env::var(&config.ai.local_model_endpoint_env)
        .ok()
        .is_some_and(|value| !value.trim().is_empty());
    let openai_configured = secret_present(secrets, "openai_api_key");
    let anthropic_configured = secret_present(secrets, "anthropic_api_key");
    let cloud_enabled = feature_enabled(config, "cloud_ai_enabled");

    vec![
        AiProviderStatus {
            id: "local.rules".to_string(),
            provider_type: "deterministic_rules".to_string(),
            configured: true,
            available: true,
            offline_capable: true,
            uses_network: false,
            secret_names: vec![],
            message: "Deterministic local planner is ready and requires no network.".to_string(),
        },
        AiProviderStatus {
            id: "local.model".to_string(),
            provider_type: "local_model_runtime".to_string(),
            configured: local_model_configured,
            available: false,
            offline_capable: true,
            uses_network: false,
            secret_names: vec!["local_model_endpoint".to_string()],
            message: if local_model_configured {
                "Endpoint is configured; model planning adapter is not enabled yet.".to_string()
            } else {
                "Set the local model endpoint env var when a local runtime adapter is added."
                    .to_string()
            },
        },
        AiProviderStatus {
            id: "cloud.openai".to_string(),
            provider_type: "cloud_model_runtime".to_string(),
            configured: openai_configured && cloud_enabled,
            available: false,
            offline_capable: false,
            uses_network: true,
            secret_names: vec!["openai_api_key".to_string()],
            message: if openai_configured {
                "Secret is detected and redacted; outbound cloud planning is disabled in this build."
                    .to_string()
            } else {
                "Missing redacted API-key readiness signal.".to_string()
            },
        },
        AiProviderStatus {
            id: "cloud.anthropic".to_string(),
            provider_type: "cloud_model_runtime".to_string(),
            configured: anthropic_configured && cloud_enabled,
            available: false,
            offline_capable: false,
            uses_network: true,
            secret_names: vec!["anthropic_api_key".to_string()],
            message: if anthropic_configured {
                "Secret is detected and redacted; outbound cloud planning is disabled in this build."
                    .to_string()
            } else {
                "Missing redacted API-key readiness signal.".to_string()
            },
        },
    ]
}

fn secret_present(secrets: &[SecretStatus], name: &str) -> bool {
    secrets
        .iter()
        .any(|secret| secret.name == name && secret.present)
}

trait AiPlanProvider {
    fn id(&self) -> &'static str;
    fn plan(&self, registry: &BTreeMap<String, Capability>, prompt: &str) -> AiPlan;
}

struct LocalRulesProvider;

impl AiPlanProvider for LocalRulesProvider {
    fn id(&self) -> &'static str {
        "local.rules"
    }

    fn plan(&self, registry: &BTreeMap<String, Capability>, prompt: &str) -> AiPlan {
        local_rules_plan(registry, prompt)
    }
}

fn build_ai_plan(
    config: &Config,
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    provider_override: Option<&str>,
) -> Result<AiPlan, String> {
    let provider_id = selected_ai_provider(config, provider_override);
    let local_provider = LocalRulesProvider;
    if provider_id == local_provider.id() {
        return Ok(local_provider.plan(registry, prompt));
    }

    let status = ai_runtime_status(config, Some(&provider_id));
    let provider = status
        .providers
        .iter()
        .find(|provider| provider.id == provider_id);
    match provider {
        Some(provider) => Err(format!(
            "AI provider {} is not executable yet: {}",
            provider.id, provider.message
        )),
        None => Err(format!("unknown AI provider: {provider_id}")),
    }
}

fn local_rules_plan(registry: &BTreeMap<String, Capability>, prompt: &str) -> AiPlan {
    let lowered = prompt.trim().to_lowercase();
    let mut steps = vec![];
    let mut warnings = vec![];

    if let Some(step) = plan_audit_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_read_file_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_list_files_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_note_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_status_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else {
        warnings.push(
            "No deterministic local rule matched this prompt; no capability will execute."
                .to_string(),
        );
    }

    AiPlan {
        plan_id: Uuid::new_v4().to_string(),
        provider: "local.rules".to_string(),
        prompt: prompt.to_string(),
        created_at: utc_now(),
        executable: !steps.is_empty(),
        steps,
        warnings,
    }
}

fn plan_audit_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("audit")
        || lowered.contains("recent action")
        || lowered.contains("recent run"))
    {
        return None;
    }
    let mut params = Map::new();
    params.insert("limit".to_string(), json!(20));
    Some(plan_step(
        registry,
        "audit.list",
        params,
        format!("List recent audit records for prompt: {prompt}"),
    ))
}

fn plan_read_file_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.starts_with("read ")
        || lowered.starts_with("open file ")
        || lowered.starts_with("show file ")
        || lowered.starts_with("cat "))
    {
        return None;
    }
    let path = extract_path_after(
        prompt,
        &[
            "open file ",
            "show file ",
            "read file ",
            "read text ",
            "read ",
            "cat ",
        ],
    )?;
    let mut params = Map::new();
    params.insert("path".to_string(), json!(path));
    Some(plan_step(
        registry,
        "fs.read_text",
        params,
        "Read a user-requested small UTF-8 text file.".to_string(),
    ))
}

fn plan_list_files_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.starts_with("list")
        || lowered.starts_with("show files")
        || lowered.starts_with("ls "))
    {
        return None;
    }
    let path = extract_path_after(
        prompt,
        &[
            "list files in ",
            "list files under ",
            "list directory ",
            "list dir ",
            "show files in ",
            "show files under ",
            "ls ",
            "list ",
        ],
    )
    .filter(|value| {
        !matches!(
            value.to_ascii_lowercase().as_str(),
            "file" | "files" | "directory" | "directories" | "dir" | "dirs"
        )
    })
    .unwrap_or_else(|| ".".to_string());
    let mut params = Map::new();
    params.insert("path".to_string(), json!(path));
    Some(plan_step(
        registry,
        "fs.list",
        params,
        "List a user-requested local directory.".to_string(),
    ))
}

fn plan_note_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.starts_with("create note")
        || lowered.starts_with("make note")
        || lowered.starts_with("write note"))
    {
        return None;
    }
    let body = extract_path_after(prompt, &["create note", "make note", "write note"])
        .unwrap_or_else(|| "AI Note".to_string());
    let (title, content) = split_note_body(&body);
    let mut params = Map::new();
    params.insert("title".to_string(), json!(title));
    if !content.trim().is_empty() {
        params.insert("content".to_string(), json!(content));
    }
    Some(plan_step(
        registry,
        "notes.create",
        params,
        "Create a safe workspace note from the prompt.".to_string(),
    ))
}

fn plan_status_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    let trimmed = lowered.trim();
    if !(trimmed == "status"
        || trimmed == "health"
        || lowered.contains("product status")
        || lowered.contains("system status")
        || lowered.contains("agent status")
        || lowered.contains("about huggingos"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "product.status",
        Map::new(),
        format!("Report product status for prompt: {prompt}"),
    ))
}

fn plan_step(
    registry: &BTreeMap<String, Capability>,
    capability: &str,
    params: Map<String, Value>,
    reason: String,
) -> AiPlanStep {
    let risk = registry
        .get(capability)
        .map(|capability| capability.metadata.risk);
    let requires_confirmation = matches!(risk, Some(RiskLevel::Medium | RiskLevel::High));
    AiPlanStep {
        step_id: Uuid::new_v4().to_string(),
        capability: capability.to_string(),
        params,
        reason,
        risk,
        requires_confirmation,
    }
}

fn extract_path_after(prompt: &str, markers: &[&str]) -> Option<String> {
    let lowered = prompt.to_ascii_lowercase();
    for marker in markers {
        if let Some(index) = lowered.find(marker) {
            let value = clean_prompt_value(&prompt[index + marker.len()..]);
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn clean_prompt_value(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch| matches!(ch, '"' | '\'' | '`' | ',' | ';' | ':'))
        .trim()
        .to_string()
}

fn split_note_body(body: &str) -> (String, String) {
    let body = clean_prompt_value(body);
    if let Some((title, content)) = body.split_once(" content:") {
        return (safe_note_title(title), clean_prompt_value(content));
    }
    if let Some((title, content)) = body.split_once(" with content ") {
        return (safe_note_title(title), clean_prompt_value(content));
    }
    let title = safe_note_title(&body);
    (title, body)
}

fn safe_note_title(value: &str) -> String {
    let title = value.trim();
    if title.is_empty() {
        "AI Note".to_string()
    } else {
        title.chars().take(80).collect()
    }
}

fn execute_ai_plan(
    config: &Config,
    registry: &BTreeMap<String, Capability>,
    plan: AiPlan,
    options: &AiOptions,
) -> AiRunReport {
    let mut results = vec![];
    if !plan.executable {
        return AiRunReport {
            run_id: Uuid::new_v4().to_string(),
            status: "no_plan".to_string(),
            summary: "No executable capability plan was produced.".to_string(),
            plan,
            results,
        };
    }

    for step in &plan.steps {
        let request = ActionRequest {
            action_id: Uuid::new_v4().to_string(),
            capability: step.capability.clone(),
            params: step.params.clone(),
            actor: options.actor.clone(),
            reason: if options.reason.trim().is_empty() {
                step.reason.clone()
            } else {
                options.reason.clone()
            },
            dry_run: options.dry_run,
            confirmed: options.confirmed,
            requested_at: utc_now(),
        };
        let result = execute_capability(config, registry, request);
        let should_stop = matches!(
            result.status,
            ActionStatus::Failed | ActionStatus::Denied | ActionStatus::ConfirmationRequired
        );
        results.push(result);
        if should_stop {
            break;
        }
    }

    let status = ai_run_status(&results);
    let summary = match status.as_str() {
        "succeeded" => "All planned capability calls executed and verified.",
        "dry_run" => "All planned capability calls completed as dry runs.",
        "needs_confirmation" => "A planned capability requires explicit confirmation.",
        "failed" => "A planned capability failed or was denied.",
        _ => "AI run finished.",
    }
    .to_string();

    AiRunReport {
        run_id: Uuid::new_v4().to_string(),
        status,
        summary,
        plan,
        results,
    }
}

fn ai_run_status(results: &[ActionResult]) -> String {
    if results.is_empty() {
        return "no_plan".to_string();
    }
    if results
        .iter()
        .any(|result| result.status == ActionStatus::ConfirmationRequired)
    {
        return "needs_confirmation".to_string();
    }
    if results
        .iter()
        .any(|result| matches!(result.status, ActionStatus::Failed | ActionStatus::Denied))
    {
        return "failed".to_string();
    }
    if results
        .iter()
        .all(|result| result.status == ActionStatus::DryRun)
    {
        return "dry_run".to_string();
    }
    "succeeded".to_string()
}

fn build_registry() -> BTreeMap<String, Capability> {
    let capabilities = [
        product_status_capability(),
        fs_list_capability(),
        fs_read_text_capability(),
        notes_create_capability(),
        audit_list_capability(),
    ];
    capabilities
        .into_iter()
        .map(|capability| (capability.metadata.name.clone(), capability))
        .collect()
}

fn product_status_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "product.status".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Report real product and host status.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["product:read".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, config| Ok(product_status(config)),
        verify: |_, _, data| Verification {
            ok: data.get("track") == Some(&json!("product")) && data.get("host").is_some(),
            message: "Product status returned host state.".to_string(),
            data: json!({}),
        },
    }
}

fn fs_list_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "fs.list".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "List a local directory without changing filesystem state.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["fs:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("path".to_string(), "string".to_string())]),
                vec!["path"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, _| {
            let raw_path = string_param(request, "path")?;
            let path = resolve_existing_path(&raw_path)?;
            if !path.is_dir() {
                return Err(format!("path is not a directory: {}", path.display()));
            }
            let mut entries = vec![];
            for entry in fs::read_dir(&path).map_err(|err| err.to_string())? {
                let entry = entry.map_err(|err| err.to_string())?;
                let metadata = entry.metadata().map_err(|err| err.to_string())?;
                entries.push(json!({
                    "name": entry.file_name().to_string_lossy(),
                    "path": entry.path(),
                    "type": if metadata.is_dir() { "directory" } else { "file" },
                    "size": metadata.len(),
                }));
            }
            entries.sort_by_key(|entry| entry["name"].as_str().unwrap_or("").to_lowercase());
            let entry_count = entries.len();
            Ok(json!({ "path": path, "entries": entries, "entry_count": entry_count }))
        },
        verify: |_, _, data| Verification {
            ok: data.get("entries").is_some(),
            message: "Directory listing verified.".to_string(),
            data: json!({}),
        },
    }
}

fn fs_read_text_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "fs.read_text".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Read a small UTF-8 text file with a size limit.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["fs:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("path".to_string(), "string".to_string())]),
                vec!["path"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, _| {
            let raw_path = string_param(request, "path")?;
            let path = resolve_existing_path(&raw_path)?;
            let metadata = fs::metadata(&path).map_err(|err| err.to_string())?;
            if !metadata.is_file() {
                return Err(format!("path is not a regular file: {}", path.display()));
            }
            if metadata.len() > MAX_TEXT_BYTES {
                return Err(format!(
                    "file is too large for Phase 2 text read: {} bytes",
                    metadata.len()
                ));
            }
            let bytes = fs::read(&path).map_err(|err| err.to_string())?;
            let text = String::from_utf8(bytes)
                .map_err(|_| format!("file is not valid UTF-8 text: {}", path.display()))?;
            Ok(json!({ "path": path, "size": metadata.len(), "text": text }))
        },
        verify: |_, _, data| Verification {
            ok: data.get("text").is_some(),
            message: "Text file read verified.".to_string(),
            data: json!({}),
        },
    }
}

fn notes_create_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "notes.create".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Create a text note inside the configured safe workspace.".to_string(),
            risk: RiskLevel::Low,
            permissions: vec!["notes:create".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("title".to_string(), "string".to_string()),
                    ("content".to_string(), "string".to_string()),
                    ("filename".to_string(), "string".to_string()),
                ]),
                vec!["title"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: true,
        },
        execute: |request, config| {
            let title = string_param(request, "title")?.trim().to_string();
            if title.is_empty() {
                return Err("note title cannot be empty".to_string());
            }
            let workspace = workspace_dir(config);
            fs::create_dir_all(&workspace).map_err(|err| err.to_string())?;
            let filename = request
                .params
                .get("filename")
                .and_then(Value::as_str)
                .map(safe_note_filename)
                .unwrap_or_else(|| safe_note_filename(&title));
            let path = absolute_path(workspace.join(filename))?;
            if !path.starts_with(&workspace) {
                return Err(
                    "refusing to create a note outside the configured workspace".to_string()
                );
            }
            let content = request
                .params
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("");
            let note_text = format!("# {title}\n\n{}\n", content.trim_end());
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|err| {
                    if err.kind() == io::ErrorKind::AlreadyExists {
                        format!("refusing to overwrite existing note: {}", path.display())
                    } else {
                        err.to_string()
                    }
                })?;
            file.write_all(note_text.as_bytes())
                .map_err(|err| err.to_string())?;
            Ok(json!({ "path": path, "workspace": workspace, "bytes": note_text.len() }))
        },
        verify: |_, _, data| {
            let ok = data
                .get("path")
                .and_then(Value::as_str)
                .map(Path::new)
                .is_some_and(Path::exists);
            Verification {
                ok,
                message: "Note exists in safe workspace.".to_string(),
                data: json!({}),
            }
        },
    }
}

fn audit_list_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "audit.list".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Show recent capability audit records.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["audit:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("limit".to_string(), "integer".to_string())]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| {
            let limit = request
                .params
                .get("limit")
                .and_then(Value::as_i64)
                .unwrap_or(20);
            if !(1..=200).contains(&limit) {
                return Err("audit list limit must be between 1 and 200".to_string());
            }
            let path = audit_log_path(config);
            let entries = list_audit_entries(&path, limit as usize)?;
            let entry_count = entries.len();
            Ok(json!({ "path": path, "entries": entries, "entry_count": entry_count }))
        },
        verify: |_, _, data| Verification {
            ok: data.get("entries").is_some(),
            message: "Audit entries loaded.".to_string(),
            data: json!({}),
        },
    }
}

fn object_schema(properties: BTreeMap<String, String>, required: Vec<&str>) -> Value {
    let properties = properties
        .into_iter()
        .map(|(name, kind)| (name, json!({ "type": kind })))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

fn execute_capability(
    config: &Config,
    registry: &BTreeMap<String, Capability>,
    request: ActionRequest,
) -> ActionResult {
    let started_at = utc_now();
    let Some(capability) = registry.get(&request.capability) else {
        let outcome = PolicyOutcome {
            decision: PolicyDecision::Deny,
            reason: format!("Unknown capability: {}", request.capability),
        };
        let result = base_result(
            &request,
            ActionStatus::Denied,
            &started_at,
            "Capability request denied.",
            &outcome.reason,
        );
        return record_audit(config, &request, &outcome, result);
    };

    if let Err(error) = validate_params(&capability.metadata.input_schema, &request.params) {
        let outcome = PolicyOutcome {
            decision: PolicyDecision::Deny,
            reason: error,
        };
        let result = base_result(
            &request,
            ActionStatus::Denied,
            &started_at,
            "Capability request denied.",
            &outcome.reason,
        );
        return record_audit(config, &request, &outcome, result);
    }

    let outcome = decide(capability, &request);
    if let Err(error) = ensure_audit_ready(config) {
        return ActionResult {
            action_id: request.action_id.clone(),
            capability: request.capability.clone(),
            status: ActionStatus::Failed,
            started_at,
            finished_at: utc_now(),
            summary: "Capability blocked because audit logging is unavailable.".to_string(),
            data: json!({}),
            error: Some(format!("Audit logging failed: {error}")),
            verification: Verification {
                ok: false,
                message: outcome.reason.clone(),
                data: json!({}),
            },
            audit_ref: None,
        };
    }

    match outcome.decision {
        PolicyDecision::Deny => {
            let result = base_result(
                &request,
                ActionStatus::Denied,
                &started_at,
                "Capability denied by policy.",
                &outcome.reason,
            );
            record_audit(config, &request, &outcome, result)
        }
        PolicyDecision::Confirm => {
            let result = base_result(
                &request,
                ActionStatus::ConfirmationRequired,
                &started_at,
                "Capability requires confirmation.",
                &outcome.reason,
            );
            record_audit(config, &request, &outcome, result)
        }
        PolicyDecision::Allow if request.dry_run => {
            let result = ActionResult {
                action_id: request.action_id.clone(),
                capability: request.capability.clone(),
                status: ActionStatus::DryRun,
                started_at,
                finished_at: utc_now(),
                summary: format!("Dry run: {} would execute.", request.capability),
                data: json!({ "params": request.params.clone() }),
                error: None,
                verification: Verification {
                    ok: true,
                    message: "Dry run completed without mutation.".to_string(),
                    data: json!({}),
                },
                audit_ref: None,
            };
            record_audit(config, &request, &outcome, result)
        }
        PolicyDecision::Allow => {
            let result = match (capability.execute)(&request, config) {
                Ok(data) => {
                    let verification = (capability.verify)(&request, config, &data);
                    ActionResult {
                        action_id: request.action_id.clone(),
                        capability: request.capability.clone(),
                        status: if verification.ok {
                            ActionStatus::Succeeded
                        } else {
                            ActionStatus::Failed
                        },
                        started_at,
                        finished_at: utc_now(),
                        summary: verification.message.clone(),
                        data,
                        error: None,
                        verification,
                        audit_ref: None,
                    }
                }
                Err(error) => base_result(
                    &request,
                    ActionStatus::Failed,
                    &started_at,
                    "Capability failed.",
                    &error,
                ),
            };
            record_audit(config, &request, &outcome, result)
        }
    }
}

fn base_result(
    request: &ActionRequest,
    status: ActionStatus,
    started_at: &str,
    summary: &str,
    error: &str,
) -> ActionResult {
    ActionResult {
        action_id: request.action_id.clone(),
        capability: request.capability.clone(),
        status,
        started_at: started_at.to_string(),
        finished_at: utc_now(),
        summary: summary.to_string(),
        data: json!({}),
        error: Some(error.to_string()),
        verification: Verification {
            ok: false,
            message: error.to_string(),
            data: json!({}),
        },
        audit_ref: None,
    }
}

fn validate_params(schema: &Value, params: &Map<String, Value>) -> Result<(), String> {
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            if !params.contains_key(key) {
                return Err(format!("Missing required parameter: {key}"));
            }
        }
    }
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Ok(());
    };
    for (key, value) in params {
        let Some(expected) = properties
            .get(key)
            .and_then(|definition| definition.get("type"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        match expected {
            "string" if !value.is_string() => {
                return Err(format!("Parameter {key} must be a string."))
            }
            "integer" if !value.is_i64() && !value.is_u64() => {
                return Err(format!("Parameter {key} must be an integer."));
            }
            _ => {}
        }
    }
    Ok(())
}

fn decide(capability: &Capability, request: &ActionRequest) -> PolicyOutcome {
    if request.dry_run {
        return PolicyOutcome {
            decision: PolicyDecision::Allow,
            reason: "Dry run allowed without mutation.".to_string(),
        };
    }
    if matches!(
        capability.metadata.name.as_str(),
        "fs.list" | "fs.read_text"
    ) && request.params.get("path").is_some_and(is_sensitive_path)
    {
        return PolicyOutcome {
            decision: PolicyDecision::Deny,
            reason: "Sensitive paths require a higher-risk capability.".to_string(),
        };
    }
    match capability.metadata.risk {
        RiskLevel::Read => PolicyOutcome {
            decision: PolicyDecision::Allow,
            reason: "Read-only capability allowed.".to_string(),
        },
        RiskLevel::Low => PolicyOutcome {
            decision: PolicyDecision::Allow,
            reason: "Low-risk capability allowed.".to_string(),
        },
        RiskLevel::Medium if request.confirmed => PolicyOutcome {
            decision: PolicyDecision::Allow,
            reason: "Medium-risk capability confirmed.".to_string(),
        },
        RiskLevel::Medium => PolicyOutcome {
            decision: PolicyDecision::Confirm,
            reason: "Medium-risk capability requires confirmation.".to_string(),
        },
        RiskLevel::High => PolicyOutcome {
            decision: PolicyDecision::Deny,
            reason: "High-risk capability denied by default.".to_string(),
        },
    }
}

fn ensure_audit_ready(config: &Config) -> Result<(), String> {
    let path = audit_log_path(config);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn record_audit(
    config: &Config,
    request: &ActionRequest,
    outcome: &PolicyOutcome,
    mut result: ActionResult,
) -> ActionResult {
    match append_audit(config, request, outcome, &result) {
        Ok(audit_ref) => {
            result.audit_ref = Some(audit_ref);
            result
        }
        Err(error) => {
            result.status = ActionStatus::Failed;
            result.summary = "Capability completed but audit logging failed.".to_string();
            result.error = Some(format!("Audit logging failed: {error}"));
            result.verification = Verification {
                ok: false,
                message: result.error.clone().unwrap_or_default(),
                data: json!({}),
            };
            result
        }
    }
}

fn append_audit(
    config: &Config,
    request: &ActionRequest,
    outcome: &PolicyOutcome,
    result: &ActionResult,
) -> Result<String, String> {
    let path = audit_log_path(config);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let record = json!({
        "recorded_at": utc_now(),
        "action_id": request.action_id,
        "actor": request.actor,
        "capability": request.capability,
        "input_summary": summarize_params(&request.params),
        "policy": outcome,
        "status": result.status,
        "summary": result.summary,
        "error": result.error,
        "started_at": result.started_at,
        "finished_at": result.finished_at,
        "verification": result.verification,
    });
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| err.to_string())?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&record).map_err(|err| err.to_string())?
    )
    .map_err(|err| err.to_string())?;
    Ok(format!("{}:{}", path.display(), request.action_id))
}

fn list_audit_entries(path: &Path, limit: usize) -> Result<Vec<Value>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(err) => return Err(err.to_string()),
    };
    let reader = io::BufReader::new(file);
    let mut entries = vec![];
    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            entries.push(value);
        }
    }
    if entries.len() > limit {
        Ok(entries.split_off(entries.len() - limit))
    } else {
        Ok(entries)
    }
}

fn summarize_params(params: &Map<String, Value>) -> Value {
    let summary = params
        .iter()
        .map(|(key, value)| {
            if is_sensitive_key(key) {
                (key.clone(), json!("<redacted>"))
            } else {
                (key.clone(), summarize_value(value))
            }
        })
        .collect::<Map<_, _>>();
    Value::Object(summary)
}

fn summarize_value(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    if is_sensitive_key(key) {
                        (key.clone(), json!("<redacted>"))
                    } else {
                        (key.clone(), summarize_value(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(summarize_value).collect()),
        Value::String(text) if text.len() > 120 => json!(format!("{}...", &text[..117])),
        _ => value.clone(),
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let lowered = key.to_lowercase();
    ["secret", "token", "key", "password"]
        .iter()
        .any(|marker| lowered.contains(marker))
}

fn parse_params(items: &[String], params_json: Option<&str>) -> Result<Map<String, Value>, String> {
    let mut params = Map::new();
    if let Some(raw_json) = params_json {
        let decoded: Value = serde_json::from_str(raw_json)
            .map_err(|err| format!("Invalid --params-json: {err}"))?;
        let object = decoded
            .as_object()
            .ok_or_else(|| "--params-json must decode to an object.".to_string())?;
        params.extend(object.clone());
    }
    for item in items {
        let (key, value) = item
            .split_once('=')
            .ok_or_else(|| format!("Parameter must be key=value: {item}"))?;
        if key.trim().is_empty() {
            return Err("Parameter key cannot be empty.".to_string());
        }
        params.insert(key.trim().to_string(), parse_value(value));
    }
    Ok(params)
}

fn parse_value(value: &str) -> Value {
    if value.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if value.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Ok(number) = value.parse::<i64>() {
        return json!(number);
    }
    json!(value)
}

fn string_param(request: &ActionRequest, name: &str) -> Result<String, String> {
    request
        .params
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("Parameter {name} must be a string."))
}

fn resolve_existing_path(raw: &str) -> Result<PathBuf, String> {
    if raw.trim().is_empty() {
        return Err("Path cannot be empty.".to_string());
    }
    let path = absolute_path(expand_home(raw))?;
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    Ok(fs::canonicalize(&path).unwrap_or(path))
}

fn expand_home(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let text = path.to_string_lossy();
    if text == "~" {
        return home_dir();
    }
    if let Some(rest) = text.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    path.to_path_buf()
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn absolute_path(path: impl AsRef<Path>) -> Result<PathBuf, String> {
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()
            .map_err(|err| err.to_string())?
            .join(path))
    }
}

fn absolute_path_lossy(path: impl AsRef<Path>) -> PathBuf {
    absolute_path(path).unwrap_or_else(|_| PathBuf::from("."))
}

static SAFE_FILENAME_RE: OnceLock<Regex> = OnceLock::new();

fn safe_note_filename(value: &str) -> String {
    let re = SAFE_FILENAME_RE.get_or_init(|| Regex::new(r"[^A-Za-z0-9._-]+").unwrap());
    let mut stem = re
        .replace_all(value.trim(), "-")
        .trim_matches(&['.', '-'][..])
        .to_lowercase();
    if stem.is_empty() {
        stem = "note".to_string();
    }
    if !stem.ends_with(".md") {
        stem.push_str(".md");
    }
    Path::new(&stem)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("note.md")
        .to_string()
}

static SENSITIVE_NAME_RE: OnceLock<Regex> = OnceLock::new();

fn is_sensitive_path(value: &Value) -> bool {
    let Some(path) = value.as_str() else {
        return false;
    };
    let pattern = SENSITIVE_NAME_RE.get_or_init(|| {
        Regex::new(r"(^|[._-])(api[-_]?keys?|credentials?|password|private[-_]?keys?|secret|token)([._-]|$)")
            .unwrap()
    });
    path.replace('\\', "/")
        .split('/')
        .map(str::to_lowercase)
        .any(|part| {
            matches!(
                part.as_str(),
                ".aws"
                    | ".azure"
                    | ".docker"
                    | ".gnupg"
                    | ".kube"
                    | ".password-store"
                    | ".ssh"
                    | ".env"
                    | ".npmrc"
                    | ".pypirc"
                    | "credentials"
                    | "credentials.json"
                    | "id_dsa"
                    | "id_ecdsa"
                    | "id_ed25519"
                    | "id_rsa"
            ) || pattern.is_match(&part)
        })
}

fn utc_now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(tmp: &TempDir) -> Config {
        Config {
            product: ProductConfig {
                version: "test".to_string(),
                ..ProductConfig::default()
            },
            runtime: RuntimeConfig {
                app_id: default_app_id(),
                config_env: default_config_env(),
                state_dir_env: default_state_dir_env(),
                workspace_dir_env: default_workspace_dir_env(),
                workspace_dir: Some(tmp.path().join("workspace").to_string_lossy().to_string()),
                state_dir: Some(tmp.path().join("state").to_string_lossy().to_string()),
            },
            ai: AiConfig::default(),
            features: BTreeMap::new(),
            policy: PolicyConfig::default(),
            config_path: tmp.path().join("defaults.toml"),
            product_root: tmp.path().join("product"),
        }
    }

    fn request(capability: &str, params: Map<String, Value>) -> ActionRequest {
        ActionRequest {
            action_id: Uuid::new_v4().to_string(),
            capability: capability.to_string(),
            params,
            actor: "test".to_string(),
            reason: String::new(),
            dry_run: false,
            confirmed: false,
            requested_at: utc_now(),
        }
    }

    #[test]
    fn product_status_writes_audit() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let result = execute_capability(
            &config,
            &build_registry(),
            request("product.status", Map::new()),
        );

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert!(audit_log_path(&config).exists());
    }

    #[test]
    fn sensitive_file_read_is_denied_before_reading() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let secret = tmp.path().join(".env");
        fs::write(&secret, "API_KEY=should-not-print").unwrap();
        let mut params = Map::new();
        params.insert("path".to_string(), json!(secret));

        let result =
            execute_capability(&config, &build_registry(), request("fs.read_text", params));
        let output = serde_json::to_string(&result).unwrap();

        assert_eq!(result.status, ActionStatus::Denied);
        assert!(!output.contains("should-not-print"));
    }

    #[test]
    fn note_dry_run_does_not_write_file() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("title".to_string(), json!("Phase Two"));
        let mut req = request("notes.create", params);
        req.dry_run = true;

        let result = execute_capability(&config, &build_registry(), req);

        assert_eq!(result.status, ActionStatus::DryRun);
        assert!(!workspace_dir(&config).join("phase-two.md").exists());
    }

    #[test]
    fn audit_summary_redacts_nested_secret_values() {
        let mut nested = Map::new();
        nested.insert("api_key".to_string(), json!("nope"));
        nested.insert("safe".to_string(), json!("ok"));
        let mut params = Map::new();
        params.insert("nested".to_string(), Value::Object(nested));

        let summary = summarize_params(&params);

        assert_eq!(summary["nested"]["api_key"], json!("<redacted>"));
        assert_eq!(summary["nested"]["safe"], json!("ok"));
    }

    #[test]
    fn audit_unavailable_blocks_execution_without_panic() {
        let tmp = TempDir::new().unwrap();
        let mut config = test_config(&tmp);
        let blocker = tmp.path().join("not-a-dir");
        fs::write(&blocker, "x").unwrap();
        config.runtime.state_dir = Some(blocker.to_string_lossy().to_string());

        let result = execute_capability(
            &config,
            &build_registry(),
            request("product.status", Map::new()),
        );

        assert_eq!(result.status, ActionStatus::Failed);
        assert!(result.error.unwrap().contains("Audit logging failed"));
    }

    #[test]
    fn local_planner_maps_status_prompt_to_capability() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let plan = build_ai_plan(
            &config,
            &build_registry(),
            "show product status",
            Some("local.rules"),
        )
        .unwrap();

        assert!(plan.executable);
        assert_eq!(plan.steps[0].capability, "product.status");
    }

    #[test]
    fn local_planner_lists_current_directory_when_no_path_given() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let plan = build_ai_plan(
            &config,
            &build_registry(),
            "list files",
            Some("local.rules"),
        )
        .unwrap();

        assert!(plan.executable);
        assert_eq!(plan.steps[0].capability, "fs.list");
        assert_eq!(plan.steps[0].params["path"], json!("."));
    }

    #[test]
    fn ai_run_executes_through_capability_engine() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();
        let plan = build_ai_plan(
            &config,
            &registry,
            "show product status",
            Some("local.rules"),
        )
        .unwrap();
        let options = AiOptions {
            actor: "test.ai".to_string(),
            ..AiOptions::default()
        };

        let report = execute_ai_plan(&config, &registry, plan, &options);

        assert_eq!(report.status, "succeeded");
        assert_eq!(report.results[0].status, ActionStatus::Succeeded);
        assert!(audit_log_path(&config).exists());
    }

    #[test]
    fn secret_status_redacts_present_values() {
        let spec = SecretSpec {
            name: "openai_api_key".to_string(),
            env_var: "HUGGINGOS_OPENAI_API_KEY".to_string(),
            required_for: vec!["cloud.openai".to_string()],
        };

        let status = secret_status_from_value(spec, Some("provider-test-value-should-not-leak"));
        let output = serde_json::to_string(&status).unwrap();

        assert!(status.present);
        assert!(status.redacted);
        assert!(!output.contains("provider-test-value-should-not-leak"));
    }

    #[test]
    fn unknown_prompt_does_not_execute() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();
        let plan = build_ai_plan(
            &config,
            &registry,
            "make the computer magically do everything",
            Some("local.rules"),
        )
        .unwrap();
        let report = execute_ai_plan(&config, &registry, plan, &AiOptions::default());

        assert_eq!(report.status, "no_plan");
        assert!(report.results.is_empty());
        assert!(!audit_log_path(&config).exists());
    }
}
