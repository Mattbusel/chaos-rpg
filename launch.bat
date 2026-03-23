@echo off
title CHAOS RPG Launcher
color 0A

echo ============================================
echo   CHAOS RPG - Auto-Update Launcher
echo ============================================
echo.

set REPO_DIR=%~dp0
set CARGO_TARGET_DIR=C:\cargo-target
set CARGO_HOME=C:\cargo-home
set TMP=C:\cargo-tmp
set TEMP=C:\cargo-tmp

cd /d "%REPO_DIR%"

echo [1/3] Checking for updates...
git pull --ff-only
if %ERRORLEVEL% NEQ 0 (
    echo WARNING: Could not pull updates. Launching existing build.
    goto LAUNCH
)

echo.
echo [2/3] Building latest version...
cargo build --release -p chaos-rpg-graphical 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Build failed. Check output above.
    pause
    exit /b 1
)

echo.
echo [3/3] Copying to dist...
copy /Y "%CARGO_TARGET_DIR%\release\chaos-rpg-graphical.exe" "%REPO_DIR%dist\chaos-rpg-graphical.exe" >nul

:LAUNCH
echo.
echo Launching CHAOS RPG (Graphical)...
start "" "%REPO_DIR%dist\chaos-rpg-graphical.exe" %*
exit
