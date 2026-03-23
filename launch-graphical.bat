@echo off
title CHAOS RPG — Graphical
color 0A

set REPO_DIR=%~dp0
set CARGO_TARGET_DIR=C:\cargo-target
set CARGO_HOME=C:\cargo-home
set TMP=C:\cargo-tmp
set TEMP=C:\cargo-tmp

cd /d "%REPO_DIR%"

echo [1/3] Checking for updates...
git pull --ff-only 2>nul

echo [2/3] Building latest graphical version...
cargo build --release -p chaos-rpg-graphical 2>&1 | findstr /C:"Compiling" /C:"Finished" /C:"error"
if %ERRORLEVEL% NEQ 0 (
    echo Build failed. Launching existing build.
    goto LAUNCH
)

echo [3/3] Updating dist...
copy /Y "%CARGO_TARGET_DIR%\release\chaos-rpg-graphical.exe" "%REPO_DIR%dist\chaos-rpg-graphical.exe" >nul

:LAUNCH
echo Launching...
start "" "%REPO_DIR%dist\chaos-rpg-graphical.exe"
exit
