param (
  [Parameter(Mandatory=$false)]
  [bool]
  $Parallel = $true
)

function ThrowOnNativeFailure
{
  if (-not $?)
  {
    throw "Native Failure"
  }
}

function Generate
{
  param (
    [Parameter(Mandatory=$true)]
    [string]
    $format,
        
    [Parameter(Mandatory=$true)]
    [string]
    $output,
                       
    [Parameter(Mandatory=$false)]
    [string]
    $args = ""
  )

  $binary = "./target/release/curlgenerator.exe"
  if (-not (Test-Path $binary))
  {
    $binary = "./target/debug/curlgenerator.exe"
  }

  Write-Host "CurlGenerator $binary ./openapi.$format --output ./Generated/$output --no-logging $args"
  $process = Start-Process $binary `
    -Args "./openapi.$format --output ./Generated/$output --no-logging $args" `
    -NoNewWindow `
    -PassThru

  $process | Wait-Process
  if ($process.ExitCode -ne 0)
  {
    throw "CurlGenerator failed"
  }

  Write-Host "CurlGenerator $binary ./openapi.$format --output ./Generated/$output --output-type OneFile --no-logging $args"
  $process = Start-Process $binary `
    -Args "./openapi.$format --output ./Generated/$output --output-type OneFile --no-logging $args" `
    -NoNewWindow `
    -PassThru

  $process | Wait-Process
  if ($process.ExitCode -ne 0)
  {
    throw "CurlGenerator failed"
  }
}

function RunTests
{
  param (
    [Parameter(Mandatory=$true)]
    [ValidateSet("dotnet-run", "CurlGenerator")]
    [string]
    $Method,
        
    [Parameter(Mandatory=$false)]
    [bool]
    $Parallel = $false
  )

  $filenames = @(
    "petstore",
    "petstore-expanded",
    "petstore-minimal",
    "petstore-simple",
    "petstore-with-external-docs",
    "api-with-examples",
    "callback-example",
    "link-example",
    "uber",
    "uspto",
    "hubspot-events",
    "hubspot-webhooks",
    "non-oauth-scopes",
    "webhook-example",
    "tictactoe"
  )
    
  Get-ChildItem '*.http' -Recurse | ForEach-Object { Remove-Item -Path $_.FullName }
  
  $cargoBuild = "cargo build --release"
  Write-Host $cargoBuild
  $process = Start-Process "cargo" -Args "build --release" -NoNewWindow -PassThru | Wait-Process
  if ($process.ExitCode -ne 0)
  {
    throw "cargo build failed"
  }
    
  "v2.0", "v3.0", "v3.1" | ForEach-Object {
    $version = $_
    "json", "yaml" | ForEach-Object {            
      $format = $_
      $filenames | ForEach-Object {
        $filename = "./OpenAPI/$version/$_.$format"
        $exists = Test-Path -Path $filename -PathType Leaf
        if ($exists -eq $true)
        {
          Write-Host "Testing $filename"
          Copy-Item $filename ./openapi.$format
          if ($version -eq "v3.1")
          {
            Generate -format $format -output $_/$version/$format -args "--skip-validation"
            Generate -format $format -output $_/$version/$format -args "--skip-validation --bash"
          } else
          {
            Generate -format $format -output $_/$version/$format
            Generate -format $format -output $_/$version/$format -args "--bash"
          }
        }
      }
    }
  }
}

Measure-Command { RunTests -Method "CurlGenerator" -Parallel $Parallel }
Write-Host "`r`n"
