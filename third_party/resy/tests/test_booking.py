#!/usr/bin/env python3
"""
Unit tests for Resy booking skill.
"""

import unittest
import sys
import os
from datetime import date, timedelta
from unittest.mock import Mock, patch, MagicMock

# Add scripts directory to path
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

from scripts.utils import (
    validate_date, validate_time, validate_party_size, validate_venue_id,
    ResyClient, ResyAPIError, format_datetime, parse_reservation_time
)


class TestValidation(unittest.TestCase):
    """Test input validation functions."""
    
    def test_validate_date_valid(self):
        """Test valid date validation."""
        future_date = (date.today() + timedelta(days=1)).strftime('%Y-%m-%d')
        result = validate_date(future_date)
        self.assertEqual(result, future_date)
    
    def test_validate_date_invalid_format(self):
        """Test invalid date format."""
        with self.assertRaises(ValueError) as ctx:
            validate_date('12-25-2024')
        self.assertIn('YYYY-MM-DD', str(ctx.exception))
    
    def test_validate_date_past(self):
        """Test past date rejection."""
        past_date = (date.today() - timedelta(days=1)).strftime('%Y-%m-%d')
        with self.assertRaises(ValueError) as ctx:
            validate_date(past_date)
        self.assertIn('future', str(ctx.exception))
    
    def test_validate_date_empty(self):
        """Test empty date rejection."""
        with self.assertRaises(ValueError):
            validate_date('')
    
    def test_validate_time_valid(self):
        """Test valid time validation."""
        result = validate_time('19:00')
        self.assertEqual(result, '19:00')
    
    def test_validate_time_with_seconds(self):
        """Test time with seconds."""
        result = validate_time('19:00:00')
        self.assertEqual(result, '19:00:00')
    
    def test_validate_time_invalid_format(self):
        """Test invalid time format."""
        with self.assertRaises(ValueError) as ctx:
            validate_time('7:00 PM')
        self.assertIn('HH:MM', str(ctx.exception))
    
    def test_validate_time_invalid_value(self):
        """Test invalid time value."""
        with self.assertRaises(ValueError):
            validate_time('25:00')
    
    def test_validate_party_size_valid(self):
        """Test valid party sizes."""
        self.assertEqual(validate_party_size(1), 1)
        self.assertEqual(validate_party_size(10), 10)
        self.assertEqual(validate_party_size(20), 20)
        self.assertEqual(validate_party_size("5"), 5)
    
    def test_validate_party_size_too_small(self):
        """Test party size too small."""
        with self.assertRaises(ValueError) as ctx:
            validate_party_size(0)
        self.assertIn('at least 1', str(ctx.exception))
    
    def test_validate_party_size_too_large(self):
        """Test party size too large."""
        with self.assertRaises(ValueError) as ctx:
            validate_party_size(21)
        self.assertIn('20', str(ctx.exception))
    
    def test_validate_party_size_invalid_type(self):
        """Test invalid party size type."""
        with self.assertRaises(ValueError):
            validate_party_size('abc')
    
    def test_validate_venue_id_valid(self):
        """Test valid venue IDs."""
        self.assertEqual(validate_venue_id(1505), 1505)
        self.assertEqual(validate_venue_id("1505"), 1505)
    
    def test_validate_venue_id_invalid(self):
        """Test invalid venue IDs."""
        with self.assertRaises(ValueError):
            validate_venue_id(0)
        with self.assertRaises(ValueError):
            validate_venue_id(-1)
        with self.assertRaises(ValueError):
            validate_venue_id('abc')


class TestDateTimeFormatting(unittest.TestCase):
    """Test date/time formatting functions."""
    
    def test_format_datetime(self):
        """Test datetime formatting."""
        result = format_datetime('2024-12-25', '19:00')
        self.assertEqual(result, '2024-12-25 19:00:00')
    
    def test_parse_reservation_time(self):
        """Test parsing reservation time."""
        date_str, time_str = parse_reservation_time('2024-12-25 19:00:00')
        self.assertEqual(date_str, '2024-12-25')
        self.assertEqual(time_str, '19:00')


