
import csv
import math

def compare():
    py_data = []
    with open("logs/algorithm_truth.csv", "r") as f:
        reader = csv.DictReader(f)
        for row in reader:
            py_data.append(row)
            
    rs_data = []
    with open("logs/rs_algo_out.csv", "r") as f:
        reader = csv.DictReader(f)
        for row in reader:
            rs_data.append(row)
            
    print(f"Python length: {len(py_data)}")
    print(f"Rust length: {len(rs_data)}")
    
    min_len = min(len(py_data), len(rs_data))
    
    max_diff_filtered = 0.0
    max_diff_sta = 0.0
    max_diff_lta = 0.0
    max_diff_ratio = 0.0
    
    mismatches = []
    
    for i in range(min_len):
        py = py_data[i]
        rs = rs_data[i]
        
        d_filt = abs(float(py['filtered']) - float(rs['filtered']))
        d_sta = abs(float(py['sta']) - float(rs['sta']))
        d_lta = abs(float(py['lta']) - float(rs['lta']))
        d_ratio = abs(float(py['ratio']) - float(rs['ratio']))
        
        max_diff_filtered = max(max_diff_filtered, d_filt)
        max_diff_sta = max(max_diff_sta, d_sta)
        max_diff_lta = max(max_diff_lta, d_lta)
        max_diff_ratio = max(max_diff_ratio, d_ratio)
        
        if d_ratio > 1e-6:
            if len(mismatches) < 10:
                mismatches.append((i, float(py['ratio']), float(rs['ratio']), d_ratio))
    
    print("-" * 30)
    print(f"Max Filter Diff: {max_diff_filtered:.6e}")
    print(f"Max STA Diff:    {max_diff_sta:.6e}")
    print(f"Max LTA Diff:    {max_diff_lta:.6e}")
    print(f"Max Ratio Diff:  {max_diff_ratio:.6e}")
    
    if mismatches:
        print("-" * 30)
        print("First 10 Ratio Mismatches (> 1e-6):")
        print("Index | Python | Rust | Diff")
        for m in mismatches:
            print(f"{m[0]:5d} | {m[1]:.6f} | {m[2]:.6f} | {m[3]:.6e}")
    else:
        print("-" * 30)
        print("PERFECT MATCH! (within 1e-6)")

if __name__ == "__main__":
    compare()
