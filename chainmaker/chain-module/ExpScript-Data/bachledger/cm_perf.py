import re 
import matplotlib.pyplot as plt
from proc import yz_perf 
import numpy as np
from plot_errbind import bcos_perf

def cm_perf(filename: str):
    with open(filename) as f:
        lines = f.readlines()
        
    tpss = []
    for i, line in enumerate(lines):
        if 'commit block [' in line:
            interval = line.split("interval")[1]
            pattern = r'\:(\d+)\)'
            interval = re.findall(pattern, interval)[0]
            
            count = line.split("count:")[1].split(",hash")[0]
            print(interval, count)
            tps = int(count)/int(interval) * 1e3
            if tps > 500:
                tpss.append(tps)
    # print(lines)
    return tpss
    
    
if __name__ == '__main__':    
    tpss = cm_perf('chainmaker-perf-64.log')
    yz_tpss, _, _ = yz_perf('perf-64-100.log')
    bcos_tpss100, _ = bcos_perf('output/output_64_100.txt')
    bcos_tpss1000, _ = bcos_perf('output/output_64_1000.txt')

    import seaborn as sns
    plt.rcParams['font.size'] = 16  # 设置全局字体大小

   # 创建一个包含所有数据的列表
    data = {
             'FISCO-BCOS(block size: 100)': np.array(bcos_tpss100)/1000
            , 'FISCO-BCOS(block size: 1000)': np.array(bcos_tpss1000)/1000
            , 'ChainMaker': np.array(tpss)/1000
            , 'BachLedger': np.array(yz_tpss)/1000
            }

    # 绘制boxenplot
    plt.figure(figsize=(10, 6))
    bplot = sns.violinplot(data=data, palette="Set2", legend='brief')
    bplot.set_xticks([])
    
    # plt.title('Comparison of Sample Distributions')
    # plt.xlabel('Sample')
    plt.ylabel('Throughput (k txn/s)')
    plt.savefig("end-to-end-perf.pdf")
    # plt.show()