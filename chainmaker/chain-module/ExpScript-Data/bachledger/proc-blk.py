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
        # print(i, d)
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
    threshold = 0.5 if '100' in filename else 1.5
    is_outlier = np.abs(z_score) > threshold 

    # 过滤异常值
    data = data[~is_outlier]
    # data = data[data > 2000]
    
    return data, data_tps2, data_tps3


if __name__ == '__main__':
    data100, _, _ = yz_perf('perf-64-100.log')
    # plt.plot(data100, label='100txs/blk')
    
    data200, _, _ = yz_perf('perf-64-200.log')
    plt.plot(data200, label='200txs/blk')
    
    data500, _, _ = yz_perf('perf-64-500.log')
    # plt.plot(data500, label='500txs/blk')
    
    data1000, _, _ = yz_perf('perf-64-1000.log')
    # plt.plot(data1000, label='1000txs/blk')
    
    plt.xlabel('block height')
    plt.ylabel('tps')
    plt.legend()
    
    box_data = [data1000, data500, data200, data100]
    plt.figure(2)
    plt.boxplot(box_data, tick_labels=[64/1000, 64/500, 64/200, 64/100], showmeans=True)
    plt.xlabel(r'Relative Computation Abundance ($\rho$)')
    plt.ylabel('Throughput (txn/s)')
    
    # 自定义图例
    mean_marker = plt.Line2D([], [], color='green', marker='^', linestyle='None', label='Mean')
    median_line = plt.Line2D([], [], color='orange', linestyle='-', label='Median')
    plt.legend(handles=[mean_marker, median_line], loc='upper left')
    
    # plt.savefig('yz-perf-64-blk-comp.pdf')
    plt.show()
    
   
