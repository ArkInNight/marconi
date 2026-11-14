# Marconi one-liner for Windows.
#
#   irm https://raw.githubusercontent.com/ArkInNight/marconi/main/scripts/get.ps1 | iex
#
# Downloads the latest release binary, verifies its SHA-256 against the
# release's SHA256SUMS, runs it, then removes the temporary files.
#
# Optional arguments (when the script is run from a file):
#   .\get.ps1 --json
# When using `irm | iex`, pass arguments through the environment instead:
#   $env:MARCONI_ARGS='--json'; irm .../get.ps1 | iex
#
# marconi only reads the game's memory locally. Nothing is uploaded.

[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
$ProgressPreference = 'SilentlyContinue'
$ErrorActionPreference = 'Stop'

$repo = 'ArkInNight/marconi'
$asset = 'marconi-windows-x64.zip'
$base = "https://github.com/$repo/releases/latest/download"

$tmp = Join-Path ([IO.Path]::GetTempPath()) ('marconi-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Write-Output 'Downloading marconi...'
    $zip = Join-Path $tmp $asset
    Invoke-WebRequest -Uri "$base/$asset" -OutFile $zip -UseBasicParsing

    $sums = (Invoke-WebRequest -Uri "$base/SHA256SUMS" -UseBasicParsing).Content
    $line = ($sums -split "`n" | Where-Object { $_ -match [regex]::Escape($asset) } | Select-Object -First 1)
    if (-not $line) { throw "SHA256SUMS does not list $asset" }
    $expected = ($line.Trim() -split '\s+')[0].ToLower()
    $actual = (Get-FileHash -Path $zip -Algorithm SHA256).Hash.ToLower()
    if ($expected -ne $actual) {
        throw "SHA-256 mismatch for $asset (expected $expected, got $actual)"
    }

    Expand-Archive -Path $zip -DestinationPath $tmp -Force
    $exe = Join-Path $tmp 'marconi.exe'
    if (-not (Test-Path $exe)) { throw 'marconi.exe is missing from the release archive' }

    $cliArgs = @()
    if ($env:MARCONI_ARGS) {
        $cliArgs = @($env:MARCONI_ARGS -split '\s+' | Where-Object { $_ })
    }

    & $exe @cliArgs
} catch {
    Write-Output "marconi: $($_.Exception.Message)"
    Write-Output "Download it manually from https://github.com/$repo/releases"
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
