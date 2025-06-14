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
