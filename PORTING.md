# Porting notes: .NET to Rust

The Rust CLI under `src/rust` replaces the .NET CLI under `src/dotnet`, which is kept as the
legacy implementation. This document records how faithful the port is and where it deliberately
differs.

## How parity was verified

Both CLIs were run over all 41 specifications in `test/OpenAPI`, in both PowerShell and Bash
output modes, producing 506 files each. The two implementations generate the **same 506 file
names**, and **442 of the 506 files are byte identical**. Every one of the remaining 64 files
differs only in one of the three intentional ways listed below.

The OpenAPI statistics table matches exactly for all 38 specifications the .NET tool is able to
validate.

To reproduce the comparison:

```bash
cargo build --release --workspace
dotnet build -c Release src/dotnet/CurlGenerator/CurlGenerator.csproj
```

then generate with both binaries into separate directories and diff them.

## Intentional differences

### 1. Request bodies with numeric examples

The .NET generator calls `JsonNode.GetValue<double>()` on schema examples. That throws for a
JSON integer, and the surrounding `catch` turns the entire request body into `-d '{}'`. Any
schema anywhere in the tree with a numeric `example` triggers it, which is why Petstore v3.0's
`Pet`, `Order`, and `User` bodies are all empty in the .NET output.

The Rust port emits the real sample body. This accounts for 50 of the 64 differing files.

### 2. Swagger 2.0 documents without a `host`

Microsoft.OpenApi falls back to the reader's `BaseUrl` when a Swagger 2.0 document declares no
`host`, so the .NET tool emits the local file path as the server URL:

```
curl -X GET file://C:/projects/christianhelle/curlgenerator/test/OpenAPI/v2.0/
```

The Rust port treats a missing `host` as "no server URL" and emits the path on its own. The
existing `--base-url` option, and the authority of the specification URL for remote documents,
both still apply. This accounts for 8 of the 64 differing files.

### 3. `date-time` sample values

`DateTime.Now.ToString("yyyy-MM-ddTHH:mm:ssZ")` uses the current culture's time separator, so on
a machine with a Danish locale the .NET tool emits `2026-09-03T00.12.59Z`. The Rust port always
uses `:`. This accounts for the last 6 of the 64 differing files.

## Other behavioural changes

- **`generator.log` is gone.** The .NET generator appended the settings, the entire serialized
  OpenAPI document, and every generated file to `generator.log` in the working directory on every
  run. It dominated the runtime and was never surfaced to users.
- **Self-referencing schemas no longer crash.** The .NET tool stack-overflows on a schema that
  references itself. The Rust port stops at the cycle and renders an empty object for it.
- **Trailing parameter commas.** The .NET `param(...)` block trims the last comma using a fixed
  offset that assumes CRLF, so on Linux it removes two characters of the parameter name instead.
  The Rust port always removes the comma, matching the .NET output on Windows and the committed
  files under `test/Generated`.
- **Exit codes.** The .NET tool returns the exception `HResult`, which truncates to arbitrary
  values. The Rust CLI returns `0` on success and `1` on failure.
- **The `--output-type onefile` example is gone** from the help screen. `curlgenerator` has never
  had an `--output-type` option; the example was copied from a sibling tool.
- **Validation is a subset.** Microsoft.OpenApi ships a large validation rule set. The Rust port
  implements the rules that the shipped test corpus actually exercises: unique path signatures and
  at-least-one-response per operation. Everything else is accepted; `--skip-validation` still
  bypasses the check entirely.
- **The document is parsed once.** The .NET command parses the specification twice, once to
  validate and once to generate.

## Structure

| .NET | Rust |
| --- | --- |
| `CurlGenerator.Core/StringExtensions.cs` | `core/src/string_extensions.rs` |
| `CurlGenerator.Core/OperationNameGenerator.cs` | `core/src/operation_name.rs` |
| `CurlGenerator.Core/OpenApiDocumentFactory.cs` | `core/src/openapi/{source,loader}.rs` |
| `CurlGenerator.Core/ScriptFileGenerator.cs` | `core/src/generator/{pipeline,powershell,bash,sample}.rs` |
| `CurlGenerator.Core/GeneratorSettings.cs` | `core/src/model.rs` |
| `CurlGenerator/Validation/OpenApiStats.cs` | `core/src/openapi/inspect.rs` |
| `CurlGenerator/Validation/OpenApiValidator.cs` | `cli/src/validation.rs` |
| `CurlGenerator/SupportInformation.cs` | `core/src/support_information.rs` |
| `CurlGenerator/Settings.cs`, `Program.cs` | `cli/src/{args,help,main}.rs` |
| `CurlGenerator/GenerateCommand.cs` | `cli/src/{run,ui}.rs` |
| `CurlGenerator/Analytics.cs` | `cli/src/telemetry.rs` |
| `CurlGenerator/PrivacyHelper.cs` | `cli/src/privacy.rs` |
| `CurlGenerator.Core/AzureEntraID.cs` | `cli/src/auth.rs` |

The Spectre.Console panels and tables are reproduced by `cli/src/ui/layout.rs`, which draws the
same box characters and colors without a console framework.
