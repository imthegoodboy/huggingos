use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
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
const MAX_SEMANTIC_FILES: usize = 200;
const MAX_WORKFLOW_EVENTS: usize = 500;

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
            phase: "Product Phase 10".to_string(),
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

#[derive(Clone, Debug, Serialize)]
struct AgentDefinition {
    id: String,
    name: String,
    description: String,
    allowed_capabilities: Vec<String>,
}

#[derive(Clone, Debug)]
struct DelegatedCapabilityCall {
    step_id: String,
    agent_id: String,
    capability: String,
    params: Map<String, Value>,
    reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginManifest {
    schema_version: String,
    id: String,
    name: String,
    version: String,
    description: String,
    #[serde(default)]
    package: Option<PluginPackageMetadata>,
    #[serde(default)]
    ui: Option<PluginUiMetadata>,
    #[serde(default)]
    sandbox: Option<PluginSandboxMetadata>,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    capabilities: Vec<PluginCapabilityManifest>,
    #[serde(default)]
    workflows: Vec<PluginWorkflowManifest>,
    #[serde(default)]
    agents: Vec<PluginAgentManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginPackageMetadata {
    format: String,
    source: String,
    sha256: String,
    #[serde(default)]
    signature: Option<PluginSignatureMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginSignatureMetadata {
    algorithm: String,
    key_id: String,
    signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginUiMetadata {
    display_name: String,
    approval_summary: String,
    #[serde(default)]
    icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginSandboxMetadata {
    code_execution: String,
    network: bool,
    filesystem: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginCapabilityManifest {
    name: String,
    description: String,
    #[serde(default = "default_plugin_capability_risk")]
    risk: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    response: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginWorkflowManifest {
    name: String,
    description: String,
    #[serde(default)]
    steps: Vec<PluginWorkflowStepManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginWorkflowStepManifest {
    capability: String,
    #[serde(default)]
    params: Map<String, Value>,
    reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PluginAgentManifest {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    allowed_capabilities: Vec<String>,
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

fn plugins_dir(config: &Config) -> PathBuf {
    absolute_path_lossy(state_dir(config).join("plugins"))
}

fn installed_plugins_dir(config: &Config) -> PathBuf {
    absolute_path_lossy(plugins_dir(config).join("installed"))
}

fn plugin_install_dir(config: &Config, plugin_id: &str) -> PathBuf {
    installed_plugins_dir(config).join(safe_plugin_id(plugin_id))
}

fn plugin_manifest_path(config: &Config, plugin_id: &str) -> PathBuf {
    plugin_install_dir(config, plugin_id).join("plugin.json")
}

fn plugin_disabled_path(config: &Config, plugin_id: &str) -> PathBuf {
    plugin_install_dir(config, plugin_id).join(".disabled")
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

fn default_plugin_capability_risk() -> String {
    "read".to_string()
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
    } else if let Some(step) = plan_memory_session_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_memory_list_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_semantic_search_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_resume_workspace_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_agent_catalog_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_agent_orchestration_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_repeated_workflow_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_proactive_suggestions_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_self_healing_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_timeline_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_plugin_catalog_intent(registry, prompt, &lowered) {
        steps.push(step);
    } else if let Some(step) = plan_plugin_workflow_intent(registry, prompt, &lowered) {
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

fn plan_plugin_catalog_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("plugin catalog")
        || lowered.contains("list plugins")
        || lowered.contains("installed plugins"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "plugins.catalog",
        Map::new(),
        format!("List installed plugins for prompt: {prompt}"),
    ))
}

fn plan_plugin_workflow_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("plugin workflow") || lowered.contains("plan plugin")) {
        return None;
    }
    let mut params = Map::new();
    if let Some(plugin_id) = extract_path_after(prompt, &["plugin workflow ", "plan plugin "]) {
        params.insert("plugin_id".to_string(), json!(safe_plugin_id(&plugin_id)));
    }
    Some(plan_step(
        registry,
        "plugins.workflow.plan",
        params,
        "Plan a plugin-provided workflow without executing it.".to_string(),
    ))
}

fn plan_repeated_workflow_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("repeated workflow")
        || lowered.contains("detect workflow")
        || lowered.contains("suggest automation")
        || lowered.contains("automation suggestion"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "proactive.workflow.detect",
        Map::new(),
        format!("Detect repeated workflows for prompt: {prompt}"),
    ))
}

fn plan_proactive_suggestions_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("proactive suggestion")
        || lowered.contains("predictive suggestion")
        || lowered.contains("optimize my system")
        || lowered.contains("make my computer faster"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "proactive.suggest",
        Map::new(),
        format!("Build safe proactive suggestions for prompt: {prompt}"),
    ))
}

fn plan_self_healing_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("self heal")
        || lowered.contains("self-heal")
        || lowered.contains("app crashed")
        || lowered.contains("app failed")
        || lowered.contains("service failed")
        || lowered.contains("memory pressure")
        || lowered.contains("slow operation"))
    {
        return None;
    }
    let mut params = Map::new();
    let symptom = if lowered.contains("service failed") {
        "service_failed"
    } else if lowered.contains("memory pressure") {
        "memory_pressure"
    } else if lowered.contains("slow operation") {
        "slow_operation"
    } else {
        "app_crashed"
    };
    params.insert("symptom".to_string(), json!(symptom));
    Some(plan_step(
        registry,
        "selfheal.diagnose",
        params,
        format!("Diagnose a recoverable failure for prompt: {prompt}"),
    ))
}

fn plan_timeline_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("explain what happened")
        || lowered.contains("what happened")
        || lowered.contains("timeline")
        || lowered.contains("activity summary"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "timeline.explain",
        Map::new(),
        format!("Explain recent local activity for prompt: {prompt}"),
    ))
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

fn plan_memory_session_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.starts_with("remember ")
        || lowered.starts_with("remember that ")
        || lowered.starts_with("save memory "))
    {
        return None;
    }
    let value = extract_path_after(prompt, &["remember that ", "remember ", "save memory "])?;
    let mut params = Map::new();
    params.insert("key".to_string(), json!(safe_memory_key(&value)));
    params.insert("value".to_string(), json!(value));
    Some(plan_step(
        registry,
        "memory.session.remember",
        params,
        "Store a user-approved session memory fact.".to_string(),
    ))
}

fn plan_memory_list_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("list memory")
        || lowered.contains("show memory")
        || lowered.contains("memory status"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "memory.session.list",
        Map::new(),
        format!("List session memory for prompt: {prompt}"),
    ))
}

fn plan_semantic_search_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("semantic search")
        || lowered.contains("search files for")
        || lowered.contains("find files about"))
    {
        return None;
    }
    let query = extract_path_after(
        prompt,
        &["semantic search ", "search files for ", "find files about "],
    )
    .unwrap_or_else(|| prompt.to_string());
    let mut params = Map::new();
    params.insert("query".to_string(), json!(query));
    Some(plan_step(
        registry,
        "files.semantic.search",
        params,
        "Search the local opt-in semantic file index.".to_string(),
    ))
}

fn plan_resume_workspace_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("resume workspace")
        || lowered.contains("resume my workspace")
        || lowered.contains("continue my work")
        || lowered.contains("resume my day"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "workspace.resume.plan",
        Map::new(),
        format!("Build a memory-backed resume plan for prompt: {prompt}"),
    ))
}

fn plan_agent_catalog_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("agent catalog")
        || lowered.contains("list agents")
        || lowered.contains("available agents"))
    {
        return None;
    }
    Some(plan_step(
        registry,
        "agents.catalog",
        Map::new(),
        format!("List available agents for prompt: {prompt}"),
    ))
}

