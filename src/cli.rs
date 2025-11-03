use clap::Parser;

const LONG_VERSION: &str = concat!(
    env!("VERSION"),
    "\ngit tag: ",
    env!("GIT_TAG"),
    "\ngit commit: ",
    env!("GIT_COMMIT"),
    "\nbuild date: ",
    env!("BUILD_DATE")
);

#[derive(Parser, Debug)]
#[command(name = "curlgenerator")]
#[command(version = env!("VERSION"))]
#[command(long_version = LONG_VERSION)]
#[command(about = "Generate cURL requests from OpenAPI specifications v2.0 and v3.0", long_about = None)]
pub struct Cli {
    /// URL or file path to OpenAPI Specification file
    #[arg(value_name = "URL or input file")]
    pub openapi_path: Option<String>,

    /// Output directory
    #[arg(short = 'o', long = "output", default_value = "./")]
    pub output: String,

    /// Generate Bash scripts
    #[arg(long = "bash")]
    pub bash: bool,

    /// Don't log errors or collect telemetry
    #[arg(long = "no-logging")]
    pub no_logging: bool,

    /// Skip validation of OpenAPI Specification file
    #[arg(long = "skip-validation")]
    pub skip_validation: bool,

    /// Authorization header to use for all requests
    #[arg(long = "authorization-header")]
    pub authorization_header: Option<String>,

    /// Default Content-Type header to use for all requests
    #[arg(long = "content-type", default_value = "application/json")]
    pub content_type: String,

    /// Default Base URL to use for all requests
    #[arg(long = "base-url")]
    pub base_url: Option<String>,
}
