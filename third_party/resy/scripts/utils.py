#!/usr/bin/env python3
"""
Shared utilities for Resy booking skill.
"""

import os
import sys
import re
import json
import logging
from datetime import datetime, date, timedelta
from typing import Optional, Dict, Any, Union, List
from urllib.parse import urljoin, urlparse

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from pytz import timezone as pytz_timezone
    PYTZ_AVAILABLE = True
except ImportError:
    PYTZ_AVAILABLE = False

try:
    import requests
except ImportError:
    print("Error: requests module not found. Install with: pip install requests", file=sys.stderr)
    sys.exit(1)

# Optional progress bar support
try:
    from tqdm import tqdm
    TQDM_AVAILABLE = True
except ImportError:
    TQDM_AVAILABLE = False
    tqdm = None

# Constants
RESY_API_BASE = "https://api.resy.com/"
VALID_API_HOST = "api.resy.com"
DEFAULT_TIMEZONE = "America/New_York"

# Setup logging to stderr
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [RESY] %(levelname)s: %(message)s',
    stream=sys.stderr
)
logger = logging.getLogger('resy_booking')


# Timezone handling functions
def get_timezone(tz_name: Optional[str] = None) -> Any:
    """
    Get timezone object.
    
    Args:
        tz_name: Timezone name (e.g., 'America/New_York'). Uses DEFAULT_TIMEZONE if not specified.
    
    Returns:
        Timezone object or None if pytz not available
    """
    if not PYTZ_AVAILABLE:
        return None
    
    tz = tz_name or os.environ.get('RESY_TIMEZONE', DEFAULT_TIMEZONE)
    try:
        return pytz_timezone(tz)
    except Exception as e:
        logger.warning(f"Invalid timezone '{tz}', using default. Error: {e}")
        return pytz_timezone(DEFAULT_TIMEZONE)


def localize_datetime(dt: datetime, tz_name: Optional[str] = None) -> datetime:
    """
    Localize a datetime to a specific timezone.
    
    Args:
        dt: Datetime object (naive or aware)
        tz_name: Target timezone name
    
    Returns:
        Timezone-aware datetime
    """
    if not PYTZ_AVAILABLE:
        return dt
    
    tz = get_timezone(tz_name)
    if tz is None:
        return dt
    
    if dt.tzinfo is None:
        # Naive datetime - localize it
        return tz.localize(dt)
    else:
        # Already aware - convert
        return dt.astimezone(tz)


def convert_to_restaurant_tz(date_str: str, time_str: str, restaurant_tz: Optional[str] = None) -> tuple:
    """
    Convert a date/time to the restaurant's timezone.
    
    Args:
        date_str: Date in YYYY-MM-DD format
        time_str: Time in HH:MM format
        restaurant_tz: Restaurant's timezone (if known)
    
    Returns:
        Tuple of (date_str, time_str) in restaurant's timezone
    """
    if not PYTZ_AVAILABLE or not restaurant_tz:
        return date_str, time_str
    
    try:
        # Parse user's local datetime
        user_tz = get_timezone()
        dt_str = f"{date_str} {time_str}"
        dt = datetime.strptime(dt_str, '%Y-%m-%d %H:%M')
        dt_aware = user_tz.localize(dt)
        
        # Convert to restaurant timezone
        rest_tz = get_timezone(restaurant_tz)
        dt_rest = dt_aware.astimezone(rest_tz)
        
        return dt_rest.strftime('%Y-%m-%d'), dt_rest.strftime('%H:%M')
    except Exception as e:
        logger.warning(f"Timezone conversion failed: {e}")
        return date_str, time_str


class ProgressSpinner:
    """Simple progress spinner for CLI feedback."""
    
    def __init__(self, message: str = "Loading", enabled: bool = True):
        self.message = message
        self.enabled = enabled and not TQDM_AVAILABLE
        self.spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
        self.idx = 0
        self.running = False
        self.thread = None
    
    def _spin(self):
        """Spinner animation loop."""
        import time
        import sys
        while self.running:
            symbol = self.spinner[self.idx % len(self.spinner)]
            sys.stderr.write(f'\r{symbol} {self.message}...')
            sys.stderr.flush()
            self.idx += 1
            time.sleep(0.1)
        sys.stderr.write('\r' + ' ' * (len(self.message) + 10) + '\r')
        sys.stderr.flush()
    
    def __enter__(self):
        if self.enabled:
            self.running = True
            import threading
            self.thread = threading.Thread(target=self._spin)
            self.thread.daemon = True
            self.thread.start()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.enabled and self.running:
            self.running = False
            if self.thread:
                self.thread.join(timeout=0.5)
        return False
    
    def update(self, message: str):
        """Update spinner message."""
        self.message = message


