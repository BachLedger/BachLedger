from plot_errbind import bcos_perf
from proc import yz_perf
from cm_perf import cm_perf
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import numpy as np


if __name__ == '__main__':    
    tpss = cm_perf('chainmaker-perf-64.log')
    yz_tpss, _, _ = yz_perf('perf-64-100.log')
    bcos_tpss100, _ = bcos_perf('output/output_64_100.txt')
    bcos_tpss1000, _ = bcos_perf('output/output_64_1000.txt')

    plt.rcParams['font.size'] = 16  # 设置全局字体大小

   # 创建一个包含所有数据的列表
    data = {
             'FISCO-BCOS(block size: 100)': np.array(bcos_tpss100)/1000
            , 'FISCO-BCOS(block size: 1000)': np.array(bcos_tpss1000)/1000
            , 'ChainMaker': np.array(tpss)/1000
            , 'BachLedger': np.array(yz_tpss)/1000
            }
    # plot these four's min, median, mean, max in one figure
    min_data = [min(data['FISCO-BCOS(block size: 100)'])
                , min(data['FISCO-BCOS(block size: 1000)'])
                , min(data['ChainMaker']), min(data['BachLedger'])
    ]
    median_data = [np.median(data['FISCO-BCOS(block size: 100)'])
                , np.median(data['FISCO-BCOS(block size: 1000)'])
                , np.median(data['ChainMaker']), np.median(data['BachLedger'])
    ]
    mean_data = [np.mean(data['FISCO-BCOS(block size: 100)'])
                , np.mean(data['FISCO-BCOS(block size: 1000)'])
                , np.mean(data['ChainMaker']), np.mean(data['BachLedger'])
    ]
    max_data = [max(data['FISCO-BCOS(block size: 100)'])
                , max(data['FISCO-BCOS(block size: 1000)'])
                , max(data['ChainMaker']), max(data['BachLedger'])
    ]
    df = pd.DataFrame(columns=['platform', 'value', 'statistic'])
    platforms = ['FISCO-BCOS(block size: 100)', 'FISCO-BCOS(block size: 1000)', 'ChainMaker', 'BachLedger']
    for i in range(4):
        new_row = pd.DataFrame({'platform': [platforms[i], platforms[i], platforms[i], platforms[i]]
                                , 'value': [min_data[i], median_data[i], mean_data[i], max_data[i]]
                                , 'statistic': ['min', 'median', 'mean', 'max']
                                })
        df = pd.concat([df, new_row], ignore_index=True)
    # df.to_csv('data.csv', index=False)
    print('Data collected and saved to data.csv')
    print(df)
    # 绘制barplot
    plt.figure(figsize=(10, 6))
    bplot = sns.barplot(df, x='statistic', y='value', hue='platform', palette="Set2")
    bplot.set_xlabel(None)
    bplot.yaxis.grid(True)
    bplot.legend(title=None)
    
    # hatches = ['/', '\\', '|', '-']
    # print(bplot.patches)
    # for i, bar in enumerate(bplot.patches):
    #     hatch_pattern = hatches[i // 5]
    #     bar.set_hatch(hatch_pattern)
        
    plt.ylabel('Throughput (k txn/s)')
    plt.savefig("end-to-end-perf-bar.pdf")
    plt.show()