
import numpy as np
from scipy.signal import butter, sosfreqz
import json

def get_coeffs(low, high, rate):
    nyq = 0.5 * rate
    low_norm = low / nyq
    high_norm = high / nyq
    sos = butter(4, [low_norm, high_norm], btype='band', output='sos')
    return sos.tolist()

if __name__ == "__main__":
    # Settings from rsudp_settings.toml (approx)
    # 0.7 - 2.0 Hz seems to be default? Or config says something else?
    # Let's assume standard defaults first, or read from config.
    # In rsudp_settings.toml: highpass=0.1, lowpass=2.0
    sos = get_coeffs(0.1, 2.0, 100.0)
    print(json.dumps(sos))
