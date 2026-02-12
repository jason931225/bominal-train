#!/usr/bin/env python3
"""
Waitlist management for Resy restaurants.
"""

import argparse
import json
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from utils import (
    ResyClient, ResyAPIError,
    validate_date, validate_time, validate_party_size, validate_venue_id,
    logger
)


def check_waitlist_availability(
    client: ResyClient,
    venue_id: int,
    date: str,
    party_size: int
) -> dict:
    """
    Check if waitlist is available for a restaurant.
    
    Args:
        client: ResyClient instance
        venue_id: Restaurant venue ID
        date: Date in YYYY-MM-DD format
        party_size: Number of guests
    
    Returns:
        Waitlist availability information
    """
    params = {
        'venue_id': venue_id,
        'day': date,
        'party_size': party_size,
    }
    
    try:
        # Check if there's a waitlist endpoint
        response = client.get('/3/waitlist/availability', params=params)
        return response.json()
    except ResyAPIError as e:
        if e.status_code == 404:
            # Waitlist not available or different endpoint
            return {'available': False, 'message': 'Waitlist not available for this venue'}
        raise
    except Exception as e:
        # Graceful fallback if endpoint doesn't exist
        logger.debug(f"Waitlist check failed: {e}")
        return {'available': False, 'message': str(e)}


def join_waitlist(
    client: ResyClient,
    venue_id: int,
    date: str,
    time: str,
    party_size: int,
    email: str = None,
    phone: str = None,
    notes: str = None
) -> dict:
    """
    Join a restaurant's waitlist.
    
    Args:
        client: ResyClient instance
        venue_id: Restaurant venue ID
        date: Date in YYYY-MM-DD format
        time: Preferred time in HH:MM format
        party_size: Number of guests
        email: Contact email (optional, uses account default)
        phone: Contact phone (optional, uses account default)
        notes: Special requests or notes
    
    Returns:
        Waitlist entry details
    """
    data = {
        'venue_id': venue_id,
        'day': date,
        'time': time,
        'party_size': party_size,
    }
    
    if email:
        data['email'] = email
    if phone:
        data['phone'] = phone
    if notes:
        data['notes'] = notes
    
    try:
        response = client.post('/3/waitlist', data=data)
        return response.json()
    except ResyAPIError:
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to join waitlist: {str(e)}")


def get_waitlist_status(client: ResyClient, waitlist_id: str) -> dict:
    """
    Check status of a waitlist entry.
    
    Args:
        client: ResyClient instance
        waitlist_id: Waitlist entry ID
    
    Returns:
        Waitlist status details
    """
    try:
        response = client.get(f'/3/waitlist/{waitlist_id}')
        return response.json()
    except ResyAPIError:
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to get waitlist status: {str(e)}")


def cancel_waitlist(client: ResyClient, waitlist_id: str) -> dict:
    """
    Cancel a waitlist entry.
    
    Args:
        client: ResyClient instance
        waitlist_id: Waitlist entry ID to cancel
    
    Returns:
        Cancellation response
    """
    try:
        response = client.post(f'/3/waitlist/{waitlist_id}/cancel')
        return response.json()
    except ResyAPIError as e:
        if e.status_code == 404:
            raise ResyAPIError(
                f"Waitlist entry '{waitlist_id}' not found. "
                "It may have already been cancelled or expired.",
                status_code=404
            )
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to cancel waitlist: {str(e)}")


def display_waitlist_entry(entry: dict):
    """Display waitlist entry details."""
    print("\n" + "=" * 50)
    print("Waitlist Entry")
    print("=" * 50)
    
    print(f"\nWaitlist ID: {entry.get('id', 'N/A')}")
    
    if entry.get('venue_name'):
        print(f"Restaurant: {entry['venue_name']}")
    
    if entry.get('date'):
        print(f"Date: {entry['date']}")
    
    if entry.get('time'):
        print(f"Preferred Time: {entry['time']}")
    
    if entry.get('party_size'):
        print(f"Party Size: {entry['party_size']}")
    
    status = entry.get('status', 'unknown')
    print(f"Status: {status}")
    
    if entry.get('position'):
        print(f"Position: #{entry['position']} on waitlist")
    
    if entry.get('estimated_wait'):
        print(f"Estimated Wait: {entry['estimated_wait']}")
    
    print("=" * 50)


