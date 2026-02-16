use crate::nix_runner::run_nix_command;
use crate::output::{limit_text_output, OutputLimits, TruncationInfo};
use crate::tools::NixLogParams;
use crate::validators::validate_installable;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NixLogResult {
    pub success: bool,
    pub log: String,
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_info: Option<TruncationInfo>,
}

pub async fn nix_log(params: NixLogParams) -> Result<NixLogResult, String> {
    validate_installable(&params.installable).map_err(|e| e.to_string())?;

    let args = vec!["log", &params.installable];

    let result = run_nix_command(&args).await.map_err(|e| e.to_string())?;

    // Apply output limits using the output module
    let limits = OutputLimits {
        head: params.head,
        tail: params.tail,
        max_bytes: params.max_bytes,
        max_lines: None,
    };

    let limited = limit_text_output(&result.stdout, &limits);

    Ok(NixLogResult {
        success: result.success,
        log: limited.content,
        stderr: result.stderr,
        truncated: if limited.truncated {
            Some(true)
        } else {
            None
        },
        truncation_info: limited.truncation_info,
    })
}
