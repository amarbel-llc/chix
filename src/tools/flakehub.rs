use crate::nix_runner::run_fh_command;
use crate::output::PaginationInfo;
use crate::tools::{
    FhAddParams, FhFetchParams, FhListFlakesParams, FhListReleasesParams, FhListVersionsParams,
    FhLoginParams, FhResolveParams, FhSearchParams,
};
use crate::validators::{validate_no_shell_metacharacters, validate_path};
use serde::Serialize;

fn paginate_json_array(
    value: serde_json::Value,
    offset: Option<usize>,
    limit: Option<usize>,
) -> (serde_json::Value, Option<PaginationInfo>) {
    if let serde_json::Value::Array(arr) = value {
        let total = arr.len();
        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(total);

        let paginated: Vec<serde_json::Value> = arr.into_iter().skip(off).take(lim).collect();
        let kept_count = paginated.len();
        let has_more = off + kept_count < total;

        let pagination = if offset.is_some() || limit.is_some() {
            Some(PaginationInfo {
                offset: off,
                limit: lim,
                total,
                has_more,
            })
        } else {
            None
        };

        (serde_json::Value::Array(paginated), pagination)
    } else {
        (value, None)
    }
}

#[derive(Debug, Serialize)]
pub struct FhSearchResult {
    pub success: bool,
    pub results: serde_json::Value,
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,
}

