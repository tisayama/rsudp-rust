import numpy as np
from obspy import read

def jma_intensity(st, sensitivity=204800):
    st.trim(st[0].stats.starttime + 1, st[0].stats.endtime - 1)
    # 1. Convert to Gal
    for tr in st:
        tr.data = tr.data.astype(float) / sensitivity * 100.0
        tr.detrend("demean")
    
    # 2. Filter (approximate JMA filter)
    st.filter("bandpass", freqmin=0.5, freqmax=10.0)
    
    # 3. Vector composition
    # Find common length
    min_len = min(len(tr.data) for tr in st)
    data_sq = np.zeros(min_len)
    for tr in st:
        data_sq += tr.data[:min_len]**2
    acc = np.sqrt(data_sq)
    
    # 4. Sort and find 0.3s duration value
    acc.sort()
    idx = int(len(acc) - 0.3 * st[0].stats.sampling_rate)
    a = acc[idx]
    
    return 2.0 * np.log10(a) + 0.94

st = read("../references/mseed/20251208_tsukuba_fdsnws.mseed")
st_acc = st.select(channel="EN*")
print(f"JMA Intensity (Simplified, sens=204800): {jma_intensity(st_acc.copy(), 204800)}")
print(f"JMA Intensity (Simplified, sens=384500): {jma_intensity(st_acc.copy(), 384500)}")