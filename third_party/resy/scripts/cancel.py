#!/usr/bin/env python3
"""
Cancel an existing reservation on Resy.
"""

import argparse
import json
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from utils import ResyClient, ResyAPIError, logger


def cancel_reservation(client: ResyClient, reservation_id: str) -> dict:
    """
    Cancel a reservation by ID.
    
    Args:
        client: ResyClient instance
        reservation_id: Reservation ID to cancel
    
    Returns:
        Cancellation response
    """
    data = {
        'reservation_id': reservation_id,
    }
    
    try:
        response = client.post('/3/cancel', data=data)
        return response.json()
        
    except ResyAPIError as e:
        if e.status_code == 404:
            raise ResyAPIError(
                f"Reservation '{reservation_id}' not found. "
                "It may have already been cancelled or expired.",
                status_code=404
            )
        elif e.status_code == 403:
            raise ResyAPIError(
                "Cancellation not allowed. The cancellation window may have passed. "
                "Contact the restaurant directly for assistance.",
                status_code=403
            )
        raise
    except Exception as e:
        raise ResyAPIError(f"Failed to cancel reservation: {str(e)}")


def display_cancellation_result(result: dict, reservation_id: str):
    """Display cancellation result."""
    status = result.get('status', 'unknown')
    
    if status == 'cancelled' or status == 'canceled':
        print(f"\n✓ Reservation {reservation_id} has been successfully cancelled.")
    else:
        print(f"\nReservation {reservation_id} status: {status}")
    
    # Check for any refund info
    if result.get('refund_amount'):
        print(f"Refund amount: {result['refund_amount']}")
    
    if result.get('cancellation_fee'):
        print(f"Cancellation fee: {result['cancellation_fee']}")
    
    # Display cancellation policy info if available
    policy = result.get('cancellation_policy', {})
    if policy.get('deadline_passed'):
        print("\n⚠️  Note: The cancellation deadline had passed.")
        print("You may have been charged a cancellation fee.")


def main():
    parser = argparse.ArgumentParser(
        description='Cancel a restaurant reservation on Resy',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --reservation-id resy_abc123
  %(prog)s --reservation-id abc123 --yes
        """
    )
    
    parser.add_argument('--reservation-id', '-r', required=True,
                        help='Reservation ID to cancel')
    parser.add_argument('--json', action='store_true',
                        help='Output result as JSON')
    parser.add_argument('--yes', '-y', action='store_true',
                        help='Skip confirmation prompt')
    
    args = parser.parse_args()
    
    # Confirmation
    if not args.yes:
        print(f"\nYou are about to cancel reservation: {args.reservation_id}")
        print("\n⚠️  This action cannot be undone!")
        
        try:
            confirm = input("\nType 'cancel' to confirm cancellation: ").strip().lower()
            if confirm != 'cancel':
                print("Cancellation aborted.")
                return 0
        except (EOFError, KeyboardInterrupt):
            print("\nCancellation aborted.")
            return 0
    
    try:
        client = ResyClient()
        
        print(f"Cancelling reservation {args.reservation_id}...")
        
        result = cancel_reservation(client, args.reservation_id)
        
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            display_cancellation_result(result, args.reservation_id)
        
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
        logger.exception("Unexpected error in cancellation")
        return 1


if __name__ == '__main__':
    sys.exit(main())
