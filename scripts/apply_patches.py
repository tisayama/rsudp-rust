import os

def patch_raspberryshake():
    path = 'references/rsudp/rsudp/raspberryshake.py'
    if not os.path.exists(path): return
    with open(path, 'r') as f: lines = f.readlines()
    new_lines = []
    patched_chn = False
    patched_chns = False
    i = 0
    while i < len(lines):
        line = lines[i]
        # Patch getCHN: support binary/error
        if "return str(DP.decode('utf-8').split(\",\")[0][1:]).strip" in line and not patched_chn:
            indent = line[:line.find('return')]
            new_lines.append(indent + "try:\n")
            new_lines.append(indent + "\t" + line.strip() + "\n")
            new_lines.append(indent + "except:\n")
            new_lines.append(indent + "\treturn 'EHZ'\n")
            patched_chn = True
            i += 1; continue
        # Patch getCHNS: sample more packets
        if "def getCHNS():" in line and not patched_chns:
            new_lines.append(line)
            while i + 1 < len(lines) and not lines[i+1].lstrip().startswith("def "):
                i += 1
            new_lines.append("\tglobal chns\n")
            new_lines.append("\tfound = set()\n")
            new_lines.append("\tfor _ in range(500):\n")
            new_lines.append("\t\ttry:\n")
            new_lines.append("\t\t\tdata = getDATA()\n")
            new_lines.append("\t\t\tch = getCHN(data)\n")
            new_lines.append("\t\t\tif ch != 'UNK': found.add(ch)\n")
            new_lines.append("\t\texcept: pass\n")
            new_lines.append("\tchns = sorted(list(found)) if found else ['EHZ']\n")
            new_lines.append("\treturn chns\n")
            patched_chns = True
            i += 1; continue
        new_lines.append(line)
        i += 1
    with open(path, 'w') as f: f.writelines(new_lines)
    print("Patched raspberryshake.py")

def patch_c_alert():
    path = 'references/rsudp/rsudp/c_alert.py'
    if not os.path.exists(path): return
    with open(path, 'r') as f: lines = f.readlines()
    new_lines = []
    patched = False
    for line in lines:
        new_lines.append(line)
        # Find print statement in _print_stalta
        if "print(COLOR['current'] + COLOR['bold'] + msg + COLOR['white']" in line and not patched:
            indent = line[:line.find('print')]
            # Add a one-liner CSV log to avoid indentation issues
            log_code = indent + "with open('logs/py_stalta.csv', 'a') as _f: _f.write('%s,%s\n' % (self.stream[0].stats.endtime.isoformat(), self.stalta[-1]))\n"
            new_lines.append(log_code)
            patched = True
    if patched:
        with open(path, 'w') as f: f.writelines(new_lines)
        print("Patched c_alert.py")

if __name__ == "__main__":
    patch_raspberryshake()
    patch_c_alert()
