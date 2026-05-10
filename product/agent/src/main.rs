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
use std::process::{Command, ExitCode, Stdio};
use std::sync::OnceLock;
use uuid::Uuid;

const MAX_TEXT_BYTES: u64 = 64 * 1024;
const MAX_OCR_IMAGE_BYTES: u64 = 10 * 1024 * 1024;

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
            phase: "Product Phase 5".to_string(),
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
struct PrivacyConfig {
    #[serde(default = "default_private_title_markers")]
    private_title_markers: Vec<String>,
    #[serde(default = "default_private_app_markers")]
    private_app_markers: Vec<String>,
    #[serde(default = "default_max_context_text_chars")]
    max_context_text_chars: usize,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            private_title_markers: default_private_title_markers(),
            private_app_markers: default_private_app_markers(),
            max_context_text_chars: default_max_context_text_chars(),
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
    privacy: PrivacyConfig,
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
    privacy: PrivacyConfig,
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

#[derive(Clone, Debug)]
struct DesktopEntry {
    id: String,
    name: String,
    exec: Option<String>,
    path: PathBuf,
    categories: Vec<String>,
    no_display: bool,
    hidden: bool,
}

#[derive(Clone, Debug)]
struct ActiveContext {
    backend: Option<String>,
    title: Option<String>,
    pid: Option<u32>,
    app: Option<String>,
    is_private: bool,
    privacy_reason: Option<String>,
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
        privacy: file_config.privacy,
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
        "desktop": desktop_status(),
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

fn default_private_title_markers() -> Vec<String> {
    vec![
        "password".to_string(),
        "secret".to_string(),
        "token".to_string(),
        "credential".to_string(),
        "private".to_string(),
        "incognito".to_string(),
        "bank".to_string(),
        "vault".to_string(),
        "2fa".to_string(),
        "otp".to_string(),
    ]
}

fn default_private_app_markers() -> Vec<String> {
    vec![
        "password".to_string(),
        "secret".to_string(),
        "credential".to_string(),
        "vault".to_string(),
        "bank".to_string(),
    ]
}

fn default_max_context_text_chars() -> usize {
    240
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
    let lowered = prompt.trim().to_ascii_lowercase();
    let mut steps = vec![];
    let mut warnings = vec![];

    if let Some(step) = plan_workspace_mode_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_browser_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_app_launch_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_app_list_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_desktop_status_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_context_snapshot_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_screen_capture_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_screen_ocr_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_screen_status_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_audit_intent(registry, prompt, &lowered) {
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

fn plan_context_snapshot_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("what is open")
        || lowered.contains("active window")
        || lowered.contains("current window")
        || lowered.contains("context snapshot")
        || lowered.contains("current context"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "context.snapshot",
        Map::new(),
        format!("Inspect active context for prompt: {prompt}"),
    ))
}

fn plan_screen_capture_intent(
    registry: &BTreeMap<String, Capability>,
    _prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("screenshot")
        || lowered.contains("screen shot")
        || lowered.contains("capture screen")
        || lowered.contains("capture screenshot"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "screen.capture",
        Map::new(),
        "Capture the screen through a permissioned desktop backend.".to_string(),
    ))
}

fn plan_screen_ocr_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.starts_with("ocr ")
        || lowered.contains("read text from image")
        || lowered.contains("extract text from image"))
    {
        return None;
    }
    let path = extract_path_after(
        prompt,
        &[
            "read text from image ",
            "extract text from image ",
            "ocr image ",
            "ocr ",
        ],
    )?;
    let mut params = Map::new();
    params.insert("path".to_string(), json!(path));
    Some(plan_step(
        registry,
        "screen.ocr_image",
        params,
        "Extract text from a user-approved image with an OCR backend.".to_string(),
    ))
}

fn plan_screen_status_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("screen status")
        || lowered.contains("screen readiness")
        || lowered.contains("capture readiness")
        || lowered.contains("ocr status"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "screen.status",
        Map::new(),
        format!("Report screen/context readiness for prompt: {prompt}"),
    ))
}

fn plan_workspace_mode_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("workspace mode")
        || lowered.contains("coding mode")
        || lowered.contains("study mode")
        || lowered.contains("deep work")
        || lowered.contains("deep-work")
        || lowered.contains("gaming mode")
        || lowered.contains("travel mode"))
    {
        return None;
    }
    let mode = extract_workspace_mode(prompt)?;
    let mut params = Map::new();
    params.insert("mode".to_string(), json!(mode));
    Some(plan_step(
        registry,
        "workspace.mode.plan",
        params,
        "Preview a workspace mode before changing desktop state.".to_string(),
    ))
}

fn plan_browser_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("http://") || lowered.contains("https://")) {
        return None;
    }
    if !(lowered.starts_with("open ")
        || lowered.starts_with("browse ")
        || lowered.contains("browser"))
    {
        return None;
    }
    let url = extract_url(prompt)?;
    let mut params = Map::new();
    params.insert("url".to_string(), json!(url));
    Some(plan_step(
        registry,
        "browser.open_url",
        params,
        "Open a user-requested URL through the desktop browser backend.".to_string(),
    ))
}

fn plan_app_launch_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.starts_with("open app ")
        || lowered.starts_with("launch app ")
        || lowered.starts_with("start app "))
    {
        return None;
    }
    let app_id = extract_path_after(prompt, &["open app ", "launch app ", "start app "])?;
    let app_id = normalize_desktop_id(&app_id);
    let mut params = Map::new();
    params.insert("app_id".to_string(), json!(app_id));
    Some(plan_step(
        registry,
        "apps.launch",
        params,
        "Launch a user-requested desktop application after confirmation.".to_string(),
    ))
}

