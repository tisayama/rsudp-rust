from obspy import read
st1 = read("../references/mseed/fdsnws.mseed")
st2 = read("../references/mseed/20251208_tsukuba_fdsnws.mseed")

def print_peaks(st, label):
    print(f"--- {label} ---")
    for tr in st:
        peak = max(abs(tr.data))
        print(f"Channel: {tr.stats.channel}, Peak: {peak}")

print_peaks(st1, "FDSNWS (R6E01)")
print_peaks(st2, "Tsukuba (S9AF3)")
