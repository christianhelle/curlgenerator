#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Comprehensive smoke tests for curlgenerator

.DESCRIPTION
    Tests the curlgenerator against various OpenAPI specifications
    including v2.0, v3.0, and v3.1 formats in both JSON and YAML.

.PARAMETER Binary
    Path to the curlgenerator binary

.EXAMPLE
    .\smoke-tests.ps1
    .\smoke-tests.ps1 -Binary ".\target\release\curlgenerator.exe"
#>

param(
    [Parameter(Mandatory=$false)]
    [string]$Binary = ""
)

$ErrorActionPreference = "Stop"

# Detect script location and set up paths
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir

# Set default binary path based on detected project root
if ([string]::IsNullOrEmpty($Binary)) {
    if ($IsWindows -or $env:OS -match "Windows") {
        $Binary = Join-Path $projectRoot "target\release\curlgenerator.exe"
    } else {
        $Binary = Join-Path $projectRoot "target/release/curlgenerator"
    }
}

# Test specifications
$testSpecs = @{
    "v2.0" = @(
        "api-with-examples",
        "petstore",
        "petstore-expanded",
        "petstore-minimal",
        "petstore-simple",
        "petstore-with-external-docs",
        "uber"
    )
    "v3.0" = @(
        "api-with-examples",
        "callback-example",
        "hubspot-events",
        "hubspot-webhooks",
        "link-example",
        "petstore",
        "petstore-expanded",
        "uspto"
    )
    "v3.1" = @(
        "non-oauth-scopes",
        "webhook-example"
    )
}

$totalTests = 0
$passedTests = 0
$failedTests = 0
$skippedTests = 0
$startTime = Get-Date

Write-Host "`n╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║          cURL Generator - Smoke Test Suite                ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

# Verify binary exists
if (-not (Test-Path $Binary)) {
    Write-Host "✗ Binary not found: $Binary" -ForegroundColor Red
    Write-Host "  Building release binary..." -ForegroundColor Yellow
    Push-Location $projectRoot
    cargo build --release
    $buildResult = $LASTEXITCODE
    Pop-Location
    if ($buildResult -ne 0) {
        Write-Host "✗ Build failed" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Project Root: $projectRoot" -ForegroundColor Gray
Write-Host "Binary: $Binary" -ForegroundColor Gray
Write-Host "Test Date: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Gray
Write-Host ""

function Test-Generation {
    param(
        [string]$Binary,
        [string]$ScriptDir,
        [string]$Version,
        [string]$SpecName,
        [string]$Format,
        [string]$OutputDir,
        [string]$ExtraArgs = ""
    )
    
    $specPath = Join-Path $ScriptDir "openapi\$Version\$SpecName.$Format"
    
    if (-not (Test-Path $specPath)) {
        return "SKIP"
    }
    
    $testName = "$Version/$SpecName.$Format"
    
    try {
        # Test PowerShell generation
        $psOutput = Join-Path $OutputDir "ps"
        $null = New-Item -ItemType Directory -Force -Path $psOutput
        
        # Use Start-Process without redirection for speed
        $process = Start-Process -FilePath $Binary -ArgumentList $specPath,"--output",$psOutput,"--skip-validation" -NoNewWindow -PassThru -Wait
        
        if ($process.ExitCode -ne 0) {
            Write-Host "  ✗ $testName (PowerShell)" -ForegroundColor Red
            return "FAIL"
        }
        
        $psFiles = Get-ChildItem "$psOutput\*.ps1" -ErrorAction SilentlyContinue
        if ($psFiles.Count -eq 0) {
            Write-Host "  ✗ $testName (PowerShell - no files)" -ForegroundColor Red
            return "FAIL"
        }
        
        # Test Bash generation
        $shOutput = Join-Path $OutputDir "sh"
        $null = New-Item -ItemType Directory -Force -Path $shOutput
        
        $bashArgs = @($specPath, "--output", $shOutput, "--skip-validation", "--bash")
        if ($ExtraArgs) { $bashArgs += $ExtraArgs.Split(' ') }
        $process = Start-Process -FilePath $Binary -ArgumentList $bashArgs -NoNewWindow -PassThru -Wait
        
        if ($process.ExitCode -ne 0) {
            Write-Host "  ✗ $testName (Bash)" -ForegroundColor Red
            return "FAIL"
        }
        
        $shFiles = Get-ChildItem "$shOutput\*.sh" -ErrorAction SilentlyContinue
        if ($shFiles.Count -eq 0) {
            Write-Host "  ✗ $testName (Bash - no files)" -ForegroundColor Red
            return "FAIL"
        }
        
        Write-Host "  ✓ $testName" -ForegroundColor Green -NoNewline
        Write-Host " ($($psFiles.Count) PS, $($shFiles.Count) SH)" -ForegroundColor Gray
        return "PASS"
        
    } catch {
        Write-Host "  ✗ $testName (Exception: $_)" -ForegroundColor Red
        return "FAIL"
    }
}

# Create output directory
$outputBase = Join-Path $scriptDir "output"
if (Test-Path $outputBase) {
    Remove-Item $outputBase -Recurse -Force
}
$null = New-Item -ItemType Directory -Force -Path $outputBase

# Run tests for each version
foreach ($version in $testSpecs.Keys | Sort-Object) {
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "  OpenAPI $version" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    
    foreach ($spec in $testSpecs[$version]) {
        foreach ($format in @("json", "yaml")) {
            $totalTests++
            $outputDir = Join-Path $outputBase "$version\$spec\$format"
            
            $result = Test-Generation -Binary $Binary -ScriptDir $scriptDir -Version $version -SpecName $spec -Format $format -OutputDir $outputDir
            
            switch ($result) {
                "PASS" { $passedTests++ }
                "FAIL" { $failedTests++ }
                "SKIP" { $skippedTests++; $totalTests-- }
            }
        }
    }
}

# Summary
$duration = (Get-Date) - $startTime
Write-Host "`n╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                      Test Summary                          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan

Write-Host "`nTotal Tests:    $totalTests" -ForegroundColor White
Write-Host "Passed:         " -NoNewline -ForegroundColor White
Write-Host $passedTests -ForegroundColor Green
Write-Host "Failed:         " -NoNewline -ForegroundColor White
if ($failedTests -gt 0) {
    Write-Host $failedTests -ForegroundColor Red
} else {
    Write-Host $failedTests -ForegroundColor Green
}
Write-Host "Skipped:        $skippedTests" -ForegroundColor Yellow
Write-Host "Duration:       $($duration.TotalSeconds.ToString('F2'))s" -ForegroundColor White

if ($failedTests -gt 0) {
    Write-Host "`n✗ Some tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "`n✓ All tests passed!" -ForegroundColor Green
    exit 0
}
