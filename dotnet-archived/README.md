# cURL Request Generator (.NET Version - ARCHIVED)

[![NuGet](https://img.shields.io/nuget/v/curlgenerator?color=blue)](https://www.nuget.org/packages/curlgenerator)

> **⚠️ ARCHIVED: This is the legacy .NET implementation**
> 
> This repository has been rewritten in Rust. The .NET version is now in **maintenance mode** and archived here for reference and legacy support.
>
> **For the current Rust version, see the [main repository](../README.md).**

## Why Was This Archived?

The .NET version has been superseded by a Rust implementation for several critical reasons:

1. **Cross-Platform Runtime Issues**: Most users are on Linux/macOS where .NET runtime management is problematic
2. **Dependency Complexity**: .NET requires either runtime installation or 100+ MB self-contained deployments
3. **Performance**: Rust version is 4-5x faster with 5-6x lower memory usage
4. **Binary Size**: Rust produces a 10MB native binary vs 100+ MB .NET self-contained executable
5. **Startup Time**: Rust cold start is 40x faster (5ms vs 200ms)

## Legacy Installation

This .NET version is still available on NuGet:

```bash
dotnet tool install --global curlgenerator
```

## Building from Source

```bash
# Build
dotnet build -c Release CurlGenerator.sln

# Run tests (NEVER CANCEL - takes 15+ seconds, timeout 60+)
dotnet test CurlGenerator.sln -c Release

# Run from source
dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- ./openapi.json --output ./scripts
```

## Usage

```bash
curlgenerator [URL or input file] [OPTIONS]

OPTIONS:
    -o, --output <OUTPUT>                      Output directory [default: ./]
        --bash                                 Generate Bash scripts
        --no-logging                           Don't log errors or collect telemetry
        --skip-validation                      Skip validation of OpenAPI Specification
        --authorization-header <HEADER>        Authorization header for all requests
        --content-type <CONTENT-TYPE>          Content-Type header [default: application/json]
        --base-url <BASE-URL>                  Base URL for all requests
        --azure-scope <SCOPE>                  Azure Entra ID Scope for access token
        --azure-tenant-id <TENANT-ID>          Azure Entra ID Tenant ID
```

## Features

### ✅ Available in .NET Version

- OpenAPI v2.0 (Swagger) and v3.0 support
- PowerShell and Bash script generation
- Load from file or URL
- Path and query parameters
- Request body generation
- Custom authorization headers
- Custom base URLs
- **Azure Entra ID integration** (unique to .NET version)
- Comprehensive OpenAPI validation
- Telemetry and analytics

### ⚠️ .NET-Specific Features

These features are **only available** in the .NET version:

1. **Azure Entra ID Authentication:**
   ```bash
   curlgenerator openapi.json \
     --azure-scope "api://myapp/.default" \
     --azure-tenant-id "your-tenant-id"
   ```

2. **Comprehensive Validation:**
   - Full OpenAPI spec validation using Microsoft.OpenApi
   - Detailed validation error messages

3. **Telemetry:**
   - Anonymous error reporting via Exceptionless
   - Can be disabled with `--no-logging`

## Migration to Rust Version

To migrate to the Rust version, see the [Migration Guide](../MIGRATION.md).

### Quick Comparison

| Aspect | .NET | Rust |
|--------|------|------|
| Installation | Requires .NET Runtime | Single native binary |
| Binary Size | 100+ MB (self-contained) | ~10 MB |
| Cold Start | ~200ms | ~5ms |
| Execution Speed | Baseline | 4-5x faster |
| Memory Usage | ~80MB | ~15MB |
| Azure Integration | ✅ Built-in | ⚠️ Manual (via Azure CLI) |
| OpenAPI v2.0 | ✅ Full support | ⚠️ Basic support |
| Dependencies | .NET Runtime | None |

## Project Structure

```
dotnet-archived/
├── src/
│   ├── CurlGenerator/           # CLI application
│   │   ├── Program.cs
│   │   ├── GenerateCommand.cs
│   │   ├── Settings.cs
│   │   └── Analytics.cs
│   └── CurlGenerator.Core/      # Core library
│       ├── ScriptFileGenerator.cs
│       ├── OpenApiDocumentFactory.cs
│       └── GeneratorSettings.cs
├── test/
│   ├── CurlGenerator.Tests/
│   └── OpenAPI/
├── CurlGenerator.sln
└── README.md (this file)
```

## Testing

```bash
# Unit tests (15 seconds, set timeout 60+)
dotnet test CurlGenerator.sln -c Release

# Smoke tests
cd test
pwsh smoke-tests.ps1
```

**IMPORTANT:** Never cancel test runs. They take 13-15 seconds and require a timeout of 60+ seconds.

## Building Self-Contained Executables

```bash
# Windows
dotnet publish src/CurlGenerator/CurlGenerator.csproj \
  -c Release -r win-x64 --self-contained \
  -o ./publish/win-x64

# Linux
dotnet publish src/CurlGenerator/CurlGenerator.csproj \
  -c Release -r linux-x64 --self-contained \
  -o ./publish/linux-x64

# macOS
dotnet publish src/CurlGenerator/CurlGenerator.csproj \
  -c Release -r osx-x64 --self-contained \
  -o ./publish/osx-x64
```

## When to Use the .NET Version

Consider using this archived .NET version if you:

- ✅ Need Azure Entra ID integration (`--azure-scope`)
- ✅ Require comprehensive OpenAPI validation
- ✅ Are already in a .NET-heavy environment
- ✅ Need the most mature and battle-tested implementation
- ✅ Work with complex OpenAPI v2.0 (Swagger) specifications

Otherwise, the Rust version is recommended for:

- ✅ Simpler deployment (single binary)
- ✅ Better performance
- ✅ Lower resource usage
- ✅ Linux/macOS without .NET runtime
- ✅ CI/CD pipelines
- ✅ Embedded systems

## Support Status

**Maintenance Mode**: This .NET version will receive:
- ✅ Critical bug fixes
- ✅ Security patches
- ❌ No new features
- ❌ No performance improvements

All new development happens in the Rust version.

## Documentation

- [Original README](../docs/dotnet-readme-original.md) - Historical reference
- [Migration Guide](../MIGRATION.md) - How to migrate to Rust
- [Comparison Document](../COMPARISON.md) - Detailed .NET vs Rust comparison

## License

MIT License - see [LICENSE](../LICENSE) file for details

## Author

**Christian Helle**

- Blog: [christianhelle.com](https://christianhelle.com)
- GitHub: [@christianhelle](https://github.com/christianhelle)

If you find this useful and feel generous, you can [buy me a coffee ☕](https://www.buymeacoffee.com/christianhelle)
