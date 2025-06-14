# make_installer_DEBUG.ps1 - Shows what's happening!

param(
    [switch]$Verbose,
    [switch]$Debug
)

# Enable verbose output if requested
if ($Verbose) {
    $VerbosePreference = 'Continue'
}

if ($Debug) {
    $DebugPreference = 'Continue'
    $VerbosePreference = 'Continue'
}

# ALWAYS show basic progress
$ErrorActionPreference = 'Continue'  # Don't stop on errors, show them

Write-Host "=== make_installer_DEBUG.ps1 Starting ===" -ForegroundColor Cyan
Write-Host "Current directory: $(Get-Location)" -ForegroundColor Gray
Write-Host "PowerShell version: $($PSVersionTable.PSVersion)" -ForegroundColor Gray
Write-Host ""

####################
# STEP 0: Check Prerequisites
####################

Write-Host "STEP 0: Checking prerequisites..." -ForegroundColor Yellow

$requiredFiles = @(
    "7za.exe",
    "file-utils.exe", 
    "install.bat",
    "UNWISE.bat",
    "README",
    "LICENSE"
)

Write-Host "Required files check:" -ForegroundColor Gray
$missingFiles = @()
foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        $size = [math]::Round((Get-Item $file).Length / 1024, 1)
        Write-Host "  ✅ Found: $file ($size KB)" -ForegroundColor Green
    } else {
        Write-Host "  ❌ Missing: $file" -ForegroundColor Red
        $missingFiles += $file
    }
}

