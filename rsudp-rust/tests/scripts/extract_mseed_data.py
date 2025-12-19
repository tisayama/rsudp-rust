import obspy
import numpy as np

mseed_path = "/home/tisayama/Development/rustrsudp_speckit/references/mseed/fdsnws.mseed"
st = obspy.read(mseed_path)
tr = st[0] # AM.R6E01.00.EHZ

data = tr.data.astype(np.int32)
# Reconstruct differences from the decoded data
# d[i] = S[i] - S[i-1]
diffs = np.diff(data)

print(f"Trace {tr.id} First 10 samples: {data[:10].tolist()}")
print(f"Trace {tr.id} First 10 differences: {diffs[:10].tolist()}")
