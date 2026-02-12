#!/usr/bin/env python3
"""
Modify existing reservations on Resy.
"""

import argparse
import json
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from utils import (
    ResyClient, ResyAPIError,
    validate_date, validate_time, validate_party_size,
    logger
)


def get_reservation_details(client: ResyClient, reservation_id: str) -> dict:
    """
    Get details of an existing reservation.
    
    Args:
        client: ResyClient instance
        reservation_id: Reservation ID
    
    Returns:
        Reservation details
    """
    try:
        # Try to get from user reservations list
        response = client.get('/2/user')
        data = response.json()
        
        for res in data.get('reservations', []):
            res_id = res.get('reservation_id') or res.get('id')
            if res_id == reservation_id:
                return res
        
        raise ResyAPIError(f"Reservation '{reservation_id}' not found", status_code=404)
        
    except ResyAPIError:
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to get reservation details: {str(e)}")


def modify_reservation(
    client: ResyClient,
    reservation_id: str,
    new_date: str = None,
    new_time: str = None,
    new_party_size: int = None,
    notes: str = None
) -> dict:
    """
    Modify an existing reservation.
    
    Note: Resy API may require cancel + rebook for some modifications.
    This attempts to use the modify endpoint if available.
    
    Args:
        client: ResyClient instance
        reservation_id: Reservation ID to modify
        new_date: New date (YYYY-MM-DD)
        new_time: New time (HH:MM)
        new_party_size: New party size
        notes: Updated special requests
    
    Returns:
        Updated reservation details
    """
    data = {
        'reservation_id': reservation_id,
    }
    
    if new_date:
        data['date'] = new_date
    if new_time:
        data['time'] = new_time
    if new_party_size:
        data['party_size'] = new_party_size
    if notes:
        data['notes'] = notes
    
    try:
        # Try the modify endpoint
        response = client.post('/3/reservation/modify', data=data)
        return response.json()
        
    except ResyAPIError as e:
        if e.status_code == 404:
            # Modify endpoint not available, suggest cancel + rebook
            raise ResyAPIError(
                "Direct modification not available for this reservation. "
                "You may need to cancel and create a new reservation.\n\n"
                f"To cancel: python3 cancel.py --reservation-id {reservation_id}\n"
                f"Then book: python3 book.py --venue-id <VENUE_ID> --date {new_date or 'DATE'} "
                f"--time {new_time or 'TIME'} --party-size {new_party_size or 'SIZE'}",
                status_code=404
            )
        elif e.status_code == 409:
            raise ResyAPIError(
                "Cannot modify - the new time/date is no longer available. "
                "Please check availability first.",
                status_code=409
            )
        elif e.status_code == 403:
            raise ResyAPIError(
                "Modification not allowed. The modification window may have passed "
                "or this reservation type cannot be modified.",
                status_code=403
            )
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to modify reservation: {str(e)}")


def modify_party_size(client: ResyClient, reservation_id: str, new_size: int) -> dict:
    """
    Modify just the party size.
    
    Args:
        client: ResyClient instance
        reservation_id: Reservation ID
        new_size: New party size
    
    Returns:
        Updated reservation
    """
    return modify_reservation(client, reservation_id, new_party_size=new_size)


def modify_time(client: ResyClient, reservation_id: str, new_time: str) -> dict:
    """
    Modify just the time.
    
    Args:
        client: ResyClient instance
        reservation_id: Reservation ID
        new_time: New time (HH:MM)
    
    Returns:
        Updated reservation
    """
    return modify_reservation(client, reservation_id, new_time=new_time)


def modify_date(client: ResyClient, reservation_id: str, new_date: str) -> dict:
    """
    Modify just the date.
    
    Args:
        client: ResyClient instance
        reservation_id: Reservation ID
        new_date: New date (YYYY-MM-DD)
    
    Returns:
        Updated reservation
    """
    return modify_reservation(client, reservation_id, new_date=new_date)


def display_modification_result(result: dict, original_id: str):
    """Display modification result."""
    print("\n" + "=" * 50)
    print("✓ Reservation Modified!")
    print("=" * 50)
    
    print(f"\nReservation ID: {result.get('reservation_id', original_id)}")
    
    if result.get('display_date'):
        print(f"New Date: {result['display_date']}")
    
    if result.get('display_time'):
        print(f"New Time: {result['display_time']}")
    
    if result.get('party_size'):
        print(f"New Party Size: {result['party_size']}")
    
    if result.get('venue', {}).get('name'):
        print(f"Restaurant: {result['venue']['name']}")
    
    notes = result.get('notes') or result.get('special_requests')
    if notes:
        print(f"Special Requests: {notes}")
    
    print("\nYou can view your updated reservation using:")
    print(f"  python3 list_reservations.py")
    print("=" * 50)


