import numpy as np
from obspy.signal.trigger import recursive_sta_lta
import csv
import sys

def generate_reference_data(output_file, n_samples=10000, nsta=50, nlta=200):
    # Generate synthetic data: Noise + Signal
    np.random.seed(42)
    data = np.random.normal(0, 0.1, n_samples)
    # Add a signal
    data[5000:5500] += np.sin(np.linspace(0, np.pi, 500)) * 5.0
    
    # Calculate recursive STA/LTA
    cft = recursive_sta_lta(data, nsta, nlta)
    
    # Write to CSV
    with open(output_file, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['input', 'ratio'])
        for inp, ratio in zip(data, cft):
            writer.writerow([inp, ratio])

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python generate_stalta_reference.py <output_csv> [n_samples] [nsta] [nlta]")
        generate_reference_data("reference.csv")
    else:
        generate_reference_data(sys.argv[1])