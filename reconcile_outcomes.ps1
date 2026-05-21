param(
    [string]$StrategyConfig = "configs/btc_ensemble.env",
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root

if (-not (Test-Path $StrategyConfig)) {
    throw "Strategy config not found: $StrategyConfig"
}

$env:STRATEGY_CONFIG = $StrategyConfig

$CargoArgs = @("run", "--locked")
if ($Release) {
    $CargoArgs += "--release"
}
$CargoArgs += @("--bin", "reconcile_outcomes")

Write-Host "Running Polymarket official outcome reconciliation"
Write-Host "Strategy config: $StrategyConfig"
cargo @CargoArgs
