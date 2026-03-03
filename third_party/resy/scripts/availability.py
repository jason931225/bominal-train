#!/usr/bin/env python3
"""
Check availability for a restaurant.
"""

import argparse
import json
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from datetime import datetime
from utils import (
    ResyClient, ResyAPIError, 
    validate_date, validate_party_size, validate_venue_id,
    parse_reservation_time, check_availability, parse_availability,
    logger, ProgressSpinner
)


def display_availability(slots: list, venue_id: int, date: str, party_size: int):
    """Display availability in a readable format."""
    if not slots:
        print(f"\nNo availability found for venue {venue_id} on {date} for {party_size} guests.")
        print("\nTips:")
        print("  - Try a different date")
        print("  - Some restaurants release reservations at specific times (often 9 AM, 30 days out)")
        print("  - Check if the restaurant offers walk-in seating")
        print("  - Try joining the waitlist if available")
        return
    
    print(f"\nAvailable times for {slots[0].get('venue_name', f'Venue {venue_id}')} on {date}:")
    print(f"Party size: {party_size}")
    print(f"Found {len(slots)} available slot(s):\n")
    
    # Group by time
    by_time: dict = {}
    for slot in slots:
        time = slot.get('time', 'Unknown')
        if time not in by_time:
            by_time[time] = []
        by_time[time].append(slot)
    
    # Display sorted by time
    for time in sorted(by_time.keys()):
        slots_at_time = by_time[time]
        types = [s['type'] for s in slots_at_time if s['type']]
        type_str = f" ({', '.join(types)})" if types else ""
        
        # Format time nicely
        try:
            dt = datetime.strptime(time, '%H:%M')
            formatted_time = dt.strftime('%I:%M %p').lstrip('0')
        except ValueError:
            formatted_time = time
        
        print(f"  {formatted_time}{type_str}")
    
    print("\nTo book a reservation, use:")
    print(f"  python3 book.py --venue-id {venue_id} --date {date} --time HH:MM --party-size {party_size}")
    
    # Suggest waitlist if no availability
    if len(slots) == 0:
        print("\nTo join the waitlist, use:")
        print(f"  python3 waitlist.py --venue-id {venue_id} --date {date} --party-size {party_size}")


def main():
    parser = argparse.ArgumentParser(
        description='Check availability for a restaurant on Resy',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --venue-id 1505 --date 2024-12-25 --party-size 2
  %(prog)s --venue-id 1505 --date 2024-12-25 --party-size 4 --json
        """
    )
    
    parser.add_argument('--venue-id', '-v', required=True, type=int,
                        help='Restaurant venue ID')
    parser.add_argument('--date', '-d', required=True,
                        help='Date in YYYY-MM-DD format')
    parser.add_argument('--party-size', '-p', required=True, type=int,
                        help='Number of guests (1-20)')
    parser.add_argument('--lat', type=float,
                        help='Latitude (optional)')
    parser.add_argument('--long', type=float,
                        help='Longitude (optional)')
    parser.add_argument('--json', action='store_true',
                        help='Output raw JSON response')
    parser.add_argument('--slots-only', action='store_true',
                        help='Output only the parsed slots as JSON')
    
    args = parser.parse_args()
    
    # Validate inputs
    try:
        validate_venue_id(args.venue_id)
        validate_date(args.date)
        validate_party_size(args.party_size)
    except ValueError as e:
        print(f"Validation error: {e}", file=sys.stderr)
        return 1
    
    if (args.lat is not None and args.long is None) or (args.lat is None and args.long is not None):
        print("Error: Must provide both --lat and --long together", file=sys.stderr)
        return 1
    
    try:
        client = ResyClient()
        
        with ProgressSpinner(f"Checking availability for venue {args.venue_id}"):
            data = check_availability(
                client=client,
                venue_id=args.venue_id,
                date=args.date,
                party_size=args.party_size,
                lat=args.lat,
                long=args.long
            )
        
        if args.json:
            print(json.dumps(data, indent=2))
        elif args.slots_only:
            slots = parse_availability(data)
            print(json.dumps(slots, indent=2))
        else:
            slots = parse_availability(data)
            display_availability(slots, args.venue_id, args.date, args.party_size)
        
        return 0
        
    except ResyAPIError as e:
        print(f"Error: {e.message}", file=sys.stderr)
        if e.status_code:
            print(f"Status Code: {e.status_code}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nCheck cancelled.", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"Unexpected error: {str(e)}", file=sys.stderr)
        logger.exception("Unexpected error in availability check")
        return 1


if __name__ == '__main__':
    sys.exit(main())