class TestResyAPIError(unittest.TestCase):
    """Test ResyAPIError exception."""
    
    def test_error_message(self):
        """Test error message."""
        err = ResyAPIError("Test error")
        self.assertEqual(str(err), "Test error")
        self.assertEqual(err.message, "Test error")
    
    def test_error_with_status(self):
        """Test error with status code."""
        err = ResyAPIError("Not found", status_code=404)
        self.assertEqual(err.status_code, 404)


class TestResyClient(unittest.TestCase):
    """Test ResyClient class."""
    
    @patch.dict(os.environ, {
        'RESY_API_KEY': 'test_api_key_12345',
        'RESY_AUTH_TOKEN': 'test_auth_token_67890'
    })
    def test_client_initialization(self):
        """Test client initialization with env vars."""
        client = ResyClient()
        self.assertEqual(client.api_key, 'test_api_key_12345')
        self.assertEqual(client.auth_token, 'test_auth_token_67890')
    
    @patch.dict(os.environ, {}, clear=True)
    def test_client_missing_api_key(self):
        """Test client with missing API key."""
        with self.assertRaises(ResyAPIError) as ctx:
            ResyClient()
        self.assertIn('RESY_API_KEY', str(ctx.exception))
    
    @patch.dict(os.environ, {'RESY_API_KEY': 'test_key'}, clear=True)
    def test_client_missing_auth_token(self):
        """Test client with missing auth token."""
        with self.assertRaises(ResyAPIError) as ctx:
            ResyClient()
        self.assertIn('RESY_AUTH_TOKEN', str(ctx.exception))
    
    @patch.dict(os.environ, {
        'RESY_API_KEY': 'test_key',
        'RESY_AUTH_TOKEN': 'test_token'
    })
    def test_headers_generation(self):
        """Test header generation."""
        client = ResyClient()
        headers = client._get_headers()
        
        self.assertIn('Authorization', headers)
        self.assertIn('X-Resy-Auth-Token', headers)
        self.assertIn('test_key', headers['Authorization'])
        self.assertEqual(headers['X-Resy-Auth-Token'], 'test_token')
    
    @patch.dict(os.environ, {
        'RESY_API_KEY': 'test_key',
        'RESY_AUTH_TOKEN': 'test_token'
    })
    def test_url_validation_valid(self):
        """Test URL validation with valid URLs."""
        client = ResyClient()
        
        # Test with full URL
        result = client._validate_url('https://api.resy.com/2/user')
        self.assertEqual(result, 'https://api.resy.com/2/user')
        
        # Test with path only
        result = client._validate_url('/2/user')
        self.assertEqual(result, 'https://api.resy.com/2/user')
    
    @patch.dict(os.environ, {
        'RESY_API_KEY': 'test_key',
        'RESY_AUTH_TOKEN': 'test_token'
    })
    def test_url_validation_invalid_host(self):
        """Test URL validation with invalid host."""
        client = ResyClient()
        
        with self.assertRaises(ResyAPIError) as ctx:
            client._validate_url('https://evil.com/api')
        self.assertIn('Invalid API host', str(ctx.exception))
    
    @patch.dict(os.environ, {
        'RESY_API_KEY': 'test_key',
        'RESY_AUTH_TOKEN': 'test_token'
    })
    def test_mask_sensitive_data(self):
        """Test sensitive data masking."""
        client = ResyClient()
        
        data = {
            'book_token': 'secret_token_123',
            'config_id': 'config_456',
            'name': 'Restaurant Name',
            'nested': {
                'token': 'nested_secret',
                'value': 'visible'
            }
        }
        
        masked = client._mask_sensitive_data(data)
        
        self.assertEqual(masked['book_token'], '***MASKED***')
        self.assertEqual(masked['config_id'], '***MASKED***')
        self.assertEqual(masked['name'], 'Restaurant Name')
        self.assertEqual(masked['nested']['token'], '***MASKED***')
        self.assertEqual(masked['nested']['value'], 'visible')


