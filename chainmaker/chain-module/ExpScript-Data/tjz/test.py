import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import numpy as np

# 创建示例数据
np.random.seed(0)
df = pd.DataFrame({
    'x': range(1, 11),
    'y1': np.random.randint(1, 10, 10),
    'y2': np.random.randint(1000, 1500, 10)
})

# 创建图表
f, (ax1, ax2) = plt.subplots(2, 1, sharex=True, figsize=(10, 8))
f.suptitle('Y轴折叠示例', fontsize=16)

# 在两个子图上绘制数据
sns.scatterplot(data=df, x='x', y='y1', ax=ax1, color='blue', label='y1')
sns.scatterplot(data=df, x='x', y='y2', ax=ax2, color='red', label='y2')

# 设置y轴范围
ax1.set_ylim(0, 15)  # y1的范围
ax2.set_ylim(950, 1550)  # y2的范围

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

# 设置标签
ax2.set_xlabel('X轴')
ax1.set_ylabel('Y1轴')
ax2.set_ylabel('Y2轴')

# 显示图例
ax1.legend()
ax2.legend()

plt.tight_layout()
plt.show()