$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$cargoToml = Join-Path $repoRoot "Cargo.toml"
$manifest = Get-Content -Path $cargoToml -Raw

if ($manifest -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw "Could not read package version from Cargo.toml"
}

$appVersion = $Matches[1]
$parts = $appVersion.Split(".")
if ($parts.Count -gt 3) {
    $productVersion = ($parts[0..2] -join ".") + ".0"
} else {
    $productVersion = ($parts + @("0", "0", "0", "0") | Select-Object -First 4) -join "."
}

$distDir = Join-Path $repoRoot "dist"
$exePath = Join-Path $repoRoot "target\release\apex_footwork.exe"
$iconPath = Join-Path $repoRoot "apex-footwork.ico"
$outFile = Join-Path $distDir "ApexFootwork-$appVersion-setup.exe"
$nsiPath = Join-Path $repoRoot "installer\apex-footwork.nsi"

if (-not (Test-Path $iconPath)) {
    throw "Application icon was not found: $iconPath"
}

$makensisPath = $null
$makensis = Get-Command "makensis" -ErrorAction SilentlyContinue
if (-not $makensis) {
    $candidate = Join-Path ${env:ProgramFiles(x86)} "NSIS\makensis.exe"
    if (Test-Path $candidate) {
        $makensisPath = $candidate
    }
} else {
    $makensisPath = $makensis.Source
}
if (-not (Test-Path $makensisPath)) {
    throw "NSIS makensis.exe was not found. Install NSIS and make sure makensis is available in PATH."
}

New-Item -ItemType Directory -Force -Path $distDir | Out-Null

Push-Location $repoRoot
try {
    cargo build --release
    & $makensisPath `
        "/DAPP_VERSION=$appVersion" `
        "/DPRODUCT_VERSION=$productVersion" `
        "/DAPP_EXE_PATH=$exePath" `
        "/DAPP_ICON_PATH=$iconPath" `
        "/DOUT_FILE=$outFile" `
        $nsiPath
}
finally {
    Pop-Location
}

Write-Host "Installer created: $outFile"
