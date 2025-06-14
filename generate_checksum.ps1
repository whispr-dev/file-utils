 generate-checksums.ps1 - Create professional checksums file

Write-Host "Generating checksums for file-utils distribution..." -ForegroundColor Green

# Files to checksum
$files = @(
    "file-utils_installer.exe",
    "README",
    "LICENSE"
)

# Output file
$checksumFile = "checksums.txt"

# Header with metadata
$output = @"
file-utils v0.3.0 - Checksums
=============================
Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss UTC")
Package: Quantum-Enhanced File Security Tool
Publisher: whispr.dev

Verification Instructions:
- Windows PowerShell: Get-FileHash filename.exe
- Windows Command: certutil -hashfile filename.exe SHA256
- Linux/macOS: sha256sum filename.exe

SHA256 Checksums:
"@

Write-Host "Calculating checksums..." -ForegroundColor Yellow

foreach ($file in $files) {
    if (Test-Path $file) {
        Write-Host "  Processing: $file" -ForegroundColor Gray
        
        $hash = Get-FileHash $file -Algorithm SHA256
        $size = [math]::Round((Get-Item $file).Length / 1024 / 1024, 2)
        
        $output += "`n$($hash.Hash.ToLower())  $file  ($size MB)"
        
        Write-Host "    SHA256: $($hash.Hash.Substring(0,16))..." -ForegroundColor Green
    } else {
        Write-Host "  WARNING: $file not found" -ForegroundColor Red
    }
}

# Add additional hash algorithms for extra security
$output += "`n`nMD5 Checksums (legacy compatibility):"

foreach ($file in $files) {
    if (Test-Path $file) {
        $md5 = Get-FileHash $file -Algorithm MD5
        $output += "`n$($md5.Hash.ToLower())  $file"
    }
}

# Save checksums file
$output | Out-File -FilePath $checksumFile -Encoding UTF8

Write-Host "`nChecksums saved to: $checksumFile" -ForegroundColor Green

# Display results
Write-Host "`nGenerated checksums:" -ForegroundColor Cyan
Get-Content $checksumFile | Where-Object { $_ -match "^[a-f0-9]{64}" } | ForEach-Object {
    $parts = $_ -split "  "
    Write-Host "  $($parts[1]): $($parts[0].Substring(0,16))..." -ForegroundColor White
}