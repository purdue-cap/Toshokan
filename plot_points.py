#!/usr/bin/env python3
from csv import DictReader
import sys
import re
from matplotlib import pyplot as plt
from matplotlib import patches

content = sys.stdin.read()
content = content.replace("\t", ",")
rdr = DictReader(content.split("\n"))
entries = list(rdr)
names = [e["Benchmark"] for e in entries]

# slow_down = [float(e['Execution Time'])/float(e['Mock Time']) if e['Mock Time'] != '$\\infty$' else 0 for e in entries]
slow_down = [float(e['Execution Time'])-float(e['Mock Time']) if e['Mock Time'] != '$\\infty$' else 0 for e in entries]
# loc_re = re.compile(r'\d+\((\d+)\\%\)') # Extracting ratio in %
loc_re = re.compile(r'(\d+)\(\d+\\%\)') # Extracting LoC raw number
extra_code = [int(loc_re.match(e['Mock LoC']).group(1)) for e in entries]
# extra_code = [x / (x + 100) for x in extra_code]

points = list(zip(extra_code, slow_down, names))

cluster_cond = lambda p: False
# clustered_x = [p[0] for p in points if cluster_cond(p)]
# clustered_y = [p[1] for p in points if cluster_cond(p)]
# clustered_n = [p[2] for p in points if cluster_cond(p)]
# xmin = min(clustered_x)
# ymin = min(clustered_y)
# xmax = max(clustered_x)
# ymax = max(clustered_y)
# xmargin = 0.15
# ymargin = 0.2
# width = xmax - xmin
# height = ymax - ymin
# box_start = (xmin - xmargin * width, ymin - ymargin * height)
# box_width = width * (1 + 2*xmargin)
# box_height = height * (1 + 2*ymargin)


other_x = [p[0] for p in points if not cluster_cond(p)]
other_y = [p[1] for p in points if not cluster_cond(p)]
other_n = [p[2] for p in points if not cluster_cond(p)]

fig = plt.figure()
ax = fig.add_subplot()

# ax.set_xscale("log")
# plt.xlabel("Mock LoC / Total LoC")
plt.xlabel("Mock LoC")
# plt.ylabel("Factor of Slow down on Toshokan as Compared to Mock")
plt.ylabel("Extra Time on Toshokan as Compared to Mock / s")


# ax.scatter(clustered_x, clustered_y, marker="o")
ax.scatter(other_x, other_y, marker="x")

for i, txt in enumerate(other_n):
    xytext = (-10, 5)
    # Special annotate adaptions
    # if txt == 'evalPoly_combined':
    #     xytext = (-10, -10)
    # if txt == 'heap_test_param':
    #     xytext = (-40, -10)
    # if txt == 'heap_test':
    #     xytext = (-35, 5)
    # if txt == 'heap_test_complex':
    #     xytext = (-50, 5)
    ax.annotate(txt, (other_x[i], other_y[i]), xytext=xytext, textcoords="offset points")

# rect = patches.Rectangle(box_start, box_width, box_height, facecolor='none', edgecolor='black')
# ax.add_patch(rect)
# ax.annotate("Clustered Benchmarks", (box_start[0]+box_width, box_start[1]+box_height))

# loc, _ = plt.yticks()
# loc = list(loc)
# del loc[0]
# loc[0] = 1.
# del loc[-1]
# lbl = [str(int(l)) for _, l in enumerate(loc)] 
# plt.yticks(loc, lbl)

plt.savefig("scatter.png")

# plt.clf()

# fig = plt.figure()
# ax = fig.add_subplot()

# plt.xlabel("% of Mock LoC as Compared to Benchmark LoC")
# plt.ylabel("Factor of Slow down on Toshokan as Compared to Mock")

# ax.scatter(clustered_x, clustered_y, marker="o")

# for i, txt in enumerate(clustered_n):
#     ax.annotate(txt, (clustered_x[i], clustered_y[i]))

# plt.savefig("scatter_cluster.png")