#Requires -Version 5.1
<#
.SYNOPSIS
    Download and verify the curated plant image/icon set declared in manifest.json.

.DESCRIPTION
    Reads src-tauri/resources/images/plants/manifest.json and, for each asset,
    downloads its source_url into its declared path (skipping files that already
    exist unless -Force is passed). Verifies the file is non-empty and that .svg
    files contain an <svg> root.

.EXAMPLE
    .\scripts\fetch-plant-icons.ps1           # download missing files only
    .\scripts\fetch-plant-icons.ps1 -Force    # re-download everything
#>
[CmdletBinding()]
param(
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repo 'src-tauri\resources\images\plants\manifest.json'
$root = Split-Path -Parent $manifestPath

if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "manifest.json not found at $manifestPath"
}
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json

$downloaded = 0
$skipped = 0
$failed = @()

foreach ($entry in $manifest.assets.PSObject.Properties) {
    $assetId = $entry.Name
    $info = $entry.Value
    $rel = $info.path
    $dest = Join-Path $root ($rel -replace '/', '\')
    $dir = Split-Path -Parent $dest
    New-Item -ItemType Directory -Force -Path $dir | Out-Null

    if ((Test-Path -LiteralPath $dest) -and -not $Force) {
        $skipped++
        Write-Host "[skip] $assetId -> $rel" -ForegroundColor DarkGray
        continue
    }
    if (-not $info.source_url) {
        Write-Host "[nourl] $assetId -> $rel" -ForegroundColor Yellow
        continue
    }

    try {
        Invoke-WebRequest -Uri $info.source_url -OutFile $dest
        $bytes = (Get-Item -LiteralPath $dest).Length
        if ($bytes -le 0) { throw "empty download" }
        $head = (Get-Content -LiteralPath $dest -TotalCount 1) -join ''
        if ($rel -like '*.svg' -and $head -notmatch '<svg') {
            Write-Warning "    $rel does not start with an <svg> root (bytes=$bytes)"
        }
        $downloaded++
        Write-Host "[ok]   $assetId -> $rel ($bytes bytes)" -ForegroundColor Green
    } catch {
        $failed += $assetId
        Write-Host "[fail] $assetId ($($_.Exception.Message))" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "Done: $downloaded downloaded, $skipped skipped, $($failed.Count) failed."
if ($failed.Count -gt 0) {
    Write-Host "Failed assets:" -ForegroundColor Red
    $failed | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
}
exit 0
