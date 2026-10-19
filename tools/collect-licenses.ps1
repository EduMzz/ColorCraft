$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskCargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path -LiteralPath $taskCargo)) { $taskCargo = 'cargo' }
Push-Location -LiteralPath $taskRoot
try {
    $taskJson = & $taskCargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao ler os metadados.' }
    $taskMetadata = $taskJson | ConvertFrom-Json
    $taskCache = @{}
    foreach ($taskDep in ($taskMetadata.packages | Where-Object source)) {
        if ($taskDep.name -like 'craft-*') { continue }
        $taskDepRoot = Split-Path -Parent $taskDep.manifest_path
        $taskNotices = @(Get-ChildItem -LiteralPath $taskDepRoot -File | Where-Object Name -Match '^(licen[cs]e|copying|notice|unlicense)')
        if ($taskNotices.Count -gt 0) { continue }
        $taskDest = Join-Path $taskRoot ('third-party-extra\' + $taskDep.name + '-' + $taskDep.version)
        if (Test-Path -LiteralPath (Join-Path $taskDest 'SOURCES.txt')) { continue }
        $taskVcs = Get-Content -LiteralPath (Join-Path $taskDepRoot '.cargo_vcs_info.json') -Raw | ConvertFrom-Json
        if ($taskDep.repository -notmatch '^https://github.com/([^/]+/[^/]+)') { throw ('Verifique a licença de ' + $taskDep.name) }
        $taskRepo = $Matches[1].TrimEnd('/')
        $taskKey = $taskRepo + '/' + $taskVcs.git.sha1
        if (-not $taskCache.ContainsKey($taskKey)) {
            $taskApi = 'https://api.github.com/repos/' + $taskRepo + '/contents?ref=' + $taskVcs.git.sha1
            $taskContents = Invoke-RestMethod -Uri $taskApi -Headers @{'User-Agent'='ColorCraft-license-packaging'}
            $taskFound = @{}
            foreach ($taskRemote in ($taskContents | Where-Object { $_.type -eq 'file' -and $_.name -match '^(licen[cs]e|copying|notice|unlicense)' })) {
                $taskResponse = Invoke-WebRequest -Uri $taskRemote.download_url
                $taskFound[$taskRemote.name] = @{ text=[string]$taskResponse.Content; url=$taskRemote.download_url }
            }
            if ($taskFound.Count -eq 0) { throw ('Nenhuma licença encontrada: ' + $taskKey) }
            $taskCache[$taskKey] = $taskFound
        }
        New-Item -ItemType Directory -Path $taskDest -Force | Out-Null
        $taskSources = @('Licenças upstream inalteradas; commit original do pacote Cargo: ' + $taskKey)
        foreach ($taskEntry in $taskCache[$taskKey].GetEnumerator()) {
            [IO.File]::WriteAllText((Join-Path $taskDest $taskEntry.Key), $taskEntry.Value.text, [Text.UTF8Encoding]::new($false))
            $taskSources += $taskEntry.Value.url
        }
        Set-Content -LiteralPath (Join-Path $taskDest 'SOURCES.txt') -Value $taskSources -Encoding utf8
        Write-Output ('Licenças: ' + $taskDep.name + ' ' + $taskDep.version)
    }
} finally { Pop-Location }
