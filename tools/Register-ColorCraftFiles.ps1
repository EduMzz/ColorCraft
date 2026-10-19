param()
$ErrorActionPreference = 'Stop'
$taskRoot = $PSScriptRoot
if (-not (Test-Path -LiteralPath (Join-Path $taskRoot 'ColorCraft.exe'))) {
    $taskRoot = Split-Path -Parent $PSScriptRoot
}
$taskExe = Join-Path $taskRoot 'ColorCraft.exe'
$taskIcon = Join-Path $taskRoot 'assets\palette-file.ico'
if (-not (Test-Path -LiteralPath $taskExe)) { throw "ColorCraft.exe não encontrado em $taskRoot" }
if (-not (Test-Path -LiteralPath $taskIcon)) { throw "Ícone de arquivo não encontrado em $taskIcon" }

$taskClasses = 'Registry::HKEY_CURRENT_USER\Software\Classes'
$taskTypes = @(
    @{ Extension = '.colorcraft'; ProgId = 'ColorCraft.Project'; Name = 'Projeto ColorCraft' },
    @{ Extension = '.colorcraft-palette'; ProgId = 'ColorCraft.Palette'; Name = 'Paleta ColorCraft' }
)
foreach ($taskType in $taskTypes) {
    $taskExtension = Join-Path $taskClasses $taskType.Extension
    $taskProgId = Join-Path $taskClasses $taskType.ProgId
    New-Item -Path $taskExtension -Force | Out-Null
    Set-Item -Path $taskExtension -Value $taskType.ProgId
    New-Item -Path (Join-Path $taskProgId 'DefaultIcon') -Force | Out-Null
    Set-Item -Path (Join-Path $taskProgId 'DefaultIcon') -Value ('"' + $taskIcon + '",0')
    New-Item -Path (Join-Path $taskProgId 'shell\open\command') -Force | Out-Null
    Set-Item -Path (Join-Path $taskProgId 'shell\open\command') -Value ('"' + $taskExe + '" "%1"')
    New-ItemProperty -Path $taskProgId -Name 'FriendlyTypeName' -Value $taskType.Name -PropertyType String -Force | Out-Null
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ColorCraftShellNotify {
    [DllImport("shell32.dll")]
    public static extern void SHChangeNotify(int eventId, uint flags, IntPtr item1, IntPtr item2);
}
'@
[ColorCraftShellNotify]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
Write-Output 'Associações do ColorCraft registradas para o usuário atual.'
