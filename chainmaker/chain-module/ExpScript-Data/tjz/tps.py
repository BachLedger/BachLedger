import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt 

def plot_with_broken_y(df, x, y, hue, xlabel, ylabel, filename):
    # 为每个平台创建一个子图
    platforms = df['platform'].unique()
    fig, (ax1, ax2) = plt.subplots(2, 1, sharex=True, figsize=(10, 8))
    
    # 绘制每个平台的数据
    for i, platform in enumerate(platforms):
        data = df[df['platform'] == platform]
        ax = ax1 if i == 0 else ax2
        sns.lineplot(data=data, x=x, y=y, ax=ax, label=platform)
        
        ax.set_xlabel(xlabel)
        ax.set_ylabel(ylabel)
    
    # 设置y轴范围
    y1_max = df[df['platform'] == platforms[0]][y].max()
    y2_max = df[df['platform'] == platforms[1]][y].max()
    ax1.set_ylim(0, y1_max * 1.1)
    ax2.set_ylim(0, y2_max * 1.1)
    
    # 隐藏ax1的底部刻度和ax2的顶部刻度
    ax1.spines['bottom'].set_visible(False)
    ax2.spines['top'].set_visible(False)
    ax1.xaxis.tick_top()
    ax1.tick_params(labeltop=False)
    ax2.xaxis.tick_bottom()

    # 添加折断线
    d = .015  # 折断线的大小
    kwargs = dict(transform=ax1.transAxes, color='k', clip_on=False)
    ax1.plot((-d, +d), (-d, +d), **kwargs)
    ax1.plot((1 - d, 1 + d), (-d, +d), **kwargs)
    kwargs.update(transform=ax2.transAxes)
    ax2.plot((-d, +d), (1 - d, 1 + d), **kwargs)
    ax2.plot((1 - d, 1 + d), (1 - d, 1 + d), **kwargs)

    # 添加图例
    lines1, labels1 = ax1.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax2.legend(lines1 + lines2, labels1 + labels2, loc='upper right')
    ax1.get_legend().remove()

    plt.tight_layout()
    plt.savefig(filename)
    plt.close()

if __name__ == '__main__':
    df = pd.read_csv('data.csv')
    # print(df)
    # line plot of node to tps
    df['tps'] = df ['txs'] / df['total_time']
    df['platform'] = 'ours'
    baseline_name = 'baseline'
    new_data = [
        {'nodes': 4, 'txs': 2515, 'total_time': 1.0, 'GetKey_time': 0.0, 'tps': 2515.0, 'platform': baseline_name},
        {'nodes': 7, 'txs': 2306, 'total_time': 1.0, 'GetKey_time': 0.0, 'tps': 2306.0, 'platform': baseline_name},
        {'nodes': 10, 'txs': 2063, 'total_time': 1.0, 'GetKey_time': 0.0, 'tps': 2063.0, 'platform': baseline_name},
        {'nodes': 13, 'txs': 1911, 'total_time': 1.0, 'GetKey_time': 0.0, 'tps': 1911.0, 'platform': baseline_name},
        {'nodes': 16, 'txs': 1748, 'total_time': 1.0, 'GetKey_time': 0.0, 'tps': 1748.0, 'platform': baseline_name}
    ]
    df = pd.concat([df, pd.DataFrame(new_data)], ignore_index=True)
    
    print(df)   
    ax = sns.lineplot(data=df, x='nodes', y='tps', hue='platform')
    ax.legend(title=None)
    ax.set_xlabel('Number of Nodes')
    ax.set_ylabel('Throughput (txn/s)')
    # plt.show()
    plt.savefig('node-to-tps.png')
    # 绘制 node-to-tps 图
    plot_with_broken_y(df, 'nodes', 'tps', 'platform', 'Number of Nodes', 'Throughput (txn/s)', 'node-to-tps-fold-y-ax.png')
    
    plt.figure(2)
    ax = sns.lineplot(data=df, x='nodes', y='GetKey_time', hue='platform')
    ax.legend(title=None)
    ax.set_xlabel('Number of Nodes')
    ax.set_ylabel('GetKey Time (s)')
    plt.savefig('node-to-getkeytime.png')
    
    df['tps_per_node'] = df['tps'] / df['nodes']
    plt.figure(3)
    ax = sns.lineplot(data=df, x='nodes', y='tps_per_node', hue='platform')
    ax.legend(title=None)
    ax.set_xlabel('Number of Nodes')
    ax.set_ylabel('Throughput per Node (txn/s)')
    plt.savefig('node-to-tps-per-node.png')
    
    
