param([string]$Target = "x86_64-pc-windows-msvc", [switch]$PortableOnly)
$ErrorActionPreference = "Stop"
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    $BuildArguments = @("scripts/build.py", "--platform", "windows", "--target", $Target)
    if ($PortableOnly) { $BuildArguments += "--portable-only" }
    python @BuildArguments
    if ($LASTEXITCODE -ne 0) { throw "Windows packaging failed" }
} finally { Pop-Location }
