import re
import csv
import sys
from datetime import datetime

def parse_log(filepath, label):
    events = []
    # Regex for rsudp log timestamp: YYYY-MM-DD HH:MM:SS.mmmmmm
    # Example: "Trigger threshold 1.1 exceeded at 2025-11-25 09:01:23.995000"
    # Python log sometimes has "Trigger threshold of 1.1 ..."
    trigger_pattern_py = re.compile(r"Trigger threshold .* exceeded at (\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d+)")
    
    # Rust: [2025-11-25 09:01:23.455000095 UTC] Channel ...
    trigger_pattern_rs = re.compile(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d+) UTC\]")
    
    with open(filepath, 'r') as f:
        for line in f:
            # Remove ANSI color codes for safer regex
            clean_line = re.sub(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])', '', line)
            
            if "Trigger threshold" in clean_line and ("exceeded" in clean_line or "ALARM" in clean_line):
                if label == 'Python':
                    match = trigger_pattern_py.search(clean_line)
                    if match:
                        # Truncate/Pad microseconds to 6 digits for consistent parsing
                        ts = match.group(1)
                        if len(ts.split('.')[1]) > 6: ts = ts[:26]
                        events.append({'type': 'Trigger', 'time': ts, 'source': label})
                
                elif label == 'Rust':
                    match = trigger_pattern_rs.search(clean_line)
                    if match:
                        ts = match.group(1)
                        if len(ts.split('.')[1]) > 6: ts = ts[:26]
                        events.append({'type': 'Trigger', 'time': ts, 'source': label})

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
