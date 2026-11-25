//! `piiguard scan|redact|restore`：输入为文件或 stdin。
//! scan 输出 JSON 报告；redact/restore 输出处理后的文本。

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use piiguard::{Action, Config, Finding, Vault, redact, restore};

#[derive(Parser)]
#[command(name = "piiguard", version, about = "PII 检测与可逆脱敏")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 扫描并输出 JSON 报告（不含命中原文）
    Scan(AnalyzeOpts),
    /// 脱敏并输出文本
    Redact {
        #[command(flatten)]
        opts: AnalyzeOpts,
        #[arg(long, value_enum, default_value_t = ActionKind::Replace)]
        action: ActionKind,
        #[arg(long, default_value_t = '*')]
        mask_char: char,
        /// pseudonymize 必填：映射表 JSON 文件（已存在则追加）
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    /// 用映射表把占位符还原
    Restore {
        input: Option<PathBuf>,
        #[arg(long)]
        vault: PathBuf,
    },
}

#[derive(Args)]
struct AnalyzeOpts {
    /// 输入文件，省略则读 stdin
    input: Option<PathBuf>,
    /// JSON 配置文件
    #[arg(long)]
    config: Option<PathBuf>,
    /// 覆盖配置中的分数阈值
    #[arg(long)]
    threshold: Option<f32>,
    /// 覆盖配置中的实体列表，逗号分隔
    #[arg(long, value_delimiter = ',')]
    entities: Option<Vec<String>>,
}

#[derive(Clone, Copy, ValueEnum)]
enum ActionKind {
    Mask,
    Replace,
    Pseudonymize,
}

#[derive(serde::Serialize)]
struct Report<'a> {
    count: usize,
    findings: &'a [Finding],
}

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

fn read_input(path: Option<&Path>) -> CliResult<String> {
    match path {
        Some(p) => Ok(fs::read_to_string(p)?),
        None => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            Ok(s)
        }
    }
}

fn analyze(opts: &AnalyzeOpts) -> CliResult<(String, Vec<Finding>)> {
    let mut cfg = match &opts.config {
        Some(p) => Config::from_json(&fs::read_to_string(p)?)?,
        None => Config::default(),
    };
    if let Some(e) = &opts.entities {
        cfg.entities = Some(e.clone());
    }
    if let Some(t) = opts.threshold {
        cfg.threshold = Some(t);
    }
    let analyzer = cfg.into_builder()?.build()?;
    let text = read_input(opts.input.as_deref())?;
    let findings = analyzer.analyze(&text);
    Ok((text, findings))
}

fn load_vault(path: &Path) -> CliResult<Vault> {
    if path.exists() {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    } else {
        Ok(Vault::new())
    }
}

fn run(cli: Cli) -> CliResult<String> {
    match cli.cmd {
        Cmd::Scan(opts) => {
            let (_, findings) = analyze(&opts)?;
            let report = Report {
                count: findings.len(),
                findings: &findings,
            };
            Ok(serde_json::to_string_pretty(&report)? + "\n")
        }
        Cmd::Redact {
            opts,
            action,
            mask_char,
            vault,
        } => {
            let (text, findings) = analyze(&opts)?;
            match action {
                ActionKind::Mask => Ok(redact(&text, &findings, Action::Mask { ch: mask_char })),
                ActionKind::Replace => Ok(redact(&text, &findings, Action::Replace)),
                ActionKind::Pseudonymize => {
                    let path = vault.ok_or("--action pseudonymize 需要 --vault <FILE>")?;
                    let mut v = load_vault(&path)?;
                    let out = redact(&text, &findings, Action::Pseudonymize(&mut v));
                    fs::write(&path, serde_json::to_string_pretty(&v)?)?;
                    Ok(out)
                }
            }
        }
        Cmd::Restore { input, vault } => {
            let v = load_vault(&vault)?;
            Ok(restore(&read_input(input.as_deref())?, &v))
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(out) => {
            if io::stdout().write_all(out.as_bytes()).is_err() {
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("piiguard: {e}");
            ExitCode::FAILURE
        }
    }
}