fn plan_app_list_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.starts_with("list apps")
        || lowered.starts_with("show apps")
        || lowered.starts_with("list applications")
        || lowered.starts_with("show applications")
        || lowered.contains("installed apps")
        || lowered.contains("installed applications"))
    {
        return None;
    }
    let mut params = Map::new();
    if let Some(query) = extract_path_after(prompt, &["search apps ", "find apps "]) {
        params.insert("query".to_string(), json!(query));
    }
    Some(plan_step(
        registry,
        "apps.list",
        params,
        "List installed desktop applications.".to_string(),
    ))
}

fn plan_desktop_status_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("desktop status")
        || lowered.contains("desktop readiness")
        || lowered.contains("gui status")
        || lowered.contains("app control status"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "desktop.status",
        Map::new(),
        format!("Report desktop readiness for prompt: {prompt}"),
    ))
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

fn extract_url(prompt: &str) -> Option<String> {
    prompt
        .split_whitespace()
        .map(clean_prompt_value)
        .find(|part| part.starts_with("http://") || part.starts_with("https://"))
}

fn extract_workspace_mode(prompt: &str) -> Option<String> {
    let lowered = prompt.to_ascii_lowercase();
    for mode in [
        "deep-work",
        "deep work",
        "coding",
        "study",
        "gaming",
        "travel",
    ] {
        if lowered.contains(mode) {
            return Some(mode.replace(' ', "-"));
        }
    }
    extract_path_after(prompt, &["workspace mode ", "mode "])
}

fn normalize_desktop_id(value: &str) -> String {
    let mut app_id = clean_prompt_value(value).replace(' ', "-");
    if !app_id.ends_with(".desktop") {
        app_id.push_str(".desktop");
    }
    app_id
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

fn desktop_status() -> Value {
    let wayland_display = env::var("WAYLAND_DISPLAY").ok();
    let x11_display = env::var("DISPLAY").ok();
    let has_graphical_session = wayland_display
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
        || x11_display
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
    json!({
        "session": {
            "current_desktop": env::var("XDG_CURRENT_DESKTOP").ok(),
            "session_desktop": env::var("XDG_SESSION_DESKTOP").ok(),
            "session_type": env::var("XDG_SESSION_TYPE").ok(),
            "wayland_display": wayland_display,
            "display": x11_display,
            "dbus_session_bus": env::var("DBUS_SESSION_BUS_ADDRESS").ok().map(|_| "<present>"),
            "has_graphical_session": has_graphical_session,
            "is_wsl": env::var("WSL_DISTRO_NAME").ok().is_some()
                || env::var("WSL_INTEROP").ok().is_some(),
        },
        "tools": {
            "xdg_open": find_command("xdg-open"),
            "gio": find_command("gio"),
            "gtk_launch": find_command("gtk-launch"),
        },
        "app_registry": {
            "directories": desktop_entry_dirs(),
        }
    })
}

fn screen_status(config: &Config) -> Value {
    json!({
        "desktop": desktop_status(),
        "capture": {
            "backends": screen_capture_backends(),
            "output_dir": screen_capture_dir(config),
            "requires_confirmation": true,
        },
        "active_context": {
            "backends": active_context_backends(),
            "requires_confirmation": true,
        },
        "ocr": {
            "backends": ocr_backends(),
            "requires_confirmation": true,
            "max_image_bytes": MAX_OCR_IMAGE_BYTES,
        },
        "clipboard": {
            "backends": clipboard_backends(),
            "content_collection": "not_enabled_in_phase5",
        },
        "privacy": privacy_status(config),
    })
}

fn ensure_desktop_session_ready() -> Result<(), String> {
    let has_display = env::var("WAYLAND_DISPLAY")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
        || env::var("DISPLAY")
            .ok()
            .is_some_and(|value| !value.trim().is_empty());
    if has_display {
        Ok(())
    } else {
        Err(
            "No graphical desktop session detected; use --dry-run in headless CI/WSL or run from a Linux desktop."
                .to_string(),
        )
    }
}

fn screen_capture_backends() -> Vec<Value> {
    [
        ("grim", "wayland_screenshot"),
        ("gnome-screenshot", "gnome_screenshot"),
        ("spectacle", "kde_screenshot"),
        ("scrot", "x11_screenshot"),
        ("import", "imagemagick_x11_screenshot"),
    ]
    .into_iter()
    .map(|(command, kind)| {
        json!({
            "command": command,
            "kind": kind,
            "path": find_command(command),
            "available": find_command(command).is_some(),
        })
    })
    .collect()
}

fn active_context_backends() -> Vec<Value> {
    [("xdotool", "x11_active_window")]
        .into_iter()
        .map(|(command, kind)| {
            json!({
                "command": command,
                "kind": kind,
                "path": find_command(command),
                "available": find_command(command).is_some(),
            })
        })
        .collect()
}

fn ocr_backends() -> Vec<Value> {
    [("tesseract", "ocr_engine")]
        .into_iter()
        .map(|(command, kind)| {
            json!({
                "command": command,
                "kind": kind,
                "path": find_command(command),
                "available": find_command(command).is_some(),
            })
        })
        .collect()
}

fn clipboard_backends() -> Vec<Value> {
    [
        ("wl-paste", "wayland_clipboard"),
        ("xclip", "x11_clipboard"),
        ("xsel", "x11_clipboard"),
    ]
    .into_iter()
    .map(|(command, kind)| {
        json!({
            "command": command,
            "kind": kind,
            "path": find_command(command),
            "available": find_command(command).is_some(),
        })
    })
    .collect()
}

fn privacy_status(config: &Config) -> Value {
    json!({
        "private_title_markers": config.privacy.private_title_markers,
        "private_app_markers": config.privacy.private_app_markers,
        "max_context_text_chars": config.privacy.max_context_text_chars,
        "clipboard_content_collection": "disabled",
    })
}

fn screen_capture_dir(config: &Config) -> PathBuf {
    absolute_path_lossy(workspace_dir(config).join("screenshots"))
}

fn capture_screenshot(config: &Config, filename: Option<&str>) -> Result<Value, String> {
    ensure_desktop_session_ready()?;
    let active = active_context_snapshot(config);
    ensure_context_observable(&active)?;

    let output_dir = screen_capture_dir(config);
    fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
    let filename = filename
        .map(safe_capture_filename)
        .unwrap_or_else(|| format!("screenshot-{}.png", Uuid::new_v4()));
    let path = absolute_path(output_dir.join(filename))?;
    if !path.starts_with(&output_dir) {
        return Err("refusing to write screenshot outside the capture workspace".to_string());
    }

    let backend = capture_screenshot_to(&path)?;
    let metadata = fs::metadata(&path).map_err(|err| err.to_string())?;
    if metadata.len() == 0 {
        return Err("screenshot backend wrote an empty file".to_string());
    }
    Ok(json!({
        "captured": true,
        "path": path,
        "bytes": metadata.len(),
        "backend": backend,
        "active_context": active_context_json(&active),
    }))
}

fn capture_screenshot_to(path: &Path) -> Result<String, String> {
    let output = path.to_string_lossy().to_string();
    if find_command("grim").is_some() {
        run_command_owned("grim", std::slice::from_ref(&output))?;
        return Ok("grim".to_string());
    }
    if find_command("gnome-screenshot").is_some() {
        run_command_owned("gnome-screenshot", &["-f".to_string(), output.clone()])?;
        return Ok("gnome-screenshot".to_string());
    }
    if find_command("spectacle").is_some() {
        run_command_owned(
            "spectacle",
            &[
                "-b".to_string(),
                "-n".to_string(),
                "-o".to_string(),
                output.clone(),
            ],
        )?;
        return Ok("spectacle".to_string());
    }
    if find_command("scrot").is_some() {
        run_command_owned("scrot", std::slice::from_ref(&output))?;
        return Ok("scrot".to_string());
    }
    if find_command("import").is_some() {
        run_command_owned(
            "import",
            &["-window".to_string(), "root".to_string(), output],
        )?;
        return Ok("import".to_string());
    }
    Err("No screenshot backend found; install grim, gnome-screenshot, spectacle, scrot, or ImageMagick import.".to_string())
}

fn active_context_snapshot(config: &Config) -> ActiveContext {
    let mut context = if find_command("xdotool").is_some() {
        active_context_from_xdotool()
    } else {
        ActiveContext {
            backend: None,
            title: None,
            pid: None,
            app: None,
            is_private: false,
            privacy_reason: None,
        }
    };
    apply_privacy_policy(config, &mut context);
    context
}

fn active_context_from_xdotool() -> ActiveContext {
    let window_id = command_output("xdotool", &["getactivewindow"]).ok();
    let title = window_id
        .as_deref()
        .and_then(|id| command_output("xdotool", &["getwindowname", id]).ok())
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty());
    let pid = window_id
        .as_deref()
        .and_then(|id| command_output("xdotool", &["getwindowpid", id]).ok())
        .and_then(|pid| pid.trim().parse::<u32>().ok());
    let app = pid.and_then(process_name);
    ActiveContext {
        backend: Some("xdotool".to_string()),
        title,
        pid,
        app,
        is_private: false,
        privacy_reason: None,
    }
}

