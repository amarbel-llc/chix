use crate::nix_runner::run_nix_command;
use crate::tools::{NixHashFileParams, NixHashPathParams};
use crate::validators::validate_path;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NixHashResult {
    pub success: bool,
    pub hash: String,
    pub stderr: String,
}

pub async fn nix_hash_path(params: NixHashPathParams) -> Result<NixHashResult, String> {
    validate_path(&params.path).map_err(|e| e.to_string())?;

    let hash_type = params.hash_type.unwrap_or_else(|| "sha256".to_string());
    let valid_types = ["sha256", "sha512", "sha1", "md5"];
    if !valid_types.contains(&hash_type.as_str()) {
        return Err(format!(
            "Invalid hash type: {}. Must be one of: {:?}",
            hash_type, valid_types
        ));
    }

    let mut args = vec!["hash", "path"];

    if params.base32.unwrap_or(false) {
        args.push("--base32");
    } else if params.sri.unwrap_or(true) {
        args.push("--sri");
    }

    args.push("--type");
    args.push(&hash_type);
    args.push(&params.path);

    let result = run_nix_command(&args).await.map_err(|e| e.to_string())?;

    Ok(NixHashResult {
        success: result.success,
        hash: result.stdout.trim().to_string(),
        stderr: result.stderr,
    })
}

pub async fn nix_hash_file(params: NixHashFileParams) -> Result<NixHashResult, String> {
    validate_path(&params.path).map_err(|e| e.to_string())?;

    let hash_type = params.hash_type.unwrap_or_else(|| "sha256".to_string());
    let valid_types = ["sha256", "sha512", "sha1", "md5"];
    if !valid_types.contains(&hash_type.as_str()) {
        return Err(format!(
            "Invalid hash type: {}. Must be one of: {:?}",
            hash_type, valid_types
        ));
    }

    let mut args = vec!["hash", "file"];

    if params.base32.unwrap_or(false) {
        args.push("--base32");
    } else if params.sri.unwrap_or(true) {
        args.push("--sri");
    }

    args.push("--type");
    args.push(&hash_type);
    args.push(&params.path);

    let result = run_nix_command(&args).await.map_err(|e| e.to_string())?;

    Ok(NixHashResult {
        success: result.success,
        hash: result.stdout.trim().to_string(),
        stderr: result.stderr,
    })
}