def with_progress(message: str = "Processing"):
    """Decorator to add progress indicator to a function."""
    def decorator(func):
        def wrapper(*args, **kwargs):
            with ProgressSpinner(message):
                return func(*args, **kwargs)
        return wrapper
    return decorator


def format_datetime_with_tz(dt_str: str, tz_name: Optional[str] = None) -> str:
    """
    Format a datetime string with timezone information.
    
    Args:
        dt_str: Datetime string from API
        tz_name: Target timezone name
    
    Returns:
        Formatted string with timezone
    """
    if not PYTZ_AVAILABLE:
        return dt_str
    
    try:
        # Parse the datetime
        for fmt in ['%Y-%m-%d %H:%M:%S', '%Y-%m-%d %H:%M', '%Y-%m-%dT%H:%M:%S']:
            try:
                dt = datetime.strptime(dt_str, fmt)
                tz = get_timezone(tz_name)
                if tz:
                    dt = tz.localize(dt)
                    return dt.strftime('%Y-%m-%d %H:%M %Z')
                break
            except ValueError:
                continue
    except Exception as e:
        logger.debug(f"Could not format datetime with timezone: {e}")
    
    return dt_str


class ResyAPIError(Exception):
    """Custom exception for Resy API errors."""
    
    def __init__(self, message: str, status_code: Optional[int] = None, response: Optional[Dict] = None):
        self.message = message
        self.status_code = status_code
        self.response = response
        super().__init__(self.message)


