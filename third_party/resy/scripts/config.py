#!/usr/bin/env python3
"""
Configuration management for Resy booking skill.
"""

import os
import json
import sys
from pathlib import Path
from typing import Dict, Any, Optional

from utils import ResyAPIError, logger


class Config:
    """Configuration manager for Resy skill."""
    
    DEFAULT_CONFIG_PATHS = [
        '~/.resy-config.json',
        '~/.config/resy/config.json',
        './.resy-config.json',
    ]
    
    def __init__(self, config_path: Optional[str] = None):
        self._config: Dict[str, Any] = {}
        self._config_path: Optional[str] = None
        
        # Load from first available config file
        if config_path:
            self.load_config(config_path)
        else:
            self._load_default_config()
    
    def _load_default_config(self):
        """Try to load config from default paths."""
        for path in self.DEFAULT_CONFIG_PATHS:
            expanded = os.path.expanduser(path)
            if os.path.exists(expanded):
                try:
                    self.load_config(expanded)
                    break
                except ResyAPIError:
                    continue
    
    def load_config(self, path: str):
        """Load configuration from JSON file."""
        expanded_path = os.path.expanduser(path)
        
        try:
            with open(expanded_path, 'r') as f:
                self._config = json.load(f)
            self._config_path = expanded_path
            logger.info(f"Loaded config from {expanded_path}")
        except FileNotFoundError:
            raise ResyAPIError(f"Config file not found: {path}")
        except json.JSONDecodeError as e:
            raise ResyAPIError(f"Invalid JSON in config file {path}: {str(e)}")
        except Exception as e:
            raise ResyAPIError(f"Failed to load config from {path}: {str(e)}")
    
    def save_config(self, path: Optional[str] = None):
        """Save current configuration to file."""
        save_path = path or self._config_path or os.path.expanduser('~/.resy-config.json')
        
        # Ensure directory exists
        os.makedirs(os.path.dirname(save_path), exist_ok=True)
        
        try:
            with open(save_path, 'w') as f:
                json.dump(self._config, f, indent=2)
            logger.info(f"Saved config to {save_path}")
        except Exception as e:
            raise ResyAPIError(f"Failed to save config: {str(e)}")
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        # Check environment first
        env_key = f'RESY_{key.upper()}'
        if env_key in os.environ:
            return os.environ[env_key]
        
        # Then check config file
        return self._config.get(key, default)
    
    def set(self, key: str, value: Any):
        """Set a configuration value."""
        self._config[key] = value
    
    @property
    def api_key(self) -> Optional[str]:
        """Get API key from config or environment."""
        return self.get('api_key') or os.environ.get('RESY_API_KEY')
    
    @property
    def auth_token(self) -> Optional[str]:
        """Get auth token from config or environment."""
        return self.get('auth_token') or os.environ.get('RESY_AUTH_TOKEN')
    
    @property
    def timezone(self) -> str:
        """Get default timezone."""
        return self.get('timezone', 'America/New_York')
    
    @property
    def default_party_size(self) -> int:
        """Get default party size."""
        return int(self.get('default_party_size', 2))
    
    @property
    def auto_confirm(self) -> bool:
        """Whether to skip confirmation prompts."""
        return self.get('auto_confirm', 'false').lower() == 'true'
    
    @property
    def notification_email(self) -> Optional[str]:
        """Email for notifications."""
        return self.get('notification_email')
    
    def setup_wizard(self):
        """Interactive configuration setup."""
        print("\n=== Resy Configuration Setup ===\n")
        
        print("This will create a configuration file with your Resy credentials.")
        print("Note: Environment variables will still take precedence.\n")
        
        # API Key
        current_key = self.api_key
        if current_key:
            masked = current_key[:4] + '...' + current_key[-4:] if len(current_key) > 8 else '****'
            print(f"Current API Key: {masked}")
        
        new_key = input("API Key (press Enter to keep current): ").strip()
        if new_key:
            self.set('api_key', new_key)
        
        # Auth Token
        current_token = self.auth_token
        if current_token:
            masked = current_token[:4] + '...' + current_token[-4:] if len(current_token) > 8 else '****'
            print(f"\nCurrent Auth Token: {masked}")
        
        new_token = input("Auth Token (press Enter to keep current): ").strip()
        if new_token:
            self.set('auth_token', new_token)
        
        # Timezone
        print(f"\nDefault timezone: {self.timezone}")
        new_tz = input("Timezone (press Enter to keep current): ").strip()
        if new_tz:
            self.set('timezone', new_tz)
        
        # Default party size
        print(f"\nDefault party size: {self.default_party_size}")
        new_size = input("Default party size (press Enter to keep current): ").strip()
        if new_size:
            try:
                self.set('default_party_size', int(new_size))
            except ValueError:
                print("Invalid number, keeping current value.")
        
        # Save
        save_path = input(f"\nSave to [{self._config_path or '~/.resy-config.json'}]: ").strip()
        if not save_path:
            save_path = self._config_path or '~/.resy-config.json'
        
        try:
            self.save_config(save_path)
            print(f"\n✓ Configuration saved to {save_path}")
            print("\nSecurity tip: Make sure this file has restricted permissions:")
            print(f"  chmod 600 {save_path}")
        except ResyAPIError as e:
            print(f"Error saving config: {e.message}")
            return False
        
        return True


def get_config() -> Config:
    """Get global configuration instance."""
    return Config()


if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(description='Resy configuration management')
    parser.add_argument('--setup', action='store_true', help='Run configuration setup wizard')
    parser.add_argument('--show', action='store_true', help='Show current configuration')
    parser.add_argument('--path', help='Path to config file')
    
    args = parser.parse_args()
    
    config = Config(args.path) if args.path else Config()
    
    if args.setup:
        config.setup_wizard()
    elif args.show:
        print(f"Config path: {config._config_path or 'Not loaded'}")
        print(f"API Key: {'Set' if config.api_key else 'Not set'}")
        print(f"Auth Token: {'Set' if config.auth_token else 'Not set'}")
        print(f"Timezone: {config.timezone}")
        print(f"Default Party Size: {config.default_party_size}")
    else:
        parser.print_help()