fn plan_agent_orchestration_intent(
    registry: &BTreeMap<String, Capability>,
    prompt: &str,
    lowered: &str,
) -> Option<AiPlanStep> {
    if !(lowered.contains("orchestrate")
        || lowered.contains("delegate")
        || lowered.contains("daily brief")
        || lowered.contains("multi agent")
        || lowered.contains("multi-agent"))
    {
        return None;
    }
    let mut params = Map::new();
    params.insert("goal".to_string(), json!(prompt.trim()));
    Some(plan_step(
        registry,
        "agents.orchestrate",
        params,
        "Delegate the goal through the local agent catalog.".to_string(),
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

fn memory_dir(config: &Config) -> PathBuf {
    absolute_path_lossy(state_dir(config).join("memory"))
}

fn session_memory_path(config: &Config) -> PathBuf {
    memory_dir(config).join("session.jsonl")
}

fn preferences_path(config: &Config) -> PathBuf {
    memory_dir(config).join("preferences.json")
}

fn semantic_index_path(config: &Config) -> PathBuf {
    memory_dir(config).join("semantic-index.json")
}

fn agent_trace_path(config: &Config) -> PathBuf {
    memory_dir(config).join("agent-traces.jsonl")
}

fn remember_session_memory(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let key = safe_memory_key(&string_param(request, "key")?);
    let value = string_param(request, "value")?.trim().to_string();
    if value.is_empty() {
        return Err("memory value cannot be empty".to_string());
    }
    let tags = request
        .params
        .get("tags")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(safe_memory_key)
                .filter(|tag| !tag.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let memory_id = Uuid::new_v4().to_string();
    let record = json!({
        "memory_id": memory_id,
        "key": key,
        "value": truncate_text(&value, 2000),
        "tags": tags,
        "source": "session",
        "actor": request.actor,
        "created_at": utc_now(),
    });
    append_json_line(&session_memory_path(config), &record)?;
    Ok(json!({
        "stored": true,
        "memory_id": memory_id,
        "key": record["key"],
        "path": session_memory_path(config),
    }))
}

fn list_session_memory(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let query = request
        .params
        .get("query")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let limit = bounded_limit(request, 50, 200)?;
    let mut items = read_json_lines(&session_memory_path(config))?;
    items.retain(|item| match &query {
        Some(query) => item.to_string().to_ascii_lowercase().contains(query),
        None => true,
    });
    if items.len() > limit {
        items = items.split_off(items.len() - limit);
    }
    let item_count = items.len();
    Ok(json!({
        "path": session_memory_path(config),
        "items": items,
        "item_count": item_count,
        "query": query,
    }))
}

fn set_preference(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let key = safe_memory_key(&string_param(request, "key")?);
    let value = string_param(request, "value")?.trim().to_string();
    if value.is_empty() {
        return Err("preference value cannot be empty".to_string());
    }
    let mut preferences = read_preferences(config)?;
    preferences.insert(
        key.clone(),
        json!({
            "value": truncate_text(&value, 2000),
            "updated_at": utc_now(),
            "actor": request.actor,
        }),
    );
    write_json_file(
        &preferences_path(config),
        &Value::Object(preferences.clone()),
    )?;
    Ok(json!({
        "updated": true,
        "key": key,
        "path": preferences_path(config),
        "preference": preferences.get(&key),
    }))
}

fn list_preferences(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let preferences = read_preferences(config)?;
    if let Some(key) = request.params.get("key").and_then(Value::as_str) {
        let key = safe_memory_key(key);
        let mut filtered = Map::new();
        if let Some(value) = preferences.get(&key) {
            filtered.insert(key, value.clone());
        }
        let count = filtered.len();
        return Ok(json!({
            "path": preferences_path(config),
            "preferences": filtered,
            "preference_count": count,
        }));
    }
    let count = preferences.len();
    Ok(json!({
        "path": preferences_path(config),
        "preferences": preferences,
        "preference_count": count,
    }))
}

fn delete_memory(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let scope = string_param(request, "scope")?;
    let scope = scope.trim().to_ascii_lowercase();
    let key = request
        .params
        .get("key")
        .and_then(Value::as_str)
        .map(safe_memory_key);
    let mut deleted = vec![];
    match scope.as_str() {
        "session" => {
            let path = session_memory_path(config);
            if let Some(key) = key {
                let mut items = read_json_lines(&path)?;
                let before = items.len();
                items.retain(|item| item.get("key").and_then(Value::as_str) != Some(key.as_str()));
                write_json_lines(&path, &items)?;
                deleted
                    .push(json!({"scope": "session", "key": key, "removed": before - items.len()}));
            } else {
                remove_file_if_exists(&path)?;
                deleted.push(json!({"scope": "session", "path": path}));
            }
        }
        "preferences" => {
            let path = preferences_path(config);
            if let Some(key) = key {
                let mut preferences = read_preferences(config)?;
                let removed = preferences.remove(&key).is_some();
                write_json_file(&path, &Value::Object(preferences))?;
                deleted.push(json!({"scope": "preferences", "key": key, "removed": removed}));
            } else {
                remove_file_if_exists(&path)?;
                deleted.push(json!({"scope": "preferences", "path": path}));
            }
        }
        "semantic_index" => {
            let path = semantic_index_path(config);
            remove_file_if_exists(&path)?;
            deleted.push(json!({"scope": "semantic_index", "path": path}));
        }
        "traces" => {
            let path = agent_trace_path(config);
            remove_file_if_exists(&path)?;
            deleted.push(json!({"scope": "traces", "path": path}));
        }
        "all" => {
            for path in [
                session_memory_path(config),
                preferences_path(config),
                semantic_index_path(config),
                agent_trace_path(config),
            ] {
                remove_file_if_exists(&path)?;
                deleted.push(json!({"scope": "all", "path": path}));
            }
        }
        _ => {
            return Err(
                "memory delete scope must be session, preferences, semantic_index, traces, or all"
                    .to_string(),
            )
        }
    }
    Ok(json!({ "deleted": deleted, "scope": scope }))
}

fn export_memory(config: &Config) -> Value {
    json!({
        "session": {
            "path": session_memory_path(config),
            "items": read_json_lines(&session_memory_path(config)).unwrap_or_default(),
        },
        "preferences": {
            "path": preferences_path(config),
            "items": read_preferences(config).unwrap_or_default(),
        },
        "semantic_index": {
            "path": semantic_index_path(config),
            "present": semantic_index_path(config).exists(),
        },
        "traces": {
            "path": agent_trace_path(config),
            "items": read_json_lines(&agent_trace_path(config)).unwrap_or_default(),
        },
    })
}

fn list_memory_events(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let limit = bounded_limit(request, 25, 200)?;
    let entries = list_audit_entries(&audit_log_path(config), limit)?;
    let events = entries
        .into_iter()
        .map(|entry| {
            json!({
                "recorded_at": entry.get("recorded_at"),
                "actor": entry.get("actor"),
                "capability": entry.get("capability"),
                "status": entry.get("status"),
                "summary": entry.get("summary"),
            })
        })
        .collect::<Vec<_>>();
    let event_count = events.len();
    Ok(json!({
        "source": audit_log_path(config),
        "events": events,
        "event_count": event_count,
    }))
}

fn detect_repeated_workflows(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let limit = bounded_limit(request, 100, MAX_WORKFLOW_EVENTS)?;
    let min_repetitions = request
        .params
        .get("min_repetitions")
        .and_then(Value::as_u64)
        .unwrap_or(2);
    if min_repetitions < 2 {
        return Err("min_repetitions must be at least 2".to_string());
    }
    let entries = list_audit_entries(&audit_log_path(config), limit)?;
    let capabilities = entries
        .iter()
        .filter(|entry| entry.get("status") == Some(&json!("succeeded")))
        .filter_map(|entry| {
            entry
                .get("capability")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect::<Vec<_>>();

    let mut pair_counts: BTreeMap<String, usize> = BTreeMap::new();
    for pair in capabilities.windows(2) {
        if pair[0] == pair[1]
            || pair[0] == "proactive.workflow.detect"
            || pair[1] == "proactive.workflow.detect"
        {
            continue;
        }
        let key = format!("{} -> {}", pair[0], pair[1]);
        *pair_counts.entry(key).or_insert(0) += 1;
    }

    let mut suggestions = pair_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_repetitions as usize)
        .map(|(sequence, count)| {
            json!({
                "kind": "workflow_repetition",
                "sequence": sequence,
                "count": count,
                "confidence": if count >= 4 { "high" } else { "medium" },
                "suggestion": format!("Offer a user-approved automation for: {sequence}"),
                "policy": "suggestion_only",
                "requires_confirmation": true,
            })
        })
        .collect::<Vec<_>>();
    suggestions.sort_by(|left, right| {
        right["count"]
            .as_u64()
            .cmp(&left["count"].as_u64())
            .then_with(|| left["sequence"].as_str().cmp(&right["sequence"].as_str()))
    });
    let suggestion_count = suggestions.len();

    Ok(json!({
        "source": audit_log_path(config),
        "event_count": capabilities.len(),
        "min_repetitions": min_repetitions,
        "suggestions": suggestions,
        "suggestion_count": suggestion_count,
        "policy": "no proactive action is executed by this capability",
    }))
}

fn self_heal_diagnose(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let symptom = request
        .params
        .get("symptom")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .trim()
        .to_ascii_lowercase();
    let target = request
        .params
        .get("target")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unspecified");
    let simulated = request
        .params
        .get("simulated")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let recent_events = list_audit_entries(&audit_log_path(config), 20)?;
    let (severity, diagnosis, recovery_steps) = match symptom.as_str() {
        "app_crashed" | "app_failed" => (
            "medium",
            format!("Application failure reported for {target}."),
            vec![
                json!({
                    "capability": "apps.list",
                    "params": {},
                    "reason": "Check whether the app is discoverable before restart."
                }),
                json!({
                    "capability": "apps.launch",
                    "params": { "app_id": "<chosen-app.desktop>" },
                    "reason": "Relaunch only after the user confirms the exact app."
                }),
            ],
        ),
        "service_failed" => (
            "medium",
            format!("Service failure reported for {target}."),
            vec![
                json!({
                    "capability": "product.status",
                    "params": {},
                    "reason": "Check product runtime state before recommending service recovery."
                }),
                json!({
                    "capability": "memory.event.list",
                    "params": { "limit": 20 },
                    "reason": "Inspect recent failures before any service action."
                }),
            ],
        ),
        "memory_pressure" => (
            "low",
            "Memory pressure reported or simulated.".to_string(),
            vec![json!({
                "capability": "timeline.explain",
                "params": { "limit": 20 },
                "reason": "Review recent activity before suggesting cleanup."
            })],
        ),
        "slow_operation" => (
            "low",
            "Slow operation reported or simulated.".to_string(),
            vec![json!({
                "capability": "proactive.workflow.detect",
                "params": { "limit": 100 },
                "reason": "Look for repeated slow workflows that could be streamlined."
            })],
        ),
        _ => (
            "low",
            format!("No known self-healing rule matched symptom: {symptom}."),
            vec![json!({
                "capability": "memory.event.list",
                "params": { "limit": 20 },
                "reason": "Inspect recent local events for clues."
            })],
        ),
    };

    Ok(json!({
        "symptom": symptom,
        "target": target,
        "simulated": simulated,
        "severity": severity,
        "diagnosis": diagnosis,
        "recommended_actions": recovery_steps,
        "recent_event_count": recent_events.len(),
        "policy": "diagnosis only; recovery capabilities still require their own policy checks and confirmations",
    }))
}

fn proactive_suggestions(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let workflows = detect_repeated_workflows(config, request)?;
    let mut suggestions = workflows
        .get("suggestions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let recent_entries = list_audit_entries(&audit_log_path(config), 50)?;
    let failure_count = recent_entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.get("status").and_then(Value::as_str),
                Some("failed") | Some("denied") | Some("confirmation_required")
            )
        })
        .count();
    if failure_count > 0 {
        suggestions.push(json!({
            "kind": "recent_failure_review",
            "count": failure_count,
            "suggestion": "Review recent failed or blocked actions before retrying.",
            "capability": "timeline.explain",
            "params": { "limit": 50 },
            "policy": "suggestion_only",
            "requires_confirmation": false,
        }));
    }
    let suggestion_count = suggestions.len();

    Ok(json!({
        "generated_at": utc_now(),
        "suggestions": suggestions,
        "suggestion_count": suggestion_count,
        "workflow_source": workflows["source"],
        "policy": "suggestions only; no action is executed automatically",
    }))
}

fn explain_timeline(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let limit = bounded_limit(request, 25, 100)?;
    let entries = list_audit_entries(&audit_log_path(config), limit)?;
    let mut timeline = vec![];
    for entry in entries {
        timeline.push(json!({
            "kind": "capability_event",
            "recorded_at": entry.get("recorded_at"),
            "actor": entry.get("actor"),
            "capability": entry.get("capability"),
            "status": entry.get("status"),
            "summary": entry.get("summary"),
        }));
    }

    let session_memory_count = read_json_lines(&session_memory_path(config))?.len();
    let agent_trace_count = read_json_lines(&agent_trace_path(config))?.len();
    let event_count = timeline.len();
    Ok(json!({
        "generated_at": utc_now(),
        "source": audit_log_path(config),
        "timeline": timeline,
        "event_count": event_count,
        "context": {
            "session_memory_count": session_memory_count,
            "agent_trace_count": agent_trace_count,
            "semantic_index_present": semantic_index_path(config).exists(),
        },
        "summary": format!("Found {event_count} recent audited events, {session_memory_count} session memories, and {agent_trace_count} agent traces."),
    }))
}

fn validate_plugin_manifest_capability(
    _config: &Config,
    request: &ActionRequest,
) -> Result<Value, String> {
    let source = string_param(request, "source")?;
    let manifest = read_plugin_manifest_from_source(&source)?;
    validate_plugin_manifest(&manifest)?;
    Ok(json!({
        "valid": true,
        "manifest": plugin_manifest_summary(&manifest, false),
        "permission_summary": plugin_permission_summary(&manifest),
        "plugin_trust_state": plugin_trust_state(&manifest),
        "source": source,
    }))
}

fn validate_plugin_package(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let source = string_param(request, "source")?;
    let manifest = read_plugin_manifest_from_source(&source)?;
    validate_plugin_manifest(&manifest)?;
    Ok(json!({
        "valid": true,
        "source": source,
        "package": manifest.package,
        "plugin": plugin_manifest_summary(&manifest, false),
        "permission_summary": plugin_permission_summary(&manifest),
        "approval": plugin_approval_summary(&manifest),
        "sandbox": plugin_sandbox_summary(&manifest),
        "plugin_trust_state": plugin_trust_state(&manifest),
        "install_preview": {
            "destination": plugin_install_dir(config, &manifest.id),
            "requires_confirmation": true,
            "arbitrary_code_execution": false
        }
    }))
}

fn review_plugin_permissions(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let manifest = if let Some(source) = request.params.get("source").and_then(Value::as_str) {
        read_plugin_manifest_from_source(source)?
    } else if let Some(plugin_id) = request.params.get("plugin_id").and_then(Value::as_str) {
        read_installed_plugin(config, &safe_plugin_id(plugin_id))?
    } else {
        return Err("plugins.permission.review requires source or plugin_id".to_string());
    };
    validate_plugin_manifest(&manifest)?;
    Ok(json!({
        "plugin_id": manifest.id,
        "plugin_name": manifest.name,
        "permission_summary": plugin_permission_summary(&manifest),
        "approval": plugin_approval_summary(&manifest),
        "sandbox": plugin_sandbox_summary(&manifest),
        "plugin_trust_state": plugin_trust_state(&manifest),
    }))
}

fn install_plugin(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let source = string_param(request, "source")?;
    let force = request
        .params
        .get("force")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let manifest = read_plugin_manifest_from_source(&source)?;
    validate_plugin_manifest(&manifest)?;
    let plugin_id = safe_plugin_id(&manifest.id);
    let trust_state = plugin_trust_state(&manifest);
    let permission_summary = plugin_permission_summary(&manifest);
    let approval = plugin_approval_summary(&manifest);
    let destination = plugin_install_dir(config, &plugin_id);
    ensure_plugin_dir_is_scoped(config, &destination)?;
    let previous_manifest = if destination.exists() {
        fs::read_to_string(destination.join("plugin.json")).ok()
    } else {
        None
    };
    if destination.exists() {
        if !force {
            return Err(format!(
                "plugin already installed: {plugin_id}; pass force=true to replace"
            ));
        }
        fs::remove_dir_all(&destination).map_err(|err| err.to_string())?;
    }
    fs::create_dir_all(&destination).map_err(|err| err.to_string())?;
    write_json_file(&destination.join("plugin.json"), &json!(manifest))?;
    remove_file_if_exists(&destination.join(".disabled"))?;
    Ok(json!({
        "installed": true,
        "plugin": plugin_manifest_summary(&read_installed_plugin(config, &plugin_id)?, true),
        "path": destination,
        "plugin_identity": plugin_id,
        "plugin_trust_state": trust_state,
        "permission_summary": permission_summary,
        "approval": approval,
        "rollback": {
            "type": if previous_manifest.is_some() { "replace" } else { "remove" },
            "previous_manifest_present": previous_manifest.is_some(),
            "disable_capability": "plugins.disable",
            "remove_capability": "plugins.remove"
        },
    }))
}

fn disable_plugin(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let plugin_id = safe_plugin_id(&string_param(request, "plugin_id")?);
    validate_plugin_id(&plugin_id)?;
    let manifest = read_installed_plugin(config, &plugin_id)?;
    let trust_state = plugin_trust_state(&manifest);
    let disabled_path = plugin_disabled_path(config, &plugin_id);
    ensure_plugin_dir_is_scoped(config, &plugin_install_dir(config, &plugin_id))?;
    if let Some(parent) = disabled_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&disabled_path, utc_now()).map_err(|err| err.to_string())?;
    Ok(json!({
        "disabled": true,
        "plugin": plugin_manifest_summary(&manifest, false),
        "path": disabled_path,
        "plugin_identity": plugin_id,
        "plugin_trust_state": trust_state,
        "rollback": {
            "type": "enable",
            "manual_remove_disabled_marker": true,
            "path": disabled_path,
        },
    }))
}

fn remove_plugin(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let plugin_id = safe_plugin_id(&string_param(request, "plugin_id")?);
    validate_plugin_id(&plugin_id)?;
    let destination = plugin_install_dir(config, &plugin_id);
    ensure_plugin_dir_is_scoped(config, &destination)?;
    if !destination.exists() {
        return Err(format!("plugin is not installed: {plugin_id}"));
    }
    let manifest = read_installed_plugin(config, &plugin_id)?;
    let trust_state = plugin_trust_state(&manifest);
    let rollback_manifest_path = destination.join("plugin.json");
    fs::remove_dir_all(&destination).map_err(|err| err.to_string())?;
    Ok(json!({
        "removed": true,
        "plugin_id": plugin_id,
        "path": destination,
        "plugin_identity": plugin_id,
        "plugin_trust_state": trust_state,
        "rollback": {
            "type": "reinstall",
            "manifest_path_before_remove": rollback_manifest_path,
            "source_required": true
        },
    }))
}

fn catalog_plugins(config: &Config) -> Result<Value, String> {
    let plugins = read_installed_plugins(config)?
        .into_iter()
        .map(|manifest| {
            let enabled = !plugin_disabled_path(config, &manifest.id).exists();
            plugin_manifest_summary(&manifest, enabled)
        })
        .collect::<Vec<_>>();
    let plugin_count = plugins.len();
    Ok(json!({
        "plugins": plugins,
        "plugin_count": plugin_count,
        "path": installed_plugins_dir(config),
        "permission_model": "plugins declare capabilities and workflows, but execution still uses huggingOS policy and audit",
    }))
}

fn plan_plugin_workflow(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let plugin_id = request
        .params
        .get("plugin_id")
        .and_then(Value::as_str)
        .map(safe_plugin_id);
    let workflow_name = request
        .params
        .get("workflow")
        .and_then(Value::as_str)
        .map(safe_plugin_id);
    let plugins = read_installed_plugins(config)?;
    let manifest = plugins
        .iter()
        .find(|manifest| {
            plugin_id
                .as_ref()
                .map(|id| manifest.id == *id)
                .unwrap_or(true)
                && !plugin_disabled_path(config, &manifest.id).exists()
        })
        .ok_or_else(|| "no enabled plugin matched workflow request".to_string())?;
    let workflow = manifest
        .workflows
        .iter()
        .find(|workflow| {
            workflow_name
                .as_ref()
                .map(|name| safe_plugin_id(&workflow.name) == *name)
                .unwrap_or(true)
        })
        .ok_or_else(|| "no workflow matched plugin request".to_string())?;
    Ok(json!({
        "plugin_id": manifest.id,
        "plugin_name": manifest.name,
        "workflow": workflow.name,
        "description": workflow.description,
        "steps": workflow.steps,
        "step_count": workflow.steps.len(),
        "policy": "workflow plan only; steps must be executed through normal capability policy",
    }))
}

fn run_plugin_capability(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let plugin_id = safe_plugin_id(&string_param(request, "plugin_id")?);
    let capability_name = safe_plugin_id(&string_param(request, "capability")?);
    validate_plugin_id(&plugin_id)?;
    let manifest = read_installed_plugin(config, &plugin_id)?;
    if plugin_disabled_path(config, &plugin_id).exists() {
        return Err(format!("plugin is disabled: {plugin_id}"));
    }
    let capability = manifest
        .capabilities
        .iter()
        .find(|capability| safe_plugin_id(&capability.name) == capability_name)
        .ok_or_else(|| format!("plugin capability not found: {capability_name}"))?;
    if !capability.risk.trim().eq_ignore_ascii_case("read") {
        return Err(
            "only read-only declarative plugin capabilities are executable in Phase 9".to_string(),
        );
    }
    Ok(json!({
        "plugin_identity": plugin_id,
        "plugin_name": manifest.name,
        "capability": capability.name,
        "description": capability.description,
        "permissions": capability.permissions,
        "plugin_trust_state": plugin_trust_state(&manifest),
        "response": capability.response,
        "policy": "declarative read-only plugin capability executed through huggingOS policy and audit",
    }))
}

fn read_plugin_manifest_from_source(source: &str) -> Result<PluginManifest, String> {
    if is_sensitive_path(&json!(source)) {
        return Err("plugin source path is sensitive".to_string());
    }
    let source = resolve_existing_path(source)?;
    let manifest_path = if source.is_dir() {
        source.join("plugin.json")
    } else {
        source
    };
    let text = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
    serde_json::from_str::<PluginManifest>(&text)
        .map_err(|err| format!("invalid plugin manifest: {err}"))
}

fn read_installed_plugin(config: &Config, plugin_id: &str) -> Result<PluginManifest, String> {
    validate_plugin_id(plugin_id)?;
    let manifest_path = plugin_manifest_path(config, plugin_id);
    let text = fs::read_to_string(&manifest_path)
        .map_err(|err| format!("plugin is not installed: {plugin_id}: {err}"))?;
    let manifest = serde_json::from_str::<PluginManifest>(&text)
        .map_err(|err| format!("invalid installed plugin manifest: {err}"))?;
    validate_plugin_manifest(&manifest)?;
    Ok(manifest)
}

fn read_installed_plugins(config: &Config) -> Result<Vec<PluginManifest>, String> {
    let root = installed_plugins_dir(config);
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut plugins = vec![];
    for entry in fs::read_dir(&root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path().join("plugin.json");
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let manifest = serde_json::from_str::<PluginManifest>(&text).map_err(|err| {
            format!(
                "invalid installed plugin manifest {}: {err}",
                path.display()
            )
        })?;
        validate_plugin_manifest(&manifest)?;
        plugins.push(manifest);
    }
    plugins.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(plugins)
}

fn validate_plugin_manifest(manifest: &PluginManifest) -> Result<(), String> {
    if manifest.schema_version != "huggingos.plugin.v1" {
        return Err("plugin schema_version must be huggingos.plugin.v1".to_string());
    }
    validate_plugin_id(&manifest.id)?;
    if manifest.name.trim().is_empty() || manifest.version.trim().is_empty() {
        return Err("plugin name and version are required".to_string());
    }
    if manifest.capabilities.is_empty() && manifest.workflows.is_empty() {
        return Err("plugin must declare at least one capability or workflow".to_string());
    }
    validate_plugin_package_metadata(manifest)?;
    validate_plugin_ui_metadata(manifest)?;
    validate_plugin_sandbox_metadata(manifest)?;
    for permission in &manifest.permissions {
        validate_plugin_permission(permission)?;
    }
    for capability in &manifest.capabilities {
        validate_plugin_id(&capability.name)?;
        match capability.risk.trim().to_ascii_lowercase().as_str() {
            "read" => {}
            _ => {
                return Err(
                    "Phase 9 plugin capabilities must be declarative read-only capabilities"
                        .to_string(),
                )
            }
        }
        for permission in &capability.permissions {
            validate_plugin_permission(permission)?;
        }
    }
    for workflow in &manifest.workflows {
        validate_plugin_id(&workflow.name)?;
        if workflow.steps.is_empty() {
            return Err("plugin workflows must contain at least one step".to_string());
        }
        for step in &workflow.steps {
            if step.capability.trim().is_empty() || step.reason.trim().is_empty() {
                return Err("plugin workflow steps require capability and reason".to_string());
            }
        }
    }
    for agent in &manifest.agents {
        validate_plugin_id(&agent.id)?;
        if agent.allowed_capabilities.is_empty() {
            return Err("plugin agents must declare allowed capabilities".to_string());
        }
    }
    Ok(())
}

fn validate_plugin_package_metadata(manifest: &PluginManifest) -> Result<(), String> {
    let Some(package) = &manifest.package else {
        return Ok(());
    };
    if package.format.trim() != "huggingos.plugin.package.v1" {
        return Err("plugin package format must be huggingos.plugin.package.v1".to_string());
    }
    if package.source.trim().is_empty() {
        return Err("plugin package source is required".to_string());
    }
    let sha = package.sha256.trim();
    if sha.len() != 64 || !sha.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("plugin package sha256 must be a 64-character hex digest".to_string());
    }
    if let Some(signature) = &package.signature {
        if signature.algorithm.trim() != "ed25519" {
            return Err("plugin package signature algorithm must be ed25519".to_string());
        }
        if signature.key_id.trim().is_empty() || signature.signature.trim().is_empty() {
            return Err("plugin package signature key_id and signature are required".to_string());
        }
    }
    Ok(())
}

fn validate_plugin_ui_metadata(manifest: &PluginManifest) -> Result<(), String> {
    let Some(ui) = &manifest.ui else {
        return Ok(());
    };
    if ui.display_name.trim().is_empty() || ui.approval_summary.trim().is_empty() {
        return Err("plugin ui display_name and approval_summary are required".to_string());
    }
    if let Some(icon) = &ui.icon {
        if icon.contains("://") || icon.contains('\\') || icon.contains('/') {
            return Err("plugin ui icon must be a local icon name, not a path or URL".to_string());
        }
    }
    Ok(())
}

fn validate_plugin_sandbox_metadata(manifest: &PluginManifest) -> Result<(), String> {
    let Some(sandbox) = &manifest.sandbox else {
        return Ok(());
    };
    if sandbox.code_execution.trim() != "disabled" {
        return Err("plugin sandbox code_execution must be disabled in Phase 10".to_string());
    }
    if sandbox.network {
        return Err("plugin sandbox network must be false in Phase 10".to_string());
    }
    match sandbox.filesystem.trim() {
        "none" | "state_read" => Ok(()),
        _ => Err("plugin sandbox filesystem must be none or state_read".to_string()),
    }
}

fn plugin_manifest_summary(manifest: &PluginManifest, enabled: bool) -> Value {
    json!({
        "id": manifest.id,
        "name": manifest.name,
        "version": manifest.version,
        "description": manifest.description,
        "enabled": enabled,
        "package": manifest.package,
        "ui": manifest.ui,
        "sandbox": plugin_sandbox_summary(manifest),
        "plugin_trust_state": plugin_trust_state(manifest),
        "permissions": manifest.permissions,
        "capabilities": manifest.capabilities.iter().map(|capability| json!({
            "name": capability.name,
            "description": capability.description,
            "risk": capability.risk,
            "permissions": capability.permissions,
        })).collect::<Vec<_>>(),
        "workflows": manifest.workflows.iter().map(|workflow| json!({
            "name": workflow.name,
            "description": workflow.description,
            "step_count": workflow.steps.len(),
        })).collect::<Vec<_>>(),
        "agents": manifest.agents,
    })
}

fn plugin_permission_summary(manifest: &PluginManifest) -> Value {
    let capability_permissions = manifest
        .capabilities
        .iter()
        .map(|capability| {
            json!({
                "capability": capability.name,
                "risk": capability.risk,
                "permissions": capability.permissions,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "plugin_id": manifest.id,
        "plugin_permissions": manifest.permissions,
        "capability_permissions": capability_permissions,
        "workflow_count": manifest.workflows.len(),
        "agent_count": manifest.agents.len(),
        "requires_install_confirmation": true,
        "arbitrary_code_execution": false,
        "network_access": manifest.sandbox.as_ref().map(|sandbox| sandbox.network).unwrap_or(false),
    })
}

fn plugin_approval_summary(manifest: &PluginManifest) -> Value {
    json!({
        "title": manifest.ui.as_ref().map(|ui| ui.display_name.as_str()).unwrap_or(manifest.name.as_str()),
        "summary": manifest.ui.as_ref().map(|ui| ui.approval_summary.as_str()).unwrap_or(manifest.description.as_str()),
        "risk": "medium_install_read_runtime",
        "requires_confirmation": true,
        "review_items": [
            "Plugin manifest identity",
            "Declared permissions",
            "Read-only capabilities",
            "Workflow steps",
            "Package trust metadata",
            "Sandbox declaration"
        ],
    })
}

fn plugin_sandbox_summary(manifest: &PluginManifest) -> Value {
    match &manifest.sandbox {
        Some(sandbox) => json!(sandbox),
        None => json!({
            "code_execution": "disabled",
            "network": false,
            "filesystem": "none",
        }),
    }
}

fn plugin_trust_state(manifest: &PluginManifest) -> Value {
    let state = match &manifest.package {
        Some(package) if package.signature.is_some() => "signed_metadata_present_unverified",
        Some(_) => "package_metadata_unsigned",
        None => "manifest_only_unsigned",
    };
    json!({
        "state": state,
        "package_metadata_present": manifest.package.is_some(),
        "signature_present": manifest.package.as_ref().and_then(|package| package.signature.as_ref()).is_some(),
        "verified": false,
        "note": "Phase 10 validates package metadata shape but does not verify signatures cryptographically.",
    })
}

fn validate_plugin_id(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return Err("plugin ids must be 1 to 80 characters".to_string());
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-'))
    {
        Ok(())
    } else {
        Err(
            "plugin ids may contain lowercase letters, numbers, dots, dashes, and underscores"
                .to_string(),
        )
    }
}

fn validate_plugin_permission(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return Err("plugin permissions must be 1 to 80 characters".to_string());
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, ':' | '_' | '-'))
    {
        Ok(())
    } else {
        Err("plugin permissions may contain lowercase letters, numbers, colons, dashes, and underscores".to_string())
    }
}

fn safe_plugin_id(value: &str) -> String {
    let re = SAFE_FILENAME_RE.get_or_init(|| Regex::new(r"[^a-z0-9._-]+").unwrap());
    let lowered = value.trim().to_ascii_lowercase();
    let id = re
        .replace_all(&lowered, "-")
        .trim_matches(&['.', '-', '_'][..])
        .to_string();
    truncate_text(if id.is_empty() { "plugin" } else { &id }, 80)
}

fn ensure_plugin_dir_is_scoped(config: &Config, path: &Path) -> Result<(), String> {
    let root = absolute_path_lossy(installed_plugins_dir(config));
    let candidate = absolute_path_lossy(path);
    if candidate.starts_with(&root) {
        Ok(())
    } else {
        Err("plugin path escaped installed plugin directory".to_string())
    }
}

fn index_semantic_files(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let root_raw = string_param(request, "root")?;
    if is_sensitive_path(&json!(root_raw)) {
        return Err("Sensitive paths require a higher-risk capability.".to_string());
    }
    let root = resolve_existing_path(&root_raw)?;
    if !root.is_dir() {
        return Err(format!(
            "semantic index root is not a directory: {}",
            root.display()
        ));
    }
    let recursive = request
        .params
        .get("recursive")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let max_files = bounded_limit(request, MAX_SEMANTIC_FILES, MAX_SEMANTIC_FILES)?;
    let mut candidates = vec![];
    collect_semantic_candidates(&root, recursive, &mut candidates, max_files)?;
    let mut documents = vec![];
    for path in candidates {
        if documents.len() >= max_files {
            break;
        }
        if is_sensitive_path(&json!(path.to_string_lossy().to_string()))
            || path_has_hidden_component(&path)
        {
            continue;
        }
        let metadata = fs::metadata(&path).map_err(|err| err.to_string())?;
        if !metadata.is_file()
            || metadata.len() > MAX_TEXT_BYTES
            || !is_supported_semantic_file(&path)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let tokens = unique_tokens(&text);
        if tokens.is_empty() {
            continue;
        }
        documents.push(json!({
            "path": path,
            "size": metadata.len(),
            "modified": metadata.modified().ok().and_then(system_time_json),
            "tokens": tokens,
            "summary": summarize_document_text(&text),
        }));
    }
    let index = json!({
        "indexed_at": utc_now(),
        "root": root,
        "recursive": recursive,
        "engine": "local.token_overlap.v1",
        "embedding_provider": null,
        "document_count": documents.len(),
        "documents": documents,
    });
    write_json_file(&semantic_index_path(config), &index)?;
    Ok(json!({
        "path": semantic_index_path(config),
        "root": index["root"],
        "engine": index["engine"],
        "document_count": index["document_count"],
    }))
}

fn search_semantic_files(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let query = string_param(request, "query")?;
    let query_tokens = unique_tokens(&query);
    if query_tokens.is_empty() {
        return Err("semantic search query must contain searchable text".to_string());
    }
    let limit = bounded_limit(request, 10, 50)?;
    let index = read_json_file(&semantic_index_path(config))?.ok_or_else(|| {
        "semantic index not found; run files.semantic.index on an opt-in root first".to_string()
    })?;
    let mut results = vec![];
    for document in index
        .get("documents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let tokens = document
            .get("tokens")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let matches = query_tokens
            .iter()
            .filter(|token| tokens.contains(*token))
            .cloned()
            .collect::<Vec<_>>();
        let score = matches.len() as i64;
        if score > 0 {
            results.push(json!({
                "path": document.get("path"),
                "score": score,
                "matches": matches,
                "summary": document.get("summary"),
                "engine": index.get("engine"),
            }));
        }
    }
    results.sort_by(|a, b| {
        b["score"]
            .as_i64()
            .cmp(&a["score"].as_i64())
            .then_with(|| a["path"].to_string().cmp(&b["path"].to_string()))
    });
    results.truncate(limit);
    let result_count = results.len();
    Ok(json!({
        "query": query,
        "query_tokens": query_tokens,
        "index_path": semantic_index_path(config),
        "results": results,
        "result_count": result_count,
    }))
}

fn resume_workspace_plan(config: &Config) -> Value {
    let events = list_audit_entries(&audit_log_path(config), 50).unwrap_or_default();
    let session = read_json_lines(&session_memory_path(config)).unwrap_or_default();
    let preferences = read_preferences(config).unwrap_or_default();
    let mut recent_capabilities = vec![];
    for event in events.iter().rev() {
        if let Some(capability) = event.get("capability").and_then(Value::as_str) {
            if !recent_capabilities.contains(&capability.to_string()) {
                recent_capabilities.push(capability.to_string());
            }
        }
        if recent_capabilities.len() >= 8 {
            break;
        }
    }
    let mut steps = vec![
        json!({"kind": "inspect", "capability": "memory.event.list", "reason": "Review recent audited activity."}),
        json!({"kind": "inspect", "capability": "memory.session.list", "reason": "Review short-term session facts."}),
    ];
    if semantic_index_path(config).exists() {
        steps.push(json!({"kind": "search", "capability": "files.semantic.search", "reason": "Search indexed files for the current task."}));
    }
    if recent_capabilities
        .iter()
        .any(|capability| capability == "apps.launch")
    {
        steps.push(json!({"kind": "desktop", "capability": "apps.list", "reason": "Inspect app registry before relaunching anything."}));
    }
    json!({
        "generated_at": utc_now(),
        "recent_capabilities": recent_capabilities,
        "session_memory_count": session.len(),
        "preference_count": preferences.len(),
        "semantic_index_present": semantic_index_path(config).exists(),
        "steps": steps,
        "note": "This is a plan only; app launch, browser open, and screen capture still require explicit capabilities and confirmation.",
    })
}

fn agent_catalog() -> Vec<AgentDefinition> {
    vec![
        AgentDefinition {
            id: "system.agent".to_string(),
            name: "System Agent".to_string(),
            description: "Reads product, desktop, and screen readiness.".to_string(),
            allowed_capabilities: vec![
                "product.status".to_string(),
                "desktop.status".to_string(),
                "screen.status".to_string(),
            ],
        },
        AgentDefinition {
            id: "memory.agent".to_string(),
            name: "Memory Agent".to_string(),
            description: "Inspects session memory, preferences, events, and resume plans."
                .to_string(),
            allowed_capabilities: vec![
                "memory.session.list".to_string(),
                "memory.preference.list".to_string(),
                "memory.event.list".to_string(),
                "memory.export".to_string(),
                "workspace.resume.plan".to_string(),
            ],
        },
        AgentDefinition {
            id: "file.agent".to_string(),
            name: "File Agent".to_string(),
            description: "Searches opt-in file indexes and reads user-approved text files."
                .to_string(),
            allowed_capabilities: vec![
                "fs.list".to_string(),
                "fs.read_text".to_string(),
                "files.semantic.search".to_string(),
            ],
        },
        AgentDefinition {
            id: "desktop.agent".to_string(),
            name: "Desktop Agent".to_string(),
            description: "Inspects desktop apps, workspace modes, and active context.".to_string(),
            allowed_capabilities: vec![
                "apps.list".to_string(),
                "workspace.mode.plan".to_string(),
                "context.snapshot".to_string(),
            ],
        },
        AgentDefinition {
            id: "writer.agent".to_string(),
            name: "Writer Agent".to_string(),
            description: "Creates safe workspace notes.".to_string(),
            allowed_capabilities: vec!["notes.create".to_string()],
        },
        AgentDefinition {
            id: "predictive.agent".to_string(),
            name: "Predictive Agent".to_string(),
            description: "Detects repeated workflows and builds suggestion-only automations."
                .to_string(),
            allowed_capabilities: vec![
                "proactive.workflow.detect".to_string(),
                "proactive.suggest".to_string(),
                "timeline.explain".to_string(),
            ],
        },
        AgentDefinition {
            id: "healing.agent".to_string(),
            name: "Self-Healing Agent".to_string(),
            description: "Diagnoses recoverable failures without taking destructive action."
                .to_string(),
            allowed_capabilities: vec![
                "selfheal.diagnose".to_string(),
                "timeline.explain".to_string(),
                "memory.event.list".to_string(),
            ],
        },
        AgentDefinition {
            id: "plugin.agent".to_string(),
            name: "Plugin Agent".to_string(),
            description: "Inspects installed plugins and plans plugin-provided workflows."
                .to_string(),
            allowed_capabilities: vec![
                "plugins.catalog".to_string(),
                "plugins.workflow.plan".to_string(),
                "plugins.capability.run".to_string(),
            ],
        },
    ]
}

fn agent_catalog_json() -> Value {
    let agents = agent_catalog();
    json!({
        "agents": agents,
        "agent_count": agents.len(),
        "permission_model": "agents may execute only listed capabilities through policy and audit",
    })
}

fn agent_plan_json(goal: &str, registry: &BTreeMap<String, Capability>) -> Result<Value, String> {
    let steps = build_agent_delegation_plan(goal, registry)?;
    Ok(json!({
        "goal": goal,
        "plan_id": Uuid::new_v4().to_string(),
        "created_at": utc_now(),
        "steps": delegated_steps_json(&steps, registry),
        "agent_count": steps
            .iter()
            .map(|step| step.agent_id.clone())
            .collect::<BTreeSet<_>>()
            .len(),
    }))
}

fn orchestrate_agents(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let goal = string_param(request, "goal")?;
    let registry = build_registry();
    let steps = build_agent_delegation_plan(&goal, &registry)?;
    let mut results = vec![];
    for step in &steps {
        ensure_agent_can_call(&step.agent_id, &step.capability)?;
        let child_request = ActionRequest {
            action_id: Uuid::new_v4().to_string(),
            capability: step.capability.clone(),
            params: step.params.clone(),
            actor: format!("agent:{}", step.agent_id),
            reason: step.reason.clone(),
            dry_run: false,
            confirmed: request.confirmed,
            requested_at: utc_now(),
        };
        let result = execute_capability(config, &registry, child_request);
        let status = result.status;
        results.push(json!({
            "step_id": step.step_id,
            "agent_id": step.agent_id,
            "capability": step.capability,
            "status": status,
            "summary": result.summary,
            "error": result.error,
            "audit_ref": result.audit_ref,
        }));
        if matches!(
            status,
            ActionStatus::Failed | ActionStatus::Denied | ActionStatus::ConfirmationRequired
        ) {
            break;
        }
    }
    let trace_id = Uuid::new_v4().to_string();
    let status = if results
        .iter()
        .all(|result| result["status"] == json!("succeeded"))
    {
        "succeeded"
    } else {
        "partial"
    };
    let trace = json!({
        "trace_id": trace_id,
        "recorded_at": utc_now(),
        "goal": goal,
        "status": status,
        "steps": delegated_steps_json(&steps, &registry),
        "results": results,
    });
    append_json_line(&agent_trace_path(config), &trace)?;
    Ok(json!({
        "trace_id": trace_id,
        "status": status,
        "goal": trace["goal"],
        "steps": trace["steps"],
        "results": trace["results"],
        "trace_path": agent_trace_path(config),
    }))
}

fn list_agent_traces(config: &Config, request: &ActionRequest) -> Result<Value, String> {
    let limit = bounded_limit(request, 20, 100)?;
    let mut traces = read_json_lines(&agent_trace_path(config))?;
    if traces.len() > limit {
        traces = traces.split_off(traces.len() - limit);
    }
    let trace_count = traces.len();
    Ok(json!({
        "path": agent_trace_path(config),
        "traces": traces,
        "trace_count": trace_count,
    }))
}

fn build_agent_delegation_plan(
    goal: &str,
    registry: &BTreeMap<String, Capability>,
) -> Result<Vec<DelegatedCapabilityCall>, String> {
    let lowered = goal.to_ascii_lowercase();
    let mut steps = vec![];
    if lowered.contains("daily brief") || lowered.contains("brief") {
        steps.push(delegated_step(
            "system.agent",
            "product.status",
            Map::new(),
            "Check product runtime status.",
        ));
        steps.push(delegated_step(
            "memory.agent",
            "memory.event.list",
            map_from_pairs([("limit", json!(10))]),
            "Summarize recent local activity.",
        ));
        steps.push(delegated_step(
            "desktop.agent",
            "context.snapshot",
            Map::new(),
            "Inspect current active context with privacy redaction.",
        ));
    } else if lowered.contains("resume") || lowered.contains("continue my work") {
        steps.push(delegated_step(
            "memory.agent",
            "workspace.resume.plan",
            Map::new(),
            "Build a resume plan from local memory.",
        ));
        steps.push(delegated_step(
            "memory.agent",
            "memory.session.list",
            Map::new(),
            "Inspect session memory facts.",
        ));
    } else if lowered.contains("search") || lowered.contains("find") {
        let query = extract_path_after(
            goal,
            &["search files for ", "find files about ", "search ", "find "],
        )
        .unwrap_or_else(|| goal.to_string());
        steps.push(delegated_step(
            "file.agent",
            "files.semantic.search",
            map_from_pairs([("query", json!(query))]),
            "Search the opt-in semantic file index.",
        ));
        steps.push(delegated_step(
            "memory.agent",
            "memory.session.list",
            Map::new(),
            "Add session memory context to the file search.",
        ));
    } else if lowered.contains("repeated workflow") || lowered.contains("automation") {
        steps.push(delegated_step(
            "predictive.agent",
            "proactive.workflow.detect",
            map_from_pairs([("limit", json!(100))]),
            "Detect repeated audited workflows.",
        ));
        steps.push(delegated_step(
            "predictive.agent",
            "proactive.suggest",
            map_from_pairs([("limit", json!(100))]),
            "Turn repeated workflows into suggestion-only automations.",
        ));
    } else if lowered.contains("self heal")
        || lowered.contains("self-heal")
        || lowered.contains("crash")
        || lowered.contains("failed")
    {
        steps.push(delegated_step(
            "healing.agent",
            "selfheal.diagnose",
            map_from_pairs([("symptom", json!("app_crashed"))]),
            "Diagnose the recoverable failure.",
        ));
        steps.push(delegated_step(
            "healing.agent",
            "timeline.explain",
            map_from_pairs([("limit", json!(20))]),
            "Explain recent activity around the failure.",
        ));
    } else if lowered.contains("timeline") || lowered.contains("what happened") {
        steps.push(delegated_step(
            "predictive.agent",
            "timeline.explain",
            map_from_pairs([("limit", json!(25))]),
            "Explain recent local activity.",
        ));
    } else if lowered.contains("plugin") {
        steps.push(delegated_step(
            "plugin.agent",
            "plugins.catalog",
            Map::new(),
            "Inspect installed plugins.",
        ));
        steps.push(delegated_step(
            "plugin.agent",
            "plugins.workflow.plan",
            Map::new(),
            "Plan the first enabled plugin workflow.",
        ));
    } else {
        steps.push(delegated_step(
            "system.agent",
            "product.status",
            Map::new(),
            "Check product runtime status.",
        ));
        steps.push(delegated_step(
            "memory.agent",
            "memory.event.list",
            map_from_pairs([("limit", json!(5))]),
            "Inspect recent local events.",
        ));
    }

    for step in &steps {
        ensure_agent_can_call(&step.agent_id, &step.capability)?;
        if !registry.contains_key(&step.capability) {
            return Err(format!(
                "agent plan references unknown capability: {}",
                step.capability
            ));
        }
    }
    Ok(steps)
}

fn delegated_step(
    agent_id: &str,
    capability: &str,
    params: Map<String, Value>,
    reason: &str,
) -> DelegatedCapabilityCall {
    DelegatedCapabilityCall {
        step_id: Uuid::new_v4().to_string(),
        agent_id: agent_id.to_string(),
        capability: capability.to_string(),
        params,
        reason: reason.to_string(),
    }
}

fn delegated_steps_json(
    steps: &[DelegatedCapabilityCall],
    registry: &BTreeMap<String, Capability>,
) -> Vec<Value> {
    steps
        .iter()
        .map(|step| {
            let risk = registry
                .get(&step.capability)
                .map(|capability| capability.metadata.risk);
            json!({
                "step_id": step.step_id,
                "agent_id": step.agent_id,
                "capability": step.capability,
                "params": step.params,
                "reason": step.reason,
                "risk": risk,
                "requires_confirmation": matches!(risk, Some(RiskLevel::Medium | RiskLevel::High)),
            })
        })
        .collect()
}

fn ensure_agent_can_call(agent_id: &str, capability: &str) -> Result<(), String> {
    let Some(agent) = agent_catalog()
        .into_iter()
        .find(|agent| agent.id == agent_id)
    else {
        return Err(format!("unknown agent: {agent_id}"));
    };
    if agent
        .allowed_capabilities
        .iter()
        .any(|allowed| allowed == capability)
    {
        Ok(())
    } else {
        Err(format!(
            "agent {agent_id} is not permitted to call capability {capability}"
        ))
    }
}

fn map_from_pairs<const N: usize>(pairs: [(&str, Value); N]) -> Map<String, Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

fn read_preferences(config: &Config) -> Result<Map<String, Value>, String> {
    match read_json_file(&preferences_path(config))? {
        Some(Value::Object(object)) => Ok(object),
        Some(_) => Err("preferences file is not a JSON object".to_string()),
        None => Ok(Map::new()),
    }
}

fn collect_semantic_candidates(
    root: &Path,
    recursive: bool,
    out: &mut Vec<PathBuf>,
    max_files: usize,
) -> Result<(), String> {
    if out.len() >= max_files {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path_has_hidden_component(&path) {
            continue;
        }
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        if metadata.is_dir() && recursive {
            collect_semantic_candidates(&path, recursive, out, max_files)?;
        } else if metadata.is_file() {
            out.push(path);
            if out.len() >= max_files {
                break;
            }
        }
    }
    Ok(())
}

fn is_supported_semantic_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(OsStr::to_str) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "md" | "txt" | "rs" | "toml" | "py" | "yml" | "yaml" | "json" | "c" | "h" | "asm"
    )
}

