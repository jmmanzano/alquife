@echo off
setlocal EnableExtensions EnableDelayedExpansion

echo Alquife Windows uninstaller
echo =============================
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\Alquife"
set "SHORTCUT_NAME=Alquife.lnk"
set "START_MENU_DIR=%APPDATA%\Microsoft\Windows\Start Menu\Programs"
set "DESKTOP_DIR=%USERPROFILE%\Desktop"

REM Check if installation directory exists
if not exist "%INSTALL_DIR%" (
  echo Alquife is not installed at "%INSTALL_DIR%".
  echo Nothing to uninstall.
  exit /b 0
)

echo.
echo This will uninstall Alquife from "%INSTALL_DIR%"
echo.
set /p "CONFIRM=Are you sure you want to continue? (Y/N): "
if /i not "%CONFIRM%"=="Y" (
  echo Uninstall cancelled.
  exit /b 0
)

echo.
echo Removing shortcuts...
if exist "%START_MENU_DIR%\%SHORTCUT_NAME%" (
  del /F /Q "%START_MENU_DIR%\%SHORTCUT_NAME%"
  echo Removed Start Menu shortcut.
)

if exist "%DESKTOP_DIR%\%SHORTCUT_NAME%" (
  del /F /Q "%DESKTOP_DIR%\%SHORTCUT_NAME%"
  echo Removed Desktop shortcut.
)

echo.
echo Removing installation directory...
if exist "%INSTALL_DIR%" (
  rmdir /S /Q "%INSTALL_DIR%"
  echo Installation directory removed.
)

echo.
echo Removing from PATH...
call :RemoveFromPath "%INSTALL_DIR%"

echo.
set /p "REMOVE_CONFIG=Do you want to remove configuration files and presets? (Y/N): "
if /i "%REMOVE_CONFIG%"=="Y" (
  if exist "%INSTALL_DIR%" (
    rmdir /S /Q "%INSTALL_DIR%" >nul 2>&1
  )
  echo Configuration files were kept at "%INSTALL_DIR%".
  echo You can manually delete this folder if desired.
) else (
  echo Configuration files preserved at "%INSTALL_DIR%".
)

echo.
echo Uninstallation complete.
exit /b 0

:RemoveFromPath
set "PATH_ENTRY=%~1"
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$target = [System.IO.Path]::GetFullPath($env:PATH_ENTRY);" ^
  "$current = [Environment]::GetEnvironmentVariable('Path', 'User');" ^
  "if ([string]::IsNullOrWhiteSpace($current)) { $parts = @() } else { $parts = $current -split ';' | Where-Object { $_ -and $_.Trim() -ne '' } }" ^
  "$newParts = @($parts | Where-Object { $_.TrimEnd('\\') -ine $target.TrimEnd('\\') });" ^
  "if ($newParts.Count -lt $parts.Count) { [Environment]::SetEnvironmentVariable('Path', ($newParts -join ';'), 'User'); Write-Host ('Removed from user PATH: ' + $target) } else { Write-Host ('Not found in user PATH: ' + $target) }"
exit /b 0