def main():
    parser = argparse.ArgumentParser(
        description='Manage Resy restaurant waitlists',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Check if waitlist is available
  %(prog)s check --venue-id 1505 --date 2024-12-25 --party-size 2
  
  # Join a waitlist
  %(prog)s join --venue-id 1505 --date 2024-12-25 --time 19:00 --party-size 2
  
  # Check waitlist status
  %(prog)s status --waitlist-id wl_abc123
  
  # Cancel waitlist entry
  %(prog)s cancel --waitlist-id wl_abc123
        """
    )
    
    subparsers = parser.add_subparsers(dest='command', help='Command to run')
    
    # Check command
    check_parser = subparsers.add_parser('check', help='Check waitlist availability')
    check_parser.add_argument('--venue-id', '-v', required=True, type=int,
                              help='Restaurant venue ID')
    check_parser.add_argument('--date', '-d', required=True,
                              help='Date in YYYY-MM-DD format')
    check_parser.add_argument('--party-size', '-p', required=True, type=int,
                              help='Number of guests (1-20)')
    
    # Join command
    join_parser = subparsers.add_parser('join', help='Join a waitlist')
    join_parser.add_argument('--venue-id', '-v', required=True, type=int,
                             help='Restaurant venue ID')
    join_parser.add_argument('--date', '-d', required=True,
                             help='Date in YYYY-MM-DD format')
    join_parser.add_argument('--time', '-t', required=True,
                             help='Preferred time in HH:MM format')
    join_parser.add_argument('--party-size', '-p', required=True, type=int,
                             help='Number of guests (1-20)')
    join_parser.add_argument('--email', '-e',
                             help='Contact email (optional)')
    join_parser.add_argument('--phone',
                             help='Contact phone (optional)')
    join_parser.add_argument('--notes', '-n',
                             help='Special requests or notes')
    join_parser.add_argument('--json', action='store_true',
                             help='Output as JSON')
    join_parser.add_argument('--yes', '-y', action='store_true',
                             help='Skip confirmation prompt')
    
    # Status command
    status_parser = subparsers.add_parser('status', help='Check waitlist status')
    status_parser.add_argument('--waitlist-id', '-w', required=True,
                               help='Waitlist entry ID')
    status_parser.add_argument('--json', action='store_true',
                               help='Output as JSON')
    
    # Cancel command
    cancel_parser = subparsers.add_parser('cancel', help='Cancel waitlist entry')
    cancel_parser.add_argument('--waitlist-id', '-w', required=True,
                               help='Waitlist entry ID to cancel')
    cancel_parser.add_argument('--json', action='store_true',
                               help='Output as JSON')
    cancel_parser.add_argument('--yes', '-y', action='store_true',
                               help='Skip confirmation prompt')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    try:
        client = ResyClient()
        
        if args.command == 'check':
            # Validate inputs
            try:
                validate_venue_id(args.venue_id)
                validate_date(args.date)
                validate_party_size(args.party_size)
            except ValueError as e:
                print(f"Validation error: {e}", file=sys.stderr)
                return 1
            
            result = check_waitlist_availability(
                client=client,
                venue_id=args.venue_id,
                date=args.date,
                party_size=args.party_size
            )
            
            if result.get('available'):
                print(f"\n✓ Waitlist is available for this restaurant!")
                print(f"To join: python3 waitlist.py join --venue-id {args.venue_id} --date {args.date} --time HH:MM --party-size {args.party_size}")
            else:
                print(f"\n✗ Waitlist not available")
                if result.get('message'):
                    print(f"Reason: {result['message']}")
            
            if args.json:
                print(json.dumps(result, indent=2))
        
        elif args.command == 'join':
            # Validate inputs
            try:
                validate_venue_id(args.venue_id)
                validate_date(args.date)
                validate_time(args.time)
                validate_party_size(args.party_size)
            except ValueError as e:
                print(f"Validation error: {e}", file=sys.stderr)
                return 1
            
            # Confirmation
            if not args.yes:
                print(f"\nJoining waitlist:")
                print(f"  Venue ID: {args.venue_id}")
                print(f"  Date: {args.date}")
                print(f"  Preferred Time: {args.time}")
                print(f"  Party Size: {args.party_size}")
                
                try:
                    confirm = input("\nConfirm? (yes/no): ").strip().lower()
                    if confirm not in ('yes', 'y'):
                        print("Cancelled.")
                        return 0
                except (EOFError, KeyboardInterrupt):
                    print("\nCancelled.")
                    return 0
            
            result = join_waitlist(
                client=client,
                venue_id=args.venue_id,
                date=args.date,
                time=args.time,
                party_size=args.party_size,
                email=args.email,
                phone=args.phone,
                notes=args.notes
            )
            
            if args.json:
                print(json.dumps(result, indent=2))
            else:
                display_waitlist_entry(result)
            
            print("\nYou'll be notified if a table becomes available!")
        
        elif args.command == 'status':
            result = get_waitlist_status(client, args.waitlist_id)
            
            if args.json:
                print(json.dumps(result, indent=2))
            else:
                display_waitlist_entry(result)
        
        elif args.command == 'cancel':
            if not args.yes:
                print(f"\nYou are about to cancel waitlist entry: {args.waitlist_id}")
                
                try:
                    confirm = input("Type 'cancel' to confirm: ").strip().lower()
                    if confirm != 'cancel':
                        print("Cancelled.")
                        return 0
                except (EOFError, KeyboardInterrupt):
                    print("\nCancelled.")
                    return 0
            
            result = cancel_waitlist(client, args.waitlist_id)
            
            if args.json:
                print(json.dumps(result, indent=2))
            else:
                print(f"\n✓ Waitlist entry {args.waitlist_id} cancelled successfully.")
        
        return 0
        
    except ResyAPIError as e:
        print(f"\nError: {e.message}", file=sys.stderr)
        if e.status_code:
            print(f"Status Code: {e.status_code}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nOperation cancelled.", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"\nUnexpected error: {str(e)}", file=sys.stderr)
        logger.exception("Unexpected error in waitlist")
        return 1


if __name__ == '__main__':
    sys.exit(main())
