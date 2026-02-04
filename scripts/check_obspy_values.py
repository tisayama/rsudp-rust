
import numpy as np
from obspy import read
from obspy.signal.trigger import recursive_sta_lta

def verify():
    # Load data
    st = read("references/mseed/fdsnws.mseed")
    tr = st.select(channel="EHZ")[0]
    
    # rsudp parameters
    sta = 6.0
    lta = 30.0
    df = tr.stats.sampling_rate
    nsta = int(sta * df)
    nlta = int(lta * df)
    
    # rsudp logic: trim to LTA + 1 seconds
    # Let's pick a window near the first event
    endtime = tr.stats.starttime + 100 # 100 seconds from start
    tr_slice = tr.slice(endtime - (lta + 1), endtime)
    
    # 1. Filter (Standard ObsPy bandpass)
    tr_filtered = tr_slice.copy()
    tr_filtered.filter('bandpass', freqmin=0.1, freqmax=2.0, corners=4, zerophase=False)
    
    # 2. recursive_sta_lta
    # This function uses squared data internally
    cft = recursive_sta_lta(tr_filtered.data, nsta, nlta)
    
    print(f"Sample count: {len(tr_filtered.data)}")
    print(f"Last 5 ratios: {cft[-5:]}")

if __name__ == "__main__":
    verify()
