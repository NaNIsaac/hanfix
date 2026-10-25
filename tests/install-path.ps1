# install.ps1 PATH merge. Only the Merge-PathEntry function is loaded, so nothing is downloaded
# and neither the user environment nor any file is touched.
$ErrorActionPreference = 'Stop'
$ast = [Management.Automation.Language.Parser]::ParseFile(
    (Join-Path $PSScriptRoot '..\install.ps1'), [ref]$null, [ref]$null)
$fn = $ast.Find({ param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq 'Merge-PathEntry' }, $true)
if (-not $fn) { throw 'FAIL install-path: Merge-PathEntry not found in install.ps1' }
Invoke-Expression $fn.Extent.Text

$dir = 'C:\Tools\hanfix'
$cases = @(
    @{ Name = 'null';              Path = $null;                  Want = $dir }
    @{ Name = 'empty';             Path = '';                     Want = $dir }
    @{ Name = 'trailing ;';        Path = 'C:\a;C:\b;';           Want = "C:\a;C:\b;$dir" }
    @{ Name = 'only ;';            Path = ';';                    Want = $dir }
    @{ Name = 'plain';             Path = 'C:\a;C:\b';            Want = "C:\a;C:\b;$dir" }
    @{ Name = 'already present';   Path = "C:\a;$dir;C:\b";       Want = "C:\a;$dir;C:\b" }
    @{ Name = 'present, trailing'; Path = "C:\a;$dir;";           Want = "C:\a;$dir;" }
)
$failed = 0
foreach ($c in $cases) {
    try { $got = Merge-PathEntry $c.Path $dir } catch { $got = "threw: $($_.Exception.Message)" }
    if ($got -ceq $c.Want) { Write-Host "ok   install-path: $($c.Name)" }
    else { Write-Host "FAIL install-path: $($c.Name): want '$($c.Want)', got '$got'"; $failed++ }
}
if ($failed) { throw "FAIL install-path: $failed case(s)" }