if ($missingFiles.Count -gt 0) {
    Write-Host ""
    Write-Host "ERROR: Missing required files:" -ForegroundColor Red
    $missingFiles | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    Write-Host ""
    Write-Host "Current directory contents:" -ForegroundColor Yellow
    Get-ChildItem | ForEach-Object { Write-Host "  $($_.Name)" -ForegroundColor White }
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host "✅ All required files found!" -ForegroundColor Green

####################
# STEP 1: Create Archive
####################

Write-Host ""
Write-Host "STEP 1: Creating 7z archive..." -ForegroundColor Yellow

# Clean up old archive
if (Test-Path "archive.7z") {
    Write-Host "Removing existing archive.7z..." -ForegroundColor Gray
    Remove-Item "archive.7z" -Force
}

try {
    Write-Host "Running 7za.exe command..." -ForegroundColor Gray
    Write-Host "Command: .\7za.exe a -t7z archive.7z file-utils.exe README LICENSE docs\ install.bat UNWISE.bat" -ForegroundColor Gray
    
    $result = Start-Process -FilePath ".\7za.exe" -ArgumentList "a", "-t7z", "archive.7z", "file-utils.exe", "README", "LICENSE", "docs\", "install.bat", "UNWISE.bat" -Wait -NoNewWindow -PassThru
    
    Write-Host "7za.exe exit code: $($result.ExitCode)" -ForegroundColor Gray
    
    if ($result.ExitCode -eq 0) {
        if (Test-Path "archive.7z") {
            $archiveSize = [math]::Round((Get-Item "archive.7z").Length / 1024, 1)
            Write-Host "✅ Created archive.7z ($archiveSize KB)" -ForegroundColor Green
        } else {
            throw "archive.7z was not created despite exit code 0"
        }
    } else {
        throw "7za.exe failed with exit code $($result.ExitCode)"
    }
} catch {
    Write-Host "❌ Failed to create archive: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Debugging info:" -ForegroundColor Yellow
    Write-Host "  Working directory: $(Get-Location)"
    Write-Host "  7za.exe exists: $(Test-Path '.\7za.exe')"
    if (Test-Path ".\7za.exe") {
        Write-Host "  7za.exe size: $([math]::Round((Get-Item '.\7za.exe').Length / 1024, 1)) KB"
    }
    Read-Host "Press Enter to exit"
    exit 1
}

####################
# STEP 2: Check/Install ps2exe
####################

Write-Host ""
Write-Host "STEP 2: Checking ps2exe..." -ForegroundColor Yellow

try {
    $ps2exeModule = Get-Module -ListAvailable -Name ps2exe
    if ($ps2exeModule) {
        Write-Host "✅ ps2exe already available (version: $($ps2exeModule.Version))" -ForegroundColor Green
    } else {
        Write-Host "Installing ps2exe module..." -ForegroundColor Yellow
        
        Write-Host "  Installing NuGet provider..." -ForegroundColor Gray
        Install-PackageProvider -Name NuGet -MinimumVersion 2.8.5.201 -Force -Scope CurrentUser | Out-Null
        
        Write-Host "  Setting PSGallery as trusted..." -ForegroundColor Gray
        Set-PSRepository -Name 'PSGallery' -InstallationPolicy Trusted
        
        Write-Host "  Installing ps2exe module..." -ForegroundColor Gray
        Install-Module ps2exe -Scope CurrentUser -Force -AllowClobber
        
        Write-Host "✅ ps2exe installed successfully" -ForegroundColor Green
    }
} catch {
    Write-Host "❌ Failed to install ps2exe: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "  You can manually install with: Install-Module ps2exe -Scope CurrentUser -Force" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

####################
# STEP 3: Create PowerShell Installer
####################

Write-Host ""
Write-Host "STEP 3: Creating PowerShell installer..." -ForegroundColor Yellow

try {
    Write-Host "Reading archive.7z..." -ForegroundColor Gray
    $archiveBytes = [System.IO.File]::ReadAllBytes("archive.7z")
    $base64Archive = [Convert]::ToBase64String($archiveBytes)
    
    Write-Host "Reading 7za.exe..." -ForegroundColor Gray
    $7zaBytes = [System.IO.File]::ReadAllBytes("7za.exe")
    $base647za = [Convert]::ToBase64String($7zaBytes)
    
    $archiveKB = [math]::Round($archiveBytes.Length / 1024, 1)
    $7zaKB = [math]::Round($7zaBytes.Length / 1024, 1)
    $base64SizeKB = [math]::Round(($base64Archive.Length + $base647za.Length) / 1024, 1)
    
    Write-Host "  Archive: $archiveKB KB -> Base64: $([math]::Round($base64Archive.Length / 1024, 1)) KB" -ForegroundColor Gray
    Write-Host "  7za.exe: $7zaKB KB -> Base64: $([math]::Round($base647za.Length / 1024, 1)) KB" -ForegroundColor Gray
    Write-Host "  Total embedded data: $base64SizeKB KB" -ForegroundColor Cyan
    
    # Create installer content (use the balanced version from previous artifact)
    $installerContent = @"
# file-utils Self-Extracting Installer v0.3.0
# Generated by make_installer.ps1

# Your balanced installer content here...
# (This is where the full installer script would go)
Write-Host "This is a test installer script"
Write-Host "Archive size: $archiveKB KB"
Write-Host "7za.exe size: $7zaKB KB"
Read-Host "Press Enter to exit test installer"
"@

    Write-Host "Writing file-utils_installer.ps1..." -ForegroundColor Gray
    $installerContent | Out-File -FilePath "file-utils_installer.ps1" -Encoding UTF8
    
    if (Test-Path "file-utils_installer.ps1") {
        $ps1Size = [math]::Round((Get-Item "file-utils_installer.ps1").Length / 1024, 1)
        Write-Host "✅ Created file-utils_installer.ps1 ($ps1Size KB)" -ForegroundColor Green
    } else {
        throw "file-utils_installer.ps1 was not created"
    }
    
} catch {
    Write-Host "❌ Failed to create PowerShell installer: $($_.Exception.Message)" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

####################
# STEP 4: Convert to EXE
####################

Write-Host ""
Write-Host "STEP 4: Converting PowerShell to EXE..." -ForegroundColor Yellow

try {
    Write-Host "Importing ps2exe module..." -ForegroundColor Gray
    Import-Module ps2exe -Force
    
    # Check for icon
    $iconExists = Test-Path "file-utils.ico"
    Write-Host "Icon file (file-utils.ico) exists: $iconExists" -ForegroundColor Gray
    
    if ($iconExists) {
        $iconSize = [math]::Round((Get-Item "file-utils.ico").Length / 1024, 1)
        Write-Host "Icon size: $iconSize KB" -ForegroundColor Gray
    }
    
    # Clean up old EXE
    if (Test-Path "file-utils_installer.exe") {
        Write-Host "Removing existing EXE..." -ForegroundColor Gray
        Remove-Item "file-utils_installer.exe" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
    }
    
    Write-Host "Running ps2exe conversion..." -ForegroundColor Gray
    
    if ($iconExists) {
        Write-Host "ps2exe command: ps2exe -inputFile file-utils_installer.ps1 -outputFile file-utils_installer.exe -iconFile file-utils.ico -title 'file-utils Installer' ..." -ForegroundColor Gray
        
        ps2exe `
            -inputFile "file-utils_installer.ps1" `
            -outputFile "file-utils_installer.exe" `
            -iconFile "file-utils.ico" `
            -title "file-utils Installer" `
            -description "Quantum-Enhanced File Security Tool Installer" `
            -company "whispr.dev" `
            -version "0.3.0" `
            -copyright "Copyright 2024 whispr.dev" `
            -product "file-utils" `
            -noConsole `
            -requireAdmin `
            -verbose
    } else {
        Write-Host "ps2exe command: ps2exe -inputFile file-utils_installer.ps1 -outputFile file-utils_installer.exe -title 'file-utils Installer' ..." -ForegroundColor Gray
        
        ps2exe `
            -inputFile "file-utils_installer.ps1" `
            -outputFile "file-utils_installer.exe" `
            -title "file-utils Installer" `
            -description "Quantum-Enhanced File Security Tool Installer" `
            -company "whispr.dev" `
            -version "0.3.0" `
            -copyright "Copyright 2024 whispr.dev" `
            -product "file-utils" `
            -noConsole `
            -requireAdmin `
            -verbose
    }
    
    Start-Sleep -Seconds 2
    
    if (Test-Path "file-utils_installer.exe") {
        $exeSize = [math]::Round((Get-Item "file-utils_installer.exe").Length / 1024 / 1024, 2)
        Write-Host "✅ Created file-utils_installer.exe ($exeSize MB)" -ForegroundColor Green
    } else {
        Write-Host "⚠️  ps2exe completed but EXE not found" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "❌ ps2exe conversion failed: $($_.Exception.Message)" -ForegroundColor Red
    
    if (Test-Path "file-utils_installer.exe") {
        $exeSize = [math]::Round((Get-Item "file-utils_installer.exe").Length / 1024 / 1024, 2)
        Write-Host "BUT EXE was created anyway ($exeSize MB)" -ForegroundColor Green
    } else {
        Write-Host "Creating batch wrapper as fallback..." -ForegroundColor Yellow
        
        $batchWrapper = @"
@echo off
title file-utils Installer
cd /d "%~dp0"
echo Starting file-utils installer...
powershell -ExecutionPolicy Bypass -WindowStyle Normal -File "file-utils_installer.ps1"
pause
"@
        
        $batchWrapper | Out-File -FilePath "file-utils_installer.bat" -Encoding ASCII
        Write-Host "✅ Created batch wrapper: file-utils_installer.bat" -ForegroundColor Green
    }
}

####################
# FINAL RESULTS
####################

Write-Host ""
Write-Host "=== FINAL RESULTS ===" -ForegroundColor Cyan
Write-Host ""

$outputFiles = @("file-utils_installer.exe", "file-utils_installer.ps1", "file-utils_installer.bat", "archive.7z")
foreach ($file in $outputFiles) {
    if (Test-Path $file) {
        $size = Get-Item $file | ForEach-Object { 
            if ($_.Length -gt 1MB) { 
                "$([math]::Round($_.Length / 1MB, 2)) MB" 
            } else { 
                "$([math]::Round($_.Length / 1KB, 1)) KB" 
            }
        }
        Write-Host "✅ $file ($size)" -ForegroundColor Green
    } else {
        Write-Host "❌ $file (not found)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "Build process complete!" -ForegroundColor Green
Write-Host ""
Write-Host "To test: .\file-utils_installer.exe" -ForegroundColor Yellow