fn path_has_hidden_component(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|value| value.starts_with('.') && value != "." && value != "..")
}

fn unique_tokens(text: &str) -> Vec<String> {
    let mut tokens = BTreeSet::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if current.len() >= 2 {
            tokens.insert(current.clone());
            current.clear();
        } else {
            current.clear();
        }
    }
    if current.len() >= 2 {
        tokens.insert(current);
    }
    tokens.into_iter().take(512).collect()
}

fn summarize_document_text(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_text(&compact, 240)
}

fn safe_memory_key(value: &str) -> String {
    let re = SAFE_FILENAME_RE.get_or_init(|| Regex::new(r"[^A-Za-z0-9._-]+").unwrap());
    let key = re
        .replace_all(value.trim(), "-")
        .trim_matches(&['.', '-', '_'][..])
        .to_ascii_lowercase();
    truncate_text(if key.is_empty() { "memory" } else { &key }, 80)
}

fn validate_memory_key(key: &str) -> Result<(), String> {
    if safe_memory_key(key).trim().is_empty() {
        Err("memory key cannot be empty".to_string())
    } else if is_sensitive_memory_key(key) {
        Err("memory keys must not contain secret-like names".to_string())
    } else {
        Ok(())
    }
}

fn is_sensitive_memory_key(key: &str) -> bool {
    let lowered = key
        .trim()
        .to_ascii_lowercase()
        .replace(['-', ' ', '.'], "_");
    [
        "secret",
        "token",
        "password",
        "credential",
        "api_key",
        "private_key",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
}

fn validate_memory_delete_scope(scope: &str) -> Result<(), String> {
    match scope.trim().to_ascii_lowercase().as_str() {
        "session" | "preferences" | "semantic_index" | "traces" | "all" => Ok(()),
        _ => Err(
            "memory delete scope must be session, preferences, semantic_index, traces, or all"
                .to_string(),
        ),
    }
}

fn validate_selfheal_symptom(symptom: &str) -> Result<(), String> {
    match symptom.trim().to_ascii_lowercase().as_str() {
        "unknown" | "app_crashed" | "app_failed" | "service_failed" | "memory_pressure"
        | "slow_operation" => Ok(()),
        _ => Err(
            "self-healing symptom must be unknown, app_crashed, app_failed, service_failed, memory_pressure, or slow_operation"
                .to_string(),
        ),
    }
}

fn bounded_limit(request: &ActionRequest, default: usize, max: usize) -> Result<usize, String> {
    let value = request
        .params
        .get("limit")
        .or_else(|| request.params.get("max_files"))
        .and_then(Value::as_u64)
        .unwrap_or(default as u64);
    if value == 0 || value > max as u64 {
        return Err(format!("limit must be between 1 and {max}"));
    }
    Ok(value as usize)
}

fn append_json_line(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| err.to_string())?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(value).map_err(|err| err.to_string())?
    )
    .map_err(|err| err.to_string())
}

