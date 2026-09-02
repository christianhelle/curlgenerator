# curlgenerator

Generate cURL requests from OpenAPI specifications v2.0 and v3.0.

```bash
cargo install curlgenerator
```

```bash
curlgenerator ./openapi.json
curlgenerator ./openapi.json --bash
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json
```

The tool writes one script per operation: a PowerShell `.ps1` file by default, or a Bash `.sh`
file with `--bash`. Run `curlgenerator --help` for the full list of options.

This is the Rust port of the .NET tool of the same name; the legacy implementation still lives
under `src/dotnet` in the repository.
