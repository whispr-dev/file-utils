# Fixed 7-Zip build script for PowerShell

Write-Host "Building file-utils installer..." -ForegroundColor Green

# Step 1: Create the 7z archive (FIXED COMMAND)
Write-Host "Creating 7z archive..." -ForegroundColor Yellow

Start-Process -FilePath ".\7za.exe" -ArgumentList "a", "-t7z", "archive.7z", "file-utils.exe", "README", "LICENSE", "docs\", "install.bat", "UNWISE.bat" -Wait -NoNewWindow

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to create 7z archive" -ForegroundColor Red
    exit 1
}

Write-Host "Created archive.7z" -ForegroundColor Green
