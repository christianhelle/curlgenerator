# cURL Request Generator

cURL Request Generator is a .NET 8 CLI tool that generates cURL requests from OpenAPI specifications (v2.0, v3.0, and v3.1). It's distributed as a .NET tool via NuGet and supports generating both PowerShell (.ps1) and Bash (.sh) scripts.

**Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.**

## Working Effectively

### Bootstrap, Build, and Test
```bash
# Build the solution (takes ~75 seconds, NEVER CANCEL)
dotnet build -c Release CurlGenerator.sln
```
**NEVER CANCEL:** Build takes up to 75 seconds. Always set timeout to 150+ seconds.

```bash
# Run unit tests (takes ~13 seconds, NEVER CANCEL)
dotnet test CurlGenerator.sln -c Release
```
**NEVER CANCEL:** Tests take up to 15 seconds. Always set timeout to 60+ seconds.
**Expected:** 99 tests pass, 1 may fail due to external network dependency (this is normal).

```bash
# Publish the CLI tool for local testing
dotnet publish src/CurlGenerator/CurlGenerator.csproj -p:TreatWarningsAsErrors=true -p:PublishReadyToRun=true -o test/bin
```

### Running the CLI Tool

#### Using dotnet run (for development):
```bash
# Show help
dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- --help

# Generate PowerShell scripts from OpenAPI spec
dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- test/OpenAPI/v3.0/petstore.yaml --output ./output --no-logging

# Generate Bash scripts from OpenAPI spec
dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- test/OpenAPI/v3.0/petstore.yaml --output ./output --bash --no-logging
```

#### Using published binary:
```bash
# After publishing, use the binary directly
./test/bin/curlgenerator test/OpenAPI/v3.0/petstore.yaml --output ./output --no-logging
```

#### Common CLI options:
- `--output <dir>`: Output directory for generated files
- `--bash`: Generate Bash scripts instead of PowerShell
- `--no-logging`: Disable telemetry and error logging
- `--skip-validation`: Skip OpenAPI spec validation (required for v3.1 specs)
- `--authorization-header <header>`: Add auth header to all requests
- `--base-url <url>`: Override base URL from spec

## Validation

### Manual Validation Requirements
**Always test CLI functionality after making changes:**

1. **Test PowerShell generation:**
   ```bash
   mkdir -p /tmp/test_ps && dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- test/OpenAPI/v3.0/petstore.yaml --output /tmp/test_ps --no-logging
   ```
   Verify: 19 .ps1 files generated, each contains valid cURL commands with PowerShell syntax.

2. **Test Bash generation:**
   ```bash
   mkdir -p /tmp/test_bash && dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- test/OpenAPI/v3.0/petstore.yaml --output /tmp/test_bash --bash --no-logging
   ```
   Verify: 19 .sh files generated, each contains valid cURL commands with Bash syntax.

3. **Test different OpenAPI versions:**
   ```bash
   # Test v2.0
   dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- test/OpenAPI/v2.0/petstore.yaml --output /tmp/test_v2 --no-logging
   
   # Test v3.1 (requires --skip-validation)
   dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- test/OpenAPI/v3.1/webhook-example.yaml --output /tmp/test_v31 --skip-validation --no-logging
   ```

### Build Validation
Always run these commands before submitting changes:
```bash
# Clean build (NEVER CANCEL - takes 75+ seconds)
dotnet clean && dotnet build -c Release CurlGenerator.sln

# Full test suite (NEVER CANCEL - takes 15+ seconds)
dotnet test CurlGenerator.sln -c Release

# Publish test
dotnet publish src/CurlGenerator/CurlGenerator.csproj -c Release -o /tmp/publish_test
```

## Project Structure

### Key Projects
- **CurlGenerator** (`src/CurlGenerator/`): Main CLI application (.NET 8)
- **CurlGenerator.Core** (`src/CurlGenerator.Core/`): Core library (.NET Standard 2.0)
- **CurlGenerator.Tests** (`src/CurlGenerator.Tests/`): Unit tests (.NET 8)

### Important Files
- `CurlGenerator.sln`: Main solution file
- `src/Directory.Build.props`: Common MSBuild properties
- `.github/workflows/build.yml`: Main CI build pipeline
- `test/OpenAPI/`: Sample OpenAPI specifications for testing
- `test/smoke-tests.ps1`: Comprehensive smoke test suite
- `qodana.yaml`: Code quality analysis configuration

### Generated Artifacts
- NuGet packages: `src/CurlGenerator/bin/Release/CurlGenerator.*.nupkg`
- Published binary: `curlgenerator` (executable)
- Generated scripts: `.ps1` (PowerShell) or `.sh` (Bash) files

## Common Tasks

### Testing Different OpenAPI Formats
The repository includes test specifications in multiple versions:
```bash
# Available test files
ls test/OpenAPI/v2.0/    # OpenAPI 2.0 specs
ls test/OpenAPI/v3.0/    # OpenAPI 3.0 specs  
ls test/OpenAPI/v3.1/    # OpenAPI 3.1 specs
```

Use these for testing your changes:
- `petstore.yaml` - Standard petstore API
- `petstore-expanded.yaml` - Extended petstore with more operations
- `api-with-examples.yaml` - API with example data

### Creating NuGet Package
```bash
# Pack for distribution
dotnet pack src/CurlGenerator/CurlGenerator.csproj -c Release
```

### Running Comprehensive Tests
The smoke test suite validates multiple scenarios:
```bash
cd test && pwsh smoke-tests.ps1
```
**Note:** Smoke tests take 2-5 minutes to complete. Do not cancel.

### Code Quality
The project uses:
- **TreatWarningsAsErrors=true**: All warnings treated as errors
- **Qodana**: JetBrains code analysis (runs in CI)
- **Nullable reference types**: Enabled across all projects

## Troubleshooting

### Common Issues
1. **Build fails with warnings**: Project treats warnings as errors. Fix all warnings.
2. **Test fails with network error**: 1 test requires external connectivity and may fail in restricted environments (this is expected).
3. **Generated files empty**: Check OpenAPI spec validity, use `--skip-validation` for v3.1 specs.
4. **PowerShell scripts vs Bash**: Default generates .ps1 files, use `--bash` flag for .sh files.

### Debugging Generation
Add verbose output by removing `--no-logging` flag:
```bash
dotnet run --project src/CurlGenerator/CurlGenerator.csproj -- test/OpenAPI/v3.0/petstore.yaml --output ./debug_output
```

## Time Expectations
- **Clean build**: 75 seconds (NEVER CANCEL - timeout: 150+ seconds)
- **Incremental build**: 5-15 seconds
- **Test suite**: 13 seconds (NEVER CANCEL - timeout: 60+ seconds)
- **CLI generation**: 1-3 seconds per OpenAPI spec
- **Smoke tests**: 2-5 minutes (NEVER CANCEL)

**CRITICAL: Always set appropriate timeouts and never cancel long-running operations.**