fn apply_privacy_policy(config: &Config, context: &mut ActiveContext) {
    let title_marker = context
        .title
        .as_deref()
        .and_then(|title| matches_marker(title, &config.privacy.private_title_markers));
    let app_marker = context
        .app
        .as_deref()
        .and_then(|app| matches_marker(app, &config.privacy.private_app_markers));

    if let Some(marker) = title_marker {
        context.is_private = true;
        context.privacy_reason = Some(format!("active title matched private marker: {marker}"));
    } else if let Some(marker) = app_marker {
        context.is_private = true;
        context.privacy_reason = Some(format!("active app matched private marker: {marker}"));
    }

    if context.is_private {
        if context.title.is_some() {
            context.title = Some("<redacted>".to_string());
        }
        if context.app.is_some() {
            context.app = Some("<redacted>".to_string());
        }
        return;
    }

    if let Some(title) = &context.title {
        context.title = Some(truncate_text(title, config.privacy.max_context_text_chars));
    }
    if let Some(app) = &context.app {
        context.app = Some(truncate_text(app, config.privacy.max_context_text_chars));
    }
}

fn ensure_context_observable(context: &ActiveContext) -> Result<(), String> {
    if context.is_private {
        return Err(context
            .privacy_reason
            .clone()
            .unwrap_or_else(|| "active context is excluded by privacy policy".to_string()));
    }
    if context.backend.is_none() {
        return Err(
            "Active context backend unavailable; refusing confirmed screen capture without privacy context."
                .to_string(),
        );
    }
    if context.title.is_none() && context.app.is_none() {
        return Err(
            "Active context metadata unavailable; refusing confirmed screen capture without privacy context."
                .to_string(),
        );
    }
    Ok(())
}

fn active_context_json(context: &ActiveContext) -> Value {
    json!({
        "backend": context.backend.clone(),
        "title": context.title.clone(),
        "pid": context.pid,
        "app": context.app.clone(),
        "is_private": context.is_private,
        "privacy_reason": context.privacy_reason.clone(),
    })
}

