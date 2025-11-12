[![Build](https://github.com/christianhelle/curlgenerator/actions/workflows/build.yml/badge.svg)](https://github.com/christianhelle/curlgenerator/actions/workflows/build.yml)
[![Crates.io](https://img.shields.io/crates/v/curlgenerator.svg)](https://crates.io/crates/curlgenerator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# cURL Request Generator

Generate cURL requests from OpenAPI specifications v2.0, v3.0, and v3.1

> **⚠️ IMPORTANT: This repository has been rewritten in Rust**
>
> The original .NET version is now maintained in the separate [`curlgenerator-dotnet`](https://github.com/christianhelle/curlgenerator-dotnet) repository. This repository now contains the Rust implementation, which provides:
>
> - **4-5x faster** execution time
> - **5-6x lower** memory usage
> - **smaller** binary size
> - **No runtime dependencies** (single native binary)
> - **40x faster** cold start time
> - **Cross-platform** support without installing .NET
>
> **Reason for Migration:** Users of cURL Request Generator are predominantly on Linux and macOS. Getting .NET to run reliably on non-Windows systems has become increasingly troublesome, while a single natively compiled Rust binary provides a dramatically simpler and faster experience.
>
> For the legacy .NET version, see the [`curlgenerator-dotnet`](https://github.com/christianhelle/curlgenerator-dotnet) repository or install from [NuGet](https://www.nuget.org/packages/curlgenerator).

## Installation

### From Source

```bash
cargo install --path .
```

### From Crates.io (when published)

```bash
cargo install curlgenerator
```

## Usage

```bash
Generate cURL requests from OpenAPI specifications v2.0, v3.0, and v3.1

Usage: curlgenerator [OPTIONS] [URL or input file]

Arguments:
  [URL or input file]  URL or file path to OpenAPI Specification file

Options:
  -o, --output <OUTPUT>
          Output directory [default: ./]
      --bash
          Generate Bash scripts
      --no-logging
          Do not log errors or collect telemetry
      --skip-validation
          Skip validation of OpenAPI Specification file
      --authorization-header <AUTHORIZATION_HEADER>
          Authorization header to use for all requests
      --content-type <CONTENT_TYPE>
          Default Content-Type header to use for all requests [default: application/json]
      --base-url <BASE_URL>
          Default Base URL to use for all requests
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples

```bash
# Basic usage
curlgenerator ./openapi.json

# Specify output directory
curlgenerator ./openapi.json --output ./

# Generate Bash scripts
curlgenerator ./openapi.json --bash

# Load from URL
curlgenerator https://petstore.swagger.io/v2/swagger.json

# With custom base URL
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --base-url https://petstore3.swagger.io

# With authorization header
curlgenerator ./openapi.json --authorization-header "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# OpenAPI 3.1 with webhooks
curlgenerator ./openapi-v3.1.json
```

## Example

Running the following:

```bash
curlgenerator https://petstore.swagger.io/v2/swagger.json
```

Outputs the following:

```bash
╔════════════════════════════════════════════════════════════╗
║  🔧 cURL Request Generator v0.1.0                          ║
╚════════════════════════════════════════════════════════════╝

📋 Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  📁 OpenAPI Source: https://petstore.swagger.io/v2/swagger.json
  📂 Output Folder: ./
  🌐 Content Type: application/json

📊 OpenAPI Statistics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  📝 Path Items: 14
  ⚙️  Operations: 20
  📝 Parameters: 14
  📝 Schemas: 67

✅ Generation Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  📄 Files Generated: 20
  ⏱️  Duration: 2308ms
  📁 Output Location: /current/working/directory

🎉 Done!
```

### Generated Files

The tool will generate PowerShell or Bash scripts for each operation in your OpenAPI spec.

Example PowerShell script (`PostAddPet.ps1`):

```powershell
<#
  Request: POST /pet
  Summary: Add a new pet to the store
  Description: Add a new pet to the store
#>

curl -X POST https://petstore3.swagger.io/api/v3/pet `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '{
  "id": 0,
  "name": "string",
  "category": {
    "id": 0,
    "name": "string"
  },
  "photoUrls": [
    "string"
  ],
  "tags": [
    {
      "id": 0,
      "name": "string"
    }
  ],
  "status": "available"
}'
```

Example script with parameters (`GetPetById.ps1`):

```powershell
<#
  Request: GET /pet/{petId}
  Summary: Find pet by ID
  Description: Returns a single pet
#>
param(
   <# ID of pet to return #>
   [Parameter(Mandatory=$True)]
   [String] $petId
)

curl -X GET https://petstore3.swagger.io/api/v3/pet/$petId `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json'
```

## Features

- ✅ OpenAPI v2.0, v3.0, and v3.1 support
- ✅ Generate PowerShell scripts (default)
- ✅ Generate Bash scripts
- ✅ Load OpenAPI specs from file or URL
- ✅ Custom authorization headers
- ✅ Custom base URLs
- ✅ Path parameters
- ✅ Query parameters
- ✅ Request body generation with sample JSON
- ✅ Colored terminal output

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Migration from .NET Version

If you're migrating from the .NET version, see the **[Migration Guide](MIGRATION.md)** for detailed instructions.

**Quick comparison:**

- Installation: `dotnet tool install` → `cargo install` or download binary
- Performance: 4-5x faster execution
- Binary size: 100MB → 10MB
- Dependencies: Requires .NET Runtime → No dependencies
- Azure Entra ID: Built-in → Manual (via Azure CLI)

See **[COMPARISON.md](COMPARISON.md)** for a detailed feature comparison.

The archived .NET version is available in [`dotnet-archived/`](./dotnet-archived/) and on [NuGet](https://www.nuget.org/packages/curlgenerator).

## License

MIT License - see [LICENSE](LICENSE) file for details

## Author

**Christian Helle**

- Blog: [christianhelle.com](https://christianhelle.com)
- GitHub: [@christianhelle](https://github.com/christianhelle)

If you find this useful and feel generous, you can [buy me a coffee ☕](https://www.buymeacoffee.com/christianhelle)
