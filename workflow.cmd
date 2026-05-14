@echo off
setlocal

set "WORKFLOW_SCRIPT=%~dp0tools\workflow\gate.py"

where py >nul 2>nul
if not errorlevel 1 goto run_with_py

where python3 >nul 2>nul
if not errorlevel 1 goto run_with_python3

where python >nul 2>nul
if not errorlevel 1 goto run_with_python

echo Python 3 was not found. Install Python 3 or add it to PATH.
exit /b 1

:run_with_py
py -3 "%WORKFLOW_SCRIPT%" %*
exit /b %ERRORLEVEL%

:run_with_python3
python3 "%WORKFLOW_SCRIPT%" %*
exit /b %ERRORLEVEL%

:run_with_python
python "%WORKFLOW_SCRIPT%" %*
exit /b %ERRORLEVEL%
