<#
.SYNOPSIS
    Compares the runtime of the Rust CLI and the legacy .NET CLI over the OpenAPI corpus.

.DESCRIPTION
    Runs both generators over every specification in test/OpenAPI, in both PowerShell and Bash
    output modes, and reports the total elapsed time for each. Writes a Markdown table to the
    GitHub step summary when running in Actions.

.PARAMETER RustCommand
    The Rust CLI to invoke.

.PARAMETER DotnetCommand
    The .NET CLI to invoke.

.PARAMETER Runs
    How many times to repeat the whole corpus per implementation.
#>
[CmdletBinding()]
param(
    [string] $RustCommand = "../target/release/curlgenerator",
    [string] $DotnetCommand = "../src/dotnet/CurlGenerator/bin/Release/net8.0/curlgenerator",
    [int] $Runs = 3
)

$ErrorActionPreference = "Stop"

$repository = Split-Path -Parent $PSScriptRoot
$specifications = Get-ChildItem -Path (Join-Path $repository "test/OpenAPI") -Recurse -Include *.json, *.yaml |
    Sort-Object FullName

if ($specifications.Count -eq 0) {
    throw "No specifications found under test/OpenAPI"
}

function Measure-Generator {
    param(
        [string] $Name,
        [string] $Command
    )

    $output = Join-Path ([System.IO.Path]::GetTempPath()) "curlgenerator-benchmark-$Name"
    $total = [TimeSpan]::Zero

    for ($run = 1; $run -le $Runs; $run++) {
        if (Test-Path $output) {
            Remove-Item $output -Recurse -Force
        }
        New-Item -ItemType Directory -Path $output -Force | Out-Null

        $elapsed = Measure-Command {
            foreach ($specification in $specifications) {
                foreach ($mode in @(@(), @("--bash"))) {
                    & $Command $specification.FullName --output $output --no-logging --skip-validation @mode *> $null
                }
            }
        }

        $total += $elapsed
        Write-Output "$Name run $run of ${Runs}: $([math]::Round($elapsed.TotalSeconds, 2))s"
    }

    if (Test-Path $output) {
        Remove-Item $output -Recurse -Force
    }

    return [TimeSpan]::FromTicks($total.Ticks / $Runs)
}

Write-Output "Benchmarking $($specifications.Count) specifications, $Runs run(s) each"

$rust = Measure-Generator -Name "rust" -Command $RustCommand
$dotnet = Measure-Generator -Name "dotnet" -Command $DotnetCommand

$rustSeconds = [math]::Round($rust.TotalSeconds, 2)
$dotnetSeconds = [math]::Round($dotnet.TotalSeconds, 2)
$speedup = if ($rust.TotalSeconds -gt 0) {
    [math]::Round($dotnet.TotalSeconds / $rust.TotalSeconds, 1)
} else {
    0
}

$summary = @(
    "## Performance comparison",
    "",
    "$($specifications.Count) specifications x 2 output modes, mean of $Runs run(s).",
    "",
    "| Implementation | Total time | Relative |",
    "| --- | ---: | ---: |",
    "| Rust CLI | ${rustSeconds}s | 1.0x |",
    "| .NET CLI (legacy) | ${dotnetSeconds}s | ${speedup}x slower |"
) -join "`n"

Write-Output ""
Write-Output $summary

if ($env:GITHUB_STEP_SUMMARY) {
    $summary | Out-File -FilePath $env:GITHUB_STEP_SUMMARY -Append -Encoding utf8
}
