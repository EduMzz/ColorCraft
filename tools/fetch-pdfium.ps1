param([Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference = 'Stop'
$url = 'https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/pdfium-win-x64.tgz'
$expectedSha256 = '73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08'
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) ('colorcraft-pdfium-' + [guid]::NewGuid().ToString('N'))
$archive = Join-Path $tempRoot 'pdfium.tgz'
$extract = Join-Path $tempRoot 'extracted'
try {
    New-Item -ItemType Directory -Path $extract -Force | Out-Null
    Invoke-WebRequest -Uri $url -OutFile $archive
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expectedSha256) { throw "Hash do pacote PDFium não confere: $actual" }
    & tar -xzf $archive -C $extract LICENSE VERSION bin/pdfium.dll licenses
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível extrair o runtime PDFium.' }
    $pdfium = Join-Path $Destination 'third-party\pdfium'
    New-Item -ItemType Directory -Path $pdfium -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $extract 'bin\pdfium.dll') -Destination $Destination
    Copy-Item -LiteralPath (Join-Path $extract 'LICENSE') -Destination $pdfium
    Copy-Item -LiteralPath (Join-Path $extract 'VERSION') -Destination $pdfium
    Copy-Item -LiteralPath (Join-Path $extract 'licenses') -Destination $pdfium -Recurse
    Set-Content -LiteralPath (Join-Path $pdfium 'ARCHIVE-SHA256.txt') -Value $expectedSha256 -Encoding ascii
} finally {
    if (Test-Path -LiteralPath $tempRoot) {
        $resolvedTemp = (Resolve-Path -LiteralPath $tempRoot).Path
        $expectedTemp = [IO.Path]::GetFullPath($tempRoot)
        $tempParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\','/') + [IO.Path]::DirectorySeparatorChar
        if ($resolvedTemp -ne $expectedTemp -or -not $resolvedTemp.StartsWith($tempParent, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Refusing to remove an unexpected PDFium temporary directory.'
        }
        Remove-Item -LiteralPath $resolvedTemp -Recurse -Force
    }
}
