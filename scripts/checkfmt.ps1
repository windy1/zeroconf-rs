$rs_files = Get-ChildItem "examples/browser/src","examples/service/src","zeroconf/src","zeroconf-macros/src" -Recurse -Filter *.rs

$exit_code = 0
foreach ($file in $rs_files)
{
    Write-Host $file.FullName
    rustfmt --check --verbose $file.FullName
    if ($LASTEXITCODE -ne 0) { $exit_code = -1; }
}
Exit $exit_code
