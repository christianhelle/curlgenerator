use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "curlgenerator", about = "Generate cURL request scripts from OpenAPI specifications")]
pub struct Cli {
    /// URL or file path to OpenAPI Specification file
    #[arg(index = 1)]
    pub openapi_path: String,

    /// Output directory
    #[arg(short = 'o', long, default_value = "./")]
    pub output: String,

    /// Generate Bash scripts
    #[arg(long)]
    pub bash: bool,

    /// Don't log errors or collect telemetry
    #[arg(long)]
    pub no_logging: bool,

    /// Skip validation of OpenAPI Specification file
    #[arg(long)]
    pub skip_validation: bool,

    /// Authorization header to use for all requests
    #[arg(long)]
    pub authorization_header: Option<String>,

    /// Default Content-Type header to use for all requests
    #[arg(long, default_value = "application/json")]
    pub content_type: String,

    /// Default Base URL to use for all requests
    #[arg(long)]
    pub base_url: Option<String>,

    /// Azure Entra ID Scope for retrieving Access Token
    #[arg(long)]
    pub azure_scope: Option<String>,

    /// Azure Entra ID Tenant ID for retrieving Access Token
    #[arg(long)]
    pub azure_tenant_id: Option<String>,
}