fn context_snapshot(config: &Config) -> Value {
    let active = active_context_snapshot(config);
    json!({
        "desktop": desktop_status(),
        "screen": {
            "status": screen_status(config),
        },
        "active_window": active_context_json(&active),
        "clipboard": {
            "available": clipboard_backends()
                .iter()
                .any(|backend| backend["available"].as_bool().unwrap_or(false)),
            "collected": false,
            "reason": "Clipboard content collection is disabled in Phase 5.",
        },
        "privacy": privacy_status(config),
    })
}

fn ocr_image(config: &Config, path: &str) -> Result<Value, String> {
    let value = json!(path);
    if is_sensitive_path(&value) {
        return Err("Sensitive paths require a higher-risk capability.".to_string());
    }
    let path = resolve_existing_path(path)?;
    let metadata = fs::metadata(&path).map_err(|err| err.to_string())?;
    if !metadata.is_file() {
        return Err(format!(
            "OCR input is not a regular file: {}",
            path.display()
        ));
    }
    if metadata.len() > MAX_OCR_IMAGE_BYTES {
        return Err(format!(
            "OCR input is too large for Phase 5: {} bytes",
            metadata.len()
        ));
    }
    if find_command("tesseract").is_none() {
        return Err("No OCR backend found; install tesseract to use screen.ocr_image.".to_string());
    }
    let text = command_output("tesseract", &[path.to_string_lossy().as_ref(), "stdout"])?;
    Ok(json!({
        "path": path,
        "backend": "tesseract",
        "text": truncate_text(&text, config.privacy.max_context_text_chars),
        "text_length": text.chars().count(),
        "truncated": text.chars().count() > config.privacy.max_context_text_chars,
    }))
}

fn process_name(pid: u32) -> Option<String> {
    let path = PathBuf::from("/proc").join(pid.to_string()).join("comm");
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn matches_marker(value: &str, markers: &[String]) -> Option<String> {
    let lowered = value.to_ascii_lowercase();
    markers
        .iter()
        .map(|marker| marker.trim().to_ascii_lowercase())
        .filter(|marker| !marker.is_empty())
        .find(|marker| lowered.contains(marker))
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    let max_chars = max_chars.max(1);
    let mut text = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        text.push_str("...");
    }
    text
}

fn safe_capture_filename(value: &str) -> String {
    let re = SAFE_FILENAME_RE.get_or_init(|| Regex::new(r"[^A-Za-z0-9._-]+").unwrap());
    let mut stem = re
        .replace_all(value.trim(), "-")
        .trim_matches(&['.', '-'][..])
        .to_lowercase();
    if stem.is_empty() {
        stem = format!("screenshot-{}", Uuid::new_v4());
    }
    if !stem.ends_with(".png") {
        stem.push_str(".png");
    }
    Path::new(&stem)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("screenshot.png")
        .to_string()
}

fn desktop_entry_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![];
    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(xdg_data_home).join("applications"));
    } else {
        dirs.push(home_dir().join(".local").join("share").join("applications"));
    }

    let data_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    for root in data_dirs
        .split(':')
        .filter(|value| !value.trim().is_empty())
    {
        dirs.push(PathBuf::from(root).join("applications"));
    }
    dedupe_paths(dirs)
}

fn discover_desktop_entries() -> Result<Vec<DesktopEntry>, String> {
    discover_desktop_entries_in(&desktop_entry_dirs())
}

fn discover_desktop_entries_in(dirs: &[PathBuf]) -> Result<Vec<DesktopEntry>, String> {
    let mut entries = BTreeMap::new();
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        collect_desktop_entries(dir, dir, &mut entries)?;
    }
    Ok(entries.into_values().collect())
}

fn collect_desktop_entries(
    root: &Path,
    current: &Path,
    entries: &mut BTreeMap<String, DesktopEntry>,
) -> Result<(), String> {
    let Ok(read_dir) = fs::read_dir(current) else {
        return Ok(());
    };
    for entry in read_dir {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_desktop_entries(root, &path, entries)?;
        } else if path.extension().and_then(OsStr::to_str) == Some("desktop") {
            let raw = fs::read_to_string(&path).unwrap_or_default();
            if let Some(parsed) = parse_desktop_entry(root, &path, &raw) {
                entries.entry(parsed.id.clone()).or_insert(parsed);
            }
        }
    }
    Ok(())
}

