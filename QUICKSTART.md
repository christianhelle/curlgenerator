# Quick Start Guide

## Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/christianhelle/curlgenerator-rs.git
cd curlgenerator-rs

# Build release binary
cargo build --release

# The binary will be at target/release/curlgenerator
./target/release/curlgenerator --help
```

### Option 2: Install with Cargo (when published)

```bash
cargo install curlgenerator
curlgenerator --help
```

## Basic Usage

### Generate from URL

```bash
# Generate PowerShell scripts from Petstore API
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json

# Generate Bash scripts
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --bash

# Specify output directory
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --output ./scripts
```

### Generate from File

```bash
# Generate from local OpenAPI spec
curlgenerator ./openapi.json

# Generate with custom base URL
curlgenerator ./openapi.json --base-url https://api.example.com

# Generate with authorization
curlgenerator ./openapi.json --authorization-header "Bearer YOUR_TOKEN"
```

## Examples

### Example 1: Basic Generation

```bash
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json
```

Output:
```
╔════════════════════════════════════════════════════════════╗
║  🔧 cURL Request Generator v0.1.0                          ║
╚════════════════════════════════════════════════════════════╝

📋 Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  📁 OpenAPI Source: https://petstore3.swagger.io/api/v3/openapi.json
  📂 Output Folder: ./
  🌐 Content Type: application/json

📊 OpenAPI Statistics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  📝 Path Items: 14
  ⚙️  Operations: 19
  📝 Parameters: 8
  📝 Schemas: 12

✅ Generation Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  📄 Files Generated: 19
  ⏱️  Duration: 535ms
  📁 Output Location: /current/directory

🎉 Done!
```

### Example 2: Generate Bash Scripts

```bash
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json \
  --bash \
  --output ./bash-scripts
```

### Example 3: With Authentication

```bash
curlgenerator https://api.example.com/openapi.json \
  --base-url https://api.example.com \
  --authorization-header "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  --output ./authenticated-scripts
```

### Example 4: Custom Content Type

```bash
curlgenerator ./openapi.json \
  --content-type "application/xml" \
  --output ./xml-scripts
```

## Generated Script Examples

### PowerShell Script with Parameters

```powershell
<#
  Request: GET /pet/{petId}
  Summary: Find pet by ID
  Description: Returns a single pet
#>
param(
   <# ID of pet to return #>
   [Parameter(Mandatory=$True)]
   [String] $petid
)

curl -X GET https://petstore3.swagger.io/api/v3/pet/$petid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
```

### Bash Script with Parameters

```bash
#
# Request: GET /pet/{petId}
# Summary: Find pet by ID
# Description: Returns a single pet
#

# ID of pet to return
petid=""

curl -X GET "https://petstore3.swagger.io/api/v3/pet/${petid}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
```

### Using Generated Scripts

#### PowerShell
```powershell
# Execute with parameter
.\GETGetPetById.ps1 -petid "12345"
```

#### Bash
```bash
# Set parameter and execute
petid="12345"
./GETGetPetById.sh
```

## Common Use Cases

### 1. API Testing

```bash
# Generate test scripts for your API
curlgenerator https://api.example.com/swagger.json --output ./api-tests

# Run a specific test
./api-tests/POSTCreateUser.ps1
```

### 2. CI/CD Integration

```bash
# In your CI/CD pipeline
curlgenerator $OPENAPI_SPEC_URL --output ./scripts --bash
chmod +x ./scripts/*.sh
./scripts/GETHealthCheck.sh
```

### 3. Documentation

```bash
# Generate scripts as API documentation
curlgenerator ./openapi.json --output ./docs/api-examples
```

### 4. Quick API Exploration

```bash
# Quickly generate scripts to explore an API
curlgenerator https://api.example.com/openapi.json
ls *.ps1  # or *.sh for bash
```

## Command Reference

```
Generate cURL requests from OpenAPI specifications v2.0 and v3.0

Usage: curlgenerator [OPTIONS] [URL or input file]

Arguments:
  [URL or input file]  URL or file path to OpenAPI Specification file

Options:
  -o, --output <OUTPUT>
          Output directory [default: ./]
      --bash
          Generate Bash scripts
      --no-logging
          Don't log errors or collect telemetry
      --skip-validation
          Skip validation of OpenAPI Specification file
      --authorization-header <AUTHORIZATION_HEADER>
          Authorization header to use for all requests
          Example: "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
      --content-type <CONTENT_TYPE>
          Default Content-Type header to use for all requests [default: application/json]
      --base-url <BASE_URL>
          Default Base URL to use for all requests
          Example: "https://api.example.com"
  -h, --help
          Print help
  -V, --version
          Print version
```

## Troubleshooting

### Issue: "Failed to parse OpenAPI document"

This usually means the OpenAPI spec is malformed or uses an unsupported version.

**Solution:**
```bash
# Try skipping validation
curlgenerator ./openapi.json --skip-validation
```

### Issue: Binary not found after building

**Solution:**
```bash
# The binary is in target/release/
./target/release/curlgenerator --help

# Or add to PATH
export PATH="$PATH:$(pwd)/target/release"
curlgenerator --help
```

### Issue: Permission denied (Linux/macOS)

**Solution:**
```bash
chmod +x target/release/curlgenerator
./target/release/curlgenerator --help
```

## Tips & Tricks

### 1. Generate for Multiple APIs

```bash
for api in api1.json api2.json api3.json; do
  curlgenerator $api --output ./scripts/$(basename $api .json)
done
```

### 2. Test Generation Speed

```bash
time curlgenerator https://petstore3.swagger.io/api/v3/openapi.json
```

### 3. Compare Output Sizes

```bash
# PowerShell
curlgenerator ./openapi.json --output ./ps
du -sh ./ps

# Bash
curlgenerator ./openapi.json --output ./sh --bash
du -sh ./sh
```

### 4. Create Alias

```bash
# Add to ~/.bashrc or ~/.zshrc
alias curlgen='curlgenerator'

# Use it
curlgen https://api.example.com/openapi.json
```

## Performance Tips

1. **Use --skip-validation** for faster generation if you trust your OpenAPI spec
2. **Specify --output** to avoid cluttering current directory
3. **Use local files** instead of URLs when possible for faster loading
4. **Build with --release** for maximum performance

## Getting Help

- **Documentation**: See [README.md](README.md)
- **Issues**: https://github.com/christianhelle/curlgenerator-rs/issues
- **Comparison**: See [COMPARISON.md](COMPARISON.md) for .NET vs Rust
- **Original .NET version**: https://github.com/christianhelle/curlgenerator

## Next Steps

1. Read the full [README.md](README.md)
2. Check out [COMPARISON.md](COMPARISON.md) to see how it compares to the .NET version
3. Explore the [examples](#examples) above
4. Report issues or contribute on GitHub

---

Happy generating! 🚀
