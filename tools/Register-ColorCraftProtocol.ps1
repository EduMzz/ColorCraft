param([string]$Executable, [string]$BridgeRoot)
$ErrorActionPreference = 'Stop'
if (-not $Executable) { $Executable = Join-Path $PSScriptRoot 'ColorCraft.exe' }
$Executable = (Resolve-Path -LiteralPath $Executable).Path
if (-not $BridgeRoot) { $BridgeRoot = Join-Path $env:LOCALAPPDATA 'ColorCraft\Bridge\v1' }
$taskKey = 'Registry::HKEY_CURRENT_USER\Software\Classes\colorcraft'
New-Item -Path $taskKey -Force | Out-Null
Set-Item -Path $taskKey -Value 'URL:ColorCraft palette navigation'
New-ItemProperty -Path $taskKey -Name 'URL Protocol' -Value '' -PropertyType String -Force | Out-Null
New-Item -Path (Join-Path $taskKey 'DefaultIcon') -Force | Out-Null
Set-Item -Path (Join-Path $taskKey 'DefaultIcon') -Value ('"' + $Executable + '",0')
New-Item -Path (Join-Path $taskKey 'shell\open\command') -Force | Out-Null
Set-Item -Path (Join-Path $taskKey 'shell\open\command') -Value ('"' + $Executable + '" "%1"')
New-Item -ItemType Directory -Path $BridgeRoot -Force | Out-Null
$taskRegistration = @{ schemaVersion = 1; executable = $Executable; version = '0.1.14' } | ConvertTo-Json
$taskTemporary = Join-Path $BridgeRoot ('registration-' + [guid]::NewGuid().ToString() + '.tmp')
[IO.File]::WriteAllText($taskTemporary, $taskRegistration, [Text.UTF8Encoding]::new($false))
Move-Item -LiteralPath $taskTemporary -Destination (Join-Path $BridgeRoot 'app.json') -Force
Write-Output 'colorcraft:// registrado para o usuário atual. Associações de arquivos preservadas.'
