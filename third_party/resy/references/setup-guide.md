# Resy Setup Guide

## Getting Your API Credentials

To use the Resy booking skill, you need to extract your API key and authentication token from the Resy website.

### Step-by-Step Instructions

#### 1. Log into Resy

1. Open your web browser (Chrome, Firefox, Safari, Edge)
2. Go to https://resy.com/
3. Click "Sign In" in the top right
4. Enter your credentials and log in
5. **Important:** Stay logged in for the next steps

#### 2. Open Developer Tools

**Chrome/Edge:**
- Press `F12` or `Ctrl+Shift+I` (Windows/Linux)
- Press `Cmd+Option+I` (Mac)

**Firefox:**
- Press `F12` or `Ctrl+Shift+I` (Windows/Linux)
- Press `Cmd+Option+I` (Mac)

**Safari:**
- First enable Developer Tools: Safari → Preferences → Advanced → Show Develop menu
- Then press `Cmd+Option+I`

#### 3. Navigate to Network Tab

1. In Developer Tools, click the "Network" tab
2. Make sure recording is enabled (red circle or "Recording" button)
3. Look for a filter box and clear any existing filters

#### 4. Visit a Restaurant Page

1. In Resy, search for any restaurant (e.g., "Nobu")
2. Click on a restaurant to view its page
3. The page will load and make API calls

#### 5. Find the API Call

1. In the Network tab, look for requests to `api.resy.com`
2. You might see endpoints like:
   - `find`
   - `search`
   - `details`
3. Click on any request to `api.resy.com`

#### 6. Extract Your Credentials

In the request details panel:

1. Look for "Request Headers" section
2. Find and copy these values:

**API Key:**
```
Authorization: ResyAPI api_key="YOUR_API_KEY_HERE"
```
Copy the value inside the quotes (without the quotes)

**Auth Token:**
```
x-resy-auth-token: YOUR_AUTH_TOKEN_HERE
```
Copy the entire value

#### 7. Set Environment Variables

Add these to your shell profile:

**Bash/Zsh (~/.bashrc or ~/.zshrc):**
```bash
export RESY_API_KEY="your_api_key_here"
export RESY_AUTH_TOKEN="your_auth_token_here"
```

**Fish (~/.config/fish/config.fish):**
```fish
set -x RESY_API_KEY "your_api_key_here"
set -x RESY_AUTH_TOKEN "your_auth_token_here"
```

Then reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

Or set temporarily for current session:
```bash
export RESY_API_KEY="your_api_key_here"
export RESY_AUTH_TOKEN="your_auth_token_here"
```

#### 8. Verify Setup

Test your credentials:
```bash
python3 ~/.openclaw/workspace/skills/resy-booking/scripts/list_reservations.py
```

If you see your reservations (or "No reservations found"), you're set up correctly!

## Finding Venue IDs

To book at a specific restaurant, you need its venue ID.

### Method 1: Using the Search Script
```bash
python3 ~/.openclaw/workspace/skills/resy-booking/scripts/search.py --query "Restaurant Name"
```

### Method 2: From Browser
1. Visit the restaurant's Resy page
2. Open Developer Tools → Network tab
3. Look for the `find` API call
4. Check the URL parameters - `venue_id` will be listed

### Method 3: From URL
Some restaurant URLs contain the venue ID:
```
https://resy.com/cities/new-york-ny/venues/don-angie
```
The slug "don-angie" can sometimes be used, but the numeric ID is more reliable.

## Tips

### Token Expiration
- Auth tokens may expire after some time
- If you get "Unauthorized" errors, re-extract your token
- Tokens typically last several days to weeks

### Multiple Devices
- You can use the same credentials on multiple devices
- However, logging out on one device may invalidate tokens on others

### Security
- Never share your API key or auth token
- Don't commit them to version control
- Use environment variables or a secrets manager

## Troubleshooting

### "Cannot find api.resy.com requests"
- Make sure you're on a restaurant page
- Try refreshing the page with Network tab open
- Clear filters in the Network tab

### "Token looks different"
- Tokens can be long strings of letters, numbers, and symbols
- Both API key and auth token should be 50+ characters
- Make sure you're copying the full value

### "Environment variables not working"
- Check that you reloaded your shell config
- Verify with: `echo $RESY_API_KEY`
- Try setting them directly in terminal for testing

## Alternative: Using Browser Cookies

If header extraction doesn't work, you can also find tokens in cookies:

1. Open Developer Tools → Application (or Storage) tab
2. Go to Cookies → https://resy.com
3. Look for `auth_token` cookie

However, header extraction is more reliable and recommended.
