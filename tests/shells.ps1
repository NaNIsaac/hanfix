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

# The README's Rename-Decoded example, run as written: a preview renames nothing, -Apply renames only
# what decodes to a new, free name. The Korean README carries the same code.
$readme = Get-Content -Raw -Encoding utf8 (Join-Path $PSScriptRoot '..' 'README.md')
if ($readme -notmatch '(?s)<!-- rename-example -->\s*```powershell\r?\n(.*?)```') { throw 'FAIL rename example: block not found' }
$example = $Matches[1]
if (-not (Get-Content -Raw -Encoding utf8 (Join-Path $PSScriptRoot '..' 'README.ko.md')).Contains($example)) {
    throw 'FAIL rename example: README.ko.md differs'
}
Invoke-Expression $example
$dir = Join-Path ([IO.Path]::GetTempPath()) "hanfix-rename-$PID"
New-Item -ItemType Directory $dir | Out-Null
try {
    $tree = [string][char]0xD2B8 + [char]0xB9AC
    $ri = [string][char]0xB9AC
    '_ED_8A_B8_EB_A6_AC.txt', 'backup_2026_10_29.txt', 'bad_ED_8A.txt', '_EB_A6_AC.md', "$ri.md" |
        ForEach-Object { New-Item -ItemType File (Join-Path $dir $_) | Out-Null }
    $before = (Get-ChildItem -LiteralPath $dir -Name | Sort-Object) -join '|'
    $preview = @(Rename-Decoded -Path $dir 3>$null)
    if (($preview -join '|') -ne "_ED_8A_B8_EB_A6_AC.txt -> $tree.txt") { throw "FAIL rename preview: got '$($preview -join '|')'" }
    if (((Get-ChildItem -LiteralPath $dir -Name | Sort-Object) -join '|') -ne $before) { throw 'FAIL rename preview changed files' }
    Rename-Decoded -Path $dir -Apply 3>$null | Out-Null
    $after = (Get-ChildItem -LiteralPath $dir -Name | Sort-Object) -join '|'
    $want = ('_EB_A6_AC.md', 'backup_2026_10_29.txt', 'bad_ED_8A.txt', "$ri.md", "$tree.txt" | Sort-Object) -join '|'
    if ($after -ne $want) { throw "FAIL rename apply: got '$after'" }
} finally { Remove-Item -LiteralPath $dir -Recurse -Force }
Write-Host 'ok   powershell Rename-Decoded example'
