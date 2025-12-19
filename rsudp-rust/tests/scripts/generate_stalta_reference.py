import numpy as np
from obspy.signal.trigger import recursive_sta_lta
import csv
import sys

def py_recursive_sta_lta(a, nsta, nlta):
    sta = 0.0
    lta = 1.0e-99 
    csta = 1.0 / nsta
    clta = 1.0 / nlta
    cft = np.zeros(len(a))
    
    for i, x in enumerate(a):
        sq = x * x
        sta += (sq - sta) * csta
        lta += (sq - lta) * clta
        
        if lta < 1.0e-99:
            sta = 0.0
            lta = 1.0e-99
        cft[i] = sta / lta
        
    cft[:nlta] = 0.0
    return cft

def generate_reference_data(output_file, n_samples=10000, nsta=50, nlta=200):
    np.random.seed(42)
    data = np.random.normal(0, 0.1, n_samples)
    data[5000:5500] += np.sin(np.linspace(0, np.pi, 500)) * 5.0
    
    cft_obspy = recursive_sta_lta(data, nsta, nlta)
    cft_py = py_recursive_sta_lta(data, nsta, nlta)
    
    diff = np.abs(cft_obspy - cft_py)
    max_diff = np.max(diff)
    print(f"DEBUG: Max diff between Obspy and Pure Python: {max_diff}")
    
    if max_diff > 1e-15:
        print("WARNING: Logic mismatch!")
    
    with open(output_file, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['input', 'ratio'])
        for inp, ratio in zip(data, cft_obspy):
            writer.writerow([f"{inp:.25f}", f"{ratio:.25f}"])

if __name__ == "__main__":
    if len(sys.argv) < 2:
        generate_reference_data("reference.csv")
    else:
        generate_reference_data(sys.argv[1])