class ResyClient:
    """HTTP client for Resy API with authentication and security controls."""
    
    def __init__(self):
        self.api_key = self._get_api_key()
        self.auth_token = self._get_auth_token()
        self.session = requests.Session()
        self.session.headers.update(self._get_headers())
    
    def _get_api_key(self) -> str:
        """Get API key from environment variable or file."""
        api_key = os.environ.get('RESY_API_KEY')
        if not api_key:
            # Try reading from file (with path validation)
            key_file = os.environ.get('RESY_API_KEY_FILE')
            if key_file:
                api_key = self._read_credential_file(key_file, 'RESY_API_KEY_FILE')
        if not api_key:
            raise ResyAPIError(
                "RESY_API_KEY environment variable not set. "
                "See setup guide: ~/.openclaw/workspace/skills/resy-booking/references/setup-guide.md"
            )
        return api_key
    
    def _get_auth_token(self) -> str:
        """Get auth token from environment variable or file."""
        auth_token = os.environ.get('RESY_AUTH_TOKEN')
        if not auth_token:
            # Try reading from file (with path validation)
            token_file = os.environ.get('RESY_AUTH_TOKEN_FILE')
            if token_file:
                auth_token = self._read_credential_file(token_file, 'RESY_AUTH_TOKEN_FILE')
        if not auth_token:
            raise ResyAPIError(
                "RESY_AUTH_TOKEN environment variable not set. "
                "See setup guide: ~/.openclaw/workspace/skills/resy-booking/references/setup-guide.md"
            )
        return auth_token
    
    def _read_credential_file(self, file_path: str, env_var_name: str) -> Optional[str]:
        """
        Read credential from file with path traversal protection.
        
        Args:
            file_path: Path to credential file
            env_var_name: Name of environment variable (for error messages)
        
        Returns:
            File contents or None if invalid
        
        Raises:
            ResyAPIError: If path traversal detected or file too large
        """
        # Expand and normalize path
        expanded_path = os.path.abspath(os.path.expanduser(file_path))
        
        # Allowed directories (home and tmp only)
        allowed_dirs = [
            os.path.expanduser('~'),
            '/tmp',
            os.environ.get('TMPDIR', '/tmp'),  # macOS temp
        ]
        
        # Check for path traversal
        if not any(expanded_path.startswith(d) for d in allowed_dirs):
            raise ResyAPIError(
                f"Invalid {env_var_name} path: {file_path}. "
                f"Credential file must be in home directory or /tmp."
            )
        
        # Check file exists and is readable
        if not os.path.exists(expanded_path):
            return None
        
        if not os.path.isfile(expanded_path):
            raise ResyAPIError(f"{env_var_name} path is not a file: {file_path}")
        
        # Check file size (prevent reading huge files)
        file_size = os.path.getsize(expanded_path)
        if file_size > 4096:  # Max 4KB
            raise ResyAPIError(
                f"{env_var_name} file too large: {file_size} bytes (max 4096)"
            )
        
        # Read and validate content
        try:
            with open(expanded_path, 'r') as f:
                content = f.read().strip()
            
            # Basic validation - should be a reasonable API key format
            if len(content) < 10:
                raise ResyAPIError(
                    f"{env_var_name} file contains invalid credential (too short)"
                )
            
            return content
            
        except (IOError, OSError) as e:
            raise ResyAPIError(f"Failed to read {env_var_name}: {str(e)}")
    
    def _get_headers(self) -> Dict[str, str]:
        """Build authentication headers."""
        return {
            'Authorization': f'ResyAPI api_key="{self.api_key}"',
            'X-Resy-Auth-Token': self.auth_token,
            'X-Resy-Universal-Auth': self.auth_token,
            'User-Agent': 'ResyBooking/1.0 (OpenClaw Skill)',
            'Accept': 'application/json',
        }
    
    def _validate_url(self, url: str) -> str:
        """Ensure URL only connects to allowed hosts."""
        if not url.startswith(('http://', 'https://')):
            url = urljoin(RESY_API_BASE, url)
        
        parsed = urlparse(url)
        host = parsed.netloc.lower().split(':')[0]  # Remove port
        
        if parsed.scheme != 'https':
            raise ResyAPIError(f"Invalid scheme: {parsed.scheme}. Only HTTPS is allowed.")
        
        if host != VALID_API_HOST:
            raise ResyAPIError(f"Invalid API host: {parsed.netloc}. Only {VALID_API_HOST} is allowed.")
        
        return url
    
    def _sanitize_for_log(self, text: str) -> str:
        """Remove control characters that could affect log parsing."""
        import re
        return re.sub(r'[\x00-\x1F\x7F]', '', text)

    def _log_request(self, method: str, url: str, **kwargs):
        """Log API request to stderr (audit logging)."""
        safe_url = self._sanitize_for_log(self._sanitize_log_url(url))
        safe_method = self._sanitize_for_log(method)
        logger.info(f"API Request: {safe_method} {safe_url}")
    
    def _sanitize_log_url(self, url: str) -> str:
        """Remove sensitive data from URLs before logging."""
        # Remove any query parameters that might contain sensitive data
        if '?' in url:
            base, _ = url.split('?', 1)
            return base + "?..."
        return url
    
    def _mask_sensitive_data(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Mask sensitive fields in logged data."""
        masked = {}
        sensitive_fields = {
            'book_token', 'config_id', 'payment_method', 'token', 'auth_token',
            'payment_method_id', 'credit_card', 'card_number', 'cvv', 'password',
            'session_token', 'refresh_token', 'secret', 'api_key'
        }
        
        for key, value in data.items():
            if key.lower() in sensitive_fields or any(s in key.lower() for s in sensitive_fields):
                masked[key] = '***MASKED***'
            elif isinstance(value, dict):
                masked[key] = self._mask_sensitive_data(value)
            elif isinstance(value, list):
                masked[key] = [self._mask_sensitive_data(item) if isinstance(item, dict) else item for item in value]
            else:
                masked[key] = value
        
        return masked
    
    def request(self, method: str, url: str, retries: int = 3, **kwargs) -> requests.Response:
        """Make authenticated request with security controls and retry logic."""
        import time
        
        for attempt in range(retries):
            try:
                url = self._validate_url(url)
                self._log_request(method, url)
                
                # Log masked request data for POST/PUT (at debug level)
                if method in ('POST', 'PUT') and 'data' in kwargs:
                    try:
                        if isinstance(kwargs['data'], dict):
                            masked_req = self._mask_sensitive_data(kwargs['data'])
                            logger.debug(f"Request data (masked): {masked_req}")
                    except Exception:
                        pass  # Don't fail logging
                
                response = self.session.request(method, url, timeout=30, **kwargs)
                logger.info(f"API Response: {response.status_code}")
                
                # Retry on transient errors
                if response.status_code in (502, 503, 504) and attempt < retries - 1:
                    wait_time = 2 ** attempt  # Exponential backoff
                    logger.warning(f"Transient error {response.status_code}, retrying in {wait_time}s...")
                    time.sleep(wait_time)
                    continue
                
                # Handle common errors
                if response.status_code == 401:
                    raise ResyAPIError(
                        "Authentication failed. Your API key or auth token may be expired or invalid. "
                        "Please re-extract credentials from your browser.",
                        status_code=401
                    )
                elif response.status_code == 429:
                    raise ResyAPIError(
                        "Rate limit exceeded. Please wait a moment before trying again.",
                        status_code=429
                    )
                elif response.status_code >= 500:
                    raise ResyAPIError(
                        f"Resy server error ({response.status_code}). Please try again later.",
                        status_code=response.status_code
                    )
                
                response.raise_for_status()
                
                # Validate JSON can be parsed and mask sensitive data for logging
                if 'application/json' in response.headers.get('Content-Type', ''):
                    try:
                        json_data = response.json()
                        # Log masked response for debugging (at debug level)
                        masked_data = self._mask_sensitive_data(json_data)
                        logger.debug(f"Response body (masked): {json.dumps(masked_data)[:500]}")
                    except json.JSONDecodeError as e:
                        raise ResyAPIError(f"Invalid JSON response from API: {str(e)}")
                
                return response
                
            except requests.exceptions.Timeout:
                if attempt < retries - 1:
                    wait_time = 2 ** attempt
                    time.sleep(wait_time)
                    continue
                raise ResyAPIError("Request timed out. Please check your internet connection and try again.")
            except requests.exceptions.ConnectionError:
                if attempt < retries - 1:
                    wait_time = 2 ** attempt
                    time.sleep(wait_time)
                    continue
                raise ResyAPIError("Connection error. Please check your internet connection.")
            except ResyAPIError:
                raise
            except requests.exceptions.RequestException as e:
                if attempt < retries - 1:
                    wait_time = 2 ** attempt
                    time.sleep(wait_time)
                    continue
                raise ResyAPIError(f"Request failed: {str(e)}")
    
    def get(self, url: str, **kwargs) -> requests.Response:
        """Make GET request."""
        return self.request('GET', url, **kwargs)
    
    def post(self, url: str, **kwargs) -> requests.Response:
        """Make POST request."""
        return self.request('POST', url, **kwargs)


def validate_date(date_str: str) -> str:
    """
    Validate date format and ensure it's in the future.
    
    Args:
        date_str: Date string in YYYY-MM-DD format
    
    Returns:
        Validated date string
    
    Raises:
        ValueError: If date is invalid or in the past
    """
    if not date_str:
        raise ValueError("Date is required")
    
    # Check format
    if not re.match(r'^\d{4}-\d{2}-\d{2}$', date_str):
        raise ValueError(f"Invalid date format: '{date_str}'. Use YYYY-MM-DD (e.g., 2024-12-25)")
    
    try:
        parsed_date = datetime.strptime(date_str, '%Y-%m-%d').date()
    except ValueError as e:
        raise ValueError(f"Invalid date: '{date_str}'. {str(e)}")
    
    # Check if date is in the future
    if parsed_date < date.today():
        raise ValueError(f"Date must be in the future. Got: {date_str}")
    
    # Check if date is not too far in the future (most restaurants book 30-90 days out)
    max_date = date.today() + timedelta(days=120)
    
    if parsed_date > max_date:
        raise ValueError(f"Date too far in the future. Most restaurants accept reservations up to 90-120 days ahead.")
    
    return date_str


def validate_time(time_str: str) -> str:
    """
    Validate time format.
    
    Args:
        time_str: Time string in HH:MM format (24-hour)
    
    Returns:
        Validated time string
    
    Raises:
        ValueError: If time is invalid
    """
    if not time_str:
        raise ValueError("Time is required")
    
    # Support both HH:MM and HH:MM:SS
    if not re.match(r'^\d{1,2}:\d{2}(?::\d{2})?$', time_str):
        raise ValueError(f"Invalid time format: '{time_str}'. Use HH:MM (e.g., 19:00 for 7 PM)")
    
    try:
        if len(time_str.split(':')) == 3:
            datetime.strptime(time_str, '%H:%M:%S')
        else:
            datetime.strptime(time_str, '%H:%M')
    except ValueError as e:
        raise ValueError(f"Invalid time: '{time_str}'. {str(e)}")
    
    return time_str


def validate_party_size(party_size: Union[str, int]) -> int:
    """
    Validate party size.
    
    Args:
        party_size: Number of guests (1-20)
    
    Returns:
        Validated party size as integer
    
    Raises:
        ValueError: If party size is invalid
    """
    try:
        size = int(party_size)
    except (ValueError, TypeError):
        raise ValueError(f"Party size must be a number. Got: '{party_size}'")
    
    if size < 1:
        raise ValueError(f"Party size must be at least 1. Got: {size}")
    
    if size > 20:
        raise ValueError(f"Party size exceeds maximum of 20. Got: {size}. For larger groups, call the restaurant directly.")
    
    return size


def validate_venue_id(venue_id: Union[str, int]) -> int:
    """
    Validate venue ID.
    
    Args:
        venue_id: Restaurant venue ID
    
    Returns:
        Validated venue ID as integer
    
    Raises:
        ValueError: If venue ID is invalid
    """
    try:
        vid = int(venue_id)
    except (ValueError, TypeError):
        raise ValueError(f"Venue ID must be a number. Got: '{venue_id}'")
    
    if vid <= 0:
        raise ValueError(f"Venue ID must be positive. Got: {vid}")
    
    return vid


def format_datetime(date_str: str, time_str: str) -> str:
    """
    Combine date and time into Resy API format.
    
    Args:
        date_str: Date in YYYY-MM-DD format
        time_str: Time in HH:MM format
    
    Returns:
        Formatted datetime string
    """
    return f"{date_str} {time_str}:00"


def parse_reservation_time(datetime_str: str) -> tuple:
    """
    Parse Resy datetime string into date and time components.
    
    Args:
        datetime_str: Datetime string (e.g., "2024-12-25 19:00:00")
    
    Returns:
        Tuple of (date_str, time_str)
    """
    try:
        dt = datetime.strptime(datetime_str, '%Y-%m-%d %H:%M:%S')
        return dt.strftime('%Y-%m-%d'), dt.strftime('%H:%M')
    except ValueError:
        # Try alternative formats
        for fmt in ['%Y-%m-%d %H:%M', '%Y-%m-%dT%H:%M:%S', '%Y-%m-%dT%H:%M']:
            try:
                dt = datetime.strptime(datetime_str, fmt)
                return dt.strftime('%Y-%m-%d'), dt.strftime('%H:%M')
            except ValueError:
                continue
        raise ValueError(f"Cannot parse datetime: {datetime_str}")


def check_availability(
    client: 'ResyClient',
    venue_id: int,
    date: str,
    party_size: int,
    lat: Optional[float] = None,
    long: Optional[float] = None
) -> Dict:
    """
    Check availability for a restaurant.
    
    Args:
        client: ResyClient instance
        venue_id: Restaurant venue ID
        date: Date in YYYY-MM-DD format
        party_size: Number of guests
        lat: Optional latitude
        long: Optional longitude
    
    Returns:
        Dict with availability information
    """
    params = {
        'venue_id': venue_id,
        'day': date,
        'party_size': party_size,
    }
    
    if lat is not None and long is not None:
        params['lat'] = lat
        params['long'] = long
    
    try:
        response = client.get('/4/find', params=params)
        return response.json()
    except ResyAPIError:
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to check availability: {str(e)}")


def parse_availability(data: Dict) -> List[Dict]:
    """
    Parse availability response into structured slots.
    
    Args:
        data: API response data
    
    Returns:
        List of available time slots
    """
    from typing import List
    slots = []
    
    venues = data.get('results', {}).get('venues', [])
    
    for venue_data in venues:
        venue_info = venue_data.get('venue', {})
        available_slots = venue_data.get('slots', [])
        
        for slot in available_slots:
            date_info = slot.get('date', {})
            config = slot.get('config', {})
            
            slot_info = {
                'datetime': date_info.get('start'),
                'date': '',
                'time': '',
                'type': config.get('type', 'Standard'),
                'token': config.get('token'),
                'venue_name': venue_info.get('name'),
            }
            
            # Parse datetime
            if slot_info['datetime']:
                try:
                    slot_info['date'], slot_info['time'] = parse_reservation_time(slot_info['datetime'])
                except ValueError:
                    pass
            
            slots.append(slot_info)
    
    return slots


if __name__ == '__main__':
    # Test validation functions
    print("Testing validate_date...")
    print(f"  validate_date('2024-12-25') = {validate_date('2024-12-25')}")
    
    print("\nTesting validate_time...")
    print(f"  validate_time('19:00') = {validate_time('19:00')}")
    
    print("\nTesting validate_party_size...")
    print(f"  validate_party_size(4) = {validate_party_size(4)}")
    
    print("\nTesting validate_venue_id...")
    print(f"  validate_venue_id(1505) = {validate_venue_id(1505)}")
    
    print("\nAll validations passed!")
