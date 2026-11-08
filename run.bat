@echo off
rem builds the game and launcher, extra arguments go to beta173_launcher
setlocal
cd /d "%~dp0"

rem dev checkouts reuse the local jar and sounds instead of downloading them
set "FILES=%~dp0..\b1.7.3_files"
if not defined B173_CLIENT_JAR if exist "%FILES%\minecraft-b1.7.3-client.jar" set "B173_CLIENT_JAR=%FILES%\minecraft-b1.7.3-client.jar"
if not defined B173_SOUNDS_DIR if exist "%FILES%\InventivetalentDev-minecraft-assets-737829f\assets\" set "B173_SOUNDS_DIR=%FILES%\InventivetalentDev-minecraft-assets-737829f\assets"

cargo build --release --bin beta173 --bin beta173_launcher || exit /b 1
target\release\beta173_launcher.exe %*
exit /b %errorlevel%
