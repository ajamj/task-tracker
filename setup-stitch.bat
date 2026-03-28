@echo off
echo ========================================
echo   Stitch Extension - API Key Setup
echo ========================================
echo.

REM Check if API_KEY environment variable is set
if "%API_KEY%"=="" (
    echo ERROR: API_KEY environment variable is not set!
    echo.
    echo Please set your API key first:
    echo   setx API_KEY "your-api-key-here"
    echo.
    echo Then run this script again.
    echo.
    pause
    exit /b 1
)

echo API_KEY found: %API_KEY:~0,10%...
echo.

REM Set the extension directory
set EXTENSION_DIR=%USERPROFILE%\.gemini\extensions\Stitch

echo Configuring Stitch extension...
echo.

REM Create the configuration file with the API key
(
echo {
echo     "name": "Stitch",
echo     "version": "0.1.4",
echo     "description": "Integrate Stitch into your workflow: Generate UI from Text, Image.",
echo     "mcpServers": {
echo         "stitch": {
echo             "httpUrl": "https://stitch.googleapis.com/mcp",
echo             "headers": {
echo                 "X-Goog-Api-Key": "%API_KEY%"
echo             },
echo             "timeout": 300000
echo         }
echo     }
echo }
) > "%EXTENSION_DIR%\gemini-extension.json"

echo.
echo ========================================
echo   Setup Complete!
echo ========================================
echo.
echo Stitch extension is now configured with your API key.
echo.
echo Next steps:
echo   1. Restart Gemini CLI if it's running
echo   2. Run: gemini
echo   3. Try: /stitch List my projects
echo.
echo For more info, visit: https://github.com/gemini-cli-extensions/stitch
echo.
pause
