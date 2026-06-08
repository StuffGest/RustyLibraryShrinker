@echo off
chcp 65001 > nul

echo Lancement de la compilation globale...
echo.

:: Exécution du script PowerShell dans le dossier courant
powershell -ExecutionPolicy Bypass -File "%~dp0build_all.ps1"

echo.
echo Processus terminé.
echo.

pause