fn parse_desktop_entry(root: &Path, path: &Path, raw: &str) -> Option<DesktopEntry> {
    let mut in_desktop_entry = false;
    let mut fields = BTreeMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        fields
            .entry(key.trim().to_string())
            .or_insert_with(|| value.trim().to_string());
    }

    if fields.get("Type").map(String::as_str) != Some("Application") {
        return None;
    }

    let id = desktop_id_from_path(root, path)?;
    let name = fields
        .get("Name")
        .cloned()
        .unwrap_or_else(|| id.trim_end_matches(".desktop").to_string());
    let exec = fields.get("Exec").cloned();
    let categories = fields
        .get("Categories")
        .map(|value| {
            value
                .split(';')
                .filter(|item| !item.trim().is_empty())
                .map(|item| item.trim().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(DesktopEntry {
        id,
        name,
        exec,
        path: absolute_path_lossy(path),
        categories,
        no_display: desktop_bool(fields.get("NoDisplay")),
        hidden: desktop_bool(fields.get("Hidden")),
    })
}

fn desktop_id_from_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let text = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("-");
    if text.ends_with(".desktop") {
        Some(text)
    } else {
        None
    }
}

fn desktop_bool(value: Option<&String>) -> bool {
    value.is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn desktop_entry_json(entry: &DesktopEntry) -> Value {
    json!({
        "id": entry.id,
        "name": entry.name,
        "exec": entry.exec,
        "path": entry.path,
        "categories": entry.categories,
        "no_display": entry.no_display,
        "hidden": entry.hidden,
    })
}

fn find_desktop_entry(app_id: &str) -> Result<Option<DesktopEntry>, String> {
    Ok(discover_desktop_entries()?
        .into_iter()
        .find(|entry| entry.id == app_id))
}

fn ensure_launchable_desktop_entry(entry: &DesktopEntry) -> Result<(), String> {
    if entry.hidden || entry.no_display {
        Err(format!(
            "refusing to launch hidden or non-display desktop entry: {}",
            entry.id
        ))
    } else {
        Ok(())
    }
}

fn launch_desktop_entry(entry: &DesktopEntry) -> Result<String, String> {
    if find_command("gio").is_some() {
        run_command("gio", &["launch", entry.path.to_string_lossy().as_ref()])?;
        return Ok("gio launch".to_string());
    }
    if find_command("gtk-launch").is_some() {
        let launcher_id = entry.id.trim_end_matches(".desktop");
        run_command("gtk-launch", &[launcher_id])?;
        return Ok("gtk-launch".to_string());
    }
    Err("No desktop launch backend found; install gio or gtk-launch.".to_string())
}

fn open_browser_url(url: &str) -> Result<String, String> {
    if find_command("xdg-open").is_some() {
        run_command("xdg-open", &[url])?;
        return Ok("xdg-open".to_string());
    }
    if find_command("gio").is_some() {
        run_command("gio", &["open", url])?;
        return Ok("gio open".to_string());
    }
    Err("No browser open backend found; install xdg-open or gio.".to_string())
}

fn run_command(command: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| format!("failed to start {command}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command} exited with status {status}"))
    }
}

fn run_command_owned(command: &str, args: &[String]) -> Result<(), String> {
    let status = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| format!("failed to start {command}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command} exited with status {status}"))
    }
}

fn command_output(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| format!("failed to start {command}: {err}"))?;
    if !output.status.success() {
        return Err(format!("{command} exited with status {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("{command} returned non-UTF-8 output"))
}

fn validate_desktop_id(app_id: &str) -> Result<(), String> {
    let app_id = app_id.trim();
    if app_id.is_empty() {
        return Err("desktop app id cannot be empty".to_string());
    }
    if !app_id.ends_with(".desktop") {
        return Err("desktop app id must end with .desktop".to_string());
    }
    if app_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        Ok(())
    } else {
        Err(
            "desktop app id may only contain letters, numbers, dots, dashes, and underscores"
                .to_string(),
        )
    }
}

fn validate_browser_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.starts_with("http://") || url.starts_with("https://") {
        if url.len() <= 2048 && !url.chars().any(char::is_whitespace) {
            return Ok(());
        }
        return Err("browser URL must be at most 2048 characters with no whitespace".to_string());
    }
    Err("browser.open_url only accepts http:// or https:// URLs".to_string())
}

fn workspace_mode_plan(mode: &str) -> Result<Value, String> {
    let mode = validate_workspace_mode(mode)?;
    let (summary, suggested_apps, policy) = match mode {
        "coding" => (
            "Prepare a focused development workspace.",
            vec![
                "code.desktop",
                "org.gnome.Terminal.desktop",
                "firefox.desktop",
            ],
            "Open editor, terminal, and browser after user confirmation.",
        ),
        "study" => (
            "Prepare a reading and notes workspace.",
            vec![
                "org.gnome.Evince.desktop",
                "org.gnome.TextEditor.desktop",
                "firefox.desktop",
            ],
            "Open reading, note-taking, and research tools after user confirmation.",
        ),
        "deep-work" => (
            "Prepare a low-distraction workspace.",
            vec!["org.gnome.TextEditor.desktop", "org.gnome.Terminal.desktop"],
            "Prefer local tools and keep browser launch optional.",
        ),
        "gaming" => (
            "Prepare a gaming workspace.",
            vec!["steam.desktop"],
            "Launch game clients only with explicit confirmation.",
        ),
        "travel" => (
            "Prepare a travel workspace.",
            vec!["firefox.desktop", "org.gnome.Maps.desktop"],
            "Open browser and maps only with explicit confirmation.",
        ),
        _ => unreachable!(),
    };
    Ok(json!({
        "mode": mode,
        "summary": summary,
        "suggested_apps": suggested_apps,
        "policy": policy,
        "steps": [
            {
                "capability": "desktop.status",
                "params": {},
                "reason": "Check desktop readiness before changing workspace state."
            },
            {
                "capability": "apps.list",
                "params": {},
                "reason": "Inspect available desktop entries before choosing app launches."
            },
            {
                "capability": "apps.launch",
                "params": { "app_id": "<chosen-app.desktop>" },
                "reason": "Launch selected apps with confirmation after the user reviews the plan."
            }
        ]
    }))
}

fn validate_workspace_mode(mode: &str) -> Result<&'static str, String> {
    match mode.trim().to_ascii_lowercase().replace('_', "-").as_str() {
        "coding" | "code" | "development" | "dev" => Ok("coding"),
        "study" | "learning" | "learn" => Ok("study"),
        "deep-work" | "deepwork" | "focus" | "focused" => Ok("deep-work"),
        "gaming" | "game" => Ok("gaming"),
        "travel" | "trip" => Ok("travel"),
        _ => Err(
            "workspace mode must be one of: coding, study, deep-work, gaming, travel".to_string(),
        ),
    }
}

fn find_command(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = BTreeMap::new();
    for path in paths {
        seen.entry(path.to_string_lossy().to_string())
            .or_insert(path);
    }
    seen.into_values().collect()
}

