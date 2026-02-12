#!/usr/bin/env python3
"""
Book a restaurant reservation on Resy.
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
    check_availability, parse_availability, logger,
    localize_datetime, get_timezone
)
from config import Config


def get_booking_token(
    client: ResyClient,
    config_id: str,
    date: str,
    party_size: int,
    special_requests: str = None
) -> tuple:
    """
    Get booking token from config ID.
    
    Args:
        client: ResyClient instance
        config_id: Config token from availability
        date: Date in YYYY-MM-DD format
        party_size: Number of guests
        special_requests: Optional special requests/dietary restrictions
    
    Returns:
        Tuple of (booking token string, details dict)
    """
    data = {
        'commit': 1,
        'config_id': config_id,
        'day': date,
        'party_size': party_size,
    }
    
    # Add special requests if provided
    if special_requests:
        data['notes'] = special_requests
    
    try:
        response = client.post('/3/details', data=data)
        result = response.json()
        
        book_token = result.get('book_token', {}).get('value')
        if not book_token:
            raise ResyAPIError("No booking token received from details endpoint")
        
        return book_token, result
        
    except ResyAPIError:
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to get booking details: {str(e)}")


def create_reservation(
    client: ResyClient,
    book_token: str,
    source_id: str = 'resy.com',
    idempotency_key: str = None
) -> dict:
    """
    Create a reservation using booking token.
    
    Args:
        client: ResyClient instance
        book_token: Booking token from details endpoint
        source_id: Source identifier
        idempotency_key: Optional idempotency key for retry safety
    
    Returns:
        Reservation details
    """
    data = {
        'book_token': book_token,
        'source_id': source_id,
    }
    
    # Add idempotency key if provided (for retry safety)
    headers = {}
    if idempotency_key:
        headers['Idempotency-Key'] = idempotency_key
    
    try:
        response = client.post('/3/book', data=data, headers=headers)
        return response.json()
        
    except ResyAPIError as e:
        # Handle specific booking errors
        if e.status_code == 409:
            raise ResyAPIError(
                "Reservation conflict. This time slot is no longer available. "
                "Please check availability again and try a different time.",
                status_code=409
            )
        elif e.status_code == 402:
            raise ResyAPIError(
                "Payment method required. Please add a credit card to your Resy account "
                "before making this reservation.",
                status_code=402
            )
        elif e.status_code == 422:
            raise ResyAPIError(
                "Booking validation failed. The time slot may have been taken. "
                "Please check availability again.",
                status_code=422
            )
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to create reservation: {str(e)}")


def find_and_book_slot(
    client: ResyClient,
    venue_id: int,
    date: str,
    time: str,
    party_size: int,
    table_type: str = None,
    special_requests: str = None,
    max_retries: int = 2
) -> dict:
    """
    Find availability and book a specific time slot with retry logic.
    
    Args:
        client: ResyClient instance
        venue_id: Restaurant venue ID
        date: Date in YYYY-MM-DD format
        time: Time in HH:MM format
        party_size: Number of guests
        table_type: Preferred table type (optional)
        special_requests: Special requests/dietary restrictions (optional)
        max_retries: Number of retries on network failure
    
    Returns:
        Reservation details
    """
    import uuid
    import time
    
    # Generate idempotency key for this booking attempt
    idempotency_key = str(uuid.uuid4())
    last_error = None
    
    for attempt in range(max_retries):
        try:
            # First, check availability
            if attempt == 0:
                print(f"Finding availability for {venue_id} on {date} at {time}...")
            else:
                print(f"Retrying... (attempt {attempt + 1}/{max_retries})")
            
            avail_data = check_availability(
                client=client,
                venue_id=venue_id,
                date=date,
                party_size=party_size
            )
            
            slots = parse_availability(avail_data)
            
            if not slots:
                raise ResyAPIError("No availability found for the requested date and party size.")
            
            # Find matching slot
            target_slot = None
            for slot in slots:
                slot_time = slot.get('time', '')
                if slot_time == time or slot_time.startswith(time):
                    if table_type is None or slot.get('type', '').lower() == table_type.lower():
                        target_slot = slot
                        break
            
            if not target_slot:
                available_times = sorted(set(s['time'] for s in slots if s.get('time')))
                times_str = ', '.join(available_times[:5]) if available_times else 'None'
                raise ResyAPIError(
                    f"Requested time {time} not available. "
                    f"Available times include: {times_str}"
                )
            
            # Get booking token
            print(f"Found slot at {target_slot['time']}. Getting booking details...")
            config_id = target_slot.get('token')
            
            if not config_id:
                raise ResyAPIError("No config token available for this slot.")
            
            book_token, details = get_booking_token(
                client, config_id, date, party_size, special_requests
            )
            
            # Check for payment method requirement
            payment_required = details.get('payment', {}).get('required', False)
            if payment_required:
                print("ℹ️  This reservation requires a credit card hold.")
            
            # Create reservation with idempotency key for safety
            print("Creating reservation...")
            reservation = create_reservation(client, book_token, idempotency_key=idempotency_key)
            
            return reservation
            
        except ResyAPIError as e:
            # Don't retry on certain errors
            if e.status_code in (401, 403, 404, 409, 422):
                raise
            last_error = e
            if attempt < max_retries - 1:
                wait_time = 2 ** attempt
                print(f"Error: {e.message}. Retrying in {wait_time}s...")
                time.sleep(wait_time)
            else:
                raise
        except Exception as e:
            last_error = e
            if attempt < max_retries - 1:
                wait_time = 2 ** attempt
                print(f"Unexpected error: {str(e)}. Retrying in {wait_time}s...")
                time.sleep(wait_time)
            else:
                raise
    
    # If we get here, all retries failed
    if last_error:
        raise last_error
    else:
        raise ResyAPIError("Booking failed after all retry attempts.")


def display_reservation(reservation: dict):
    """Display reservation details."""
    print("\n" + "=" * 50)
    print("✓ Reservation Confirmed!")
    print("=" * 50)
    
    print(f"\nReservation ID: {reservation.get('reservation_id', reservation.get('resy_token', 'N/A'))}")
    
    if reservation.get('display_date'):
        print(f"Date: {reservation['display_date']}")
    
    if reservation.get('display_time'):
        print(f"Time: {reservation['display_time']}")
    
    if reservation.get('venue_name'):
        print(f"Restaurant: {reservation['venue_name']}")
    elif reservation.get('venue', {}).get('name'):
        print(f"Restaurant: {reservation['venue']['name']}")
    
    # Display special requests if any
    notes = reservation.get('notes') or reservation.get('special_requests')
    if notes:
        print(f"Special Requests: {notes}")
    
    # Display cancellation policy
    cancel_policy = reservation.get('cancellation_policy', {})
    if cancel_policy.get('deadline'):
        print(f"\nCancellation deadline: {cancel_policy['deadline']}")
    
    print("\nYou can view or cancel this reservation using:")
    print(f"  python3 list_reservations.py")
    print("=" * 50)


def main():
    parser = argparse.ArgumentParser(
        description='Book a restaurant reservation on Resy',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --venue-id 1505 --date 2024-12-25 --time 19:00 --party-size 2
  %(prog)s --venue-id 1505 --date 2024-12-25 --time 18:30 --party-size 4 --json
  %(prog)s --venue-id 1505 --date 2024-12-25 --time 19:00 -p 2 --notes "Anniversary dinner, vegan options please"
        """
    )
    
    parser.add_argument('--venue-id', '-v', required=True, type=int,
                        help='Restaurant venue ID')
    parser.add_argument('--date', '-d', required=True,
                        help='Date in YYYY-MM-DD format')
    parser.add_argument('--time', '-t', required=True,
                        help='Time in HH:MM format (24-hour)')
    parser.add_argument('--party-size', '-p', required=True, type=int,
                        help='Number of guests (1-20)')
    parser.add_argument('--table-type',
                        help='Preferred table type (e.g., "Dining Room", "Patio")')
    parser.add_argument('--notes', '-n',
                        help='Special requests or dietary restrictions')
    parser.add_argument('--json', action='store_true',
                        help='Output result as JSON')
    parser.add_argument('--yes', '-y', action='store_true',
                        help='Skip confirmation prompt')
    parser.add_argument('--retry', '-r', type=int, default=2,
                        help='Number of retries on network failure (default: 2)')
    
    args = parser.parse_args()
    
    # Load config
    config = Config()
    
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
    if not args.yes and not config.auto_confirm:
        print(f"\nBooking Details:")
        print(f"  Venue ID: {args.venue_id}")
        print(f"  Date: {args.date}")
        print(f"  Time: {args.time}")
        print(f"  Party Size: {args.party_size}")
        if args.table_type:
            print(f"  Table Type: {args.table_type}")
        if args.notes:
            print(f"  Special Requests: {args.notes}")
        
        try:
            confirm = input("\nConfirm booking? (yes/no): ").strip().lower()
            if confirm not in ('yes', 'y'):
                print("Booking cancelled.")
                return 0
        except (EOFError, KeyboardInterrupt):
            print("\nBooking cancelled.")
            return 0
    
    try:
        client = ResyClient()
        
        reservation = find_and_book_slot(
            client=client,
            venue_id=args.venue_id,
            date=args.date,
            time=args.time,
            party_size=args.party_size,
            table_type=args.table_type,
            special_requests=args.notes,
            max_retries=args.retry
        )
        
        if args.json:
            print(json.dumps(reservation, indent=2))
        else:
            display_reservation(reservation)
        
        return 0
        
    except ResyAPIError as e:
        print(f"\nError: {e.message}", file=sys.stderr)
        if e.status_code:
            print(f"Status Code: {e.status_code}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nBooking cancelled.", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"\nUnexpected error: {str(e)}", file=sys.stderr)
        logger.exception("Unexpected error in booking")
        return 1


if __name__ == '__main__':
    sys.exit(main())
