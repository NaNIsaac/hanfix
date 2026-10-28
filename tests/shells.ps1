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

# With codigest's hook: both names are registered as runners, and the fixed command goes to its
# __cdg_run, here a stand-in defined after hanfix (looked up at call time).
$ko = [string][char]0x3157 + [char]0x3139
if (($global:CODIGEST_RUNNERS -join ' ') -ne "hf $ko") { throw "FAIL powershell runners: '$global:CODIGEST_RUNNERS'" }
function global:__cdg_run([string]$line) { "cdg_run:$line"; . ([scriptblock]::Create($line)) }
Add-History -InputObject ([pscustomobject]@{
    CommandLine = 'ㄷ초ㅐ hanfix-ok'; ExecutionStatus = 'Completed'; StartExecutionTime = $now; EndExecutionTime = $now })
$out = (hf -y) -join '|'
if ($out -ne 'cdg_run:echo hanfix-ok|hanfix-ok') { throw "FAIL powershell __cdg_run: got '$out'" }
Write-Host 'ok   powershell with codigest'
