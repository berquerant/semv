use crate::proc::{
    filter_by_requirement, filter_semver, format_output, parse_versions, sort_lines,
};
use crate::ver::VersionInfo;
use rmcp::{
    ServiceExt,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, ErrorData},
    tool, tool_router,
    transport::io::stdio,
};
use schemars::JsonSchema;
use semver::VersionReq;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct SemvMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SemvParams {
    /// List of version strings to process.
    pub targets: Vec<String>,
    /// Sort results in ascending order.
    pub sort: Option<bool>,
    /// Sort results in descending order.
    pub reverse_sort: Option<bool>,
    /// Verbose JSON output for each version.
    pub verbose: Option<bool>,
    /// SemVer version requirement filter (e.g., '>=1.2.0, <2.0.0').
    pub requirement: Option<String>,
    /// Display non-semver strings only.
    pub nonsemver: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ParseSemverParams {
    /// The version string to parse.
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ParsedSemver {
    pub original: String,
    pub is_valid: bool,
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub pre: Option<String>,
    pub build: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MatchRequirementParams {
    /// The version string to check.
    pub version: String,
    /// The SemVer requirement expression.
    pub requirement: String,
}

#[tool_router(server_handler)]
impl SemvMcpServer {
    /// Filter, sort, and inspect SemVer strings with optional version requirements.
    #[tool(
        name = "semv",
        description = "Inspect, filter, and sort SemVer strings. Supports requirement matching, reverse sorting, and verbose JSON output."
    )]
    pub async fn semv(
        &self,
        Parameters(params): Parameters<SemvParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let filter_non_semver = params.nonsemver.unwrap_or(false);
        let mut source = filter_semver(
            parse_versions(Box::new(params.targets.into_iter())),
            filter_non_semver,
        );

        if let Some(req_str) = params.requirement {
            let req = VersionReq::parse(&req_str).map_err(|e| {
                ErrorData::invalid_params(
                    format!("Invalid version requirement '{}': {}", req_str, e),
                    None,
                )
            })?;
            source = filter_by_requirement(source, req);
        }

        if params.reverse_sort.unwrap_or(false) {
            source = sort_lines(source, true);
        } else if params.sort.unwrap_or(false) {
            source = sort_lines(source, false);
        }

        let output_iter = format_output(source, params.verbose.unwrap_or(false));
        let results: Vec<String> = output_iter.collect();
        let content_text = results.join("\n");
        Ok(CallToolResult::success(vec![ContentBlock::text(
            content_text,
        )]))
    }

    /// Parse a version string and return detailed SemVer components.
    #[tool(
        name = "parse_semver",
        description = "Parse a version string (with optional leading 'v') and return its major, minor, patch, pre-release, and build metadata."
    )]
    pub async fn parse_semver(
        &self,
        Parameters(params): Parameters<ParseSemverParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let info = VersionInfo::parse(&params.target);
        let parsed = if let Some(v) = info.version {
            ParsedSemver {
                original: info.original,
                is_valid: true,
                major: Some(v.major),
                minor: Some(v.minor),
                patch: Some(v.patch),
                pre: if v.pre.is_empty() {
                    None
                } else {
                    Some(v.pre.as_str().to_string())
                },
                build: if v.build.is_empty() {
                    None
                } else {
                    Some(v.build.as_str().to_string())
                },
            }
        } else {
            ParsedSemver {
                original: info.original,
                is_valid: false,
                major: None,
                minor: None,
                patch: None,
                pre: None,
                build: None,
            }
        };

        let json_str = serde_json::to_string(&parsed).map_err(|e| {
            ErrorData::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;
        Ok(CallToolResult::success(vec![ContentBlock::text(json_str)]))
    }

    /// Check if a version string matches a given SemVer requirement.
    #[tool(
        name = "match_requirement",
        description = "Test whether a version string satisfies a SemVer requirement (e.g. '>=1.0.0, <2.0.0')."
    )]
    pub async fn match_requirement(
        &self,
        Parameters(params): Parameters<MatchRequirementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = VersionReq::parse(&params.requirement).map_err(|e| {
            ErrorData::invalid_params(
                format!(
                    "Invalid version requirement '{}': {}",
                    params.requirement, e
                ),
                None,
            )
        })?;
        let info = VersionInfo::parse(&params.version);
        if let Some(v) = info.version {
            let matched = req.matches(&v);
            Ok(CallToolResult::success(vec![ContentBlock::text(
                matched.to_string(),
            )]))
        } else {
            Err(ErrorData::invalid_params(
                format!("'{}' is not a valid semver string", params.version),
                None,
            ))
        }
    }
}

pub async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = SemvMcpServer;
    let (stdin, stdout) = stdio();
    let running = service.serve((stdin, stdout)).await?;
    running.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_semv_filter_and_sort() {
        let server = SemvMcpServer;
        let params = SemvParams {
            targets: vec![
                "invalid".to_string(),
                "1.3.0".to_string(),
                "0.2.0".to_string(),
                "2.0.0".to_string(),
            ],
            sort: Some(true),
            reverse_sort: None,
            verbose: None,
            requirement: None,
            nonsemver: None,
        };
        let result = server.semv(Parameters(params)).await.unwrap();
        assert_eq!(result.content.len(), 1);
        if let ContentBlock::Text(t) = &result.content[0] {
            assert_eq!(t.text, "0.2.0\n1.3.0\n2.0.0");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_mcp_semv_requirement() {
        let server = SemvMcpServer;
        let params = SemvParams {
            targets: vec![
                "1.0.0".to_string(),
                "1.2.5".to_string(),
                "2.0.0".to_string(),
            ],
            sort: None,
            reverse_sort: None,
            verbose: None,
            requirement: Some(">=1.2.0, <2.0.0".to_string()),
            nonsemver: None,
        };
        let result = server.semv(Parameters(params)).await.unwrap();
        if let ContentBlock::Text(t) = &result.content[0] {
            assert_eq!(t.text, "1.2.5");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_mcp_parse_semver() {
        let server = SemvMcpServer;
        let params = ParseSemverParams {
            target: "v1.2.3-alpha.1+build.12".to_string(),
        };
        let result = server.parse_semver(Parameters(params)).await.unwrap();
        if let ContentBlock::Text(t) = &result.content[0] {
            let parsed: ParsedSemver = serde_json::from_str(&t.text).unwrap();
            assert!(parsed.is_valid);
            assert_eq!(parsed.original, "v1.2.3-alpha.1+build.12");
            assert_eq!(parsed.major, Some(1));
            assert_eq!(parsed.minor, Some(2));
            assert_eq!(parsed.patch, Some(3));
            assert_eq!(parsed.pre, Some("alpha.1".to_string()));
            assert_eq!(parsed.build, Some("build.12".to_string()));
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_mcp_match_requirement() {
        let server = SemvMcpServer;
        let params = MatchRequirementParams {
            version: "1.5.0".to_string(),
            requirement: ">=1.0.0, <2.0.0".to_string(),
        };
        let result = server.match_requirement(Parameters(params)).await.unwrap();
        if let ContentBlock::Text(t) = &result.content[0] {
            assert_eq!(t.text, "true");
        } else {
            panic!("Expected text content");
        }

        let params_no_match = MatchRequirementParams {
            version: "2.1.0".to_string(),
            requirement: ">=1.0.0, <2.0.0".to_string(),
        };
        let result_no_match = server
            .match_requirement(Parameters(params_no_match))
            .await
            .unwrap();
        if let ContentBlock::Text(t) = &result_no_match.content[0] {
            assert_eq!(t.text, "false");
        } else {
            panic!("Expected text content");
        }
    }
}
