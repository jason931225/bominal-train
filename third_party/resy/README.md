# Resy Booking Skill for OpenClaw

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![Security](https://img.shields.io/badge/security-audited-brightgreen.svg)]()

A complete, secure restaurant reservation management system for Resy, built as an OpenClaw skill.

## 🍽️ Features

- **Search Restaurants** - Find restaurants by name, location, or cuisine
- **Check Availability** - View available time slots for any restaurant
- **Book Reservations** - Make reservations with confirmation prompts
- **List Reservations** - View all your upcoming (and past) reservations
- **Cancel Reservations** - Cancel existing bookings with safety confirmations
- **Secure by Design** - No credentials stored, environment-only auth, audit logging

## 🔒 Security

This skill is designed with security as a priority:

- ✅ **No hardcoded credentials** - All auth via environment variables
- ✅ **No credential storage** - Tokens never written to disk
- ✅ **Input validation** - Sanitizes all user inputs
- ✅ **Audit logging** - All API calls logged to stderr
- ✅ **URL validation** - Only connects to official Resy API (`api.resy.com`)
- ✅ **Sensitive data masking** - Tokens redacted in logs
- ✅ **Error message safety** - No credential exposure in errors

## 📋 Prerequisites

- Python 3.8+
- `requests` library (`pip install requests`)
- Resy account with valid credentials

## 🚀 Quick Start

### 1. Get Your Resy Credentials

See [Setup Guide](references/setup-guide.md) for detailed instructions.

Quick version:
1. Log into [Resy](https://resy.com/) in your browser
2. Open Developer Tools (F12) → Network tab
3. Visit any restaurant page
4. Extract from request headers:
   - `Authorization: ResyAPI api_key="YOUR_KEY"` → API key
   - `X-Resy-Auth-Token: YOUR_TOKEN` → Auth token

### 2. Set Environment Variables

```bash
export RESY_API_KEY="your_api_key"
export RESY_AUTH_TOKEN="your_auth_token"
```

### 3. Install

```bash
# Clone or copy skill to OpenClaw workspace
cp -r resy-booking ~/.openclaw/workspace/skills/

# Verify setup
python3 ~/.openclaw/workspace/skills/resy-booking/scripts/auth.py
```

## 📖 Usage

### Search for Restaurants

```bash
python3 scripts/search.py --query "Nobu"
python3 scripts/search.py --query "Italian" --location "New York"
```

### Check Availability

```bash
python3 scripts/availability.py --venue-id 1505 --date 2024-12-25 --party-size 2
```

### Book a Table

```bash
python3 scripts/book.py --venue-id 1505 --date 2024-12-25 --time 19:00 --party-size 2
```

### List Your Reservations

```bash
python3 scripts/list_reservations.py
```

### Cancel a Reservation

```bash
python3 scripts/cancel.py --reservation-id resy_abc123
```

## 📁 Project Structure

```
resy-booking/
├── SKILL.md                    # OpenClaw skill definition
├── README.md                   # This file
├── LICENSE                     # MIT License
├── requirements.txt            # Python dependencies
├── .gitignore                  # Git ignore rules
├── scripts/
│   ├── __init__.py            # Package init
│   ├── auth.py                # Authentication handling
│   ├── utils.py               # Shared utilities & ResyClient
│   ├── search.py              # Restaurant search
│   ├── availability.py        # Check availability
│   ├── book.py                # Book reservations
│   ├── cancel.py              # Cancel reservations
│   └── list_reservations.py   # List user reservations
├── references/
│   ├── api-docs.md            # Complete API documentation
│   ├── setup-guide.md         # Credential extraction guide
│   └── error-codes.md         # Error handling reference
└── tests/
    └── test_booking.py        # Unit tests
```

## 🧪 Testing

```bash
# Run all tests
python3 tests/test_booking.py

# Run syntax check
python3 -m py_compile scripts/*.py

# Test authentication
python3 scripts/auth.py
```

## 🔐 Security Audit

This skill has been security audited for:

- **Credential handling** - No hardcoded secrets, env-only
- **Network security** - HTTPS only, validated hosts
- **Input validation** - All inputs sanitized
- **Error handling** - Safe error messages
- **Logging** - Sensitive data masked

See [SECURITY.md](SECURITY.md) for detailed security information.

## 🤝 Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) file.

## ⚠️ Disclaimer

This is an unofficial skill. Resy may change their API at any time. Use at your own risk.

## 🔗 Links

- [OpenClaw Documentation](https://docs.openclaw.ai)
- [Resy Website](https://resy.com)
- [Issue Tracker](../../issues)

---

Built with ❤️ for the OpenClaw community.
