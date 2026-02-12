#!/usr/bin/env python3
"""
Search for restaurants on Resy.
"""

import argparse
import json
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from utils import ResyClient, ResyAPIError, validate_venue_id, logger, ProgressSpinner


def search_restaurants(
    client: ResyClient,
    query: str,
    location: str = None,
    lat: float = None,
    long: float = None,
    limit: int = 20
) -> list:
    """
    Search for restaurants by name or query.
    
    Args:
        client: ResyClient instance
        query: Search query (restaurant name, cuisine, etc.)
        location: Location string (city, neighborhood)
        lat: Latitude for location-based search
        long: Longitude for location-based search
        limit: Maximum results to return
    
    Returns:
        List of restaurant dictionaries
    """
    params = {
        'query': query,
        'per_page': min(limit, 50),  # API max is typically 50
        'types': 'venue',
    }
    
    if location:
        params['location'] = location
    
    if lat is not None and long is not None:
        params['lat'] = lat
        params['long'] = long
    
    try:
        # Use the search endpoint
        response = client.get('/3/venues', params=params)
        data = response.json()
        
        # Extract venues from response
        venues = data.get('venues', [])
        
        # Format results
        results = []
        for venue in venues:
            venue_data = venue.get('venue', venue)  # Handle nested structure
            
            formatted = {
                'venue_id': venue_data.get('id', {}).get('resy') or venue_data.get('id'),
                'name': venue_data.get('name', 'Unknown'),
                'type': venue_data.get('type', 'restaurant'),
                'location': venue_data.get('location', {}).get('name', 'Unknown'),
                'neighborhood': venue_data.get('neighborhood', ''),
                'price_range': venue_data.get('price_range', ''),
                'cuisine': [c.get('name') for c in venue_data.get('cuisine', [])],
                'rating': venue_data.get('rating', {}),
                'url': f"https://resy.com/cities/{venue_data.get('url_slug', '')}" if venue_data.get('url_slug') else None,
                'timezone': venue_data.get('timezone', 'America/New_York'),
            }
            results.append(formatted)
        
        return results
        
    except ResyAPIError:
        raise
    except Exception as e:
        raise ResyAPIError(f"Search failed: {str(e)}")


def display_results(results: list, verbose: bool = False):
    """Display search results in a readable format."""
    if not results:
        print("No restaurants found matching your search.")
        print("\nTips:")
        print("  - Try broader search terms")
        print("  - Check spelling")
        print("  - Try searching by neighborhood or cuisine")
        return
    
    print(f"\nFound {len(results)} restaurant(s):\n")
    
    for i, venue in enumerate(results, 1):
        print(f"{i}. {venue['name']}")
        print(f"   Venue ID: {venue['venue_id']}")
        
        if venue['neighborhood']:
            print(f"   Location: {venue['neighborhood']}")
        elif venue['location']:
            print(f"   Location: {venue['location']}")
        
        if venue['cuisine']:
            cuisine_str = ', '.join(venue['cuisine'][:3])  # Show max 3 cuisines
            print(f"   Cuisine: {cuisine_str}")
        
        if venue['price_range']:
            print(f"   Price: {venue['price_range']}")
        
        if venue['rating'] and venue['rating'].get('average'):
            print(f"   Rating: {venue['rating']['average']}/5")
        
        if verbose:
            if venue['url']:
                print(f"   URL: {venue['url']}")
            if venue['timezone']:
                print(f"   Timezone: {venue['timezone']}")
        
        print()


def main():
    parser = argparse.ArgumentParser(
        description='Search for restaurants on Resy',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --query "Nobu"
  %(prog)s --query "Italian" --location "New York"
  %(prog)s --query "Sushi" --limit 10
        """
    )
    
    parser.add_argument('--query', '-q', required=True,
                        help='Search query (restaurant name, cuisine, etc.)')
    parser.add_argument('--location', '-l',
                        help='Location filter (city, neighborhood)')
    parser.add_argument('--lat', type=float,
                        help='Latitude for location-based search')
    parser.add_argument('--long', type=float,
                        help='Longitude for location-based search')
    parser.add_argument('--limit', type=int, default=20,
                        help='Maximum number of results (default: 20)')
    parser.add_argument('--json', action='store_true',
                        help='Output results as JSON')
    parser.add_argument('--verbose', '-v', action='store_true',
                        help='Show additional details')
    
    args = parser.parse_args()
    
    # Validate args
    if (args.lat is not None and args.long is None) or (args.lat is None and args.long is not None):
        print("Error: Must provide both --lat and --long together", file=sys.stderr)
        return 1
    
    try:
        client = ResyClient()
        
        print(f"Searching for: {args.query}")
        if args.location:
            print(f"Location: {args.location}")
        
        with ProgressSpinner("Searching restaurants"):
            results = search_restaurants(
                client=client,
                query=args.query,
                location=args.location,
                lat=args.lat,
                long=args.long,
                limit=args.limit
            )
        
        if args.json:
            print(json.dumps(results, indent=2))
        else:
            display_results(results, verbose=args.verbose)
        
        return 0
        
    except ResyAPIError as e:
        print(f"Error: {e.message}", file=sys.stderr)
        if e.status_code:
            print(f"Status Code: {e.status_code}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nSearch cancelled.", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"Unexpected error: {str(e)}", file=sys.stderr)
        logger.exception("Unexpected error in search")
        return 1


if __name__ == '__main__':
    sys.exit(main())