class TestSecurity(unittest.TestCase):
    """Test security features."""
    
    @patch.dict(os.environ, {
        'RESY_API_KEY': 'test_key',
        'RESY_AUTH_TOKEN': 'test_token'
    })
    def test_no_hardcoded_credentials(self):
        """Verify no hardcoded credentials in client."""
        client = ResyClient()
        
        # Credentials should only come from env vars
        self.assertEqual(client.api_key, os.environ['RESY_API_KEY'])
        self.assertEqual(client.auth_token, os.environ['RESY_AUTH_TOKEN'])
    
    @patch.dict(os.environ, {
        'RESY_API_KEY': 'test_key',
        'RESY_AUTH_TOKEN': 'test_token'
    })
    def test_sanitize_log_url(self):
        """Test URL sanitization for logging."""
        client = ResyClient()
        
        # URL with query params should be truncated
        result = client._sanitize_log_url('https://api.resy.com/find?token=secret')
        self.assertEqual(result, 'https://api.resy.com/find?...')
        
        # URL without query params should remain unchanged
        result = client._sanitize_log_url('https://api.resy.com/user')
        self.assertEqual(result, 'https://api.resy.com/user')


class TestAuthModule(unittest.TestCase):
    """Test auth module functions."""
    
    def test_check_credentials_set_both_set(self):
        """Test when both credentials are set."""
        with patch.dict(os.environ, {
            'RESY_API_KEY': 'test_key',
            'RESY_AUTH_TOKEN': 'test_token'
        }):
            from scripts.auth import ResyAuth
            self.assertTrue(ResyAuth.check_credentials_set())
    
    def test_check_credentials_set_only_one(self):
        """Test when only one credential is set."""
        with patch.dict(os.environ, {'RESY_API_KEY': 'test_key'}, clear=True):
            from scripts.auth import ResyAuth
            self.assertFalse(ResyAuth.check_credentials_set())
    
    def test_check_credentials_set_none(self):
        """Test when no credentials are set."""
        with patch.dict(os.environ, {}, clear=True):
            from scripts.auth import ResyAuth
            self.assertFalse(ResyAuth.check_credentials_set())
    
    def test_get_credentials_status(self):
        """Test credential status check."""
        with patch.dict(os.environ, {
            'RESY_API_KEY': 'test_key',
            'RESY_AUTH_TOKEN': 'test_token'
        }):
            from scripts.auth import ResyAuth
            status = ResyAuth.get_credentials_status()
            self.assertTrue(status['api_key_set'])
            self.assertTrue(status['auth_token_set'])
            self.assertTrue(status['all_set'])


class TestIntegration(unittest.TestCase):
    """Integration-style tests."""
    
    def test_end_to_end_validation(self):
        """Test complete validation flow."""
        future_date = (date.today() + timedelta(days=7)).strftime('%Y-%m-%d')
        
        # Should not raise
        validated_date = validate_date(future_date)
        validated_time = validate_time('19:00')
        validated_party = validate_party_size(4)
        validated_venue = validate_venue_id(1505)
        
        self.assertEqual(validated_date, future_date)
        self.assertEqual(validated_time, '19:00')
        self.assertEqual(validated_party, 4)
        self.assertEqual(validated_venue, 1505)


def run_tests():
    """Run all tests."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    
    # Add all test classes
    suite.addTests(loader.loadTestsFromTestCase(TestValidation))
    suite.addTests(loader.loadTestsFromTestCase(TestDateTimeFormatting))
    suite.addTests(loader.loadTestsFromTestCase(TestResyAPIError))
    suite.addTests(loader.loadTestsFromTestCase(TestResyClient))
    suite.addTests(loader.loadTestsFromTestCase(TestSecurity))
    suite.addTests(loader.loadTestsFromTestCase(TestAuthModule))
    suite.addTests(loader.loadTestsFromTestCase(TestIntegration))
    
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    return 0 if result.wasSuccessful() else 1


if __name__ == '__main__':
    sys.exit(run_tests())
