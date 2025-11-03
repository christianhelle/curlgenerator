# Migration Guide: .NET to Rust

This document explains the migration from the .NET version to the Rust implementation of cURL Request Generator.

> **📦 .NET Version Repository**
>
> The .NET version is now maintained in a separate repository: https://github.com/christianhelle/curlgenerator-dotnet
>
> This allows both implementations to be actively developed and maintained independently.

## Why Rust?

The decision to rewrite cURL Request Generator in Rust was driven by real-world user feedback and pain points:

### Primary Reason: Cross-Platform Runtime Issues

**The Problem:**
- Most users of cURL Request Generator are on Linux and macOS
- Getting .NET to run on non-Windows systems has become increasingly troublesome
- .NET requires either:
  - Installing the .NET Runtime (large download, version conflicts)
  - Or distributing 100+ MB self-contained executables
- Version mismatches between system .NET and required version
- Complex installation procedures that vary by Linux distribution

**The Solution:**
- Single native binary with **zero runtime dependencies**
- Works immediately after download on any supported platform
- No installation required beyond downloading the binary
- Consistent experience across all operating systems

### Performance Benefits

While not the primary reason, the performance improvements are substantial:

| Metric | .NET | Rust | Improvement |
|--------|------|------|-------------|
| Cold Start | ~200ms | ~5ms | **40x faster** |
| Execution (Petstore v3) | ~2,300ms | ~535ms | **4-5x faster** |
| Memory Usage | ~80MB | ~15MB | **5-6x lower** |
| Binary Size | ~100MB | ~10MB | **10x smaller** |

## What Changed

### Installation

**Before (.NET):**
```bash
# Required .NET SDK to be installed first
dotnet tool install --global curlgenerator
```

**After (Rust):**
```bash
# From crates.io (when published)
cargo install curlgenerator

# Or download pre-built binary from GitHub Releases
# No installation needed - just download and run
```

### Command-Line Interface

The CLI remains **nearly identical** for a smooth transition:

```bash
# Both versions use the same command syntax
curlgenerator ./openapi.json --output ./ --bash
curlgenerator https://petstore.swagger.io/v2/swagger.json
curlgenerator ./openapi.json --authorization-header "Bearer token"
```

### Generated Output

The generated scripts are **functionally identical**:

**PowerShell Scripts (.ps1):**
```powershell
<#
  Request: GET /pet/{petId}
  Summary: Find pet by ID
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $petId
)

curl -X GET https://petstore3.swagger.io/api/v3/pet/$petId `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json'
```

**Bash Scripts (.sh):**
```bash
#!/bin/bash
# Request: GET /pet/{petId}
# Summary: Find pet by ID

curl -X GET "https://petstore3.swagger.io/api/v3/pet/$1" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json"
```

## Feature Parity

### ✅ Fully Supported Features

These features work identically in both versions:

- ✅ OpenAPI v3.0 support
- ✅ PowerShell script generation
- ✅ Bash script generation
- ✅ Load from file or URL
- ✅ Path parameters
- ✅ Query parameters
- ✅ Request body generation with sample JSON
- ✅ Custom authorization headers (`--authorization-header`)
- ✅ Custom base URLs (`--base-url`)
- ✅ Custom content types (`--content-type`)
- ✅ Skip validation (`--skip-validation`)
- ✅ Disable logging (`--no-logging`)
- ✅ Colored terminal output

### ⚠️ Limited Support

- **OpenAPI v2.0 (Swagger)**: Basic support available, but less comprehensive than .NET version due to library limitations

### ❌ Not Yet Implemented

Features from the .NET version not yet available in Rust:

- **Azure Entra ID Integration**: `--azure-scope` and `--azure-tenant-id` options
  - **Workaround**: Use Azure CLI to get token manually:
    ```bash
    TOKEN=$(az account get-access-token --scope <scope> --query accessToken -o tsv)
    curlgenerator openapi.json --authorization-header "Bearer $TOKEN"
    ```

- **Telemetry/Analytics**: Intentionally omitted in Rust version for privacy and simplicity

### 🔄 Behavioral Differences

1. **Error Messages**: Rust version provides more concise error messages
2. **Validation**: OpenAPI validation is simplified in Rust version
3. **Startup Time**: Rust version starts almost instantly (5ms vs 200ms)

## Migration Steps

### For Individual Users

1. **Uninstall .NET version** (optional):
   ```bash
   dotnet tool uninstall --global curlgenerator
   ```

2. **Install Rust version**:
   ```bash
   # From source
   git clone https://github.com/christianhelle/curlgenerator.git
   cd curlgenerator
   cargo install --path .
   
   # Or download pre-built binary from GitHub Releases
   ```

3. **Test your existing workflows**:
   ```bash
   # Your existing commands should work as-is
   curlgenerator ./your-openapi.json --output ./scripts
   ```

### For CI/CD Pipelines

**Before (.NET):**
```yaml
- name: Install curlgenerator
  run: dotnet tool install --global curlgenerator

- name: Generate cURL scripts
  run: curlgenerator ./openapi.json --output ./scripts
```

**After (Rust):**
```yaml
- name: Download curlgenerator
  run: |
    curl -L -o curlgenerator https://github.com/christianhelle/curlgenerator/releases/latest/download/curlgenerator-linux-x64
    chmod +x curlgenerator

- name: Generate cURL scripts
  run: ./curlgenerator ./openapi.json --output ./scripts
