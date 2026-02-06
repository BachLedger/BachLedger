import time
import datetime
import matplotlib.pyplot as plt

def parse_line(line):
    parts = line.split('|')
    timestamp = parts[1]
    message = parts[3]

    # Convert timestamp to Unix milliseconds
    dt = datetime.datetime.strptime(timestamp, "%Y-%m-%d %H:%M:%S.%f")
    unix_milliseconds = int(time.mktime(dt.timetuple()) * 1000 + dt.microsecond / 1000)

    # Extract key-value pairs
    data = {}
    data['timestamp'] = unix_milliseconds
    message_parts = message.split(',')

    for part in message_parts:
        if '=' in part:
            key, value = part.split('=', 1)
            data[key.strip()] = value.strip()

    return data

def bcos_perf(filename):
    
    with open(filename) as f:
        lines = f.readlines()
    print(len(lines))   
    datas = []
    for i, line in enumerate(lines):
        if i == 0:
            continue
        if '^^^^^^^^Report' in line:
            # print(i, line)
            data = parse_line(line)
            datas.append(data)
    tps = []
    for i, d in enumerate(datas):
        if i == 0:
            continue
        delta = int(d['timestamp']) - int(datas[i-1]['timestamp'])
        d['tps'] = int(d['tx']) / delta * 1e3
        tps.append(d['tps'])
    return tps, datas

if __name__ == '__main__':
    tps, _  = bcos_perf('bcos.log')

    plt.plot(tps[10:])
    plt.show()