fn write_json_lines(path: &Path, values: &[Value]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let mut file = File::create(path).map_err(|err| err.to_string())?;
    for value in values {
        writeln!(
            file,
            "{}",
            serde_json::to_string(value).map_err(|err| err.to_string())?
        )
        .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(err) => return Err(err.to_string()),
    };
    let reader = io::BufReader::new(file);
    let mut values = vec![];
    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            values.push(value);
        }
    }
    Ok(values)
}

fn read_json_file(path: &Path) -> Result<Option<Value>, String> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|err| err.to_string()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(path, text).map_err(|err| err.to_string())
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

fn system_time_json(value: std::time::SystemTime) -> Option<Value> {
    value
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| json!(duration.as_secs()))
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
        memory_session_remember_capability(),
        memory_session_list_capability(),
        memory_preference_set_capability(),
        memory_preference_list_capability(),
        memory_delete_capability(),
        memory_export_capability(),
        memory_event_list_capability(),
        files_semantic_index_capability(),
        files_semantic_search_capability(),
        workspace_resume_plan_capability(),
        agents_catalog_capability(),
        agents_plan_capability(),
        agents_orchestrate_capability(),
        agents_trace_list_capability(),
        proactive_workflow_detect_capability(),
        proactive_suggest_capability(),
        selfheal_diagnose_capability(),
        timeline_explain_capability(),
        plugins_validate_capability(),
        plugins_package_validate_capability(),
        plugins_permission_review_capability(),
        plugins_install_capability(),
        plugins_disable_capability(),
        plugins_remove_capability(),
        plugins_catalog_capability(),
        plugins_workflow_plan_capability(),
        plugins_capability_run_capability(),
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

