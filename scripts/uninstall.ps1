# Ferronote Uninstaller for Windows

$ErrorActionPreference = "Stop"

function Write-Info ($msg) { Write-Host -ForegroundColor Cyan "[info] $msg" }
function Write-Success ($msg) { Write-Host -ForegroundColor Green "[success] $msg" }
function Write-WarningMsg ($msg) { Write-Host -ForegroundColor Yellow "[warn] $msg" }
function Write-ErrorMsg ($msg) { Write-Host -ForegroundColor Red "[error] $msg"; exit 1 }

$install_dir = Join-Path $env:USERPROFILE ".ferronote\bin"

Write-Info "Uninstalling Ferronote..."

if (Test-Path $install_dir) {
    Remove-Item -Path $install_dir -Recurse -Force
    Write-Success "Removed $install_dir."
} else {
    Write-WarningMsg "Ferronote directory not found at $install_dir."
}

# Remove from User PATH
$path_var = [Environment]::GetEnvironmentVariable("Path", "User")
if ($path_var -like "*$install_dir*") {
    $new_path = ($path_var -split ";" | Where-Object { $_ -and $_ -ne $install_dir }) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $new_path, "User")
    Write-Success "Removed $install_dir from User PATH."
}

Write-Success "Ferronote has been successfully uninstalled!"
