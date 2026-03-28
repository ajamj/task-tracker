# 🔐 Stitch Setup Guide

## Quick Start (3 Steps)

### 1. Get Your Stitch API Key

1. Go to https://stitch.google.com/
2. Login with your Google account
3. Click **Profile Icon** → **Stitch Settings**
4. Go to **API Keys** tab
5. Click **Create Key**
6. **Copy** the API key (starts with `AIza...`)

### 2. Set API Key Environment Variable

**Option A: Temporary (Current Session Only)**
```bash
# Windows PowerShell
$env:API_KEY="AIza-your-api-key-here"

# Windows CMD
set API_KEY=AIza-your-api-key-here
```

**Option B: Permanent (Recommended)**
```bash
# Windows - Run once
setx API_KEY "AIza-your-api-key-here"

# Then restart your terminal
```

### 3. Run Setup Script

```bash
# Navigate to project directory
cd D:\GRC-Ajam\rust-playground

# Run setup script
setup-stitch.bat
```

The script will:
- ✅ Check if API_KEY is set
- ✅ Create configuration file
- ✅ Configure Stitch extension

---

## Manual Setup (Alternative)

If you prefer to setup manually:

### Step 1: Copy Configuration Template

```bash
# Navigate to Stitch extension directory
cd %USERPROFILE%\.gemini\extensions\Stitch

# Copy API key template
copy gemini-extension-apikey.json gemini-extension.json
```

### Step 2: Edit Configuration

Open `%USERPROFILE%\.gemini\extensions\Stitch\gemini-extension.json` in a text editor and replace `YOUR_API_KEY` with your actual API key:

```json
{
    "name": "Stitch",
    "version": "0.1.4",
    "mcpServers": {
        "stitch": {
            "httpUrl": "https://stitch.googleapis.com/mcp",
            "headers": {
                "X-Goog-Api-Key": "AIza-your-actual-api-key-here"
            },
            "timeout": 300000
        }
    }
}
```

---

## Verify Installation

### 1. Check Extension is Installed

```bash
gemini extensions list
```

You should see:
```
✅ Stitch (0.1.4) - Integrate Stitch into your workflow
```

### 2. Test Stitch Connection

```bash
gemini
/stitch List my projects
```

If successful, you'll see your Stitch projects listed.

---

## Usage Examples

Once configured, you can use Stitch commands:

### List Projects
```
/stitch What Stitch projects do I have?
```

### Get Project Details
```
/stitch Tell me details about project 3677573127824787033
```

### List Screens
```
/stitch Give me all the screens of project 3677573127824787033
```

### Download Assets
```
/stitch Download the image of screen 6393b8177be0490f89eb8f2c1e4cfb37
```

### Generate New Screen
```
/stitch Design a mobile app for people who love skiing in the Alps
```

### Enhance Prompt
```
/stitch Enhance: "Design a landing page for a podcast about AI"
```

---

## Troubleshooting

### Error: "API keys are not supported"

**Cause:** Stitch requires OAuth2 or API key authentication.

**Solution:**
1. Make sure API_KEY environment variable is set
2. Verify gemini-extension.json has correct API key
3. Restart Gemini CLI

### Error: "Extension not found"

**Solution:**
```bash
# Reinstall extension
gemini extensions uninstall Stitch
gemini extensions install https://github.com/gemini-cli-extensions/stitch --auto-update
```

### Error: "Invalid API key"

**Solution:**
1. Check API key is correct (starts with `AIza...`)
2. Make sure there are no extra spaces in the key
3. Verify API key hasn't expired

---

## Alternative: ADC Authentication (Enterprise)

For enterprise use with Google Cloud:

```bash
# 1. Login to gcloud
gcloud auth login

# 2. Set project
export PROJECT_ID="your-project-id"
gcloud config set project $PROJECT_ID

# 3. Enable Stitch MCP API
gcloud beta services mcp enable stitch.googleapis.com --project=$PROJECT_ID

# 4. Setup ADC
gcloud auth application-default login

# 5. Configure extension
cd %USERPROFILE%\.gemini\extensions\Stitch
copy gemini-extension-adc.json gemini-extension.json
```

---

## Resources

| Resource | Link |
|----------|------|
| Stitch Web App | https://stitch.google.com/ |
| GitHub Repository | https://github.com/gemini-cli-extensions/stitch |
| Documentation | https://geminicli.com/extensions/ |
| Report Issues | https://github.com/gemini-cli-extensions/stitch/issues |

---

## Pricing

✅ **Stitch MCP is FREE of charge**

---

**Last Updated:** 2026-03-28
