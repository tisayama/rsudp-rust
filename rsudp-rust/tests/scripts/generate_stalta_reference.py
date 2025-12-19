import numpy as np
from obspy.signal.trigger import recursive_sta_lta
import csv
import sys

def generate_reference_data(output_file, n_samples=10000, nsta=50, nlta=200):
    """
    Generates reference data using Obspy's recursive_sta_lta (C implementation).
    """
    np.random.seed(42)
    data = np.random.normal(0, 0.1, n_samples)
    # Add a seismic-like signal
    data[5000:5500] += np.sin(np.linspace(0, np.pi, 500)) * 5.0
    
    # Calculate using Obspy (Ground Truth)
    cft_obspy = recursive_sta_lta(data, nsta, nlta)
    
    # Write to CSV with high precision
    with open(output_file, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['input', 'ratio'])
        for inp, ratio in zip(data, cft_obspy):
            writer.writerow([f"{inp:.25f}", f"{ratio:.25f}"])

if __name__ == "__main__":
    if len(sys.argv) < 2:
        output_file = "reference.csv"
    else:
        output_file = sys.argv[1]
        
    generate_reference_data(output_file)