# Fixed 7-Zip build script for PowerShell

Write-Host "Building file-utils installer..." -ForegroundColor Green

# Step 1: Create the 7z archive (FIXED COMMAND)
Write-Host "Creating 7z archive..." -ForegroundColor Yellow

Start-Process -FilePath ".\7za.exe" -ArgumentList "a", "-t7z", "archive.7z", "file-utils.exe", "README", "LICENSE", "docs\", "install.bat", "UNWISE.bat" -Wait -NoNewWindow

# if ($LASTEXITCODE -ne 0) {
#     Write-Host "Failed to create 7z archive" -ForegroundColor Red
#     exit 1
# }

Write-Host "Created archive.7z" -ForegroundColor Green

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

# Use cmd for the binary copy operation (this works reliably)
cmd /c 'copy /b "C:\7z-extra\7za.exe" + "config.txt" + "archive.7z" "file-utils_installer.exe"'

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
