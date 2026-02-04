
import numpy as np
from obspy import read
from scipy.signal import butter, sosfilt
import csv

def generate():
    # 1. Load Data
    st = read("references/mseed/fdsnws.mseed")
    tr = st.select(channel="EHZ")[0]
    data = tr.data.astype(np.float64)
    
    # 2. Preprocessing (Demean only for simple verification)
    data -= np.mean(data)
    
    # 3. Filter (Bandpass 0.1-2.0 Hz)
    sr = tr.stats.sampling_rate
    # Standard ObsPy uses corners=4 -> order=4 for butter
    sos = butter(4, [0.1, 2.0], btype='band', fs=sr, output='sos')
    
    # Apply filter (using sosfilt)
    # Scipy's sosfilt implements Direct Form II Transposed by default
    filtered, _ = sosfilt(sos, data, zi=np.zeros((sos.shape[0], 2)))
    
    # 4. STA/LTA (Recursive)
    sta_len = 6.0
    lta_len = 30.0
    nsta = int(sta_len * sr)
    nlta = int(lta_len * sr)
    
    a = 1.0 / nsta
    b = 1.0 / nlta
    
    # Squared energy
    sq = filtered ** 2
    
    sta_arr = np.zeros_like(sq)
    lta_arr = np.zeros_like(sq)
    ratio_arr = np.zeros_like(sq)
    
    # Initial values (ObsPy C-core style)
    sta = sq[0] * a
    lta = sq[0] * b + 1e-99
    
    sta_arr[0] = sta
    lta_arr[0] = lta
    ratio_arr[0] = 0.0
    
    # Loop
    for i in range(1, len(sq)):
        sta = a * sq[i] + (1.0 - a) * sta
        lta = b * sq[i-1] + (1.0 - b) * lta # Delayed LTA
        
        sta_arr[i] = sta
        lta_arr[i] = lta
        ratio_arr[i] = sta / lta

    # 5. Output to CSV
    with open("logs/algorithm_truth.csv", "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["index", "input_demeaned", "filtered", "sta", "lta", "ratio"])
        for i in range(len(data)):
            # Only dump every 10th sample to save space/time, or dump all?
            # Let's dump all but focus on comparison later.
            writer.writerow([i, data[i], filtered[i], sta_arr[i], lta_arr[i], ratio_arr[i]])
            
    print(f"Generated {len(data)} rows of truth data.")
    print("Filter SOS coefficients:")
    print(sos.tolist())

if __name__ == "__main__":
    generate()
