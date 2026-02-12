#!/usr/bin/env python3
"""
Authentication module for Resy booking skill.
"""

import os
import sys
from typing import Optional, Dict

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from utils import ResyClient, ResyAPIError, logger


class ResyAuth:
    """
    Handles Resy API authentication and credential validation.
    
    This class provides methods to verify credentials and get user information
    without storing any sensitive data.
    """
    
    def __init__(self):
        self.client: Optional[ResyClient] = None
        self._user_info: Optional[Dict] = None
    
    def authenticate(self) -> bool:
        """
        Verify that credentials are valid by making a test API call.
        
        Returns:
            True if authentication successful, False otherwise
        """
        try:
            self.client = ResyClient()
            # Make a lightweight API call to verify credentials
            response = self.client.get('/2/user')
            self._user_info = response.json()
            logger.info("Authentication successful")
            return True
        except ResyAPIError as e:
            logger.error(f"Authentication failed: {e.message}")
            return False
        except Exception as e:
            logger.error(f"Unexpected error during authentication: {str(e)}")
            return False
    
    def get_user_info(self) -> Optional[Dict]:
        """
        Get authenticated user information.
        
        Returns:
            User info dict or None if not authenticated
        """
        if not self._user_info and self.client:
            try:
                response = self.client.get('/2/user')
                self._user_info = response.json()
            except Exception as e:
                logger.error(f"Failed to get user info: {str(e)}")
                return None
        
        return self._user_info
    
    def get_user_id(self) -> Optional[int]:
        """Get the authenticated user's ID."""
        info = self.get_user_info()
        return info.get('id') if info else None
    
    def get_user_email(self) -> Optional[str]:
        """Get the authenticated user's email."""
        info = self.get_user_info()
        return info.get('email') if info else None
    
    def get_user_name(self) -> Optional[str]:
        """Get the authenticated user's full name."""
        info = self.get_user_info()
        if info:
            first = info.get('first_name', '')
            last = info.get('last_name', '')
            return f"{first} {last}".strip()
        return None
    
    def get_client(self) -> ResyClient:
        """
        Get the authenticated API client.
        
        Returns:
            ResyClient instance
        
        Raises:
            ResyAPIError: If client not initialized
        """
        if not self.client:
            self.client = ResyClient()
        return self.client
    
    @staticmethod
    def check_credentials_set() -> bool:
        """
        Check if required environment variables or files are set.
        
        Returns:
            True if both API key and auth token are available
        """
        has_key = bool(
            os.environ.get('RESY_API_KEY') or 
            os.environ.get('RESY_API_KEY_FILE')
        )
        has_token = bool(
            os.environ.get('RESY_AUTH_TOKEN') or 
            os.environ.get('RESY_AUTH_TOKEN_FILE')
        )
        return has_key and has_token
    
    @staticmethod
    def get_credentials_status() -> Dict[str, bool]:
        """
        Get status of credential sources.
        
        Returns:
            Dict with credential status values
        """
        api_key_env = bool(os.environ.get('RESY_API_KEY'))
        api_key_file = bool(os.environ.get('RESY_API_KEY_FILE'))
        auth_token_env = bool(os.environ.get('RESY_AUTH_TOKEN'))
        auth_token_file = bool(os.environ.get('RESY_AUTH_TOKEN_FILE'))
        
        return {
            'api_key_env': api_key_env,
            'api_key_file': api_key_file,
            'auth_token_env': auth_token_env,
            'auth_token_file': auth_token_file,
            'all_set': (api_key_env or api_key_file) and (auth_token_env or auth_token_file)
        }


def main():
    """CLI entry point for testing authentication."""
    import json
    
    auth = ResyAuth()
    
    # Check if credentials are set
    status = auth.get_credentials_status()
    if not status['all_set']:
        print("Error: Missing credentials", file=sys.stderr)
        if not (status['api_key_env'] or status['api_key_file']):
            print("  - RESY_API_KEY or RESY_API_KEY_FILE not set", file=sys.stderr)
        if not (status['auth_token_env'] or status['auth_token_file']):
            print("  - RESY_AUTH_TOKEN or RESY_AUTH_TOKEN_FILE not set", file=sys.stderr)
        print("\nSee setup guide: ~/.openclaw/workspace/skills/resy-booking/references/setup-guide.md", file=sys.stderr)
        return 1
    
    # Try to authenticate
    print("Testing authentication...")
    if auth.authenticate():
        print("✓ Authentication successful")
        
        user_info = auth.get_user_info()
        if user_info:
            print(f"\nUser Information:")
            print(f"  ID: {user_info.get('id')}")
            print(f"  Email: {user_info.get('email')}")
            print(f"  Name: {auth.get_user_name()}")
            
            # Show reservation count
            reservations = user_info.get('reservations', [])
            print(f"  Upcoming reservations: {len(reservations)}")
        
        return 0
    else:
        print("✗ Authentication failed")
        print("\nPlease check that your credentials are correct and not expired.")
        return 1


if __name__ == '__main__':
    sys.exit(main())
