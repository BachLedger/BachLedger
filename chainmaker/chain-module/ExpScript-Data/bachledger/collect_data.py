import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
from plot_errbind import bcos_perf
from proc import yz_perf
from cm_perf import cm_perf

def collect_data(n_cores=[64,],
    blk_size=[100, 200, 500, 1000]):

    df = pd.DataFrame(columns=['tps', 'cores', 'blk_size', 'platform'])

    for n_core in n_cores:
        for blk_tx in blk_size:
            yz_tpss, _, _ = yz_perf(f'perf-{n_core}-{blk_tx}.log')
            bcos_tpss, _ = bcos_perf(f'output/output_{n_core}_{blk_tx}.txt')
            
            for i, tps in enumerate(yz_tpss):
                if blk_size == 200:
                    if i < 200 or (i > 360 and i < 600) or i > 760:
                        continue
                # Assuming df is your existing DataFrame
                new_row = pd.DataFrame({'tps': [tps], 'cores': [n_core], 'blk_size': [blk_tx], 'platform': ['BachLedger']})
                df = df.astype({'tps': float, 'cores': int, 'blk_size': int, 'platform': str})
                new_row = new_row.astype({'tps': float, 'cores': int, 'blk_size': int, 'platform': str})
                df = pd.concat([df, new_row], ignore_index=True)
            for tps in bcos_tpss:
                # Assuming df is your existing DataFrame
                new_row = pd.DataFrame({'tps': [tps], 'cores': [n_core], 'blk_size': [blk_tx], 'platform': ['FISCO-BCOS']})
                df = df.astype({'tps': float, 'cores': int, 'blk_size': int, 'platform': str})
                new_row = new_row.astype({'tps': float, 'cores': int, 'blk_size': int, 'platform': str})
                df = pd.concat([df, new_row], ignore_index=True)
        cm_tpss = cm_perf(f'chainmaker-perf-{n_core}.log')
        for tps in cm_tpss:
            # Assuming df is your existing DataFrame
            new_row = pd.DataFrame({'tps': [tps], 'cores': [n_core], 'blk_size': [blk_tx], 'platform': ['ChainMaker']})
            df = df.astype({'tps': float, 'cores': int, 'blk_size': int, 'platform': str})
            new_row = new_row.astype({'tps': float, 'cores': int, 'blk_size': int, 'platform': str})
            df = pd.concat([df, new_row], ignore_index=True)
    return df

if __name__ == '__main__':
    df = collect_data()
    print(df)
    df.to_csv('data.csv', index=False)
    print('Data collected and saved to data.csv')
    
    df = pd.read_csv('data.csv')
    
    df = df[df['platform'] != 'ChainMaker']
    df['rho'] = df['cores'] / df['blk_size']
    df['tps'] = df['tps'] / 1000
    
    ax = sns.violinplot(data=df, x='rho', y='tps', hue='platform', legend='brief', split=True, gap=.1, inner="quart")
    ax.set_xlabel(r'Relative Computation Abundance ($\rho$)')
    ax.set_ylabel('Throughput (k txn/s)')
    ax.yaxis.grid(True)
    ax.legend(title=None)

    # plt.show()
    plt.savefig('blk-comp.pdf')
