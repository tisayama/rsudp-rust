import re
import csv
import sys
from datetime import datetime

def parse_log(filepath, label):
    events = []
    # Regex for rsudp log timestamp: YYYY-MM-DD HH:MM:SS.mmmmmm
    # Example: "Trigger threshold 1.1 exceeded at 2025-11-25 09:01:23.995000"
    trigger_pattern = re.compile(r"Trigger threshold .* exceeded .*at (\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d+)")
    
    # Example: "Earthquake trigger reset ... at 2025-11-25 09:01:33.054999" (simplified matching)
    reset_pattern = re.compile(r"trigger reset .* at (\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d+)")

    # Rust log format might differ slightly, let's assume it matches the rsudp-compatible output we implemented.
    # Rust: "[2025-11-25 09:01:23.995 UTC] Channel ...: Trigger threshold ..." 
    # We need to handle both or standardize. 
    # Let's assume the Rust implementation output format matches what we saw in previous logs or rsudp.
    # Rust trigger.rs: "[{}] Channel {}: {}" -> [Timestamp] Channel ID: Message
    # Rust Message: "Trigger threshold {} exceeded (ratio: {:.4}). ALARM!"
    
    # Adjust regex for Rust format if needed. 
    # Let's look for timestamps generally associated with "ALARM!" or "Trigger threshold"
    
    with open(filepath, 'r') as f:
        for line in f:
            if "Trigger threshold" in line and ("exceeded" in line or "ALARM" in line):
                # Try rsudp style first
                match = trigger_pattern.search(line)
                if match:
                    events.append({'type': 'Trigger', 'time': match.group(1), 'source': label})
                    continue
                
                # Try Rust style: [2025-11-25T09:01:23.995000095Z] ...
                # Or the format from AlertEvent::fmt: [2025-11-25 09:01:23.995000095 UTC]
                rust_match = re.search(r"\[(.*?)\].*Trigger threshold", line)
                if rust_match:
                    ts_str = rust_match.group(1).replace(" UTC", "").replace("T", " ").replace("Z", "")
                    events.append({'type': 'Trigger', 'time': ts_str, 'source': label})

    return events

def main():
    if len(sys.argv) != 4:
        print("Usage: python compare_logs.py <python_log> <rust_log> <output_csv>")
        sys.exit(1)

    py_log = sys.argv[1]
    rs_log = sys.argv[2]
    out_csv = sys.argv[3]

    py_events = parse_log(py_log, 'Python')
    rs_events = parse_log(rs_log, 'Rust')

    print(f"Found {len(py_events)} Python events and {len(rs_events)} Rust events.")

    # Simple matching by sequence for now (assuming identical data stream)
    # Ideally match by timestamp proximity
    
    with open(out_csv, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['Event Index', 'Type', 'Python Time', 'Rust Time', 'Diff (s)', 'Status'])
        
        max_len = max(len(py_events), len(rs_events))
        
        for i in range(max_len):
            py_evt = py_events[i] if i < len(py_events) else None
            rs_evt = rs_events[i] if i < len(rs_events) else None
            
            p_time = py_evt['time'] if py_evt else "N/A"
            r_time = rs_evt['time'] if rs_evt else "N/A"
            diff = "N/A"
            status = "Mismatch"
            
            if py_evt and rs_evt:
                # Parse times
                try:
                    # Handle variable precision
                    p_dt = datetime.strptime(p_time[:26], "%Y-%m-%d %H:%M:%S.%f")
                    r_dt = datetime.strptime(r_time[:26], "%Y-%m-%d %H:%M:%S.%f")
                    delta = (r_dt - p_dt).total_seconds()
                    diff = f"{delta:.4f}"
                    
                    if abs(delta) < 0.5:
                        status = "Match"
                    else:
                        status = "Time Drift"
                except Exception as e:
                    diff = f"Error: {e}"
            
            elif py_evt:
                status = "Python Only"
            elif rs_evt:
                status = "Rust Only"

            writer.writerow([i+1, 'Trigger', p_time, r_time, diff, status])

    print(f"Comparison report written to {out_csv}")

if __name__ == "__main__":
    main()
