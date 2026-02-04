from obspy import read, UTCDateTime

def main():
    st = read("references/mseed/fdsnws.mseed")
    tr = st.select(channel="EHZ")[0]
    
    # Target time: 2025-11-25 09:02:44
    target = UTCDateTime("2025-11-25T09:02:44")
    
    # Slice a small window
    sub = tr.slice(target, target + 1)
    print(f"Data for EHZ at {target}:")
    print(sub.data)
    print(f"Number of samples: {len(sub.data)}")
    if len(sub.data) > 0:
        print(f"Sample range: {sub.data.min()} to {sub.data.max()}")

if __name__ == "__main__":
    main()