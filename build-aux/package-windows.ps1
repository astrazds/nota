$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
$packageRoot = Join-Path $repoRoot 'dist/Nota-windows-x64'
$archive = Join-Path $repoRoot 'dist/Nota-windows-x64.zip'

foreach ($required in @('Nota.Windows.exe', 'Nota.Windows.pri', 'nota_ffi.dll', 'Microsoft.UI.Xaml.dll', 'Assets/Fonts/source-sans-3-latin-wght-normal.woff2')) {
    if (-not (Test-Path -LiteralPath (Join-Path $packageRoot $required) -PathType Leaf)) {
        throw "The published app is missing $required. Run mise run package:windows."
    }
}

Copy-Item -LiteralPath (Join-Path $repoRoot 'LICENSE') -Destination $packageRoot
Copy-Item -LiteralPath (Join-Path $repoRoot 'docs/windows.md') -Destination (Join-Path $packageRoot 'README.md')
Compress-Archive -Path (Join-Path $packageRoot '*') -DestinationPath $archive -Force
$stream = [IO.File]::OpenRead($archive)
$sha256 = [Security.Cryptography.SHA256]::Create()
try {
    $hash = [BitConverter]::ToString($sha256.ComputeHash($stream)).Replace('-', '').ToLowerInvariant()
    Write-Output "SHA256 $hash  $archive"
} finally {
    $stream.Dispose()
    $sha256.Dispose()
}
