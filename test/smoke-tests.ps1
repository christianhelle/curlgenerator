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

$binary = if ($IsWindows -or $env:OS -eq "Windows_NT") { "../target/release/curlgenerator.exe" } else { "../target/release/curlgenerator" }

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

  Write-Host "curlgenerator ./openapi.$format --output ./Generated/$output --no-logging $args"
  $process = Start-Process $binary `
    -Args "./openapi.$format --output ./Generated/$output --no-logging $args" `
    -NoNewWindow `
    -PassThru

  $process | Wait-Process
  if ($process.ExitCode -ne 0)
  {
    throw "curlgenerator failed"
  }
}

function RunTests
{
  param (
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
    
  Get-ChildItem '*.ps1' -Recurse -Path ./Generated -ErrorAction SilentlyContinue | ForEach-Object { Remove-Item -Path $_.FullName }
  Get-ChildItem '*.sh' -Recurse -Path ./Generated -ErrorAction SilentlyContinue | ForEach-Object { Remove-Item -Path $_.FullName }
  Write-Host "cargo build --release"
  Start-Process "cargo" -Args "build --release" -NoNewWindow -PassThru -WorkingDirectory ".." | Wait-Process
    
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

Measure-Command { RunTests -Parallel $Parallel }
Write-Host "`r`n"
