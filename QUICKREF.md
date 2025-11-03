# Quick Reference: .NET to Rust Migration

This is a quick reference for users migrating from the .NET version to the Rust version.

## Installation

| Task | .NET | Rust |
|------|------|------|
| **Install** | `dotnet tool install --global curlgenerator` | `cargo install curlgenerator` |
| **Update** | `dotnet tool update --global curlgenerator` | `cargo install curlgenerator --force` |
| **Uninstall** | `dotnet tool uninstall --global curlgenerator` | `cargo uninstall curlgenerator` |
| **Requirements** | .NET 8.0 SDK/Runtime | None (single binary) |

## Basic Commands

| Task | Command (Same for Both) |
|------|-------------------------|
| **Help** | `curlgenerator --help` |
| **Version** | `curlgenerator --version` |
| **Generate** | `curlgenerator openapi.json` |
| **Output dir** | `curlgenerator openapi.json --output ./scripts` |
| **Bash mode** | `curlgenerator openapi.json --bash` |
| **From URL** | `curlgenerator https://example.com/openapi.json` |
| **Custom auth** | `curlgenerator openapi.json --authorization-header "Bearer token"` |
| **Custom base URL** | `curlgenerator openapi.json --base-url https://api.example.com` |

## Changed Features

### Azure Entra ID Authentication

**Before (.NET):**
```bash
curlgenerator openapi.json \
  --azure-scope "api://myapp/.default" \
  --azure-tenant-id "tenant-id"
```

**After (Rust):**
```bash
# Get token first
TOKEN=$(az account get-access-token \
  --scope "api://myapp/.default" \
  --tenant "tenant-id" \
  --query accessToken -o tsv)

# Then use it
curlgenerator openapi.json \
  --authorization-header "Bearer $TOKEN"
```

### OpenAPI v2.0 (Swagger) Support

| Version | .NET | Rust |
|---------|------|------|
| **Support Level** | Full | Basic |
| **Recommendation** | Use for complex v2.0 specs | Use for simple v2.0 specs |
| **Workaround** | N/A | Convert to v3.0 or use .NET version |

## Performance Comparison

| Metric | .NET | Rust |
|--------|------|------|
| **Cold Start** | ~200ms | ~5ms |
| **Petstore v3 (19 ops)** | ~2,300ms | ~535ms |
| **Memory Usage** | ~80MB | ~15MB |
| **Binary Size** | ~100MB | ~10MB |

## CI/CD Snippets

### GitHub Actions

**Before (.NET):**
```yaml
- name: Setup .NET
  uses: actions/setup-dotnet@v3
  with:
    dotnet-version: 8.0.x
    
- name: Install curlgenerator
  run: dotnet tool install --global curlgenerator
  
- name: Generate cURL scripts
  run: curlgenerator ./openapi.json --output ./scripts
```

**After (Rust):**
```yaml
- name: Download curlgenerator
  run: |
    curl -L -o curlgenerator \
      https://github.com/christianhelle/curlgenerator/releases/latest/download/curlgenerator-${{ runner.os }}-${{ runner.arch }}
    chmod +x curlgenerator
    
- name: Generate cURL scripts
  run: ./curlgenerator ./openapi.json --output ./scripts
```

### GitLab CI

**Before (.NET):**
```yaml
generate-curl:
  image: mcr.microsoft.com/dotnet/sdk:8.0
  script:
    - dotnet tool install --global curlgenerator
    - export PATH="$PATH:$HOME/.dotnet/tools"
    - curlgenerator ./openapi.json --output ./scripts
```

**After (Rust):**
```yaml
generate-curl:
  image: alpine:latest
  script:
    - apk add --no-cache curl
    - curl -L -o curlgenerator https://github.com/.../curlgenerator-linux-x64
    - chmod +x curlgenerator
    - ./curlgenerator ./openapi.json --output ./scripts
```

### Docker

**Before (.NET):**
```dockerfile
FROM mcr.microsoft.com/dotnet/sdk:8.0
RUN dotnet tool install --global curlgenerator
ENV PATH="${PATH}:/root/.dotnet/tools"
```

**After (Rust):**
```dockerfile
FROM alpine:latest
COPY curlgenerator /usr/local/bin/
RUN chmod +x /usr/local/bin/curlgenerator
# No runtime dependencies!
```

## Generated Output

The generated PowerShell and Bash scripts are **functionally identical** between versions.

### PowerShell (.ps1)
```powershell
<#
  Request: GET /pet/{petId}
  Summary: Find pet by ID
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $petId
)

curl -X GET https://api.example.com/pet/$petId `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json'
```

### Bash (.sh)
```bash
#!/bin/bash
# Request: GET /pet/{petId}
# Summary: Find pet by ID

curl -X GET "https://api.example.com/pet/$1" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json"
```

## When to Use Which Version

### Use Rust Version (Recommended)
- ✅ Linux/macOS without .NET installed
- ✅ CI/CD pipelines
- ✅ Docker containers
- ✅ Performance-critical scenarios
- ✅ Minimal dependencies required
- ✅ Simple OpenAPI v3.0 specs

### Use .NET Version (Legacy)
- ✅ Azure Entra ID integration required
- ✅ Complex OpenAPI v2.0 (Swagger) specs
- ✅ Already in .NET ecosystem
- ✅ Need comprehensive validation

## Getting Help

| Resource | Link |
|----------|------|
| **Migration Guide** | [MIGRATION.md](MIGRATION.md) |
| **Feature Comparison** | [COMPARISON.md](COMPARISON.md) |
| **.NET Version Docs** | [dotnet-archived/README.md](dotnet-archived/README.md) |
| **Issues** | [GitHub Issues](https://github.com/christianhelle/curlgenerator/issues) |
| **Discussions** | [GitHub Discussions](https://github.com/christianhelle/curlgenerator/discussions) |

## Common Issues

### "command not found: curlgenerator"

**Solution:**
```bash
# Add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Need .NET version back

**Solution:**
```bash
# Install from NuGet
dotnet tool install --global curlgenerator

# Or build from archived source
cd dotnet-archived
dotnet build -c Release CurlGenerator.sln
```

### Azure authentication not working

**Solution:**
```bash
# Use Azure CLI to get token
TOKEN=$(az account get-access-token --scope <scope> --query accessToken -o tsv)
curlgenerator openapi.json --authorization-header "Bearer $TOKEN"
```

## Version Numbers

- **Rust version:** Starts at v2.0.0 (major rewrite)
- **.NET version:** Last version archived (check `dotnet-archived/`)
- **NuGet package:** Still available for .NET version

---

**TL;DR:** 
- Installation: `cargo install curlgenerator` (or download binary)
- Usage: Exactly the same CLI
- Azure auth: Now manual (use Azure CLI first)
- Performance: Much faster
- Binary: Much smaller
- Dependencies: None

**For detailed migration instructions, see [MIGRATION.md](MIGRATION.md)**
