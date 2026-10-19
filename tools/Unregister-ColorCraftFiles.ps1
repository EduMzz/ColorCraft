param()
$ErrorActionPreference = 'Stop'
$taskClasses = 'Registry::HKEY_CURRENT_USER\Software\Classes'
foreach ($taskType in @(
    @{ Extension = '.colorcraft'; ProgId = 'ColorCraft.Project' },
    @{ Extension = '.colorcraft-palette'; ProgId = 'ColorCraft.Palette' }
)) {
    $taskExtension = Join-Path $taskClasses $taskType.Extension
    $taskKey = Get-Item -LiteralPath $taskExtension -ErrorAction SilentlyContinue
    if ($null -ne $taskKey -and $taskKey.GetValue('') -eq $taskType.ProgId) {
        Remove-Item -LiteralPath $taskExtension -Recurse -Force
    }
    Remove-Item -LiteralPath (Join-Path $taskClasses $taskType.ProgId) -Recurse -Force -ErrorAction SilentlyContinue
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
Write-Output 'Associações do ColorCraft removidas para o usuário atual.'
