import re
import numpy as np
import matplotlib.pyplot as plt

def get_idles(filename):
    with open(filename) as f:
        lines = f.readlines()

    data_lines = {}
    for i, line in enumerate(lines):
        line = line.split('yy')[1]
        # print(i, line)
        pattern = r'\[(\d+)\]'
        matches = re.findall(pattern, line) 
        # height, size, tb, tt
        data_lines[int(matches[0])] = [int(matches[1]), int(matches[2]), int(matches[3]), int(matches[4])]

    # print(data_lines)

    # sort the dictionary by key and output a array
    data = []
    for key in sorted(data_lines.keys()):
        data.append(data_lines[key])

    # print(data)

    idles = []

    for i, d in enumerate(data):
        idle = (1 - d[2]/(d[1]*(d[3]/4)))
        # if idle > 0.85:
        idles.append(idle)
    return idles


if __name__ == '__main__':
    idles8 = get_idles('chainmaker-idle-test-8.log')
    idles16 = get_idles('chainmaker-idle-test-16.log')
    idles24 = get_idles('chainmaker-idle-test-24.log')
    idles32 = get_idles('chainmaker-idle-test-32.log')
    idles40 = get_idles('chainmaker-idle-test-40.log')
    idles64 = get_idles('chainmaker-idle-test-64.log')
    plt.plot(idles8, label='8')
    plt.plot(idles16, label='16')
    plt.plot(idles24, label='24')
    plt.plot(idles32, label='32')
    plt.plot(idles40, label='40')
    plt.plot(idles64, label='64')
    plt.legend()

    mean_idles = [np.mean(idles8), np.mean(idles16), np.mean(idles24)
                , np.mean(idles32)
                , np.mean(idles40), np.mean(idles64)]
    medium_idles = [np.median(idles8), np.median(idles16), np.median(idles24)
                    , np.median(idles32)
                    , np.median(idles40), np.median(idles64)]
    max_idles = [np.max(idles8), np.max(idles16), np.max(idles24)
                    , np.max(idles32)
                    , np.max(idles40), np.max(idles64)]
    x = [8, 16, 24, 32, 40, 64]
    plt.figure(2)
    plt.plot(mean_idles, label='mean')
    plt.plot(medium_idles, label='median')
    plt.plot(max_idles, label='max')
    plt.xticks(np.arange(len(x)), x)
    plt.legend()
    # plt.show()

    plt.figure(3)
    data = [idles8, idles16, idles24
            , idles32
            , idles40, idles64]

    # 创建箱线图
    plt.boxplot(data, whis=1.0, showfliers=False, showmeans=True)
    plt.plot(np.arange(1, len(x)+1), mean_idles, label='mean', color='green', linestyle='--')
    plt.plot(np.arange(1, len(x)+1), medium_idles, label='median', color='orange', linestyle='-')


    # 自定义图例
    mean_marker = plt.Line2D([], [], color='green', marker='^', linestyle='None', label='Mean')
    median_line = plt.Line2D([], [], color='orange', linestyle='-', label='Median')
    plt.legend(handles=[mean_marker, median_line], loc='lower right')


    # 添加标题和标签
    # plt.title('Thread Idle Time While Increasing the Number of Cores')
    plt.xlabel('Number of Cores (N)')
    plt.ylabel(r'Relative Thread Idle Time ($\delta$)')

    plt.xticks(np.arange(1, len(x)+1), x)
    plt.savefig('idle.pdf')
