import numpy as np
import matplotlib.pyplot as plt
import re

def yz_perf(filename):
    # read lines of the file

    with open(filename) as f:
        lines = f.readlines()
        
    data = []
    for i, line in enumerate(lines):
        if 'commit blk[' in line:
            line = line.split('perf: commit blk')[1]
            # print(i, line)
            pattern = r'\[(\d+)\]'
            matches = re.findall(pattern, line) 
            # height, size, timestamp
            # print(matches)
            data.append([int(matches[1]), int(matches[2])])

    # print(data_lines)

    # # sort the dictionary by key and output a array
    # data = []
    # for key in sorted(data.keys()):
    #     data.append(data[key])

    # print(data)

    data_tps = []
    for i, d in enumerate(data):
        print(i, d)
        if i == 0:
            continue
        delta = d[1] - data[i-1][1]
        tps = d[0] / delta * 1e6
        data_tps.append([d[0], delta, tps])
        
    data_tps = np.array(data_tps)


    data_tps2 = np.zeros(data_tps.shape)
    data_tps3 = np.zeros(data_tps.shape)
    for i in range(1, data_tps.shape[0]):
        data_tps2[i, 0] = data_tps[i, 0]
        data_tps2[i, 1] = data_tps[i, 1]
        data_tps2[i, 2] = data_tps[i, 2] if data_tps[i, 0] > 80 else 0
        
        data_tps3[i, 0] = data_tps[i, 0]
        data_tps3[i, 1] = data_tps[i, 1]
        data_tps3[i, 2] = data_tps[i, 2] if data_tps[i, 0] > 10 and data_tps[i, 0] < 80 else 0
    
    data = data_tps[:, 2]
    # data = data[100:]
    # 使用滑动窗口计算Z分数来检测异常值
    window = 300
    rolling_mean = np.convolve(data, np.ones(window)/window, mode='same')
    rolling_std = np.sqrt(np.convolve((data - rolling_mean)**2, np.ones(window)/window, mode='same'))
    z_score = (data - rolling_mean) / rolling_std

    # 设定Z分数阈值来标记异常值
    threshold = 1
    is_outlier = np.abs(z_score) > threshold 

    # 过滤异常值
    data = data[~is_outlier]
    # data = data[data > 2000]
    
    return data, data_tps2, data_tps3


if __name__ == '__main__':
    data_tps18, data_tps28, data_tps38 = yz_perf('perf-8-100.log')
    data_tps116, data_tps216, data_tps316 = yz_perf('perf-16-100.log')
    data_tps124, data_tps224, data_tps324 = yz_perf('perf-24-100.log')
    data_tps132, data_tps232, data_tps332 = yz_perf('perf-32-100.log')
    data_tps140, data_tps240, data_tps340 = yz_perf('perf-40-100.log')
    data_tps148, data_tps248, data_tps348 = yz_perf('perf-48-100.log')
    data_tps156, data_tps256, data_tps356 = yz_perf('perf-56-100.log')
    data_tps164, data_tps264, data_tps364 = yz_perf('perf-64-100.log')

    plt.plot(data_tps18, label='tps18')
    
    plt.plot(data_tps116, label='tps116')

    plt.plot(data_tps124, label='tps124')
    
    plt.plot(data_tps132, label='tps132')
    plt.plot(data_tps140, label='tps140')
    plt.plot(data_tps148, label='tps148')
    plt.plot(data_tps156, label='tps156')
    plt.plot(data_tps164, label='tps1643')
    

    plt.legend()
    # plt.show()

    plt.figure(5)
    box_data = [data_tps18[100:], data_tps116[100:], data_tps124, data_tps132[100:]
                , data_tps140
                , data_tps148, data_tps156
                , data_tps164
                ]
    max_data = [np.max(data_tps18[100:]), np.max(data_tps116[100:])
                , np.max(data_tps124)
                , np.max(data_tps132[100:])
                , np.max(data_tps140), np.max(data_tps148)
                , np.max(data_tps156)
                , np.max(data_tps164)
                ]
    plt.boxplot(box_data, meanline=True, showmeans=True, labels=['8', '16', '24', '32', '40', '48', '56'
                                                                 , '64'
                                                                 ])
    plt.plot(np.arange(1, len(max_data)+1), max_data, label='max', color='green', linestyle='--')
    plt.xticks(np.arange(len(max_data)), ['8', '16', '24', '32', '40', '48', '56', '64'])
    plt.figure(6)
    mean_data = [np.mean(data_tps18[100:]), np.mean(data_tps116[100:]), np.mean(data_tps124)
                 , np.mean(data_tps132[100:]), np.mean(data_tps140), np.mean(data_tps148), np.mean(data_tps156)
                 , np.mean(data_tps164)
    ]
    plt.plot(mean_data, label='mean', color='green', linestyle='--')
    plt.xticks(np.arange(len(mean_data)), ['8', '16', '24', '32', '40', '48', '56', '64'])
    plt.legend()
    plt.show()
