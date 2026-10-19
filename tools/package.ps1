param([switch]$SkipBuild, [string]$OutputTag = '')
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskCargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path -LiteralPath $taskCargo)) { $taskCargo = 'cargo' }
Push-Location -LiteralPath $taskRoot
try {
    if (-not $SkipBuild) {
        & $taskCargo build --locked --release -p colorcraft
        if ($LASTEXITCODE -ne 0) { throw 'A compilação falhou.' }
    }
    & $taskCargo run --locked -p colorcraft-core --example create_examples -- (Join-Path $taskRoot 'examples')
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível gerar os exemplos.' }
    & (Join-Path $PSScriptRoot 'collect-licenses.ps1')
    $taskManifest = Get-Content -LiteralPath (Join-Path $taskRoot 'Cargo.toml') -Raw
    $taskVersion = [regex]::Match($taskManifest, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
    if (-not $taskVersion) { throw 'Versão ausente no manifesto.' }
    if ($OutputTag -and $OutputTag -notmatch '^[a-z0-9-]+$') { throw 'OutputTag deve conter somente letras minúsculas, números e hífen.' }
    $taskPackage = Join-Path $taskRoot ('dist\ColorCraft-' + $taskVersion + $(if ($OutputTag) { '-' + $OutputTag }) + '-windows-x64')
    if (Test-Path -LiteralPath $taskPackage) { throw 'A pasta de distribuição já existe; preserve-a ou escolha outro destino antes de empacotar novamente.' }
    New-Item -ItemType Directory -Path $taskPackage -Force | Out-Null
    Copy-Item -LiteralPath 'target\release\colorcraft.exe' -Destination (Join-Path $taskPackage 'ColorCraft.exe')
    foreach ($taskFile in @('README.md','ROADMAP.md','CONTRIBUTING.md','AGENTS.md','ASSETS.md','NOTICE.md','LICENSE-MIT','LICENSE-APACHE','Cargo.lock','tools\Register-ColorCraftFiles.ps1','tools\Unregister-ColorCraftFiles.ps1','tools\Register-ColorCraftProtocol.ps1')) {
        Copy-Item -LiteralPath $taskFile -Destination $taskPackage
    }
    foreach ($taskFolder in @('docs','schemas','examples')) {
        Copy-Item -LiteralPath $taskFolder -Destination $taskPackage -Recurse
    }
    New-Item -ItemType Directory -Path (Join-Path $taskPackage 'assets') -Force | Out-Null
    Copy-Item -LiteralPath 'assets\ASSETS.md','assets\colorcraft-128.png','assets\palette-file.ico','assets\palette-file.png','assets\palette-file-small.png' -Destination (Join-Path $taskPackage 'assets')
    Set-Content -LiteralPath (Join-Path $taskPackage 'portable.txt') -Value 'ColorCraft portable: preserve ColorCraftData when updating.' -Encoding utf8
    $taskThird = Join-Path $taskPackage 'third-party'
    New-Item -ItemType Directory -Path $taskThird -Force | Out-Null
    & (Join-Path $PSScriptRoot 'fetch-pdfium.ps1') -Destination $taskPackage
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível preparar o runtime PDFium e suas licenças.' }
    $taskMetadataJson = & $taskCargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível inventariar as licenças.' }
    $taskMetadata = $taskMetadataJson | ConvertFrom-Json
    $taskInventory = @()
    foreach ($taskDep in $taskMetadata.packages) {
        if (-not $taskDep.source) { continue }
        $taskDepRoot = Split-Path -Parent $taskDep.manifest_path
        $taskDepDest = Join-Path $taskThird ($taskDep.name + '-' + $taskDep.version)
        New-Item -ItemType Directory -Path $taskDepDest -Force | Out-Null
        $taskNotices = @(Get-ChildItem -LiteralPath $taskDepRoot -File | Where-Object { $_.Name -match '^(licen[cs]e|copying|notice|unlicense)' })
        if ($taskDep.license_file) {
            $taskLicensePath = $taskDep.license_file
            if (-not [IO.Path]::IsPathRooted($taskLicensePath)) { $taskLicensePath = Join-Path $taskDepRoot $taskLicensePath }
            if (Test-Path -LiteralPath $taskLicensePath) { $taskNotices += Get-Item -LiteralPath $taskLicensePath }
        }
        foreach ($taskNotice in ($taskNotices | Sort-Object FullName -Unique)) {
            Copy-Item -LiteralPath $taskNotice.FullName -Destination $taskDepDest
        }
        $taskExtra = Join-Path $taskRoot ('third-party-extra\' + $taskDep.name + '-' + $taskDep.version)
        if (Test-Path -LiteralPath $taskExtra) {
            Get-ChildItem -LiteralPath $taskExtra -File | Copy-Item -Destination $taskDepDest
        }
        if ($taskDep.license -eq 'MPL-2.0') {
            Copy-Item -LiteralPath (Join-Path $taskDepRoot 'src') -Destination $taskDepDest -Recurse
            Copy-Item -LiteralPath (Join-Path $taskDepRoot 'Cargo.toml') -Destination $taskDepDest
            Copy-Item -LiteralPath (Join-Path $taskDepRoot 'README.md') -Destination $taskDepDest
        }
        if ($taskDep.name -eq 'epaint_default_fonts') {
            foreach ($taskFontNotice in (Get-ChildItem -LiteralPath $taskDepRoot -Recurse -File | Where-Object { $_.Name -match '(OFL|UFL|LICENSE|LICENCE|COPYING)|\.txt$' })) {
                $taskRelative = $taskFontNotice.FullName.Substring($taskDepRoot.Length).TrimStart('\','/')
                $taskFontTarget = Join-Path $taskDepDest $taskRelative
                New-Item -ItemType Directory -Path (Split-Path -Parent $taskFontTarget) -Force | Out-Null
                Copy-Item -LiteralPath $taskFontNotice.FullName -Destination $taskFontTarget
            }
        }
        $taskInventory += [pscustomobject]@{ name=$taskDep.name; version=$taskDep.version; license=$taskDep.license; source=$taskDep.source; repository=$taskDep.repository; notices=($taskNotices.Name -join ', ') }
    }
    $taskInventory | Export-Csv -LiteralPath (Join-Path $taskThird 'dependencies.csv') -NoTypeInformation -Encoding utf8
    $taskCraftManifest = $taskMetadata.packages | Where-Object name -eq 'craft-cms' | Select-Object -First 1
    $taskCraftRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $taskCraftManifest.manifest_path))
    $taskCraftDest = Join-Path $taskThird 'craft-libs'
    New-Item -ItemType Directory -Path $taskCraftDest -Force | Out-Null
    foreach ($taskNotice in (Get-ChildItem -LiteralPath $taskCraftRoot -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|NOTICE)' })) {
        Copy-Item -LiteralPath $taskNotice.FullName -Destination $taskCraftDest
    }
    $taskSums = Get-ChildItem -LiteralPath $taskPackage -Recurse -File | ForEach-Object {
        (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() + '  ' + $_.FullName.Substring($taskPackage.Length + 1).Replace('\','/')
    }
    Set-Content -LiteralPath (Join-Path $taskPackage 'SHA256SUMS.txt') -Value $taskSums -Encoding utf8
    $taskZip = $taskPackage + '.zip'
    Compress-Archive -LiteralPath $taskPackage -DestinationPath $taskZip -CompressionLevel Optimal
    $taskZipHash = Get-FileHash -LiteralPath $taskZip -Algorithm SHA256
    Set-Content -LiteralPath ($taskZip + '.sha256') -Value ($taskZipHash.Hash.ToLowerInvariant() + '  ' + [IO.Path]::GetFileName($taskZip)) -Encoding utf8
    Write-Output $taskZip
    Write-Output ('SHA256 ' + $taskZipHash.Hash)
} finally {
    Pop-Location
}
