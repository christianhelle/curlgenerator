[![Build](https://github.com/christianhelle/curlgenerator/actions/workflows/build.yml/badge.svg)](https://github.com/christianhelle/curlgenerator/actions/workflows/build.yml)
[![Smoke Tests](https://github.com/christianhelle/curlgenerator/actions/workflows/smoke-tests.yml/badge.svg)](https://github.com/christianhelle/curlgenerator/actions/workflows/smoke-tests.yml)

# cURL Request Generator

Generate cURL requests from OpenAPI specifications (v2.0, v3.0 and v3.1).

This is a command line tool written in [Rust](https://www.rust-lang.org/) that reads an
OpenAPI specification (from a local file or a URL, in JSON or YAML) and generates a
cURL request script for every operation. Scripts can be generated either as PowerShell
(`.ps1`) or Bash (`.sh`).

## Installation

### From crates.io

```bash
cargo install curlgenerator
```

### From source

```bash
git clone https://github.com/christianhelle/curlgenerator
cd curlgenerator
cargo install --path .
```

### Pre-built binaries

Pre-built binaries for Linux, macOS and Windows are attached to every
[GitHub release](https://github.com/christianhelle/curlgenerator/releases).

## Usage

```text
Generate cURL requests from OpenAPI specifications (v2.0, v3.0 and v3.1)

Usage: curlgenerator [OPTIONS] <URL or input file>

Arguments:
  <URL or input file>  URL or file path to OpenAPI Specification file

Options:
  -o, --output <OUTPUT>                Output directory [default: ./]
      --bash                           Generate Bash scripts
      --no-logging                     Don't log errors or collect telemetry (accepted for compatibility; no-op)
      --skip-validation                Skip validation of OpenAPI Specification file
      --authorization-header <HEADER>  Authorization header to use for all requests
      --content-type <CONTENT-TYPE>    Default Content-Type header to use for all requests [default: application/json]
      --base-url <BASE-URL>            Default Base URL to use for all requests
      --azure-scope <SCOPE>            Azure Entra ID Scope to use for retrieving an Access Token
      --azure-tenant-id <TENANT-ID>    Azure Entra ID Tenant ID to use for retrieving an Access Token
  -h, --help                           Print help
  -V, --version                        Print version

EXAMPLES:
  curlgenerator ./openapi.json
  curlgenerator ./openapi.json --output ./
  curlgenerator ./openapi.json --bash
  curlgenerator https://petstore.swagger.io/v2/swagger.json
  curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --base-url https://petstore3.swagger.io
  curlgenerator ./openapi.json --azure-scope [Some Application ID URI]/.default
```

Running the following:

```sh
curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --base-url https://petstore3.swagger.io
```

Outputs something like:

```sh
cURL Request Generator v1.0.0
Support key: mbmbqvd

OpenAPI statistics:
 - Path Items: 13
 - Operations: 19
 - Parameters: 11
 - Request Bodies: 9
 - Responses: 19
 - Links: 0
 - Callbacks: 0
 - Schemas: 8

Files: 19
Duration: 00:00:00.42
```

Which will produce one script file per operation, e.g.:

```sh
DeleteOrder.ps1
DeletePet.ps1
DeleteUser.ps1
GetFindPetsByStatus.ps1
GetFindPetsByTags.ps1
GetInventory.ps1
GetLoginUser.ps1
GetLogoutUser.ps1
GetOrderById.ps1
GetPetById.ps1
GetUserByName.ps1
PostAddPet.ps1
PostCreateUser.ps1
PostCreateUsersWithListInput.ps1
PostPlaceOrder.ps1
PostUpdatePetWithForm.ps1
PostUploadFile.ps1
PutUpdatePet.ps1
PutUpdateUser.ps1
```

In this example, the contents of `PostAddPet.ps1` looks like this:

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
  "id": 10,
  "name": "doggie",
  "category": {
    "id": 1,
    "name": "Dogs"
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
  "status": "string"
}'
```

The generated script will contain mandatory parameters for operations where the path
contains parameters, it will look something like this:

```powershell
<#
  Request: GET /pet/{petId}
  Summary: Find pet by ID
  Description: Returns a single pet
#>
param(
   <# ID of pet to return #>
   [Parameter(Mandatory=$True)]
   [String] $petid
)

curl -X GET https://petstore3.swagger.io/api/v3/pet/$petId?petId=$petid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json'
```

Use the `--bash` flag to generate Bash (`.sh`) scripts instead of PowerShell.

## Authorization

Use `--authorization-header` to add an `Authorization` header to every generated request:

```sh
curlgenerator ./openapi.json --authorization-header "Bearer <token>"
```

### Azure Entra ID

For APIs protected by Microsoft Entra ID you can let the tool retrieve an access token
for you using the [Azure CLI](https://learn.microsoft.com/cli/azure/). You must be logged
in (`az login`) and have the Azure CLI installed and on your `PATH`.

```powershell
curlgenerator `
  https://api.example.com/swagger/v1/swagger.json `
  --azure-scope [Some Application ID URI]/.default `
  --base-url https://api.example.com `
  --output ./HttpFiles
```

Internally this runs `az account get-access-token --scope <scope>` (optionally with
`--tenant <tenant-id>`) and uses the returned token as the `Authorization` header.

Alternatively, you can pipe a token in yourself:

```powershell
az account get-access-token --scope [Some Application ID URI]/.default `
| ConvertFrom-Json `
| %{
    curlgenerator `
        https://api.example.com/swagger/v1/swagger.json `
        --authorization-header ("Bearer " + $_.accessToken) `
        --base-url https://api.example.com `
        --output ./HttpFiles
}
```

## Building and testing

```sh
cargo build --release   # build the CLI
cargo test              # run unit and integration tests
cargo fmt --all         # format the code
cargo clippy --all-targets -- -D warnings   # lint
```

#

For tips and tricks on software development, check out [my blog](https://christianhelle.com)

If you find this useful and feel a bit generous then feel free to [buy me a coffee ☕](https://www.buymeacoffee.com/christianhelle)
