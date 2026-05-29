$binary = if ($IsWindows -or $env:OS -eq "Windows_NT") { "../target/release/curlgenerator.exe" } else { "../target/release/curlgenerator" }

cargo build --release --manifest-path ../Cargo.toml

az account get-access-token `
| ConvertFrom-Json `
| %{
    & $binary `
    https://petstore3.swagger.io/api/v3/openapi.json `
        --authorization-header ("Bearer " + $_.accessToken) `
        --base-url https://petstore3.swagger.io `
        --output ./HttpFiles
}