fn memory_session_remember_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "memory.session.remember".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Store a user-approved short-term session memory fact.".to_string(),
            risk: RiskLevel::Low,
            permissions: vec!["memory:write".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("key".to_string(), "string".to_string()),
                    ("value".to_string(), "string".to_string()),
                    ("tags".to_string(), "array".to_string()),
                ]),
                vec!["key", "value"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: true,
        },
        execute: |request, config| remember_session_memory(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("memory_id").is_some() && data.get("stored") == Some(&json!(true)),
            message: "Session memory fact stored.".to_string(),
            data: json!({}),
        },
    }
}

fn memory_session_list_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "memory.session.list".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "List inspectable short-term session memory facts.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["memory:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("query".to_string(), "string".to_string()),
                    ("limit".to_string(), "integer".to_string()),
                ]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| list_session_memory(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("items").is_some(),
            message: "Session memory list returned.".to_string(),
            data: json!({}),
        },
    }
}

fn memory_preference_set_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "memory.preference.set".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Create or update a user preference in local memory.".to_string(),
            risk: RiskLevel::Low,
            permissions: vec!["memory:write".to_string(), "preferences:write".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("key".to_string(), "string".to_string()),
                    ("value".to_string(), "string".to_string()),
                ]),
                vec!["key", "value"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: true,
        },
        execute: |request, config| set_preference(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("key").is_some() && data.get("updated") == Some(&json!(true)),
            message: "Preference stored.".to_string(),
            data: json!({}),
        },
    }
}

