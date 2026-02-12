#!/usr/bin/env python3
"""
Output formatting utilities for Resy booking skill.
"""

import csv
import json
import sys
from io import StringIO
from typing import List, Dict, Any, Optional

# Try to import tabulate for table formatting
try:
    from tabulate import tabulate
    TABULATE_AVAILABLE = True
except ImportError:
    TABULATE_AVAILABLE = False


def format_as_table(data: List[Dict[str, Any]], 
                    headers: Optional[List[str]] = None,
                    tablefmt: str = 'simple') -> str:
    """
    Format data as a table.
    
    Args:
        data: List of dictionaries
        headers: Column headers (uses dict keys if None)
        tablefmt: Table format (simple, grid, pipe, etc.)
    
    Returns:
        Formatted table string
    """
    if not data:
        return "No data to display."
    
    # Auto-detect headers from first row
    if headers is None:
        headers = list(data[0].keys())
    
    # Extract rows in header order
    rows = []
    for item in data:
        row = []
        for h in headers:
            val = item.get(h, '')
            # Format values nicely
            if val is None:
                val = ''
            elif isinstance(val, dict):
                val = json.dumps(val)
            elif isinstance(val, list):
                val = ', '.join(str(x) for x in val[:3])
                if len(val) > 50:
                    val = val[:47] + '...'
            else:
                val = str(val)
                if len(val) > 50:
                    val = val[:47] + '...'
            row.append(val)
        rows.append(row)
    
    if TABULATE_AVAILABLE:
        return tabulate(rows, headers=headers, tablefmt=tablefmt)
    else:
        # Simple fallback formatting
        return _simple_table_format(rows, headers)


def _simple_table_format(rows: List[List[str]], headers: List[str]) -> str:
    """Simple table formatting without tabulate."""
    if not rows:
        return "No data."
    
    # Calculate column widths
    widths = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            widths[i] = max(widths[i], len(str(cell)))
    
    # Build output
    lines = []
    
    # Header
    header_row = ' | '.join(h.ljust(widths[i]) for i, h in enumerate(headers))
    lines.append(header_row)
    lines.append('-' * len(header_row))
    
    # Rows
    for row in rows:
        lines.append(' | '.join(str(cell).ljust(widths[i]) for i, cell in enumerate(row)))
    
    return '\n'.join(lines)


def format_as_csv(data: List[Dict[str, Any]], 
                  headers: Optional[List[str]] = None) -> str:
    """
    Format data as CSV.
    
    Args:
        data: List of dictionaries
        headers: Column headers (uses dict keys if None)
    
    Returns:
        CSV string
    """
    if not data:
        return ""
    
    if headers is None:
        headers = list(data[0].keys())
    
    output = StringIO()
    writer = csv.DictWriter(output, fieldnames=headers, extrasaction='ignore')
    writer.writeheader()
    writer.writerows(data)
    return output.getvalue()


def format_reservation_for_table(res: Dict[str, Any]) -> Dict[str, str]:
    """Format a reservation dict for table display."""
    venue = res.get('venue', {})
    
    return {
        'ID': res.get('reservation_id', res.get('id', 'N/A'))[:12],
        'Restaurant': venue.get('name', 'Unknown')[:25],
        'Date': res.get('scheduled_date', res.get('start_date', {}).get('date', 'N/A')),
        'Time': res.get('scheduled_time', res.get('start_date', {}).get('time', 'N/A')),
        'Party': str(res.get('party_size', res.get('num_people', '?'))),
        'Status': res.get('status', 'confirmed'),
    }


def format_venue_for_table(venue: Dict[str, Any]) -> Dict[str, str]:
    """Format a venue dict for table display."""
    venue_data = venue.get('venue', venue)
    
    cuisine = venue_data.get('cuisine', [])
    cuisine_str = ', '.join(c.get('name', '') for c in cuisine[:2]) if cuisine else ''
    
    rating = venue_data.get('rating', {})
    rating_str = f"{rating.get('average', '?')}/5" if rating else ''
    
    return {
        'ID': str(venue_data.get('id', {}).get('resy') or venue_data.get('id', 'N/A')),
        'Name': venue_data.get('name', 'Unknown')[:30],
        'Neighborhood': venue_data.get('neighborhood', '')[:20],
        'Cuisine': cuisine_str[:20],
        'Price': venue_data.get('price_range', ''),
        'Rating': rating_str,
    }


def format_slots_for_table(slots: List[Dict[str, Any]]) -> List[Dict[str, str]]:
    """Format availability slots for table display."""
    result = []
    for slot in slots:
        result.append({
            'Time': slot.get('time', 'N/A'),
            'Type': slot.get('type', 'Standard'),
            'Date': slot.get('date', 'N/A'),
        })
    return result


def print_summary(title: str, items: List[Dict], 
                  headers: Optional[List[str]] = None,
                  format: str = 'table'):
    """
    Print a summary of items in the specified format.
    
    Args:
        title: Title to display
        items: List of dictionaries
        headers: Column headers
        format: 'table', 'csv', or 'json'
    """
    if not items:
        print(f"\nNo {title.lower()} found.")
        return
    
    print(f"\n{title} ({len(items)} items):")
    
    if format == 'csv':
        print(format_as_csv(items, headers))
    elif format == 'json':
        print(json.dumps(items, indent=2))
    else:  # table
        print(format_as_table(items, headers))
    
    print()


class OutputFormatter:
    """Flexible output formatter for CLI tools."""
    
    FORMATS = ['table', 'csv', 'json', 'simple']
    
    def __init__(self, format_type: str = 'table'):
        self.format_type = format_type.lower()
        if self.format_type not in self.FORMATS:
            raise ValueError(f"Unknown format: {format_type}. Use: {', '.join(self.FORMATS)}")
    
    def format(self, data: List[Dict], headers: Optional[List[str]] = None) -> str:
        """Format data according to the set format type."""
        if self.format_type == 'csv':
            return format_as_csv(data, headers)
        elif self.format_type == 'json':
            return json.dumps(data, indent=2)
        elif self.format_type == 'simple':
            # Simple text format
            lines = []
            for item in data:
                for key, val in item.items():
                    lines.append(f"{key}: {val}")
                lines.append("-" * 40)
            return '\n'.join(lines)
        else:  # table
            return format_as_table(data, headers)
    
    def print(self, data: List[Dict], headers: Optional[List[str]] = None, 
              title: Optional[str] = None):
        """Print formatted data."""
        if title:
            print(f"\n{title}")
            print("=" * len(title))
        print(self.format(data, headers))


if __name__ == '__main__':
    # Test the formatter
    test_data = [
        {'name': 'Nobu', 'cuisine': 'Japanese', 'rating': 4.5},
        {'name': 'Carbone', 'cuisine': 'Italian', 'rating': 4.8},
        {'name': 'Le Bernardin', 'cuisine': 'French', 'rating': 4.9},
    ]
    
    print("Table format:")
    print(format_as_table(test_data))
    
    print("\n\nCSV format:")
    print(format_as_csv(test_data))
    
    print("\n\nSimple format:")
    formatter = OutputFormatter('simple')
    print(formatter.format(test_data))
