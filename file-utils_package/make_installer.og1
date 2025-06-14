
#  make install script, da?

####################

####################
#  step 1. make archive
####################

# Fixed 7-Zip build script for PowerShell

Write-Host "Building file-utils installer..." -ForegroundColor Green

# Step 1: Create the 7z archive (FIXED COMMAND)
Write-Host "Creating 7z archive..." -ForegroundColor Yellow

Start-Process -FilePath ".\7za.exe" -ArgumentList "a", "-t7z", "archive.7z", "file-utils.exe", "README", "LICENSE", "docs\", "install.bat", "UNWISE.bat" -Wait -NoNewWindow

#  if ($LASTEXITCODE -ne 0) {
#      Write-Host "Failed to create 7z archive" -ForegroundColor Red
#      exit 1
#  }

Write-Host "Created archive.7z" -ForegroundColor Green

#####################
#  step 2. make ps1 script
#####################

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

# Create temp directory
`$tempDir = "`$env:TEMP\file-utils_install-`$(Get-Random)"
New-Item -ItemType Directory -Path `$tempDir -Force | Out-Null
Write-Host "Temp directory: `$tempDir" -ForegroundColor Gray

try {
    Write-Host "Extracting embedded 7za.exe..." -ForegroundColor Yellow
    
    # Extract embedded 7za.exe
    `$7zaData = "$base647za"
    `$7zaBytes = [Convert]::FromBase64String(`$7zaData)
    `$7zaPath = "`$tempDir\7za.exe"
    [System.IO.File]::WriteAllBytes(`$7zaPath, `$7zaBytes)
    
    Write-Host "Extracting embedded archive..." -ForegroundColor Yellow
    
    # Extract embedded archive
    `$archiveData = "$base64Archive"
    `$archiveBytes = [Convert]::FromBase64String(`$archiveData)
    `$archivePath = "`$tempDir\package.7z"
    [System.IO.File]::WriteAllBytes(`$archivePath, `$archiveBytes)
    
    Write-Host "Extracting package contents..." -ForegroundColor Yellow
    
    # Extract the package using our embedded 7za
    Push-Location `$tempDir
    & ".\7za.exe" x "package.7z" -y
    `$extractResult = `$LASTEXITCODE
    Pop-Location
    
    if (`$extractResult -ne 0) {
        Write-Host "ERROR: Failed to extract package" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Package extracted successfully!" -ForegroundColor Green
    
    # Run the installer
    `$installScript = "`$tempDir\install.bat"
    if (Test-Path `$installScript) {
        Write-Host "Running installation script..." -ForegroundColor Yellow
        Push-Location `$tempDir
        & ".\install.bat"
        `$installResult = `$LASTEXITCODE
        Pop-Location
        
        if (`$installResult -eq 0) {
            Write-Host "Installation completed successfully!" -ForegroundColor Green
        } else {
            Write-Host "Installation completed with warnings" -ForegroundColor Yellow
        }
    } else {
        Write-Host "WARNING: install.bat not found in package" -ForegroundColor Yellow
        Write-Host "Available files:" -ForegroundColor Gray
        Get-ChildItem `$tempDir | Where-Object { `$_.Name -ne "7za.exe" -and `$_.Name -ne "package.7z" } | ForEach-Object { Write-Host "  - `$(`$_.Name)" -ForegroundColor White }
    }
    
} catch {
    Write-Host "Installation failed: `$(`$_.Exception.Message)" -ForegroundColor Red
    Write-Host "Stack trace: `$(`$_.ScriptStackTrace)" -ForegroundColor Gray
    exit 1
} finally {
    # Clean up temp directory
    Write-Host "Cleaning up temporary files..." -ForegroundColor Gray
    if (Test-Path `$tempDir) {
        Remove-Item `$tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

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
$installerContent | Out-File -FilePath "file-utils_installer.ps1" -Encoding UTF8

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
powershell -ExecutionPolicy Bypass -File "file-utils_installer.ps1"
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

$batchContent | Out-File -FilePath "file-utils_installer.bat" -Encoding ASCII

# Show results
$installerSize = (Get-Item "file-utils_installer.ps1").Length
$installerMB = [math]::Round($installerSize / 1024 / 1024, 2)

Write-Host ""
Write-Host "=================================================================" -ForegroundColor Green
Write-Host "                        SUCCESS!                               " -ForegroundColor Green
Write-Host "=================================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Created TRULY self-extracting installer:" -ForegroundColor Cyan
Write-Host "  file-utils_installer.ps1 ($installerMB MB)" -ForegroundColor White
Write-Host "  file-utils_installer.bat (double-click version)" -ForegroundColor White
Write-Host ""
Write-Host "This installer contains:" -ForegroundColor Yellow
Write-Host "  - Your file-utils package" -ForegroundColor White
Write-Host "  - Embedded 7za.exe extractor" -ForegroundColor White
Write-Host "  - Installation script" -ForegroundColor White
Write-Host "  - NO EXTERNAL DEPENDENCIES!" -ForegroundColor Green
Write-Host ""
Write-Host "Test with: .\file-utils_installer.ps1" -ForegroundColor Cyan
Write-Host "Or double-click: file-utils_installer.bat" -ForegroundColor Cyan


######################
#  step 3. .ps1 -> .exe
######################

# Install ps2exe
Install-Module ps2exe -Scope CurrentUser -Force

# Convert to EXE (one line)
ps2exe -inputFile "file-utils_installer.ps1" -outputFile "file-utils_installer.exe" -iconFile "file-utils.ico" -title "file-utils Installer" -description "Quantum-Enhanced File Security Tool Installer" -company "whispr.dev" -version "0.3.0" -copyright "Copyright © 2024 whispr.dev" -product "file-utils" -noConsole -requireAdmin

# Test it
.\file-utils-installer.exe


######################
#  done! congratz.
######################