fn memory_preference_list_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "memory.preference.list".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Inspect local user preferences.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["memory:read".to_string(), "preferences:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("key".to_string(), "string".to_string())]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| list_preferences(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("preferences").is_some(),
            message: "Preferences returned.".to_string(),
            data: json!({}),
        },
    }
}

fn memory_delete_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "memory.delete".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Delete selected local memory, preferences, index, or traces.".to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["memory:delete".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("scope".to_string(), "string".to_string()),
                    ("key".to_string(), "string".to_string()),
                ]),
                vec!["scope"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| delete_memory(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("deleted").is_some(),
            message: "Memory delete request completed.".to_string(),
            data: json!({}),
        },
    }
}

fn memory_export_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "memory.export".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Export inspectable local memory state without secrets.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["memory:read".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, config| Ok(export_memory(config)),
        verify: |_, _, data| Verification {
            ok: data.get("session").is_some() && data.get("preferences").is_some(),
            message: "Memory export returned.".to_string(),
            data: json!({}),
        },
    }
}

fn memory_event_list_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "memory.event.list".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "List recent local capability events from the audit log.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["memory:read".to_string(), "audit:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("limit".to_string(), "integer".to_string())]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| list_memory_events(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("events").is_some(),
            message: "Memory events returned from audit history.".to_string(),
            data: json!({}),
        },
    }
}

fn files_semantic_index_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "files.semantic.index".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Build an opt-in local semantic file index for text files.".to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["files:index".to_string(), "memory:write".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("root".to_string(), "string".to_string()),
                    ("recursive".to_string(), "boolean".to_string()),
                    ("max_files".to_string(), "integer".to_string()),
                ]),
                vec!["root"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: true,
        },
        execute: |request, config| index_semantic_files(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("document_count").and_then(Value::as_u64).is_some(),
            message: "Semantic file index written.".to_string(),
            data: json!({}),
        },
    }
}

fn files_semantic_search_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "files.semantic.search".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Search the opt-in local semantic file index.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["files:search".to_string(), "memory:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("query".to_string(), "string".to_string()),
                    ("limit".to_string(), "integer".to_string()),
                ]),
                vec!["query"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| search_semantic_files(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("results").is_some(),
            message: "Semantic search returned results.".to_string(),
            data: json!({}),
        },
    }
}

fn workspace_resume_plan_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "workspace.resume.plan".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Build a local memory-backed plan to resume recent work.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["workspace:plan".to_string(), "memory:read".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, config| Ok(resume_workspace_plan(config)),
        verify: |_, _, data| Verification {
            ok: data.get("steps").is_some(),
            message: "Workspace resume plan returned.".to_string(),
            data: json!({}),
        },
    }
}

fn agents_catalog_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "agents.catalog".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "List built-in agents and their allowed capabilities.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["agents:read".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, _| Ok(agent_catalog_json()),
        verify: |_, _, data| Verification {
            ok: data.get("agents").is_some(),
            message: "Agent catalog returned.".to_string(),
            data: json!({}),
        },
    }
}

fn agents_plan_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "agents.plan".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Create a deterministic multi-agent delegation plan.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["agents:plan".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("goal".to_string(), "string".to_string())]),
                vec!["goal"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, _| {
            let goal = string_param(request, "goal")?;
            agent_plan_json(&goal, &build_registry())
        },
        verify: |_, _, data| Verification {
            ok: data.get("steps").is_some(),
            message: "Agent delegation plan returned.".to_string(),
            data: json!({}),
        },
    }
}

fn agents_orchestrate_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "agents.orchestrate".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Run a deterministic multi-agent plan through permitted capabilities."
                .to_string(),
            risk: RiskLevel::Medium,
            permissions: vec![
                "agents:run".to_string(),
                "capabilities:delegate".to_string(),
            ],
            input_schema: object_schema(
                BTreeMap::from([("goal".to_string(), "string".to_string())]),
                vec!["goal"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| orchestrate_agents(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("results").is_some() && data.get("trace_id").is_some(),
            message: "Agent orchestration completed with replayable trace.".to_string(),
            data: json!({}),
        },
    }
}

fn agents_trace_list_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "agents.trace.list".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "List recent multi-agent orchestration traces.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["agents:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("limit".to_string(), "integer".to_string())]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| list_agent_traces(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("traces").is_some(),
            message: "Agent traces returned.".to_string(),
            data: json!({}),
        },
    }
}

fn proactive_workflow_detect_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "proactive.workflow.detect".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Detect repeated audited workflows and suggest automations.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["proactive:read".to_string(), "audit:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("limit".to_string(), "integer".to_string()),
                    ("min_repetitions".to_string(), "integer".to_string()),
                ]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| detect_repeated_workflows(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("suggestions").is_some(),
            message: "Repeated workflow detection returned suggestions.".to_string(),
            data: json!({}),
        },
    }
}

fn proactive_suggest_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "proactive.suggest".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Build safe proactive suggestions from local events.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["proactive:read".to_string(), "audit:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("limit".to_string(), "integer".to_string()),
                    ("min_repetitions".to_string(), "integer".to_string()),
                ]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| proactive_suggestions(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("suggestions").is_some(),
            message: "Proactive suggestions returned without executing actions.".to_string(),
            data: json!({}),
        },
    }
}

fn selfheal_diagnose_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "selfheal.diagnose".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Diagnose recoverable failures and return safe recovery steps."
                .to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["selfheal:read".to_string(), "audit:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("symptom".to_string(), "string".to_string()),
                    ("target".to_string(), "string".to_string()),
                    ("simulated".to_string(), "boolean".to_string()),
                ]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| self_heal_diagnose(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("recommended_actions").is_some(),
            message: "Self-healing diagnosis returned safe recommendations.".to_string(),
            data: json!({}),
        },
    }
}