def display_current_reservation(res: dict):
    """Display current reservation details."""
    venue = res.get('venue', {})
    
    print("\nCurrent Reservation Details:")
    print("-" * 40)
    print(f"ID: {res.get('reservation_id', res.get('id', 'N/A'))}")
    print(f"Restaurant: {venue.get('name', 'Unknown')}")
    print(f"Date: {res.get('scheduled_date', res.get('start_date', {}).get('date', 'N/A'))}")
    print(f"Time: {res.get('scheduled_time', res.get('start_date', {}).get('time', 'N/A'))}")
    print(f"Party Size: {res.get('party_size', res.get('num_people', 'N/A'))}")
    
    notes = res.get('notes')
    if notes:
        print(f"Special Requests: {notes}")
    print("-" * 40)


def main():
    parser = argparse.ArgumentParser(
        description='Modify an existing Resy reservation',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # View current reservation
  %(prog)s --reservation-id resy_abc123 --show
  
  # Change party size
  %(prog)s --reservation-id resy_abc123 --party-size 4
  
  # Change time
  %(prog)s --reservation-id resy_abc123 --time 20:00
  
  # Change date and time
  %(prog)s --reservation-id resy_abc123 --date 2024-12-26 --time 19:30
  
  # Update special requests
  %(prog)s --reservation-id resy_abc123 --notes "Allergic to nuts, window table please"
        """
    )
    
    parser.add_argument('--reservation-id', '-r', required=True,
                        help='Reservation ID to modify')
    parser.add_argument('--date', '-d',
                        help='New date in YYYY-MM-DD format')
    parser.add_argument('--time', '-t',
                        help='New time in HH:MM format (24-hour)')
    parser.add_argument('--party-size', '-p', type=int,
                        help='New party size (1-20)')
    parser.add_argument('--notes', '-n',
                        help='Update special requests/dietary restrictions')
    parser.add_argument('--show', '-s', action='store_true',
                        help='Show current reservation details only')
    parser.add_argument('--json', action='store_true',
                        help='Output result as JSON')
    parser.add_argument('--yes', '-y', action='store_true',
                        help='Skip confirmation prompt')
    
    args = parser.parse_args()
    
    # Validate that at least one modification is requested (unless --show)
    if not args.show:
        modifications = [args.date, args.time, args.party_size, args.notes]
        if not any(modifications):
            print("Error: No modifications specified. Use --date, --time, --party-size, or --notes.",
                  file=sys.stderr)
            print("Use --show to view current reservation details.", file=sys.stderr)
            return 1
    
    try:
        client = ResyClient()
        
        # Get current reservation details
        print(f"Fetching reservation {args.reservation_id}...")
        current = get_reservation_details(client, args.reservation_id)
        
        if args.show:
            display_current_reservation(current)
            return 0
        
        # Validate new values
        if args.date:
            try:
                validate_date(args.date)
            except ValueError as e:
                print(f"Validation error: {e}", file=sys.stderr)
                return 1
        
        if args.time:
            try:
                validate_time(args.time)
            except ValueError as e:
                print(f"Validation error: {e}", file=sys.stderr)
                return 1
        
        if args.party_size:
            try:
                validate_party_size(args.party_size)
            except ValueError as e:
                print(f"Validation error: {e}", file=sys.stderr)
                return 1
        
        # Display current and proposed changes
        display_current_reservation(current)
        
        print("\nProposed Changes:")
        print("-" * 40)
        if args.date:
            current_date = current.get('scheduled_date', current.get('start_date', {}).get('date', 'N/A'))
            print(f"  Date: {current_date} → {args.date}")
        if args.time:
            current_time = current.get('scheduled_time', current.get('start_date', {}).get('time', 'N/A'))
            print(f"  Time: {current_time} → {args.time}")
        if args.party_size:
            current_size = current.get('party_size', current.get('num_people', 'N/A'))
            print(f"  Party Size: {current_size} → {args.party_size}")
        if args.notes:
            print(f"  Notes: Updated")
        print("-" * 40)
        
        # Confirmation
        if not args.yes:
            try:
                confirm = input("\nConfirm modification? (yes/no): ").strip().lower()
                if confirm not in ('yes', 'y'):
                    print("Modification cancelled.")
                    return 0
            except (EOFError, KeyboardInterrupt):
                print("\nModification cancelled.")
                return 0
        
        # Perform modification
        print("\nModifying reservation...")
        result = modify_reservation(
            client=client,
            reservation_id=args.reservation_id,
            new_date=args.date,
            new_time=args.time,
            new_party_size=args.party_size,
            notes=args.notes
        )
        
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            display_modification_result(result, args.reservation_id)
        
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
        logger.exception("Unexpected error in modify")
        return 1


if __name__ == '__main__':
    sys.exit(main())
