#!/usr/bin/env python3
"""
List existing reservations for the authenticated user.
"""

import argparse
import json
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from datetime import datetime, date
from utils import ResyClient, ResyAPIError, parse_reservation_time, logger, ProgressSpinner


def get_user_reservations(client: ResyClient) -> list:
    """
    Get all reservations for the authenticated user.
    
    Args:
        client: ResyClient instance
    
    Returns:
        List of reservation dictionaries
    """
    try:
        response = client.get('/2/user')
        data = response.json()
        
        reservations = data.get('reservations', [])
        return reservations
        
    except ResyAPIError:
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to get reservations: {str(e)}")


def parse_reservation(res: dict) -> dict:
    """
    Parse reservation data into a clean format.
    
    Args:
        res: Raw reservation data from API
    
    Returns:
        Cleaned reservation dict
    """
    venue = res.get('venue', {})
    
    # Extract date and time
    scheduled_date = res.get('scheduled_date', '')
    scheduled_time = res.get('scheduled_time', '')
    
    # Try to parse combined datetime if available
    start_date = res.get('start_date', {}).get('date', '')
    start_time = res.get('start_date', {}).get('time', '')
    
    parsed_date = scheduled_date or start_date
    parsed_time = scheduled_time or start_time
    
    # Format time nicely
    formatted_time = parsed_time
    if parsed_time and ':' in parsed_time:
        try:
            # Parse 24-hour time and convert to 12-hour with AM/PM
            t = datetime.strptime(parsed_time[:5], '%H:%M')
            formatted_time = t.strftime('%I:%M %p').lstrip('0')
        except ValueError:
            pass
    
    return {
        'reservation_id': res.get('reservation_id', res.get('id', 'Unknown')),
        'venue_name': venue.get('name', 'Unknown Restaurant'),
        'venue_id': venue.get('id', {}).get('resy') if isinstance(venue.get('id'), dict) else venue.get('id'),
        'date': parsed_date,
        'time': parsed_time,
        'formatted_time': formatted_time,
        'party_size': res.get('party_size', res.get('num_people', 'Unknown')),
        'status': res.get('status', 'confirmed'),
        'table_type': res.get('config', {}).get('type', ''),
        'cancellation_policy': res.get('cancellation_policy', {}),
        'special_requests': res.get('notes', ''),
    }


def filter_upcoming(reservations: list) -> list:
    """Filter to show only upcoming reservations."""
    today = date.today()
    upcoming = []
    
    for res in reservations:
        try:
            res_date = datetime.strptime(res['date'], '%Y-%m-%d').date()
            if res_date >= today:
                upcoming.append(res)
        except (ValueError, KeyError):
            logger.debug(f"Could not parse date: {res.get('date')}")
            # If we can't parse the date, include it anyway
            upcoming.append(res)
    
    return upcoming


def sort_by_date(reservations: list) -> list:
    """Sort reservations by date and time."""
    def sort_key(res):
        try:
            date_str = res.get('date', '9999-12-31')
            time_str = res.get('time', '23:59')
            return (date_str, time_str)
        except (ValueError, KeyError):
            return ('9999-12-31', '23:59')
    
    return sorted(reservations, key=sort_key)


def display_reservations(reservations: list, show_all: bool = False):
    """Display reservations in a readable format."""
    if not reservations:
        print("\nNo reservations found.")
        return
    
    # Parse and filter
    parsed = [parse_reservation(r) for r in reservations]
    
    if not show_all:
        parsed = filter_upcoming(parsed)
        if not parsed:
            print("\nNo upcoming reservations found.")
            print("Use --all to see past reservations.")
            return
    
    # Sort by date
    parsed = sort_by_date(parsed)
    
    print(f"\n{'Upcoming' if not show_all else 'All'} Reservations ({len(parsed)}):\n")
    
    current_date = None
    for res in parsed:
        # Group by date
        if res['date'] != current_date:
            current_date = res['date']
            try:
                d = datetime.strptime(current_date, '%Y-%m-%d')
                date_display = d.strftime('%A, %B %d, %Y')
            except ValueError:
                date_display = current_date
            print(f"\n{date_display}")
            print("-" * 40)
        
        print(f"  {res['formatted_time']} - {res['venue_name']}")
        print(f"    Party: {res['party_size']} | ID: {res['reservation_id']}")
        
        if res['table_type']:
            print(f"    Table: {res['table_type']}")
        
        if res['special_requests']:
            print(f"    Notes: {res['special_requests']}")
        
        if res['status'] != 'confirmed':
            print(f"    Status: {res['status']}")
        
        # Show cancellation policy
        policy = res.get('cancellation_policy', {})
        if policy.get('deadline'):
            print(f"    Cancel by: {policy['deadline']}")
    
    print("\nTo cancel a reservation:")
    print("  python3 cancel.py --reservation-id <ID>")
    print("\nTo modify a reservation:")
    print("  python3 modify.py --reservation-id <ID> --time <NEW_TIME>")


def display_summary(reservations: list):
    """Display a brief summary."""
    parsed = [parse_reservation(r) for r in reservations]
    upcoming = filter_upcoming(parsed)
    
    print(f"\nReservation Summary:")
    print(f"  Total reservations: {len(parsed)}")
    print(f"  Upcoming: {len(upcoming)}")
    
    if upcoming:
        next_res = upcoming[0]
        print(f"\n  Next reservation:")
        print(f"    {next_res['venue_name']}")
        print(f"    {next_res['date']} at {next_res['formatted_time']}")


def main():
    parser = argparse.ArgumentParser(
        description='List your Resy reservations',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s
  %(prog)s --all
  %(prog)s --json
  %(prog)s --summary
        """
    )
    
    parser.add_argument('--all', '-a', action='store_true',
                        help='Show all reservations including past ones')
    parser.add_argument('--json', action='store_true',
                        help='Output as JSON')
    parser.add_argument('--summary', '-s', action='store_true',
                        help='Show brief summary only')
    parser.add_argument('--limit', '-l', type=int,
                        help='Limit number of results shown')
    
    args = parser.parse_args()
    
    try:
        client = ResyClient()
        
        with ProgressSpinner("Fetching reservations"):
            reservations = get_user_reservations(client)
        
        # Apply limit if specified
        if args.limit:
            reservations = reservations[:args.limit]
        
        if args.json:
            print(json.dumps(reservations, indent=2))
        elif args.summary:
            display_summary(reservations)
        else:
            display_reservations(reservations, show_all=args.all)
        
        return 0
        
    except ResyAPIError as e:
        print(f"Error: {e.message}", file=sys.stderr)
        if e.status_code:
            print(f"Status Code: {e.status_code}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nOperation cancelled.", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"Unexpected error: {str(e)}", file=sys.stderr)
        logger.exception("Unexpected error in list_reservations")
        return 1


if __name__ == '__main__':
    sys.exit(main())