```

### For Docker Containers

**Before (.NET):**
```dockerfile
FROM mcr.microsoft.com/dotnet/sdk:8.0
RUN dotnet tool install --global curlgenerator
ENV PATH="${PATH}:/root/.dotnet/tools"
```

**After (Rust):**
```dockerfile
FROM debian:bookworm-slim
COPY curlgenerator /usr/local/bin/
# No runtime dependencies needed!
```

## Handling Azure Entra ID Authentication

If you used the `--azure-scope` and `--azure-tenant-id` options, you'll need to adapt:

**Before (.NET):**
```bash
curlgenerator openapi.json \
  --azure-scope "api://myapp/.default" \
  --azure-tenant-id "tenant-id"
```

**After (Rust + Azure CLI):**
```bash
# Get token using Azure CLI
TOKEN=$(az account get-access-token \
  --scope "api://myapp/.default" \
  --tenant "tenant-id" \
  --query accessToken -o tsv)

# Use token with curlgenerator
curlgenerator openapi.json \
  --authorization-header "Bearer $TOKEN"
```

**After (Rust + Script):**
```bash
#!/bin/bash
# generate-with-auth.sh

SCOPE="api://myapp/.default"
TENANT="tenant-id"

echo "🔑 Obtaining access token..."
TOKEN=$(az account get-access-token --scope "$SCOPE" --tenant "$TENANT" --query accessToken -o tsv)

if [ -z "$TOKEN" ]; then
  echo "❌ Failed to obtain access token"
  exit 1
fi

echo "✅ Token obtained"
echo "🔧 Generating cURL scripts..."

curlgenerator "$1" \
  --authorization-header "Bearer $TOKEN" \
  --output ./scripts

echo "✅ Done!"
```

## Testing Your Migration

After switching to the Rust version, verify everything works:

1. **Test basic generation:**
   ```bash
   curlgenerator https://petstore.swagger.io/v2/swagger.json --output ./test-output
   ```

2. **Verify output:**
   ```bash
   ls -la ./test-output
   # Should see .ps1 files (or .sh if using --bash)
   ```

3. **Test a generated script:**
   ```bash
   # PowerShell
   pwsh ./test-output/GetPetById.ps1 -petId 123
   
   # Bash
   bash ./test-output/GetPetById.sh 123
   ```

4. **Compare output with .NET version:**
   ```bash
   # The generated scripts should be functionally identical
   diff <(cat old-dotnet-output/GetPetById.ps1) <(cat new-rust-output/GetPetById.ps1)
   ```

## Troubleshooting

### "Command not found" after installation

**Problem:** Shell can't find the `curlgenerator` binary.

**Solution:**
```bash
# Add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Or use full path
~/.cargo/bin/curlgenerator --version
```

### OpenAPI v2.0 (Swagger) not working properly

**Problem:** Some Swagger 2.0 specs fail to parse or generate incomplete scripts.

**Solution:**
- Use `--skip-validation` flag
- Consider converting to OpenAPI v3.0 using online converters
- Or use the archived .NET version for complex Swagger 2.0 specs

### Missing Azure Entra ID integration

**Problem:** Need to authenticate with Azure Entra ID.

**Solution:** See the "Handling Azure Entra ID Authentication" section above for workarounds.

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/christianhelle/curlgenerator/issues)
- **Discussions**: [GitHub Discussions](https://github.com/christianhelle/curlgenerator/discussions)
- **Legacy .NET Version**: Available in [`dotnet-archived/`](./dotnet-archived/) folder

## Rollback Plan

If you need to revert to the .NET version:

1. **The .NET version is still available:**
   ```bash
   dotnet tool install --global curlgenerator
   ```

2. **Or use the archived code:**
   ```bash
   cd dotnet-archived
   dotnet build -c Release CurlGenerator.sln
   dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- --help
   ```

## Frequently Asked Questions

### Q: Will the .NET version continue to receive updates?

**A:** The .NET version is now in maintenance mode. Critical bug fixes may be applied, but new features will be developed for the Rust version. The code remains available in the `dotnet-archived/` folder and on NuGet.

### Q: Can I use both versions side by side?

**A:** Yes! Install them with different names or use the .NET version via `dotnet run` from the archived folder.

### Q: What about Windows users?

**A:** The Rust version works excellently on Windows and provides the same benefits. Pre-built Windows binaries are available in GitHub Releases.

### Q: Is the output exactly the same?

**A:** The generated scripts are functionally identical and will produce the same cURL commands. Minor formatting differences may exist.

### Q: How do I contribute to the Rust version?

**A:** Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines. The codebase is significantly simpler than the .NET version (~700 lines vs 2,000+ lines).

## Performance Benchmarks

Real-world comparison using Petstore OpenAPI v3 (19 operations):

```bash
# .NET version
$ time dotnet tool run curlgenerator https://petstore3.swagger.io/api/v3/openapi.json
Duration: 00:00:02.3089450

# Rust version
$ time curlgenerator https://petstore3.swagger.io/api/v3/openapi.json
Duration: 535ms
```

**Result:** Rust version is **4.3x faster** in this real-world scenario.

## Conclusion

The migration to Rust provides a superior user experience, especially for Linux and macOS users who no longer need to manage .NET runtime dependencies. The CLI and output formats remain compatible, making the transition smooth for existing users.

For questions or issues during migration, please [open an issue](https://github.com/christianhelle/curlgenerator/issues) on GitHub.
