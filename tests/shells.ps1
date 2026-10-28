# PowerShell integration: the same check as tests/shells.sh, with the history entry added directly.
$ErrorActionPreference = 'Stop'
Invoke-Expression (hanfix --init powershell | Out-String)
$now = Get-Date
Add-History -InputObject ([pscustomobject]@{
    CommandLine = 'ㄷ초ㅐ hanfix-ok'; ExecutionStatus = 'Completed'; StartExecutionTime = $now; EndExecutionTime = $now })
$out = hf -y
if (-not $? -or $out -ne 'hanfix-ok') { throw "FAIL powershell: got '$out'" }
if ((Get-Alias ([string][char]0x3157 + [char]0x3139)).Definition -ne 'hf') { throw 'FAIL powershell: Korean alias' }

# A fixed command that fails leaves $? False, as it would typed (hanfix#11).
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$failing = (hanfix --to ko -- 'pwsh -nop -c "exit 3"' | Out-String).Trim()
$now = Get-Date
Add-History -InputObject ([pscustomobject]@{
    CommandLine = $failing; ExecutionStatus = 'Completed'; StartExecutionTime = $now; EndExecutionTime = $now })
hf -y
$ok = $?
if ($ok -or $LASTEXITCODE -ne 3) { throw "FAIL powershell status: `$?=$ok exit=$LASTEXITCODE" }
Write-Host 'ok   powershell'

# With codigest's hook: both names are registered as runners, once even when the setup is sourced
# again, and the fixed command goes to its __cdg_run, here a stand-in defined after hanfix (looked up
# at call time).
Invoke-Expression (hanfix --init powershell | Out-String)
$ko = [string][char]0x3157 + [char]0x3139
if (($global:CODIGEST_RUNNERS -join ' ') -ne "hf $ko") { throw "FAIL powershell runners: '$global:CODIGEST_RUNNERS'" }
function global:__cdg_run([string]$line) { "cdg_run:$line"; . ([scriptblock]::Create($line)) }
Add-History -InputObject ([pscustomobject]@{
    CommandLine = 'ㄷ초ㅐ hanfix-ok'; ExecutionStatus = 'Completed'; StartExecutionTime = $now; EndExecutionTime = $now })
$out = (hf -y) -join '|'
if ($out -ne 'cdg_run:echo hanfix-ok|hanfix-ok') { throw "FAIL powershell __cdg_run: got '$out'" }
Write-Host 'ok   powershell with codigest'
