@echo off
setlocal enabledelayedexpansion

REM Start frontend server

echo.
echo ============================================
echo   VoIP Application - Frontend Server
echo ============================================
echo.

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0
cd /d "!SCRIPT_DIR!frontend"

if errorlevel 1 (
    echo ERROR: Could not navigate to frontend directory
    pause
    exit /b 1
)

echo Starting frontend on http://localhost:3000
echo Close this window to stop the server.
echo.

python -m http.server 3000

if errorlevel 1 (
    echo.
    echo Error: Python not found!
    echo Please ensure Python is installed and in PATH.
    pause
    exit /b 1
)

endlocal
