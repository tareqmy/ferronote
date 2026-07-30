param (
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'

$Repo = "tareqmy/ferronote"
$InstallDir = "$env:LocalAppData\Programs\ferronote"

# Handle uninstall flag or environment variable
if ($Uninstall -or $env:UNINSTALL -eq 'true') {
    Write-Host "🗑️ Uninstalling Ferronote..." -ForegroundColor Yellow
    if (Test-Path $InstallDir) {
        Remove-Item -Path $InstallDir -Recurse -Force
        Write-Host "Removed $InstallDir." -ForegroundColor Green
    } else {
        Write-Host "Ferronote directory not found at $InstallDir." -ForegroundColor DarkGray
    }

    # Remove from User PATH
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -like "*$InstallDir*") {
        $NewPath = ($UserPath -split ';' | Where-Object { $_ -and $_ -ne $InstallDir }) -join ';'
        [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
        Write-Host "Removed $InstallDir from User PATH." -ForegroundColor Green
    }

    Write-Host "`n✨ Ferronote successfully uninstalled!" -ForegroundColor Green
    exit 0
}

Write-Host "⚡ Installing Ferronote for Windows..." -ForegroundColor Cyan

# Determine latest version tag
try {
    $ReleaseApi = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
    $Tag = $ReleaseApi.tag_name
} catch {
    if (Test-Path ".version") {
        $Ver = (Get-Content .version).Trim()
        $Tag = "v$Ver"
    } else {
        $Tag = "v1.1.3"
    }
}

$ZipName = "ferronote-windows-amd64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/$ZipName"
$TempZip = Join-Path $env:TEMP $ZipName

Write-Host "📥 Downloading Ferronote $Tag..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip -UseBasicParsing

# Create install directory if it doesn't exist
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Host "📦 Extracting archive to $InstallDir..." -ForegroundColor Yellow
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item -Path $TempZip -Force

# Create fnt.exe shortcut copy
if (Test-Path "$InstallDir\ferronote.exe") {
    Copy-Item -Path "$InstallDir\ferronote.exe" -Destination "$InstallDir\fnt.exe" -Force
}

# Add InstallDir to User PATH environment variable if not already present
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    $env:PATH = "$env:PATH;$InstallDir"
    Write-Host "🔗 Added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "`n✨ Ferronote $Tag successfully installed to $InstallDir\ferronote.exe (with 'fnt' shortcut)!" -ForegroundColor Green
Write-Host "Open a new terminal window and run 'ferronote' or 'fnt' to start taking notes." -ForegroundColor White
