param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$Debug
)

$arguments = @("build", "-p", "joysharp-native", "--target", $Target)
if (-not $Debug) { $arguments += "--release" }

& cargo @arguments
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$profile = if ($Debug) { "debug" } else { "release" }
$library = Join-Path $PSScriptRoot "..\..\target\$Target\$profile\joysharp_native.dll"
Write-Host "Native library built: $(Resolve-Path $library)"