#[derive(Debug, Serialize)]
pub struct FhAddResult {
    pub success: bool,
    pub output: String,
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub struct FhListResult {
    pub success: bool,
    pub results: serde_json::Value,
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,
}

#[derive(Debug, Serialize)]
pub struct FhResolveResult {
    pub success: bool,
    pub result: serde_json::Value,
    pub stderr: String,
}

pub async fn fh_search(params: FhSearchParams) -> Result<FhSearchResult, String> {
    validate_no_shell_metacharacters(&params.query).map_err(|e| e.to_string())?;

    let mut args = vec!["search", "--json"];
    args.push(&params.query);

    let max_results_str;
    if let Some(max_results) = params.max_results {
        max_results_str = max_results.to_string();
        args.push("--max-results");
        args.push(&max_results_str);
    }

    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    let parsed = if result.success {
        serde_json::from_str(&result.stdout).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };

    let (results, pagination) = paginate_json_array(parsed, params.offset, params.limit);

    Ok(FhSearchResult {
        success: result.success,
        results,
        stderr: result.stderr,
        pagination,
    })
}

pub async fn fh_add(params: FhAddParams) -> Result<FhAddResult, String> {
    validate_no_shell_metacharacters(&params.input_ref).map_err(|e| e.to_string())?;

    let mut args = vec!["add"];

    let flake_path;
    if let Some(ref path) = params.flake_path {
        validate_no_shell_metacharacters(path).map_err(|e| e.to_string())?;
        flake_path = path.clone();
        args.push("--flake-path");
        args.push(&flake_path);
    }

    let input_name;
    if let Some(ref name) = params.input_name {
        validate_no_shell_metacharacters(name).map_err(|e| e.to_string())?;
        input_name = name.clone();
        args.push("--input-name");
        args.push(&input_name);
    }

    args.push(&params.input_ref);

    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    Ok(FhAddResult {
        success: result.success,
        output: result.stdout,
        stderr: result.stderr,
    })
}

pub async fn fh_list_flakes(params: FhListFlakesParams) -> Result<FhListResult, String> {
    let mut args = vec!["list", "flakes", "--json"];

    let limit_str;
    if let Some(limit) = params.limit {
        limit_str = limit.to_string();
        args.push("--limit");
        args.push(&limit_str);
    }

    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    let parsed = if result.success {
        serde_json::from_str(&result.stdout).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };

    let (results, pagination) = paginate_json_array(parsed, params.offset, params.limit);

    Ok(FhListResult {
        success: result.success,
        results,
        stderr: result.stderr,
        pagination,
    })
}

pub async fn fh_list_releases(params: FhListReleasesParams) -> Result<FhListResult, String> {
    validate_no_shell_metacharacters(&params.flake).map_err(|e| e.to_string())?;

    let mut args = vec!["list", "releases", "--json"];
    args.push(&params.flake);

    let limit_str;
    if let Some(limit) = params.limit {
        limit_str = limit.to_string();
        args.push("--limit");
        args.push(&limit_str);
    }

    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    let parsed = if result.success {
        serde_json::from_str(&result.stdout).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };

    let (results, pagination) = paginate_json_array(parsed, params.offset, params.limit);

    Ok(FhListResult {
        success: result.success,
        results,
        stderr: result.stderr,
        pagination,
    })
}

pub async fn fh_list_versions(params: FhListVersionsParams) -> Result<FhListResult, String> {
    validate_no_shell_metacharacters(&params.flake).map_err(|e| e.to_string())?;
    validate_no_shell_metacharacters(&params.version_constraint).map_err(|e| e.to_string())?;

    let mut args = vec!["list", "versions", "--json"];
    args.push(&params.flake);
    args.push(&params.version_constraint);

    let limit_str;
    if let Some(limit) = params.limit {
        limit_str = limit.to_string();
        args.push("--limit");
        args.push(&limit_str);
    }

    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    let parsed = if result.success {
        serde_json::from_str(&result.stdout).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };

    let (results, pagination) = paginate_json_array(parsed, params.offset, params.limit);

    Ok(FhListResult {
        success: result.success,
        results,
        stderr: result.stderr,
        pagination,
    })
}

pub async fn fh_resolve(params: FhResolveParams) -> Result<FhResolveResult, String> {
    validate_no_shell_metacharacters(&params.flake_ref).map_err(|e| e.to_string())?;

    let args = vec!["resolve", "--json", &params.flake_ref];

    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    let resolve_result = if result.success {
        serde_json::from_str(&result.stdout).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };

    Ok(FhResolveResult {
        success: result.success,
        result: resolve_result,
        stderr: result.stderr,
    })
}

#[derive(Debug, Serialize)]
pub struct FhStatusResult {
    pub success: bool,
    pub logged_in: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub struct FhFetchResult {
    pub success: bool,
    pub store_path: Option<String>,
    pub target_link: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub struct FhLoginResult {
    pub success: bool,
    pub message: String,
    pub stderr: String,
}

pub async fn fh_status() -> Result<FhStatusResult, String> {
    let args = vec!["status"];
    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    // Check if output indicates logged in status
    let logged_in = result.success
        && (result.stdout.contains("Logged in") || result.stdout.contains("authenticated"));

    Ok(FhStatusResult {
        success: result.success,
        logged_in,
        stdout: result.stdout,
        stderr: result.stderr,
    })
}

pub async fn fh_fetch(params: FhFetchParams) -> Result<FhFetchResult, String> {
    validate_no_shell_metacharacters(&params.flake_ref).map_err(|e| e.to_string())?;
    validate_path(&params.target_link).map_err(|e| e.to_string())?;

    let args = vec!["fetch", &params.flake_ref, &params.target_link];
    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    // Extract store path from output if present
    let store_path = result
        .stdout
        .lines()
        .find(|line| line.starts_with("/nix/store/"))
        .map(|s| s.to_string());

    Ok(FhFetchResult {
        success: result.success,
        store_path,
        target_link: params.target_link,
        stdout: result.stdout,
        stderr: result.stderr,
    })
}

pub async fn fh_login(params: FhLoginParams) -> Result<FhLoginResult, String> {
    let mut args = vec!["login"];

    let token_file;
    if let Some(ref path) = params.token_file {
        validate_path(path).map_err(|e| e.to_string())?;
        token_file = path.clone();
        args.push("--token-file");
        args.push(&token_file);
    }

    let result = run_fh_command(&args).await.map_err(|e| e.to_string())?;

    let message = if result.success {
        "Login initiated. Follow browser prompts to complete authentication.".to_string()
    } else {
        format!("Login failed: {}", result.stderr)
    };

    Ok(FhLoginResult {
        success: result.success,
        message,
        stderr: result.stderr,
    })
}
