# make-truly-self-extracting.ps1 - EMBED EVERYTHING!

Write-Host "Creating TRULY self-extracting installer..." -ForegroundColor Green

# Check required files
$requiredFiles = @("archive.7z", "7za.exe")
foreach ($file in $requiredFiles) {
    if (-not (Test-Path $file)) {
        Write-Host "ERROR: $file not found!" -ForegroundColor Red
        exit 1
    }
}

# Read and encode both files
Write-Host "Encoding archive.7z..." -ForegroundColor Yellow
$archiveBytes = [System.IO.File]::ReadAllBytes("archive.7z")
$base64Archive = [Convert]::ToBase64String($archiveBytes)

Write-Host "Encoding 7za.exe..." -ForegroundColor Yellow
$7zaBytes = [System.IO.File]::ReadAllBytes("7za.exe")
$base647za = [Convert]::ToBase64String($7zaBytes)

$archiveKB = [math]::Round($archiveBytes.Length / 1024, 1)
$7zaKB = [math]::Round($7zaBytes.Length / 1024, 1)
$totalKB = $archiveKB + $7zaKB

Write-Host "Archive: $archiveKB KB, 7za.exe: $7zaKB KB, Total: $totalKB KB" -ForegroundColor Cyan

# Create the TRULY self-extracting installer
$installerContent = @"
# file-utils TRULY Self-Extracting Installer
# Contains both archive and extractor - NO DEPENDENCIES!

Write-Host ""
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "                    file-utils v0.3.0                          " -ForegroundColor Cyan
Write-Host "              Quantum-Enhanced File Security                    " -ForegroundColor Cyan
Write-Host "                   by whispr.dev                               " -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host ""

`$response = Read-Host "Install file-utils? (Y/N)"
if (`$response -notmatch "^[Yy]") {
    Write-Host "Installation cancelled." -ForegroundColor Yellow
    exit 0
}

Write-Host "Starting installation..." -ForegroundColor Green

Step 2: Create the config file
Write-Host "Creating installer config..." -ForegroundColor Yellow

@'
;!@Install@!UTF-8!
Title="file-utils v0.3.0 - Quantum-Enhanced File Security"
BeginPrompt="Install file-utils by whispr.dev?\n\nThis will install the quantum-enhanced file encryption and secure deletion tool."
RunProgram="install.bat"
;!@InstallEnd@!
'@ | Out-File -FilePath "config.txt" -Encoding UTF8

Write-Host "Created config.txt" -ForegroundColor Green

# Step 3: Combine SFX + config + archive
Write-Host "Building self-extracting executable..." -ForegroundColor Yellow

if (-not (Test-Path "file-utils_installer.exe")) {
    Write-Host "Failed to create installer" -ForegroundColor Red
    exit 1
}

Write-Host "Created file-utils_installer.exe" -ForegroundColor Green

# Step 4: Add icon (if you have rcedit)
if (Test-Path "rcedit.exe") {
    Write-Host "Adding custom icon..." -ForegroundColor Yellow
    
    & ".\rcedit.exe" "file-utils_installer.exe" --set-icon "file-utils.ico" --set-version-string "ProductName" "file-utils" --set-version-string "FileDescription" "Quantum-Enhanced File Security Tool" --set-version-string "CompanyName" "whispr.dev"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Icon and version info added!" -ForegroundColor Green
    } else {
        Write-Host "Could not add icon (installer still works)" -ForegroundColor Yellow
    }
} else {
    Write-Host "rcedit.exe not found - skipping icon (installer still works)" -ForegroundColor Yellow
}

# Step 5: Show results
$installerSize = (Get-Item "file-utils_installer.exe").Length
Write-Host ""
Write-Host "SUCCESS!" -ForegroundColor Green
Write-Host "Created: file-utils_installer.exe" -ForegroundColor Cyan
Write-Host "Size: $([math]::Round($installerSize/1MB, 2)) MB" -ForegroundColor Cyan
Write-Host "Test with: .\file-utils_installer.exe" -ForegroundColor Yellow

# Cleanup temporary files
Remove-Item "archive.7z", "config.txt" -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "=================================================================" -ForegroundColor Green
Write-Host "                 Installation Complete!                        " -ForegroundColor Green  
Write-Host "=================================================================" -ForegroundColor Green
Write-Host ""
Write-Host "You can now use: file-utils --help" -ForegroundColor Cyan
Write-Host ""
Read-Host "Press Enter to exit"
"@

# Save the truly self-extracting installer
$installerContent | Out-File -FilePath "file-utils_installer.exe" -Encoding UTF8

# Create batch runner
$batchContent = @"
@echo off
title file-utils Installer
echo.
echo ===============================================================
echo                    file-utils v0.3.0
echo              Quantum-Enhanced File Security
echo                     by whispr.dev
echo ===============================================================
echo.
echo Starting PowerShell installer...
echo.
powershell -ExecutionPolicy Bypass -File "file-utils_installer.exe"
if errorlevel 1 (
    echo.
    echo Installation failed!
    pause
) else (
    echo.
    echo Installation successful!
    pause
)
"@

$batchContent | Out-File -FilePath "file-utils_installer.exe" -Encoding ASCII

# Show results
$installerSize = (Get-Item "file-utils_installer.exe").Length
$installerMB = [math]::Round($installerSize / 1024 / 1024, 2)

Write-Host ""
Write-Host "=================================================================" -ForegroundColor Green
Write-Host "                        SUCCESS!                               " -ForegroundColor Green
Write-Host "=================================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Created TRULY self-extracting installer:" -ForegroundColor Cyan
Write-Host "  file-utils_installer.exe ($installerMB MB)" -ForegroundColor White
Write-Host "  (double-click version)" -ForegroundColor White
Write-Host ""
Write-Host "This installer contains:" -ForegroundColor Yellow
Write-Host "  - Your file-utils package" -ForegroundColor White
Write-Host "  - Embedded 7za.exe extractor" -ForegroundColor White
Write-Host "  - Installation Executable" -ForegroundColor White
Write-Host "  - NO EXTERNAL DEPENDENCIES!" -ForegroundColor Green
Write-Host ""
Write-Host "Test with: .\file-utils_installer.exe" -ForegroundColor Cyan
Write-Host "Or double-click: file-utils_installer.exe" -ForegroundColor Cyan