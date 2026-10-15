# PowerShell integration: the same check as tests/shells.sh, with the history entry added directly.
$ErrorActionPreference = 'Stop'
Invoke-Expression (hanfix --init powershell | Out-String)
$now = Get-Date
Add-History -InputObject ([pscustomobject]@{
    CommandLine = 'ㄷ초ㅐ hanfix-ok'; ExecutionStatus = 'Completed'; StartExecutionTime = $now; EndExecutionTime = $now })
$out = hf -y
if ($out -ne 'hanfix-ok') { throw "FAIL powershell: got '$out'" }
if ((Get-Alias ([string][char]0x3157 + [char]0x3139)).Definition -ne 'hf') { throw 'FAIL powershell: Korean alias' }
Write-Host 'ok   powershell'