fn timeline_explain_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "timeline.explain".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Explain recent activity from audit events, memory, and agent traces."
                .to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["timeline:read".to_string(), "audit:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("limit".to_string(), "integer".to_string())]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| explain_timeline(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("timeline").is_some(),
            message: "Timeline explanation returned.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_validate_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.validate".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Validate a local plugin manifest without installing it.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["plugins:read".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("source".to_string(), "string".to_string())]),
                vec!["source"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| validate_plugin_manifest_capability(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("valid") == Some(&json!(true)),
            message: "Plugin manifest validated.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_package_validate_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.package.validate".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Validate plugin package trust metadata before install.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["plugins:read".to_string(), "plugins:trust".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("source".to_string(), "string".to_string())]),
                vec!["source"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| validate_plugin_package(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("valid") == Some(&json!(true)) && data.get("plugin_trust_state").is_some(),
            message: "Plugin package metadata validated.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_permission_review_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.permission.review".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Generate a user-facing plugin permission review.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["plugins:read".to_string(), "plugins:review".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("source".to_string(), "string".to_string()),
                    ("plugin_id".to_string(), "string".to_string()),
                ]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| review_plugin_permissions(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("permission_summary").is_some() && data.get("approval").is_some(),
            message: "Plugin permission review returned.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_install_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.install".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Install a validated local plugin manifest.".to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["plugins:install".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("source".to_string(), "string".to_string()),
                    ("force".to_string(), "boolean".to_string()),
                ]),
                vec!["source"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: true,
        },
        execute: |request, config| install_plugin(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("installed") == Some(&json!(true))
                && data.get("plugin_identity").is_some(),
            message: "Plugin installed with identity.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_disable_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.disable".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Disable an installed plugin without deleting its manifest.".to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["plugins:disable".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("plugin_id".to_string(), "string".to_string())]),
                vec!["plugin_id"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: true,
        },
        execute: |request, config| disable_plugin(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("disabled") == Some(&json!(true)) && data.get("plugin_identity").is_some(),
            message: "Plugin disabled with identity.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_remove_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.remove".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Remove an installed plugin manifest from local state.".to_string(),
            risk: RiskLevel::Medium,
            permissions: vec!["plugins:remove".to_string()],
            input_schema: object_schema(
                BTreeMap::from([("plugin_id".to_string(), "string".to_string())]),
                vec!["plugin_id"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| remove_plugin(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("removed") == Some(&json!(true)) && data.get("plugin_identity").is_some(),
            message: "Plugin removed with identity.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_catalog_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.catalog".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "List installed plugins, capabilities, workflows, and enabled state."
                .to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["plugins:read".to_string()],
            input_schema: object_schema(BTreeMap::<String, String>::new(), vec![]),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |_, config| catalog_plugins(config),
        verify: |_, _, data| Verification {
            ok: data.get("plugins").is_some(),
            message: "Plugin catalog returned.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_workflow_plan_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.workflow.plan".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Plan a plugin-provided workflow without executing it.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["plugins:read".to_string(), "workflows:plan".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("plugin_id".to_string(), "string".to_string()),
                    ("workflow".to_string(), "string".to_string()),
                ]),
                vec![],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| plan_plugin_workflow(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("steps").is_some() && data.get("plugin_id").is_some(),
            message: "Plugin workflow plan returned.".to_string(),
            data: json!({}),
        },
    }
}

fn plugins_capability_run_capability() -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: "plugins.capability.run".to_string(),
            version: "1.0.0".to_string(),
            owner: "huggingos".to_string(),
            description: "Run a declarative read-only plugin capability.".to_string(),
            risk: RiskLevel::Read,
            permissions: vec!["plugins:run".to_string()],
            input_schema: object_schema(
                BTreeMap::from([
                    ("plugin_id".to_string(), "string".to_string()),
                    ("capability".to_string(), "string".to_string()),
                ]),
                vec!["plugin_id", "capability"],
            ),
            result_schema: json!({ "type": "object" }),
            reversible: false,
        },
        execute: |request, config| run_plugin_capability(config, request),
        verify: |_, _, data| Verification {
            ok: data.get("plugin_identity").is_some() && data.get("response").is_some(),
            message: "Plugin capability returned with identity.".to_string(),
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
            "boolean" if !value.is_boolean() => {
                return Err(format!("Parameter {key} must be a boolean."));
            }
            "array" if !value.is_array() => {
                return Err(format!("Parameter {key} must be an array."));
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
        "memory.session.remember" | "memory.preference.set" => {
            let key = params
                .get("key")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter key must be a string.".to_string())?;
            validate_memory_key(key)?;
            let value = params
                .get("value")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter value must be a string.".to_string())?;
            if value.trim().is_empty() {
                return Err("memory value cannot be empty".to_string());
            }
            Ok(())
        }
        "memory.preference.list" => {
            if let Some(key) = params.get("key").and_then(Value::as_str) {
                validate_memory_key(key)?;
            }
            Ok(())
        }
        "memory.delete" => {
            let scope = params
                .get("scope")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter scope must be a string.".to_string())?;
            validate_memory_delete_scope(scope)?;
            if let Some(key) = params.get("key").and_then(Value::as_str) {
                validate_memory_key(key)?;
            }
            Ok(())
        }
        "files.semantic.index" => {
            let root = params
                .get("root")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter root must be a string.".to_string())?;
            if is_sensitive_path(&json!(root)) {
                return Err("Sensitive paths require a higher-risk capability.".to_string());
            }
            Ok(())
        }
        "files.semantic.search" => {
            let query = params
                .get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter query must be a string.".to_string())?;
            if query.trim().is_empty() {
                return Err("semantic search query cannot be empty".to_string());
            }
            Ok(())
        }
        "agents.plan" | "agents.orchestrate" => {
            let goal = params
                .get("goal")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter goal must be a string.".to_string())?;
            if goal.trim().is_empty() {
                return Err("agent goal cannot be empty".to_string());
            }
            Ok(())
        }
        "selfheal.diagnose" => {
            if let Some(symptom) = params.get("symptom").and_then(Value::as_str) {
                validate_selfheal_symptom(symptom)?;
            }
            Ok(())
        }
        "plugins.validate" | "plugins.package.validate" | "plugins.install" => {
            let source = params
                .get("source")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter source must be a string.".to_string())?;
            if is_sensitive_path(&json!(source)) {
                return Err("plugin source path is sensitive".to_string());
            }
            Ok(())
        }
        "plugins.permission.review" => {
            let source = params.get("source").and_then(Value::as_str);
            let plugin_id = params.get("plugin_id").and_then(Value::as_str);
            if source.is_none() && plugin_id.is_none() {
                return Err("plugins.permission.review requires source or plugin_id".to_string());
            }
            if let Some(source) = source {
                if is_sensitive_path(&json!(source)) {
                    return Err("plugin source path is sensitive".to_string());
                }
            }
            if let Some(plugin_id) = plugin_id {
                validate_plugin_id(&safe_plugin_id(plugin_id))?;
            }
            Ok(())
        }
        "plugins.disable" | "plugins.remove" => {
            let plugin_id = params
                .get("plugin_id")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter plugin_id must be a string.".to_string())?;
            validate_plugin_id(&safe_plugin_id(plugin_id))
        }
        "plugins.workflow.plan" => {
            if let Some(plugin_id) = params.get("plugin_id").and_then(Value::as_str) {
                validate_plugin_id(&safe_plugin_id(plugin_id))?;
            }
            if let Some(workflow) = params.get("workflow").and_then(Value::as_str) {
                validate_plugin_id(&safe_plugin_id(workflow))?;
            }
            Ok(())
        }
        "plugins.capability.run" => {
            let plugin_id = params
                .get("plugin_id")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter plugin_id must be a string.".to_string())?;
            let capability = params
                .get("capability")
                .and_then(Value::as_str)
                .ok_or_else(|| "Parameter capability must be a string.".to_string())?;
            validate_plugin_id(&safe_plugin_id(plugin_id))?;
            validate_plugin_id(&safe_plugin_id(capability))
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
        "plugin_identity": request.params.get("plugin_id"),
        "plugin_trust_state": result.data.get("plugin_trust_state"),
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

    fn write_sample_plugin(tmp: &TempDir) -> PathBuf {
        let plugin_dir = tmp.path().join("sample-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("plugin.json"),
            serde_json::to_string_pretty(&json!({
                "schema_version": "huggingos.plugin.v1",
                "id": "sample.hello",
                "name": "Sample Hello Plugin",
                "version": "1.0.0",
                "description": "Sample third-party plugin for tests.",
                "package": {
                    "format": "huggingos.plugin.package.v1",
                    "source": "test-fixture",
                    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                    "signature": {
                        "algorithm": "ed25519",
                        "key_id": "test-key",
                        "signature": "metadata-only"
                    }
                },
                "ui": {
                    "display_name": "Sample Hello Plugin",
                    "approval_summary": "Adds one read-only greeting capability and one workflow.",
                    "icon": "plugin-sample"
                },
                "sandbox": {
                    "code_execution": "disabled",
                    "network": false,
                    "filesystem": "none"
                },
                "permissions": ["plugins:read"],
                "capabilities": [
                    {
                        "name": "hello",
                        "description": "Return a sample greeting.",
                        "risk": "read",
                        "permissions": ["plugins:read"],
                        "response": {
                            "message": "hello from plugin"
                        }
                    }
                ],
                "workflows": [
                    {
                        "name": "hello-workflow",
                        "description": "Run the sample greeting capability.",
                        "steps": [
                            {
                                "capability": "plugins.capability.run",
                                "params": {
                                    "plugin_id": "sample.hello",
                                    "capability": "hello"
                                },
                                "reason": "Run the sample plugin greeting."
                            }
                        ]
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        plugin_dir
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
    fn memory_session_remember_and_list_round_trips() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("key".to_string(), json!("editor"));
        params.insert("value".to_string(), json!("Use Helix for quick edits"));

        let result = execute_capability(
            &config,
            &build_registry(),
            request("memory.session.remember", params),
        );
        assert_eq!(result.status, ActionStatus::Succeeded);

        let list = execute_capability(
            &config,
            &build_registry(),
            request("memory.session.list", Map::new()),
        );
        assert_eq!(list.status, ActionStatus::Succeeded);
        assert_eq!(list.data["item_count"], json!(1));
        assert_eq!(list.data["items"][0]["key"], json!("editor"));
    }

    #[test]
    fn memory_delete_removes_session_key() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("key".to_string(), json!("project"));
        params.insert("value".to_string(), json!("huggingOS"));
        execute_capability(
            &config,
            &build_registry(),
            request("memory.session.remember", params),
        );

        let mut delete_params = Map::new();
        delete_params.insert("scope".to_string(), json!("session"));
        delete_params.insert("key".to_string(), json!("project"));
        let mut delete_request = request("memory.delete", delete_params);
        delete_request.confirmed = true;
        let result = execute_capability(&config, &build_registry(), delete_request);
        assert_eq!(result.status, ActionStatus::Succeeded);

        let list = execute_capability(
            &config,
            &build_registry(),
            request("memory.session.list", Map::new()),
        );
        assert_eq!(list.data["item_count"], json!(0));
    }

    #[test]
    fn preference_set_and_list_round_trips() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("key".to_string(), json!("theme"));
        params.insert("value".to_string(), json!("dark"));

        let result = execute_capability(
            &config,
            &build_registry(),
            request("memory.preference.set", params),
        );
        assert_eq!(result.status, ActionStatus::Succeeded);

        let list = execute_capability(
            &config,
            &build_registry(),
            request("memory.preference.list", Map::new()),
        );
        assert_eq!(list.status, ActionStatus::Succeeded);
        assert_eq!(list.data["preferences"]["theme"]["value"], json!("dark"));
    }

    #[test]
    fn memory_rejects_secret_like_keys() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("key".to_string(), json!("api_key"));
        params.insert("value".to_string(), json!("do-not-store"));

        let result = execute_capability(
            &config,
            &build_registry(),
            request("memory.preference.set", params),
        );

        assert_eq!(result.status, ActionStatus::Denied);
        assert!(result.error.unwrap().contains("secret-like"));
    }

    #[test]
    fn semantic_index_and_search_are_opt_in_and_local() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let docs = tmp.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("memory.md"),
            "semantic memory search helps resume workspace context",
        )
        .unwrap();
        fs::write(docs.join(".env.local"), "SECRET=hidden").unwrap();

        let mut params = Map::new();
        params.insert("root".to_string(), json!(docs.to_string_lossy()));
        params.insert("recursive".to_string(), json!(true));
        let mut req = request("files.semantic.index", params);
        req.confirmed = true;
        let index = execute_capability(&config, &build_registry(), req);
        assert_eq!(index.status, ActionStatus::Succeeded);
        assert_eq!(index.data["document_count"], json!(1));

        let mut search_params = Map::new();
        search_params.insert("query".to_string(), json!("workspace memory"));
        let search = execute_capability(
            &config,
            &build_registry(),
            request("files.semantic.search", search_params),
        );
        assert_eq!(search.status, ActionStatus::Succeeded);
        assert_eq!(search.data["result_count"], json!(1));
        assert!(search.data["results"][0]["path"]
            .as_str()
            .unwrap()
            .ends_with("memory.md"));
    }

    #[test]
    fn agent_catalog_enforces_allowed_capabilities() {
        assert!(ensure_agent_can_call("system.agent", "product.status").is_ok());
        assert!(ensure_agent_can_call("memory.agent", "apps.launch").is_err());
    }

    #[test]
    fn agent_orchestration_delegates_and_records_trace() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("goal".to_string(), json!("daily brief"));
        let mut req = request("agents.orchestrate", params);
        req.confirmed = true;

        let result = execute_capability(&config, &build_registry(), req);

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert_eq!(result.data["results"].as_array().unwrap().len(), 3);
        assert!(agent_trace_path(&config).exists());
        let agents = result.data["steps"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|step| step["agent_id"].as_str())
            .collect::<BTreeSet<_>>();
        assert!(agents.len() >= 2);
    }

    #[test]
    fn local_planner_maps_phase6_and_phase7_prompts() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();

        let memory_plan = build_ai_plan(
            &config,
            &registry,
            "remember that color is blue",
            Some("local.rules"),
        )
        .unwrap();
        let resume_plan = build_ai_plan(
            &config,
            &registry,
            "resume my workspace",
            Some("local.rules"),
        )
        .unwrap();
        let agent_plan =
            build_ai_plan(&config, &registry, "daily brief", Some("local.rules")).unwrap();

        assert_eq!(memory_plan.steps[0].capability, "memory.session.remember");
        assert_eq!(resume_plan.steps[0].capability, "workspace.resume.plan");
        assert_eq!(agent_plan.steps[0].capability, "agents.orchestrate");
    }

    #[test]
    fn repeated_workflow_detection_suggests_automation_without_execution() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();
        execute_capability(&config, &registry, request("product.status", Map::new()));
        execute_capability(&config, &registry, request("memory.event.list", Map::new()));
        execute_capability(&config, &registry, request("product.status", Map::new()));
        execute_capability(&config, &registry, request("memory.event.list", Map::new()));

        let result = execute_capability(
            &config,
            &registry,
            request("proactive.workflow.detect", Map::new()),
        );

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert!(result.data["suggestion_count"].as_u64().unwrap() >= 1);
        assert_eq!(
            result.data["suggestions"][0]["requires_confirmation"],
            json!(true)
        );
        assert_eq!(
            result.data["suggestions"][0]["policy"],
            json!("suggestion_only")
        );
    }

    #[test]
    fn self_heal_diagnoses_simulated_app_failure() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let mut params = Map::new();
        params.insert("symptom".to_string(), json!("app_crashed"));
        params.insert("target".to_string(), json!("editor"));
        params.insert("simulated".to_string(), json!(true));

        let result = execute_capability(
            &config,
            &build_registry(),
            request("selfheal.diagnose", params),
        );

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert_eq!(result.data["simulated"], json!(true));
        assert!(result.data["recommended_actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|step| step["capability"] == json!("apps.launch")));
    }

    #[test]
    fn timeline_explain_combines_recent_events_and_context_counts() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        execute_capability(
            &config,
            &build_registry(),
            request("product.status", Map::new()),
        );

        let result = execute_capability(
            &config,
            &build_registry(),
            request("timeline.explain", Map::new()),
        );

        assert_eq!(result.status, ActionStatus::Succeeded);
        assert!(result.data["event_count"].as_u64().unwrap() >= 1);
        assert!(result.data["context"]["session_memory_count"].is_number());
    }

    #[test]
    fn local_planner_maps_phase8_prompts() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();

        let workflow_plan = build_ai_plan(
            &config,
            &registry,
            "detect repeated workflow",
            Some("local.rules"),
        )
        .unwrap();
        let healing_plan = build_ai_plan(
            &config,
            &registry,
            "app crashed, self heal it",
            Some("local.rules"),
        )
        .unwrap();
        let timeline_plan = build_ai_plan(
            &config,
            &registry,
            "explain what happened",
            Some("local.rules"),
        )
        .unwrap();

        assert_eq!(
            workflow_plan.steps[0].capability,
            "proactive.workflow.detect"
        );
        assert_eq!(healing_plan.steps[0].capability, "selfheal.diagnose");
        assert_eq!(timeline_plan.steps[0].capability, "timeline.explain");
    }

    #[test]
    fn plugin_manifest_install_run_disable_and_remove_round_trips() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let source = write_sample_plugin(&tmp);
        let registry = build_registry();

        let mut validate_params = Map::new();
        validate_params.insert("source".to_string(), json!(source));
        let validate = execute_capability(
            &config,
            &registry,
            request("plugins.validate", validate_params),
        );
        assert_eq!(validate.status, ActionStatus::Succeeded);
        assert_eq!(validate.data["manifest"]["id"], json!("sample.hello"));
        assert_eq!(
            validate.data["plugin_trust_state"]["state"],
            json!("signed_metadata_present_unverified")
        );

        let mut package_params = Map::new();
        package_params.insert("source".to_string(), json!(source));
        let package = execute_capability(
            &config,
            &registry,
            request("plugins.package.validate", package_params),
        );
        assert_eq!(package.status, ActionStatus::Succeeded);
        assert_eq!(
            package.data["install_preview"]["requires_confirmation"],
            json!(true)
        );
        assert_eq!(package.data["sandbox"]["code_execution"], json!("disabled"));

        let mut review_params = Map::new();
        review_params.insert("source".to_string(), json!(source));
        let review = execute_capability(
            &config,
            &registry,
            request("plugins.permission.review", review_params),
        );
        assert_eq!(review.status, ActionStatus::Succeeded);
        assert_eq!(
            review.data["permission_summary"]["arbitrary_code_execution"],
            json!(false)
        );

        let mut install_params = Map::new();
        install_params.insert("source".to_string(), json!(source));
        let mut install_request = request("plugins.install", install_params);
        install_request.confirmed = true;
        let install = execute_capability(&config, &registry, install_request);
        assert_eq!(install.status, ActionStatus::Succeeded);
        assert_eq!(install.data["plugin_identity"], json!("sample.hello"));
        assert_eq!(
            install.data["plugin_trust_state"]["state"],
            json!("signed_metadata_present_unverified")
        );
        assert_eq!(install.data["rollback"]["type"], json!("remove"));

        let catalog =
            execute_capability(&config, &registry, request("plugins.catalog", Map::new()));
        assert_eq!(catalog.status, ActionStatus::Succeeded);
        assert_eq!(catalog.data["plugin_count"], json!(1));
        assert_eq!(catalog.data["plugins"][0]["enabled"], json!(true));

        let mut workflow_params = Map::new();
        workflow_params.insert("plugin_id".to_string(), json!("sample.hello"));
        let workflow = execute_capability(
            &config,
            &registry,
            request("plugins.workflow.plan", workflow_params),
        );
        assert_eq!(workflow.status, ActionStatus::Succeeded);
        assert_eq!(workflow.data["step_count"], json!(1));

        let mut run_params = Map::new();
        run_params.insert("plugin_id".to_string(), json!("sample.hello"));
        run_params.insert("capability".to_string(), json!("hello"));
        let run = execute_capability(
            &config,
            &registry,
            request("plugins.capability.run", run_params),
        );
        assert_eq!(run.status, ActionStatus::Succeeded);
        assert_eq!(run.data["response"]["message"], json!("hello from plugin"));

        let audit = list_audit_entries(&audit_log_path(&config), 20).unwrap();
        assert!(audit.iter().any(|entry| {
            entry["capability"] == json!("plugins.capability.run")
                && entry["plugin_identity"] == json!("sample.hello")
                && entry["plugin_trust_state"]["state"]
                    == json!("signed_metadata_present_unverified")
        }));

        let mut disable_params = Map::new();
        disable_params.insert("plugin_id".to_string(), json!("sample.hello"));
        let mut disable_request = request("plugins.disable", disable_params);
        disable_request.confirmed = true;
        let disable = execute_capability(&config, &registry, disable_request);
        assert_eq!(disable.status, ActionStatus::Succeeded);

        let mut disabled_run_params = Map::new();
        disabled_run_params.insert("plugin_id".to_string(), json!("sample.hello"));
        disabled_run_params.insert("capability".to_string(), json!("hello"));
        let disabled_run = execute_capability(
            &config,
            &registry,
            request("plugins.capability.run", disabled_run_params),
        );
        assert_eq!(disabled_run.status, ActionStatus::Failed);

        let mut remove_params = Map::new();
        remove_params.insert("plugin_id".to_string(), json!("sample.hello"));
        let mut remove_request = request("plugins.remove", remove_params);
        remove_request.confirmed = true;
        let remove = execute_capability(&config, &registry, remove_request);
        assert_eq!(remove.status, ActionStatus::Succeeded);
        assert!(!plugin_install_dir(&config, "sample.hello").exists());
    }

    #[test]
    fn plugin_install_requires_confirmation() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let source = write_sample_plugin(&tmp);
        let mut params = Map::new();
        params.insert("source".to_string(), json!(source));

        let result = execute_capability(
            &config,
            &build_registry(),
            request("plugins.install", params),
        );

        assert_eq!(result.status, ActionStatus::ConfirmationRequired);
    }

    #[test]
    fn local_planner_maps_phase9_plugin_prompts() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(&tmp);
        let registry = build_registry();

        let catalog_plan =
            build_ai_plan(&config, &registry, "list plugins", Some("local.rules")).unwrap();
        let workflow_plan = build_ai_plan(
            &config,
            &registry,
            "plugin workflow sample.hello",
            Some("local.rules"),
        )
        .unwrap();

        assert_eq!(catalog_plan.steps[0].capability, "plugins.catalog");
        assert_eq!(workflow_plan.steps[0].capability, "plugins.workflow.plan");
        assert_eq!(
            workflow_plan.steps[0].params["plugin_id"],
            json!("sample.hello")
        );
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
