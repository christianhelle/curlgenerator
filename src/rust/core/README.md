# curlgenerator-core

Core library behind [cURL Request Generator](https://github.com/christianhelle/curlgenerator).

It loads an OpenAPI specification from a file or URL, normalizes Swagger 2.0, OpenAPI 3.0, and
OpenAPI 3.1 documents into a single model, and renders a PowerShell or Bash script per operation
that invokes `curl`.

```rust
use curlgenerator_core::{GeneratorSettings, generator::generate};

let result = generate(&GeneratorSettings::new("./openapi.json"))?;

for file in result.files {
    println!("{}", file.filename);
}
# Ok::<(), curlgenerator_core::openapi::OpenApiLoadError>(())
```

This crate is the Rust port of the legacy `CurlGenerator.Core` .NET library, which still lives
under `src/dotnet` in the repository.
