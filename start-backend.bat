@echo off
setlocal enabledelayedexpansion

REM Quick start script for VoIP Application on Windows

echo.
echo ============================================
echo   VoIP Application - Quick Start
echo ============================================
echo.

echo Checking for Rust installation...
rustc --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Rust is not installed!
    echo Please download from: https://rustup.rs/
    pause
    exit /b 1
)

echo Rust found!
echo.
echo Starting backend server...
echo.

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0
cd /d "!SCRIPT_DIR!backend"

if errorlevel 1 (
    echo ERROR: Could not navigate to backend directory
    pause
    exit /b 1
)

echo Building and running backend...
cargo run --release

pause
endlocal
