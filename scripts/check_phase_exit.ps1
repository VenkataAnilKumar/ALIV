param(
    [string]$ProjectRoot = ".",
    [int]$Runs = 2
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path $ProjectRoot).Path
Set-Location $root

Write-Host "Running Phase exit gate with $Runs consecutive full-suite run(s)..."
python .\benchmarks\run_full_benchmark_suite.py --runs $Runs

$latest = Join-Path $root "benchmarks\results\full_suite_latest.json"
if (-not (Test-Path $latest)) {
    Write-Error "Missing benchmark results file: $latest"
}

$result = Get-Content $latest -Raw | ConvertFrom-Json
if (-not $result.phase4_exit.pass) {
    Write-Error "Phase 4 exit gate failed: $($result.phase4_exit.reason)"
}

Write-Host "Phase 4 exit gate passed."
