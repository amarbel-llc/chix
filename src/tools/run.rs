use crate::nix_runner::run_nix_command_in_dir;
use crate::tools::{NixDevelopRunParams, NixRunParams};
use crate::validators::{validate_args, validate_flake_ref, validate_installable, validate_path};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NixRunResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

pub async fn nix_run(params: NixRunParams) -> Result<NixRunResult, String> {
    let installable = params
        .installable
        .unwrap_or_else(|| ".#default".to_string());
    validate_installable(&installable).map_err(|e| e.to_string())?;

    let flake_dir = params.flake_dir.as_deref();
    if let Some(dir) = flake_dir {
        validate_path(dir).map_err(|e| e.to_string())?;
    }

    if let Some(ref args) = params.args {
        validate_args(args).map_err(|e| e.to_string())?;
    }

    let mut args: Vec<&str> = vec!["run", &installable];

    let user_args: Vec<String> = params.args.unwrap_or_default();
    if !user_args.is_empty() {
        args.push("--");
        for arg in &user_args {
            args.push(arg);
        }
    }

    let result = run_nix_command_in_dir(&args, flake_dir)
        .await
        .map_err(|e| e.to_string())?;

    Ok(NixRunResult {
        success: result.success,
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
    })
}

pub async fn nix_develop_run(params: NixDevelopRunParams) -> Result<NixRunResult, String> {
    let flake_ref = params.flake_ref.unwrap_or_else(|| ".".to_string());
    validate_flake_ref(&flake_ref).map_err(|e| e.to_string())?;

    let flake_dir = params.flake_dir.as_deref();
    if let Some(dir) = flake_dir {
        validate_path(dir).map_err(|e| e.to_string())?;
    }

    if let Some(ref args) = params.args {
        validate_args(args).map_err(|e| e.to_string())?;
    }

    let mut args: Vec<&str> = vec!["develop", &flake_ref, "-c", &params.command];

    let user_args: Vec<String> = params.args.unwrap_or_default();
    for arg in &user_args {
        args.push(arg);
    }

    let result = run_nix_command_in_dir(&args, flake_dir)
        .await
        .map_err(|e| e.to_string())?;

    Ok(NixRunResult {
        success: result.success,
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
    })
}
