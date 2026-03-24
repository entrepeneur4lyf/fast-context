[CmdletBinding()]
param(
    [string]$PythonExe = "",
    [switch]$SkipValidation,
    [switch]$SkipBuild,
    [switch]$SkipPublish,
    [switch]$SkipCargoPublish,
    [switch]$SkipNpmPublish,
    [switch]$SkipPythonPublish,
    [switch]$TagAfterPublish
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Invoke-Step {
    param([string]$Command)
    Write-Host "PS> $Command" -ForegroundColor DarkGray
    Invoke-Expression $Command
}

function Get-PythonCommand {
    param([string]$Requested)

    if ($Requested) {
        return $Requested
    }

    $preferred = "E:\models\bin\conda\envs\fast-context-py311\python.exe"
    if (Test-Path $preferred) {
        return $preferred
    }

    return "python"
}

function Get-CargoVersion {
    $match = Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if (-not $match) {
        throw "Unable to read version from Cargo.toml"
    }
    return $match.Matches[0].Groups[1].Value
}

function Get-NpmVersion {
    $package = Get-Content "package.json" -Raw | ConvertFrom-Json
    return $package.version
}

function Get-PythonVersion {
    $content = Get-Content "pyproject.toml" -Raw
    $match = [regex]::Match($content, '^\s*version\s*=\s*"([^"]+)"', [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if ($match.Success) {
        return $match.Groups[1].Value
    }

    $cargoVersion = Get-CargoVersion
    return $cargoVersion
}

function Assert-VersionAlignment {
    $cargoVersion = Get-CargoVersion
    $npmVersion = Get-NpmVersion
    $pythonVersion = Get-PythonVersion

    if (($cargoVersion -ne $npmVersion) -or ($cargoVersion -ne $pythonVersion)) {
        throw "Version mismatch detected. Cargo=$cargoVersion npm=$npmVersion python=$pythonVersion"
    }

    return $cargoVersion
}

function Assert-ReleaseEnv {
    param(
        [bool]$NeedCargo,
        [bool]$NeedNpm,
        [bool]$NeedPython
    )

    if ($NeedCargo -and -not $env:CARGO_REGISTRY_TOKEN) {
        throw "CARGO_REGISTRY_TOKEN is not set"
    }

    if ($NeedNpm -and -not $env:NPM_TOKEN) {
        throw "NPM_TOKEN is not set"
    }

    if ($NeedPython -and -not $env:PYPI_API_TOKEN) {
        throw "PYPI_API_TOKEN is not set"
    }
}

$python = Get-PythonCommand -Requested $PythonExe
$version = Assert-VersionAlignment

$shouldPublishCargo = -not $SkipPublish -and -not $SkipCargoPublish
$shouldPublishNpm = -not $SkipPublish -and -not $SkipNpmPublish
$shouldPublishPython = -not $SkipPublish -and -not $SkipPythonPublish

Write-Step "Preparing local coordinated release for version $version"
Write-Host "Python interpreter: $python"

if (-not $SkipValidation) {
    Write-Step "Running validation"
    Invoke-Step "cargo check"
    Invoke-Step "cargo test"
    Invoke-Step "cargo test --test cli_tests --features cli"
    Invoke-Step "cargo clippy --all-targets --all-features -- -D warnings"
    Invoke-Step "npm install"
    Invoke-Step "npm run build:debug"
    Invoke-Step "npm test"
    Invoke-Step "& `"$python`" -m pytest tests/python"
    Invoke-Step "cargo audit"
    Invoke-Step "npm audit --audit-level moderate"
}

if (-not $SkipBuild) {
    Write-Step "Building release artifacts"
    Invoke-Step "cargo build --release --bin fast-context --features cli"
    Invoke-Step "cargo build --release --bin fast-context-mcp --features mcp"
    Invoke-Step "npm run build"
    Invoke-Step "npm pack"
    Invoke-Step "& `"$python`" -m maturin build --release --features python"
}

if (-not $SkipPublish) {
    Write-Step "Checking publish credentials"
    Assert-ReleaseEnv -NeedCargo:$shouldPublishCargo -NeedNpm:$shouldPublishNpm -NeedPython:$shouldPublishPython

    Write-Step "Publishing release channels"

    if ($shouldPublishCargo) {
        Invoke-Step "cargo publish --locked"
    }

    if ($shouldPublishNpm) {
        Invoke-Step "npm publish"
    }

    if ($shouldPublishPython) {
        Invoke-Step "& `"$python`" -m maturin publish --features python"
    }

    if ($TagAfterPublish) {
        Write-Step "Tagging release"
        Invoke-Step "git tag v$version"
        Invoke-Step "git push origin main --tags"
    }
}
else {
    Write-Step "Publish skipped"
}

Write-Step "Release script completed"
