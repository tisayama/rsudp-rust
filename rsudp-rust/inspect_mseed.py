from obspy import read
st = read("../references/mseed/20251208_tsukuba_fdsnws.mseed")
print(st)
for tr in st:
    print(f"Channel: {tr.stats.channel}, Start: {tr.stats.starttime}, Duration: {tr.stats.endtime - tr.stats.starttime}")