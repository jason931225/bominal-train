# Resy API Error Codes and Solutions

## Authentication Errors

### 401 Unauthorized

**Cause:** Invalid or expired credentials

**Symptoms:**
```
Error: Authentication failed (401)
Message: Unauthorized
```

**Solutions:**
1. Re-extract your API key and auth token from browser
2. Verify environment variables are set correctly:
   ```bash
   echo $RESY_API_KEY
   echo $RESY_AUTH_TOKEN
   ```
3. Ensure tokens haven't expired (re-login to Resy if needed)

---

## Availability Errors

### No Availability (Empty Slots)

**Cause:** Restaurant has no open tables for the requested criteria

**Symptoms:**
```
No availability found for venue_id=12345 on 2024-12-25
```

**Solutions:**
1. Try different dates or times
2. Check if restaurant is closed that day
3. Some restaurants release reservations at specific times:
   - Many release 30 days in advance at 9 AM local time
   - Some release 60 days in advance
   - Popular spots may have different schedules
4. Check for waitlist availability (if supported by restaurant)
5. Try walk-in availability

### 422 Unprocessable Entity (Invalid Date/Time)

**Cause:** Invalid date format or past date

**Symptoms:**
```
Error: Invalid request (422)
Message: Date must be in the future
```

**Solutions:**
1. Use correct date format: `YYYY-MM-DD`
2. Ensure date is in the future
3. Check that time format is `HH:MM` (24-hour)

---

## Booking Errors

### 409 Conflict (Already Booked)

**Cause:** You already have a reservation at this time/venue

**Symptoms:**
```
Error: Reservation conflict (409)
Message: You already have a reservation at this venue
```

**Solutions:**
1. Check your existing reservations:
   ```bash
   python3 ~/.openclaw/workspace/skills/resy-booking/scripts/list_reservations.py
   ```
2. Cancel existing reservation first if needed
3. Choose a different time or date

### 409 Conflict (Slot Taken)

**Cause:** Someone else booked the slot while you were attempting

**Symptoms:**
```
Error: Reservation conflict (409)
Message: This slot is no longer available
```

**Solutions:**
1. Re-check availability for alternative slots
2. Try booking immediately when slots are released
3. High-demand restaurants require quick action

### 402 Payment Required

**Cause:** No credit card on file for the restaurant

**Symptoms:**
```
Error: Payment required (402)
Message: Credit card required for this reservation
```

**Solutions:**
1. Log into Resy website
2. Go to Account → Payment Methods
3. Add a valid credit card
4. Some restaurants require credit cards for all reservations
5. Others only require them for large parties or special events

### 422 Unprocessable (Invalid Party Size)

**Cause:** Party size outside restaurant's allowed range

**Symptoms:**
```
Error: Invalid party size (422)
Message: Party size must be between 1 and 8
```

**Solutions:**
1. Check restaurant's party size limits
2. For large groups, call the restaurant directly
3. Split into multiple smaller reservations (if policy allows)

---

## Cancellation Errors

### 404 Not Found (Invalid Reservation ID)

**Cause:** Reservation ID doesn't exist or already cancelled

**Symptoms:**
```
Error: Reservation not found (404)
Message: Reservation does not exist
```

**Solutions:**
1. List your current reservations to get correct ID:
   ```bash
   python3 ~/.openclaw/workspace/skills/resy-booking/scripts/list_reservations.py
   ```
2. Check if reservation was already cancelled
3. Reservation may have expired (past date)

### 403 Forbidden (Cancellation Policy)

**Cause:** Outside cancellation window

**Symptoms:**
```
Error: Cancellation not allowed (403)
Message: Cancellation window has passed
```

**Solutions:**
1. Check restaurant's cancellation policy
2. Many restaurants require 24-48 hours notice
3. Contact restaurant directly for late cancellations
4. You may be charged a no-show fee

---

## Rate Limiting Errors

### 429 Too Many Requests

**Cause:** Making too many requests too quickly

**Symptoms:**
```
Error: Rate limit exceeded (429)
Message: Please slow down
```

**Solutions:**
1. Add delays between requests (1+ second)
2. Implement exponential backoff
3. Don't run multiple scripts simultaneously
4. Wait a few minutes before retrying

---

## Server Errors

### 500 Internal Server Error

**Cause:** Resy server issue

**Symptoms:**
```
Error: Server error (500)
Message: Internal server error
```

**Solutions:**
1. Wait a few minutes and retry
2. Check Resy website status
3. Try again later
4. If persistent, Resy may be experiencing outages

### 503 Service Unavailable

**Cause:** Resy temporarily down for maintenance

**Symptoms:**
```
Error: Service unavailable (503)
Message: Service temporarily unavailable
```

**Solutions:**
1. Wait and retry in a few minutes
2. Check Resy social media for maintenance announcements
3. Use Resy website/app as alternative

---

## Network Errors

### Connection Timeout

**Cause:** Network connectivity issues

**Symptoms:**
```
Error: Connection timeout
Message: Could not connect to api.resy.com
```

**Solutions:**
1. Check internet connection
2. Verify firewall/proxy settings
3. Try from different network
4. Check if Resy is accessible in browser

### SSL Certificate Error

**Cause:** SSL/TLS handshake failure

**Symptoms:**
```
Error: SSL certificate verification failed
```

**Solutions:**
1. Update your system's SSL certificates
2. Check system time is correct
3. Verify not behind corporate proxy intercepting SSL

---

## Search Errors

### No Results Found

**Cause:** Search query too specific or restaurant not on Resy

**Symptoms:**
```
No restaurants found for query: "xyz"
```

**Solutions:**
1. Try broader search terms
2. Search by neighborhood or cuisine type
3. Verify restaurant uses Resy (not OpenTable, Tock, etc.)
4. Check spelling of restaurant name

### Invalid Location

**Cause:** Location format not recognized

**Symptoms:**
```
Error: Invalid location format
```

**Solutions:**
1. Use city names (e.g., "New York", "Los Angeles")
2. Try neighborhood names (e.g., "West Village")
3. Use zip codes for specific areas

---

## Debugging Tips

### Enable Debug Logging

All scripts support verbose output:
```bash
python3 script.py --verbose [other options]
```

### Check API Response

View full API response for debugging:
```bash
python3 script.py --debug [other options] 2>&1 | less
```

### Verify Credentials

Quick credential check:
```bash
curl -H "Authorization: ResyAPI api_key=\"$RESY_API_KEY\"" \
     -H "X-Resy-Auth-Token: $RESY_AUTH_TOKEN" \
     https://api.resy.com/2/user
```

### Common Fixes Checklist

- [ ] Credentials set correctly in environment variables
- [ ] Date format is YYYY-MM-DD and in the future
- [ ] Time format is HH:MM (24-hour)
- [ ] Party size within restaurant's limits
- [ ] Venue ID is correct (use search to verify)
- [ ] Credit card on file (if required)
- [ ] Not rate limited (wait between requests)
- [ ] Internet connection stable

## Getting Help

If issues persist:

1. Check Resy Help Center: https://help.resy.com/
2. Contact Resy Support through the app/website
3. Review this error code reference
4. Check if Resy has announced API changes
