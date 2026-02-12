#!/usr/bin/env python3
"""
Health check and diagnostics for Resy booking skill.
"""

import argparse
import sys
import os
import time

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from utils import ResyClient, ResyAPIError, logger


class HealthChecker:
    """Comprehensive health check for Resy skill."""
    
    def __init__(self):
        self.client = None
        self.results = []
        self.errors = []
    
    def run_all_checks(self) -> dict:
        """Run all health checks and return results."""
        print("🔍 Running Resy Skill Health Check\n")
        print("=" * 50)
        
        checks = [
            ("Environment", self.check_environment),
            ("Credentials", self.check_credentials),
            ("Network", self.check_network),
            ("API Authentication", self.check_api_auth),
            ("API Endpoints", self.check_api_endpoints),
            ("Rate Limits", self.check_rate_limits),
        ]
        
        results = {}
        for name, check_func in checks:
            print(f"\n📝 Checking {name}...")
            try:
                result = check_func()
                results[name] = result
                status = "✅" if result.get('status') == 'ok' else "⚠️"
                print(f"   {status} {result.get('message', 'OK')}")
            except Exception as e:
                results[name] = {'status': 'error', 'message': str(e)}
                print(f"   ❌ Error: {str(e)}")
        
        return results
    
    def check_environment(self) -> dict:
        """Check Python environment and dependencies."""
        import importlib
        
        required = ['requests']
        optional = ['pytz', 'tabulate', 'tqdm']
        
        missing_required = []
        missing_optional = []
        
        for pkg in required:
            try:
                importlib.import_module(pkg)
            except ImportError:
                missing_required.append(pkg)
        
        for pkg in optional:
            try:
                importlib.import_module(pkg)
            except ImportError:
                missing_optional.append(pkg)
        
        result = {
            'python_version': f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
            'required_packages': {'ok': len(missing_required) == 0, 'missing': missing_required},
            'optional_packages': {'ok': len(missing_optional) == 0, 'missing': missing_optional},
        }
        
        if missing_required:
            return {
                'status': 'error',
                'message': f"Missing required packages: {', '.join(missing_required)}",
                'details': result
            }
        
        msg = "All dependencies installed"
        if missing_optional:
            msg += f" (optional missing: {', '.join(missing_optional)})"
        
        return {
            'status': 'ok',
            'message': msg,
            'details': result
        }
    
    def check_credentials(self) -> dict:
        """Check if credentials are configured."""
        import os
        
        api_key_env = bool(os.environ.get('RESY_API_KEY'))
        api_key_file = bool(os.environ.get('RESY_API_KEY_FILE'))
        auth_token_env = bool(os.environ.get('RESY_AUTH_TOKEN'))
        auth_token_file = bool(os.environ.get('RESY_AUTH_TOKEN_FILE'))
        
        has_api_key = api_key_env or api_key_file
        has_auth_token = auth_token_env or auth_token_file
        
        result = {
            'api_key_source': 'env' if api_key_env else ('file' if api_key_file else 'none'),
            'auth_token_source': 'env' if auth_token_env else ('file' if auth_token_file else 'none'),
        }
        
        if not has_api_key or not has_auth_token:
            missing = []
            if not has_api_key:
                missing.append('RESY_API_KEY or RESY_API_KEY_FILE')
            if not has_auth_token:
                missing.append('RESY_AUTH_TOKEN or RESY_AUTH_TOKEN_FILE')
            return {
                'status': 'error',
                'message': f"Missing credentials: {', '.join(missing)}",
                'details': result
            }
        
        # Try to read files if specified
        if api_key_file:
            key_file = os.path.expanduser(os.environ['RESY_API_KEY_FILE'])
            if not os.path.exists(key_file):
                return {
                    'status': 'error',
                    'message': f"API key file not found: {key_file}",
                    'details': result
                }
            # Check permissions
            mode = os.stat(key_file).st_mode
            if mode & 0o077:
                result['warning'] = f"API key file has loose permissions ({oct(mode & 0o777)})"
        
        if auth_token_file:
            token_file = os.path.expanduser(os.environ['RESY_AUTH_TOKEN_FILE'])
            if not os.path.exists(token_file):
                return {
                    'status': 'error',
                    'message': f"Auth token file not found: {token_file}",
                    'details': result
                }
            mode = os.stat(token_file).st_mode
            if mode & 0o077:
                result['warning'] = f"Auth token file has loose permissions ({oct(mode & 0o777)})"
        
        return {
            'status': 'ok',
            'message': 'Credentials configured',
            'details': result
        }
    
    def check_network(self) -> dict:
        """Check network connectivity to Resy API."""
        import socket
        import ssl
        
        hostname = 'api.resy.com'
        port = 443
        
        try:
            # DNS resolution
            ip = socket.gethostbyname(hostname)
            
            # TCP connection
            sock = socket.create_connection((hostname, port), timeout=10)
            
            # SSL handshake
            context = ssl.create_default_context()
            with context.wrap_socket(sock, server_hostname=hostname) as ssock:
                cipher = ssock.cipher()
                version = ssock.version()
            
            sock.close()
            
            return {
                'status': 'ok',
                'message': f'Connected to {hostname} ({ip})',
                'details': {
                    'ip': ip,
                    'ssl_version': version,
                    'cipher': cipher[0] if cipher else 'unknown'
                }
            }
        except socket.gaierror:
            return {
                'status': 'error',
                'message': f'Cannot resolve {hostname} - DNS issue',
                'details': {}
            }
        except socket.timeout:
            return {
                'status': 'error',
                'message': f'Connection to {hostname} timed out',
                'details': {}
            }
        except Exception as e:
            return {
                'status': 'error',
                'message': f'Network error: {str(e)}',
                'details': {}
            }
    
    def check_api_auth(self) -> dict:
        """Check API authentication."""
        try:
            self.client = ResyClient()
            response = self.client.get('/2/user')
            data = response.json()
            
            user_id = data.get('id')
            email = data.get('email')
            first_name = data.get('first_name', '')
            last_name = data.get('last_name', '')
            
            return {
                'status': 'ok',
                'message': f'Authenticated as {first_name} {last_name} (ID: {user_id})',
                'details': {
                    'user_id': user_id,
                    'email': email,
                    'name': f"{first_name} {last_name}".strip()
                }
            }
        except ResyAPIError as e:
            return {
                'status': 'error',
                'message': f'Authentication failed: {e.message}',
                'details': {'status_code': e.status_code}
            }
        except Exception as e:
            return {
                'status': 'error',
                'message': f'Unexpected error: {str(e)}',
                'details': {}
            }
    
    def check_api_endpoints(self) -> dict:
        """Test key API endpoints."""
        if not self.client:
            return {
                'status': 'skipped',
                'message': 'Skipped (authentication required)',
                'details': {}
            }
        
        endpoints = [
            ('/2/user', 'GET', 'User data'),
            ('/3/venues', 'GET', 'Venue search'),
        ]
        
        results = []
        all_ok = True
        
        for path, method, description in endpoints:
            try:
                start = time.time()
                if method == 'GET':
                    if 'venues' in path:
                        response = self.client.get(path, params={'query': 'test', 'per_page': 1})
                    else:
                        response = self.client.get(path)
                elapsed = (time.time() - start) * 1000
                
                results.append({
                    'path': path,
                    'status': response.status_code,
                    'latency_ms': round(elapsed, 2),
                    'ok': response.status_code == 200
                })
            except Exception as e:
                results.append({
                    'path': path,
                    'status': 'error',
                    'error': str(e),
                    'ok': False
                })
                all_ok = False
        
        failed = [r for r in results if not r.get('ok')]
        
        if failed:
            return {
                'status': 'warning',
                'message': f"{len(failed)} endpoint(s) failed",
                'details': {'endpoints': results}
            }
        
        avg_latency = sum(r.get('latency_ms', 0) for r in results) / len(results)
        return {
            'status': 'ok',
            'message': f'All {len(results)} endpoints OK (avg {avg_latency:.0f}ms)',
            'details': {'endpoints': results, 'avg_latency_ms': round(avg_latency, 2)}
        }
    
    def check_rate_limits(self) -> dict:
        """Check current rate limit status."""
        # Resy doesn't expose rate limit headers consistently
        # But we can track our own request count
        return {
            'status': 'ok',
            'message': 'Rate limit tracking active (see logs for usage)',
            'details': {
                'note': 'Resy rate limits are not exposed via API headers'
            }
        }
    
    def print_summary(self, results: dict):
        """Print summary of health check results."""
        print("\n" + "=" * 50)
        print("📊 HEALTH CHECK SUMMARY")
        print("=" * 50)
        
        ok_count = sum(1 for r in results.values() if r.get('status') == 'ok')
        warning_count = sum(1 for r in results.values() if r.get('status') == 'warning')
        error_count = sum(1 for r in results.values() if r.get('status') == 'error')
        skipped_count = sum(1 for r in results.values() if r.get('status') == 'skipped')
        
        print(f"\n✅ Passed: {ok_count}")
        if warning_count:
            print(f"⚠️  Warnings: {warning_count}")
        if error_count:
            print(f"❌ Errors: {error_count}")
        if skipped_count:
            print(f"⏭️  Skipped: {skipped_count}")
        
        if error_count == 0:
            print("\n🎉 All critical checks passed! Skill is ready to use.")
        elif error_count > 0:
            print("\n⚠️  Some checks failed. Please review the errors above.")
        
        print("=" * 50)


def main():
    parser = argparse.ArgumentParser(
        description='Health check and diagnostics for Resy booking skill',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s              # Run all checks
  %(prog)s --json       # Output as JSON
        """
    )
    
    parser.add_argument('--json', action='store_true',
                        help='Output results as JSON')
    parser.add_argument('--quiet', '-q', action='store_true',
                        help='Minimal output')
    
    args = parser.parse_args()
    
    checker = HealthChecker()
    results = checker.run_all_checks()
    
    if args.json:
        import json
        print(json.dumps(results, indent=2))
    elif not args.quiet:
        checker.print_summary(results)
    
    # Exit with error code if any critical checks failed
    error_count = sum(1 for r in results.values() if r.get('status') == 'error')
    return 1 if error_count > 0 else 0


if __name__ == '__main__':
    sys.exit(main())
