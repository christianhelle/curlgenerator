[![Build](https://github.com/christianhelle/curlgenerator/actions/workflows/build.yml/badge.svg)](https://github.com/christianhelle/curlgenerator/actions/workflows/build.yml)
[![Smoke Tests](https://github.com/christianhelle/curlgenerator/actions/workflows/smoke-tests.yml/badge.svg)](https://github.com/christianhelle/curlgenerator/actions/workflows/smoke-tests.yml)
[![NuGet](https://img.shields.io/nuget/v/curlgenerator?color=blue)](https://www.nuget.org/packages/curlgenerator)
[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=christianhelle_curlgenerator&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=christianhelle_curlgenerator)
[![codecov](https://codecov.io/gh/christianhelle/curlgenerator/graph/badge.svg?token=242YT1N6T2)](https://codecov.io/gh/christianhelle/curlgenerator)
[![Qodana](https://github.com/christianhelle/curlgenerator/actions/workflows/qodana.yml/badge.svg)](https://github.com/christianhelle/curlgenerator/actions/workflows/qodana.yml)

# cURL Request Generator

Generate cURL requests from OpenAPI specifications v2.0 and v3.0

The CLI was rewritten in Rust for performance. Over the 41 specifications in `test/OpenAPI`, in
both output modes, the Rust CLI runs the whole corpus roughly **70x faster** than the legacy .NET
tool. The .NET implementation is kept under `src/dotnet`; the Rust implementation lives under
`src/rust`. See [PORTING.md](PORTING.md) for how output parity was verified and where the two
deliberately differ.

## Installation

### Cargo

```bash
cargo install curlgenerator
```

### .NET tool (legacy)

The original implementation is still distributed as a .NET Tool on NuGet.org

```bash
dotnet tool install --global curlgenerator
```

## Usage

```pwsh
USAGE:
    curlgenerator [URL or input file] [OPTIONS]

EXAMPLES:
    curlgenerator ./openapi.json
    curlgenerator ./openapi.json --output ./
    curlgenerator ./openapi.json --bash
    curlgenerator https://petstore.swagger.io/v2/swagger.json
    curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --base-url https://petstore3.swagger.io
    curlgenerator ./openapi.json --authorization-header Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c
    curlgenerator ./openapi.json --azure-scope [Some Application ID URI]/.default

ARGUMENTS:
    [URL or input file]    URL or file path to OpenAPI Specification file

OPTIONS:
                                           DEFAULT
    -h, --help                                                 Prints help information
    -v, --version                                              Prints version information
    -o, --output <OUTPUT>                  ./                  Output directory
        --bash                                                 Generate Bash scripts
        --no-logging                                           Don't log errors or collect telemetry
        --skip-validation                                      Skip validation of OpenAPI
                                                               Specification file
        --authorization-header <HEADER>                        Authorization header to use for all
                                                               requests
        --content-type <CONTENT-TYPE>      application/json    Default Content-Type header to use
                                                               for all requests
        --base-url <BASE-URL>                                  Default Base URL to use for all
                                                               requests. Use this if the OpenAPI
                                                               spec doesn't explicitly specify a
                                                               server URL
        --azure-scope <SCOPE>                                  Azure Entra ID Scope to use for
                                                               retrieving Access Token for
                                                               Authorization header
        --azure-tenant-id <TENANT-ID>                          Azure Entra ID Tenant ID to use for
                                                               retrieving Access Token for
                                                               Authorization header
```

Running the following:

```sh
curlgenerator https://petstore.swagger.io/v2/swagger.json
```

Outputs the following:

```sh
┌──────────────────────────────────────────────────────────────────────────────┐
│ 🔧 cURL Request Generator v0.4.1                                             │
└──────────────────────────────────────────────────────────────────────────────┘

⚠️  Unavailable when logging is disabled

┌─📋 Configuration────────────────────────────────────────────────────┐
│ ┌───────────────────┬─────────────────────────────────────────────┐ │
│ │ Setting           │ Value                                       │ │
│ ├───────────────────┼─────────────────────────────────────────────┤ │
│ │ 📁 OpenAPI Source │ https://petstore.swagger.io/v2/swagger.json │ │
│ │ 📂 Output Folder  │ ./                                          │ │
│ │ 🌐 Content Type   │ application/json                            │ │
│ └───────────────────┴─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘


┌─📊 OpenAPI Statistics─────────┐
│ ┌───────────────────┬───────┐ │
│ │ Component         │ Count │ │
│ ├───────────────────┼───────┤ │
│ │ 📝 Path Items     │    14 │ │
│ │ ⚙️  Operations     │    20 │ │
│ │ 📝 Parameters     │    14 │ │
│ │ 📦 Request Bodies │     9 │ │
│ │ 📋 Responses      │    20 │ │
│ │ 🔗 Links          │     0 │ │
│ │ 📞 Callbacks      │     0 │ │
│ │ 📝 Schemas        │    67 │ │
│ └───────────────────┴───────┘ │
└───────────────────────────────┘


┌─✅ Generation Complete───────────────────────────────────────────────────────┐
│ ┌────────────────────┬─────────────────────────────────────────────────────┐ │
│ │ Metric             │ Value                                               │ │
│ ├────────────────────┼─────────────────────────────────────────────────────┤ │
│ │ 📄 Files Generated │ 20                                                  │ │
│ │ ⏱️  Duration        │ 606ms                                               │ │
│ │ 📁 Output Location │ ./                                                  │ │
│ └────────────────────┴─────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘


📁 Generated Files:
  📝 PostUploadFile.ps1 (336 bytes)
  📝 PostAddPet.ps1 (438 bytes)
  📝 PutUpdatePet.ps1 (432 bytes)
  📝 GetFindPetsByStatus.ps1 (444 bytes)
  📝 GetFindPetsByTags.ps1 (424 bytes)
  📝 GetPetById.ps1 (345 bytes)
  📝 PostUpdatePetWithForm.ps1 (352 bytes)
  📝 DeletePet.ps1 (310 bytes)
  📝 GetInventory.ps1 (289 bytes)
  📝 PostPlaceOrder.ps1 (355 bytes)
  📝 GetOrderById.ps1 (477 bytes)
  📝 DeleteOrder.ps1 (510 bytes)
  📝 PostCreateUsersWithListInput.ps1 (465 bytes)
  📝 GetUserByName.ps1 (371 bytes)
  📝 PutUpdateUser.ps1 (582 bytes)
  📝 DeleteUser.ps1 (403 bytes)
  📝 GetLoginUser.ps1 (456 bytes)
  📝 GetLogoutUser.ps1 (227 bytes)
  📝 PostCreateUsersWithArrayInput.ps1 (467 bytes)
  📝 PostCreateUser.ps1 (437 bytes)

🎉 Done!
```

Which will produce the following files:

```sh
-rw-r--r-- 1 christian 197121  593 Dec 10 10:44 DeleteOrder.ps1        
-rw-r--r-- 1 christian 197121  231 Dec 10 10:44 DeletePet.ps1
-rw-r--r-- 1 christian 197121  358 Dec 10 10:44 DeleteUser.ps1
-rw-r--r-- 1 christian 197121  432 Dec 10 10:44 GetFindPetsByStatus.ps1
-rw-r--r-- 1 christian 197121  504 Dec 10 10:44 GetFindPetsByTags.ps1  
-rw-r--r-- 1 christian 197121  371 Dec 10 10:44 GetInventory.ps1       
-rw-r--r-- 1 christian 197121  247 Dec 10 10:44 GetLoginUser.ps1       
-rw-r--r-- 1 christian 197121  291 Dec 10 10:44 GetLogoutUser.ps1      
-rw-r--r-- 1 christian 197121  540 Dec 10 10:44 GetOrderById.ps1
-rw-r--r-- 1 christian 197121  275 Dec 10 10:44 GetPetById.ps1
-rw-r--r-- 1 christian 197121  245 Dec 10 10:44 GetUserByName.ps1
-rw-r--r-- 1 christian 197121  513 Dec 10 10:44 PostAddPet.ps1
-rw-r--r-- 1 christian 197121  521 Dec 10 10:44 PostCreateUser.ps1
-rw-r--r-- 1 christian 197121  610 Dec 10 10:44 PostCreateUsersWithListInput.ps1
-rw-r--r-- 1 christian 197121  464 Dec 10 10:44 PostPlaceOrder.ps1
-rw-r--r-- 1 christian 197121  299 Dec 10 10:44 PostUpdatePetWithForm.ps1
-rw-r--r-- 1 christian 197121  274 Dec 10 10:44 PostUploadFile.ps1
-rw-r--r-- 1 christian 197121  513 Dec 10 10:44 PutUpdatePet.ps1
-rw-r--r-- 1 christian 197121  541 Dec 10 10:44 PutUpdateUser.ps1
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
  "id": 0,
  "name": "name",
  "category": {
    "id": 0,
    "name": "name"
  },
  "photoUrls": [
    ""
  ],
  "tags": [
    {
      "id": 0,
      "name": "name"
    }
  ],
  "status": "available"
}'
```

The generated script will contain mandatory parameters for operations where the path contains parameters, it will look something like this:

```powershell
<#
  Request: GET /pet/{petId}
  Summary: Find pet by ID
  Description: Returns a single pet
#>
param(
   <# ID of pet to return #>
   [Parameter(Mandatory=$True)]
   [String] $petId
)

curl -X GET https://petstore3.swagger.io/api/v3/pet/$petId `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json'
```

Here's an advanced example of generating cURL requests for a REST API hosted on Microsoft Azure that uses the Microsoft Entra ID service as an STS. For this example, I use PowerShell and Azure CLI to retrieve an access token for the user I'm currently logged in with.

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

You can also use the `--azure-scope` and `--azure-tenant-id` arguments, which retrieve an access token for the specified `scope` using the Azure CLI credentials and the local developer tooling credentials.

```powershell
curlgenerator `
  https://api.example.com/swagger/v1/swagger.json `
  --azure-scope [Some Application ID URI]/.default `
  --base-url https://api.example.com `
  --output ./HttpFiles 
```

## Building from source

```bash
make build   # cargo build --workspace, then the legacy .NET solution
make test    # cargo test --workspace, then dotnet test
make lint    # cargo fmt --check and cargo clippy -D warnings
```

The Rust workspace on its own:

```bash
cargo build --workspace
cargo test --workspace
```

#

For tips and tricks on software development, check out [my blog](https://christianhelle.com)

If you find this useful and feel a bit generous then feel free to [buy me a coffee ☕](https://www.buymeacoffee.com/christianhelle)