fn build_registry() -> BTreeMap<String, Capability> {
    let capabilities = [
        product_status_capability(),
        fs_list_capability(),
        fs_read_text_capability(),
        notes_create_capability(),
        audit_list_capability(),
        desktop_status_capability(),
        apps_list_capability(),
        apps_launch_capability(),
        browser_open_url_capability(),
        workspace_mode_plan_capability(),
        screen_status_capability(),
        screen_capture_capability(),
        context_snapshot_capability(),
        screen_ocr_image_capability(),
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

fn desktop_status_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "desktop.status".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Report Linux desktop session and backend readiness.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["desktop:read".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, _| Ok(desktop_status()),
        verify: |_, _, data| Verification {
            ok: data.get("session").is_some() && data.get("tools").is_some(),
            message: "Desktop status returned host session readiness.".to_string(),
            data: json!({}),
        },
    }
}

fn apps_list_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "apps.list".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "List installed Linux desktop applications from .desktop entries."
                .to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["desktop:read".to_string(), "apps:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("query".to_string(), "string".to_string())]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, _| {
            let query = request
                .params
                .get("query")
                .and_then(Value::as_str)
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty());
            let mut apps = discover_desktop_entries()?
                .into_iter()
                .filter(|entry| !entry.hidden && !entry.no_display)
                .filter(|entry| match &query {
                    Some(query) => {
                        entry.id.to_ascii_lowercase().contains(query)
                            || entry.name.to_ascii_lowercase().contains(query)
                            || entry
                                .categories
                                .iter()
                                .any(|category| category.to_ascii_lowercase().contains(query))
                    }
                    None => true,
                })
                .map(|entry| desktop_entry_json(&entry))
                .collect::<Vec<_>>();
            apps.sort_by_key(|entry| entry["name"].as_str().unwrap_or("").to_ascii_lowercase());
            let app_count = apps.len();
            Ok(json!({
                "apps": apps,
                "app_count": app_count,
                "query": query,
                "desktop": desktop_status(),
            }))
        },
        verify: |_, _, data| Verification {
            ok: data.get("apps").is_some(),
            message: "Desktop application registry loaded.".to_string(),
            data: json!({}),
        },
    }
}

fn apps_launch_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "apps.launch".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Launch an installed desktop application by .desktop ID.".to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["desktop:control".to_string(), "apps:launch".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("app_id".to_string(), "string".to_string())]),
                vec!["app_id"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, _| {
            let app_id = string_param(request, "app_id")?;
            validate_desktop_id(&app_id)?;
            let entry = find_desktop_entry(&app_id)?
                .ok_or_else(|| format!("desktop app not found: {app_id}"))?;
            ensure_launchable_desktop_entry(&entry)?;
            ensure_desktop_session_ready()?;
            let backend = launch_desktop_entry(&entry)?;
            Ok(json!({
                "launched": true,
                "app": desktop_entry_json(&entry),
                "backend": backend,
            }))
        },
        verify: |_, _, data| {
            let ok = data.get("launched") == Some(&json!(true));
            Verification {
                ok,
                message: if ok {
                    "Desktop launch command completed.".to_string()
                } else {
                    "Desktop launch was not confirmed.".to_string()
                },
                data: json!({}),
            }
        },
    }
}

fn browser_open_url_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "browser.open_url".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Open an HTTP or HTTPS URL through the Linux desktop browser backend."
                .to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["desktop:control".to_string(), "browser:open".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("url".to_string(), "string".to_string())]),
                vec!["url"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, _| {
            let url = string_param(request, "url")?;
            validate_browser_url(&url)?;
            ensure_desktop_session_ready()?;
            let backend = open_browser_url(&url)?;
            Ok(json!({
                "opened": true,
                "url": url,
                "backend": backend,
            }))
        },
        verify: |_, _, data| {
            let ok = data.get("opened") == Some(&json!(true));
            Verification {
                ok,
                message: if ok {
                    "Browser open command completed.".to_string()
                } else {
                    "Browser open was not confirmed.".to_string()
                },
                data: json!({}),
            }
        },
    }
}

fn workspace_mode_plan_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "workspace.mode.plan".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Preview a workspace mode as explicit future capability steps."
                .to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["workspace:plan".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("mode".to_string(), "string".to_string())]),
                vec!["mode"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, _| {
            let mode = string_param(request, "mode")?;
            let plan = workspace_mode_plan(&mode)?;
            Ok(plan)
        },
        verify: |_, _, data| {
            let ok = data.get("mode").is_some() && data.get("steps").is_some();
            Verification {
                ok,
                message: "Workspace mode plan is inspectable.".to_string(),
                data: json!({}),
            }
        },
    }
}

fn screen_status_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "screen.status".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Report screen capture, OCR, context, and privacy readiness.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["screen:readiness".to_string(), "privacy:read".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, config| Ok(screen_status(config)),
        verify: |_, _, data| Verification {
            ok: data.get("capture").is_some()
                && data.get("active_context").is_some()
                && data.get("privacy").is_some(),
            message: "Screen/context readiness returned.".to_string(),
            data: json!({}),
        },
    }
}

fn screen_capture_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "screen.capture".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description:
                "Capture a screenshot through a real desktop backend into the safe workspace."
                    .to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["screen:capture".to_string(), "privacy:observe".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("filename".to_string(), "string".to_string())]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: true,
        },
        execute: |request, config| {
            let filename = request.params.get("filename").and_then(Value::as_str);
            capture_screenshot(config, filename)
        },
        verify: |_, _, data| {
            let ok = data
                .get("path")
                .and_then(Value::as_str)
                .map(Path::new)
                .is_some_and(Path::exists);
            Verification {
                ok,
                message: if ok {
                    "Screenshot exists in safe workspace.".to_string()
                } else {
                    "Screenshot output was not verified.".to_string()
                },
                data: json!({}),
            }
        },
    }
}

