# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-11-03

### 🚀 MAJOR RELEASE: Complete Rewrite in Rust

This is a **breaking change** that replaces the .NET implementation with a complete Rust rewrite.

#### Why Rust?

The primary motivation for this rewrite is **cross-platform usability**:
- Most cURL Request Generator users are on Linux and macOS
- Getting .NET to run reliably on non-Windows systems has become increasingly problematic
- A single natively compiled binary provides a dramatically simpler and faster experience
- No runtime dependencies or version conflicts

#### Changed

- **[BREAKING]** Repository now contains Rust implementation as primary codebase
- **[BREAKING]** .NET version moved to separate repository: https://github.com/christianhelle/curlgenerator-dotnet
- **[BREAKING]** Installation method changed from `dotnet tool install` to `cargo install`
- **[BREAKING]** Azure Entra ID integration (`--azure-scope`, `--azure-tenant-id`) removed
  - Workaround: Use Azure CLI to obtain token manually, then pass via `--authorization-header`

#### Added

- Complete Rust implementation with identical CLI interface
- Zero runtime dependencies (single native binary)
- [docs/MIGRATION.md](docs/MIGRATION.md) - Comprehensive migration guide
- [docs/COMPARISON.md](docs/COMPARISON.md) - Detailed comparison between implementations
- .NET version available at: https://github.com/christianhelle/curlgenerator-dotnet

#### Performance Improvements

- ⚡ **4-5x faster** execution time
- 🚀 **40x faster** cold start (5ms vs 200ms)
- 💾 **5-6x lower** memory usage (15MB vs 80MB)
- 📦 **10x smaller** binary size (10MB vs 100MB)

#### Feature Parity

Maintained from .NET version:
- ✅ OpenAPI v3.0 support
- ✅ PowerShell script generation
- ✅ Bash script generation
- ✅ Load from file or URL
- ✅ Path and query parameters
- ✅ Request body generation
- ✅ Custom authorization headers
- ✅ Custom base URLs
- ✅ Colored terminal output

Limited/Changed:
- ⚠️ OpenAPI v2.0: Basic support (library limitations)
- ⚠️ Azure Entra ID: Manual token acquisition required
- ❌ Telemetry: Intentionally omitted for privacy

#### Migration

See [MIGRATION.md](MIGRATION.md) for detailed migration instructions.

For users requiring .NET-specific features:
- .NET version remains available on NuGet: `dotnet tool install --global curlgenerator`
- Archived source code available in `dotnet-archived/` folder
- .NET version enters maintenance mode (critical fixes only)

---

## .NET Version History (Archived)

The following releases were for the .NET implementation (now archived):

## [0.1.0] - 2024-11-01

### Added
- Initial Rust port of cURL Request Generator
- Generate PowerShell scripts from OpenAPI v2.0 and v3.0 specs
- Generate Bash scripts from OpenAPI v2.0 and v3.0 specs
- Load OpenAPI specs from file or URL
- Support for custom authorization headers
- Support for custom base URLs
- Support for path parameters
- Support for query parameters
- Request body generation with sample JSON
- Colored terminal output with statistics
- Command-line interface with clap
- Cross-platform support (Windows, Linux, macOS)

### Features Compared to .NET Version
- ✅ OpenAPI v2.0 and v3.0 support
- ✅ PowerShell script generation
- ✅ Bash script generation
- ✅ Load from file or URL
- ✅ Custom authorization headers
- ✅ Custom base URLs
- ✅ Path parameters
- ✅ Query parameters
- ✅ Request body generation
- ⏳ Azure Entra ID authentication (planned)
- ⏳ Full OpenAPI validation (planned)

[0.1.0]: https://github.com/christianhelle/curlgenerator-rs/releases/tag/v0.1.0
