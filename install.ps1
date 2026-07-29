$ErrorActionPreference = 'Stop'

$Repo = "tareqmy/ferronote"
$InstallDir = "$env:LocalAppData\Programs\ferronote"

Write-Host "⚡ Installing Ferronote for Windows..." -ForegroundColor Cyan

# Determine latest version tag
try {
    $ReleaseApi = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
    $Tag = $ReleaseApi.tag_name
} catch {
    $Tag = "v1.0.1"
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

# Add InstallDir to User PATH environment variable if not already present
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    $env:PATH = "$env:PATH;$InstallDir"
    Write-Host "🔗 Added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "`n✨ Ferronote $Tag successfully installed to $InstallDir\ferronote.exe!" -ForegroundColor Green
Write-Host "Open a new terminal window and run 'ferronote' to start taking notes." -ForegroundColor White
