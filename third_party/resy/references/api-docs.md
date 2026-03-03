# Resy API Documentation

## Base URL

```
https://api.resy.com/
```

## Authentication

All API requests require two headers:

### Headers

| Header | Value Format | Description |
|--------|--------------|-------------|
| `Authorization` | `ResyAPI api_key="YOUR_API_KEY"` | API authentication |
| `X-Resy-Auth-Token` | `YOUR_AUTH_TOKEN` | User session token |

### Obtaining Credentials

1. Log into Resy at https://resy.com/
2. Open browser Developer Tools (F12)
3. Go to Network tab
4. Visit any restaurant page
5. Look for API calls to `api.resy.com`
6. Extract credentials from request headers:
   - `Authorization` header contains API key
   - `X-Resy-Auth-Token` header contains auth token

## Endpoints

### 1. Find Availability

Search for available reservation slots.

**Endpoint:** `GET /4/find`

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `venue_id` | integer | Yes | Restaurant ID |
| `day` | string | Yes | Date in YYYY-MM-DD format |
| `party_size` | integer | Yes | Number of guests (1-20) |
| `lat` | float | No | Latitude for location context |
| `long` | float | No | Longitude for location context |

**Example Request:**
```bash
curl -X GET "https://api.resy.com/4/find?venue_id=1505&day=2024-12-25&party_size=2" \
  -H "Authorization: ResyAPI api_key=\"YOUR_KEY\"" \
  -H "X-Resy-Auth-Token: YOUR_TOKEN"
```

**Example Response:**
```json
{
  "results": {
    "venues": [{
      "venue": {
        "id": {"resy": 1505},
        "name": "Don Angie",
        "location": {"name": "West Village"}
      },
      "slots": [
        {
          "date": {"start": "2024-12-25 18:00:00"},
          "config": {"type": "Dining Room", "token": "eyJ..."}
        }
      ]
    }]
  }
}
```

### 2. Get Booking Details

Retrieve booking token and details before creating a reservation.

**Endpoint:** `POST /3/details`

**Content-Type:** `application/x-www-form-urlencoded`

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `commit` | integer | Yes | Set to 1 |
| `config_id` | string | Yes | Config token from find endpoint |
| `day` | string | Yes | Date in YYYY-MM-DD format |
| `party_size` | integer | Yes | Number of guests |

**Example Request:**
```bash
curl -X POST "https://api.resy.com/3/details" \
  -H "Authorization: ResyAPI api_key=\"YOUR_KEY\"" \
  -H "X-Resy-Auth-Token: YOUR_TOKEN" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "commit=1&config_id=TOKEN&day=2024-12-25&party_size=2"
```

**Example Response:**
```json
{
  "book_token": {"value": "book_token_value"},
  "user": {"id": 12345},
  "reservation_id": null
}
```

### 3. Create Reservation

Book a reservation using the booking token.

**Endpoint:** `POST /3/book`

**Content-Type:** `application/x-www-form-urlencoded`

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `book_token` | string | Yes | Token from details endpoint |
| `source_id` | string | No | Source identifier ("resy.com") |
| `struct_payment_method` | string | No | Payment method JSON |

**Example Request:**
```bash
curl -X POST "https://api.resy.com/3/book" \
  -H "Authorization: ResyAPI api_key=\"YOUR_KEY\"" \
  -H "X-Resy-Auth-Token: YOUR_TOKEN" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "book_token=TOKEN&source_id=resy.com"
```

**Example Response:**
```json
{
  "reservation_id": "resy_abc123",
  "resy_token": "abc123",
  "display_date": "December 25, 2024",
  "display_time": "7:00 PM"
}
```

### 4. Cancel Reservation

Cancel an existing reservation.

**Endpoint:** `POST /3/cancel`

**Content-Type:** `application/x-www-form-urlencoded`

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `reservation_id` | string | Yes | Reservation ID to cancel |

**Example Request:**
```bash
curl -X POST "https://api.resy.com/3/cancel" \
  -H "Authorization: ResyAPI api_key=\"YOUR_KEY\"" \
  -H "X-Resy-Auth-Token: YOUR_TOKEN" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "reservation_id=resy_abc123"
```

**Example Response:**
```json
{
  "status": "cancelled"
}
```

### 5. Get User Info

Retrieve user profile and existing reservations.

**Endpoint:** `GET /2/user`

**Example Request:**
```bash
curl -X GET "https://api.resy.com/2/user" \
  -H "Authorization: ResyAPI api_key=\"YOUR_KEY\"" \
  -H "X-Resy-Auth-Token: YOUR_TOKEN"
```

**Example Response:**
```json
{
  "id": 12345,
  "email": "user@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "payment_methods": [...],
  "reservations": [
    {
      "reservation_id": "resy_abc123",
      "venue": {"name": "Don Angie"},
      "scheduled_date": "2024-12-25",
      "scheduled_time": "19:00:00"
    }
  ]
}
```

## Rate Limiting

The Resy API implements rate limiting. Best practices:

- Don't exceed 1 request per second
- Cache results when appropriate
- Implement exponential backoff for retries

## Response Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 400 | Bad Request | Invalid parameters |
| 401 | Unauthorized | Invalid credentials |
| 403 | Forbidden | Action not permitted |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Booking conflict |
| 422 | Unprocessable | Validation error |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Server Error | Resy server error |

## Common Error Responses

### Authentication Error (401)
```json
{
  "message": "Unauthorized",
  "status": 401
}
```

### No Availability (200 with empty slots)
```json
{
  "results": {
    "venues": [{
      "venue": {...},
      "slots": []
    }]
  }
}
```

### Booking Conflict (409)
```json
{
  "message": "Reservation conflict",
  "status": 409
}
```

## Notes

- All times are in the restaurant's local timezone
- Venue IDs are stable and can be cached
- Config tokens expire quickly (use within seconds)
- Some restaurants require credit cards on file