fn context_snapshot_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "context.snapshot".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Return active desktop/window context metadata with privacy redaction."
                .to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["context:read".to_string(), "privacy:observe".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, config| Ok(context_snapshot(config)),
        verify: |_, _, data| Verification {
            ok: data.get("active_window").is_some() && data.get("privacy").is_some(),
            message: "Context snapshot returned with privacy state.".to_string(),
            data: json!({}),
        },
    }
}

fn screen_ocr_image_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "screen.ocr_image".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Extract text from a user-approved image through a real OCR backend."
                .to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["screen:ocr".to_string(), "privacy:observe".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("path".to_string(), "string".to_string())]),
                vec!["path"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| {
            let path = string_param(request, "path")?;
            ocr_image(config, &path)
        },
        verify: |_, _, data| Verification {
            ok: data.get("text").is_some() && data.get("backend").is_some(),
            message: "OCR output returned from backend.".to_string(),
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

    if let Err(error) = validate_capability_request(&capability.metadata.name, &request.params) {
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

fn validate_capability_request(
    capability_name: &str,
    params: &Map<String, Value>,
) -> Result<(), String> {
    match capability_name {
        "apps.launch" => {
            let app_id = params
                .get("app_id")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter app_id must be a string.".to_string())?;
            validate_desktop_id(app_id)
        }
        "browser.open_url" => {
            let url = params
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter url must be a string.".to_string())?;
            validate_browser_url(url)
        }
        "workspace.mode.plan" => {
            let mode = params
                .get("mode")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter mode must be a string.".to_string())?;
            validate_workspace_mode(mode).map(|_| ())
        }
        "screen.capture" => {
            if let Some(filename) = params.get("filename").and_then(Value::as_str) {
                let safe = safe_capture_filename(filename);
                if safe.trim().is_empty() {
                    return Err("screen capture filename cannot be empty".to_string());
                }
            }
            Ok(())
        }
        "screen.ocr_image" => {
            let path = params
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter path must be a string.".to_string())?;
            if is_sensitive_path(&json!(path)) {
                Err("Sensitive paths require a higher-risk capability.".to_string())
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
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
        Value::String(text) if text.chars().count() > 120 => {
            json!(format!("{}...", text.chars().take(117).collect::<String>()))
        }
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
                    | ".npmrc"
                    | ".pypirc"
                    | "credentials"
                    | "credentials.json"
                    | "id_dsa"
                    | "id_ecdsa"
                    | "id_ed25519"
                    | "id_rsa"
            ) || part == ".env"
                || part.starts_with(".env.")
                || pattern.is_match(&part)
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
            privacy: PrivacyConfig::default(),
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
    fn desktop_status_reports_session_shape() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);

        let result = execute_capability(
            &config,
            &build_registry(),
            request("desktop.status", Map::new()),
        );

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert!(result.data.get("session").is_some());
        assert!(result.data.get("tools").is_some());
    }

    #[test]
    fn desktop_entry_parser_reads_application_metadata() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("applications");
        let nested = root.join("org").join("example");
        fs::create_dir_all(&nested).unwrap();
        let desktop_file = nested.join("Demo.desktop");
        let raw = r#"
[Desktop Entry]
Type=Application
Name=Demo App
Exec=demo --flag
Categories=Utility;Development;
"#;

        let entry = parse_desktop_entry(&root, &desktop_file, raw).unwrap();

        assert_eq!(entry.id, "org-example-Demo.desktop");
        assert_eq!(entry.name, "Demo App");
        assert_eq!(entry.categories, vec!["Utility", "Development"]);
    }

    #[test]
    fn hidden_desktop_entry_is_not_launchable() {
        let tmp = TempDir::new().unwrap();
        let entry = DesktopEntry {
            id: "hidden.desktop".to_string(),
            name: "Hidden App".to_string(),
            exec: Some("hidden-app".to_string()),
            path: tmp.path().join("hidden.desktop"),
            categories: vec![],
            no_display: false,
            hidden: true,
        };

        let error = ensure_launchable_desktop_entry(&entry).unwrap_err();

        assert!(error.contains("hidden or non-display"));
    }

    #[test]
    fn browser_url_validation_denies_non_web_schemes_even_for_dry_run() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("url".to_string(), json!("file:///etc/passwd"));
        let mut req = request("browser.open_url", params);
        req.dry_run = true;

        let result = execute_capability(&config, &build_registry(), req);

        assert_eq!(result.status, ActionStatus::Denied);
        assert!(result.error.unwrap().contains("http:// or https://"));
    }

    #[test]
    fn medium_desktop_actions_require_confirmation_before_launch() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("app_id".to_string(), json!("firefox.desktop"));

        let result = execute_capability(&config, &build_registry(), request("apps.launch", params));

        assert_eq!(result.status, ActionStatus::ConfirmationRequired);
    }

    #[test]
    fn workspace_mode_plan_is_read_only_and_inspectable() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("mode".to_string(), json!("coding"));

        let result = execute_capability(
            &config,
            &build_registry(),
            request("workspace.mode.plan", params),
        );

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert_eq!(result.data["mode"], json!("coding"));
        assert!(result.data["steps"]
            .as_array()
            .is_some_and(|steps| !steps.is_empty()));
    }

    #[test]
    fn local_planner_maps_phase4_desktop_prompts() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();

        let browser_plan = build_ai_plan(
            &config,
            &registry,
            "open browser https://example.com",
            Some("local.rules"),
        )
        .unwrap();
        let app_plan = build_ai_plan(
            &config,
            &registry,
            "launch app firefox",
            Some("local.rules"),
        )
        .unwrap();
        let mode_plan = build_ai_plan(
            &config,
            &registry,
            "switch to coding mode",
            Some("local.rules"),
        )
        .unwrap();

        assert_eq!(browser_plan.steps[0].capability, "browser.open_url");
        assert_eq!(
            browser_plan.steps[0].params["url"],
            json!("https://example.com")
        );
        assert_eq!(app_plan.steps[0].capability, "apps.launch");
        assert_eq!(app_plan.steps[0].params["app_id"], json!("firefox.desktop"));
        assert_eq!(mode_plan.steps[0].capability, "workspace.mode.plan");
        assert_eq!(mode_plan.steps[0].params["mode"], json!("coding"));
    }

    #[test]
    fn screen_status_reports_backend_shape() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);

        let result = execute_capability(
            &config,
            &build_registry(),
            request("screen.status", Map::new()),
        );

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert!(result.data.get("capture").is_some());
        assert!(result.data.get("active_context").is_some());
        assert!(result.data.get("privacy").is_some());
    }

    #[test]
    fn screen_capture_dry_run_does_not_require_backend() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("filename".to_string(), json!("audit-shot.png"));
        let mut req = request("screen.capture", params);
        req.dry_run = true;

        let result = execute_capability(&config, &build_registry(), req);

        assert_eq!(result.status, ActionStatus::DryRun);
        assert!(!screen_capture_dir(&config).join("audit-shot.png").exists());
    }

    #[test]
    fn context_snapshot_requires_confirmation() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);

        let result = execute_capability(
            &config,
            &build_registry(),
            request("context.snapshot", Map::new()),
        );

        assert_eq!(result.status, ActionStatus::ConfirmationRequired);
    }

    #[test]
    fn privacy_policy_redacts_private_active_context() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut context = ActiveContext {
            backend: Some("test".to_string()),
            title: Some("Password vault".to_string()),
            pid: None,
            app: Some("browser".to_string()),
            is_private: false,
            privacy_reason: None,
        };

        apply_privacy_policy(&config, &mut context);

        assert!(context.is_private);
        assert_eq!(context.title.as_deref(), Some("<redacted>"));
        assert_eq!(context.app.as_deref(), Some("<redacted>"));
        assert!(context
            .privacy_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("password")));
    }

    #[test]
    fn privacy_policy_redacts_title_when_app_is_private() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut context = ActiveContext {
            backend: Some("test".to_string()),
            title: Some("Checking account dashboard".to_string()),
            pid: None,
            app: Some("bank-browser".to_string()),
            is_private: false,
            privacy_reason: None,
        };

        apply_privacy_policy(&config, &mut context);

        assert!(context.is_private);
        assert_eq!(context.title.as_deref(), Some("<redacted>"));
        assert_eq!(context.app.as_deref(), Some("<redacted>"));
        assert!(context
            .privacy_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("bank")));
    }

    #[test]
    fn screen_capture_requires_observable_active_context() {
        let context = ActiveContext {
            backend: Some("xdotool".to_string()),
            title: None,
            pid: None,
            app: None,
            is_private: false,
            privacy_reason: None,
        };

        let error = ensure_context_observable(&context).unwrap_err();
        assert!(error.contains("metadata unavailable"));
    }

    #[test]
    fn ocr_sensitive_path_is_denied_even_for_dry_run() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("path".to_string(), json!(".env"));
        let mut req = request("screen.ocr_image", params);
        req.dry_run = true;

        let result = execute_capability(&config, &build_registry(), req);

        assert_eq!(result.status, ActionStatus::Denied);
        assert!(result
            .error
            .unwrap()
            .contains("Sensitive paths require a higher-risk capability"));
    }

    #[test]
    fn ocr_env_variant_path_is_denied_even_for_dry_run() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("path".to_string(), json!(".env.local"));
        let mut req = request("screen.ocr_image", params);
        req.dry_run = true;

        let result = execute_capability(&config, &build_registry(), req);

        assert_eq!(result.status, ActionStatus::Denied);
        assert!(result
            .error
            .unwrap()
            .contains("Sensitive paths require a higher-risk capability"));
    }

    #[test]
    fn audit_summary_truncates_unicode_without_panicking() {
        let mut params = Map::new();
        params.insert("title".to_string(), json!("\u{00e9}".repeat(130)));

        let summary = summarize_params(&params);

        let title = summary["title"].as_str().unwrap();
        assert!(title.ends_with("..."));
        assert_eq!(title.chars().count(), 120);
    }

    #[test]
    fn local_planner_maps_phase5_screen_prompts() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();

        let status_plan =
            build_ai_plan(&config, &registry, "screen status", Some("local.rules")).unwrap();
        let context_plan =
            build_ai_plan(&config, &registry, "what is open", Some("local.rules")).unwrap();
        let capture_plan =
            build_ai_plan(&config, &registry, "take a screenshot", Some("local.rules")).unwrap();
        let ocr_plan = build_ai_plan(
            &config,
            &registry,
            "ocr image product/README.md",
            Some("local.rules"),
        )
        .unwrap();

        assert_eq!(status_plan.steps[0].capability, "screen.status");
        assert_eq!(context_plan.steps[0].capability, "context.snapshot");
        assert_eq!(capture_plan.steps[0].capability, "screen.capture");
        assert_eq!(ocr_plan.steps[0].capability, "screen.ocr_image");
        assert_eq!(ocr_plan.steps[0].params["path"], json!("product/README.md"));
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
