#!/usr/bin/env python3
from csv import DictReader
import sys
import re
from matplotlib import pyplot as plt
from matplotlib import patches

content = sys.stdin.read()
content = content.replace("\t", ",")
content = content.replace("{", '"{')
content = content.replace("}", '}"')
rdr = DictReader(content.split("\n"))
entries = list(rdr)
names_mock = [e["Benchmark"] for e in entries]
names_model = [e["Benchmark"] for e in entries if e['Model Time'] != "N/A"]

# slow_down = [float(e['Execution Time'])/float(e['Mock Time']) if e['Mock Time'] != '$\\infty$' else 0 for e in entries]
slow_down_mock = [float(e['Execution Time'])-float(e['Mock Time']) if e['Mock Time'] != '$\\infty$' else 0 for e in entries]
slow_down_model = [float(e['Execution Time'])-float(e['Model Time']) for e in entries if e['Model Time'] != "N/A"]
slow_down_ratio_mock = [float(e['Execution Time'])/float(e['Mock Time']) if e['Mock Time'] != '$\\infty$' else 0 for e in entries]
slow_down_ratio_model = [float(e['Execution Time'])/float(e['Model Time']) for e in entries if e['Model Time'] != "N/A"]
# loc_re = re.compile(r'\d+\((\d+)\\%\)') # Extracting ratio in %
loc_re = re.compile(r'(\d+)\((\d+)\\%\)') # Extracting LoC raw number
extra_code_mock = [int(loc_re.match(e['Mock LoC']).group(1)) for e in entries]
extra_code_ratio_mock = [int(loc_re.match(e['Mock LoC']).group(2)) for e in entries]
extra_code_ratio_mock = [(e+100)/100 for e in extra_code_ratio_mock]
extra_code_model = [int(loc_re.match(e['Model LoC']).group(1)) for e in entries if e['Model Time'] != "N/A"]
extra_code_ratio_model = [int(loc_re.match(e['Model LoC']).group(2)) for e in entries if e['Model Time'] != "N/A"]
extra_code_ratio_model = [(e+100)/100 for e in extra_code_ratio_model]
# extra_code = [x / (x + 100) for x in extra_code]

points_mock = list(zip(extra_code_mock, slow_down_mock, names_mock))
points_model = list(zip(extra_code_model, slow_down_model, names_model))
points_mock_ratio = list(zip(extra_code_ratio_mock, slow_down_ratio_mock, names_mock))
points_model_ratio = list(zip(extra_code_ratio_model, slow_down_ratio_model, names_model))

# points_model = [p for p in points_model if p[2] != "primality_sqrt_generators"]

# cluster_cond = lambda p: False
cluster_cond = lambda p: p[0] < 40 and p[1] < 25
clustered_x_mock = [p[0] for p in points_mock if cluster_cond(p)]
clustered_y_mock = [p[1] for p in points_mock if cluster_cond(p)]
clustered_n_mock = [p[2] for p in points_mock if cluster_cond(p)]
clustered_x_model = [p[0] for p in points_model if cluster_cond(p)]
clustered_y_model = [p[1] for p in points_model if cluster_cond(p)]
clustered_n_model = [p[2] for p in points_model if cluster_cond(p)]
xmin = min(clustered_x_mock + clustered_x_model)
ymin = min(clustered_y_mock + clustered_y_model)
xmax = max(clustered_x_mock + clustered_x_model)
ymax = max(clustered_y_mock + clustered_y_model)
xmargin = 0.1
ymargin = 0.1
width = xmax - xmin
height = ymax - ymin
box_start = (xmin - xmargin * width, ymin - ymargin * height)
box_width = width * (1 + 2*xmargin)
box_height = height * (1 + 2*ymargin)

mock_x = [p[0] for p in points_mock]
mock_y = [p[1] for p in points_mock]
mock_n = [p[2] if not cluster_cond(p) else None for p in points_mock ]
model_x = [p[0] for p in points_model]
model_y = [p[1] for p in points_model]
model_n = [p[2] if not cluster_cond(p) else None for p in points_model ]

fig = plt.figure()
ax = fig.add_subplot()

# ax.set_yscale("log")
# plt.xlabel("Mock LoC / Total LoC")
plt.xlabel("Mock or Model LoC")
# plt.ylabel("Factor of Slow down on Toshokan as Compared to Mock")
plt.ylabel("Toshokan Time - Mock or Model Time (s)")


# ax.scatter(clustered_x, clustered_y, marker="o")
ax.scatter(mock_x, mock_y, marker="x")
ax.scatter(model_x, model_y, marker="o")
ax.legend(["Mock", "Model"], loc="lower right")

for i, txt in enumerate(mock_n):
    xytext = (-10, 5)
    # Special annotate adaptions
    if txt == 'arraylist_match':
        xytext = (-20, -12)
    if txt == 'stack_match':
        xytext = (-20, -12)
    if txt == 'primality_sqrt':
        xytext = (-10, -12)
    if txt == 'heap_sort':
        xytext = (-40, -10)
    if txt:
        ax.annotate(txt, (mock_x[i], mock_y[i]), xytext=xytext, textcoords="offset points")
for i, txt in enumerate(model_n):
    xytext = (-10, 5)
    # Special annotate adaptions
    if txt == 'arraylist_match':
        xytext = (0, 5)
    if txt:
        ax.annotate(txt, (model_x[i], model_y[i]), xytext=xytext, textcoords="offset points")

rect = patches.Rectangle(box_start, box_width, box_height, facecolor='none', edgecolor='black')
ax.add_patch(rect)
ax.annotate("Clustered Benchmarks", (box_start[0]+box_width+1, box_start[1]))

# loc, _ = plt.yticks()
# loc = list(loc)
# del loc[0]
# loc[0] = 1.
# del loc[-1]
# lbl = [str(int(l)) for _, l in enumerate(loc)] 
# plt.yticks(loc, lbl)

plt.savefig("scatter.pdf")

plt.clf()

fig = plt.figure()
ax = fig.add_subplot()

plt.xlabel("Mock or Model LoC")
plt.ylabel("Toshokan Time - Mock or Model Time (s)")

ax.scatter(clustered_x_mock, clustered_y_mock, marker="x")
ax.scatter(clustered_x_model, clustered_y_model, marker="o")
ax.legend(["Mock", "Model"], loc="upper left")

for i, txt in enumerate(clustered_n_mock):
    xytext = (-15, 5)
    arrow = None
    # Special annotate adaptions
    if txt == 'evalPoly_combined':
        xytext = (-15, -10)
    if txt == 'set_match':
        xytext = (-35, 5)
    if txt:
        ax.annotate(txt, (clustered_x_mock[i], clustered_y_mock[i]), xytext=xytext, textcoords="offset points", arrowprops=arrow)
for i, txt in enumerate(clustered_n_model):
    xytext = (-15, 5)
    arrow = None
    # Special annotate adaptions
    if txt == 'evalPoly_combined':
        xytext = (-60, 5)
    if txt == 'set_match':
        xytext = (-45, 5)
    if txt:
        ax.annotate(txt, (clustered_x_model[i], clustered_y_model[i]), xytext=xytext, textcoords="offset points", arrowprops=arrow)
# loc = list(range(8,24,3)) 
# lbl = [str(l) for l in loc]
# plt.xticks(loc, lbl)

plt.savefig("scatter_cluster.pdf")

plt.clf()

# cluster_cond = lambda p: False
cluster_cond = lambda p: p[0] < 1.6 and p[1] < 7.5
clustered_x_mock = [p[0] for p in points_mock_ratio if cluster_cond(p)]
clustered_y_mock = [p[1] for p in points_mock_ratio if cluster_cond(p)]
clustered_n_mock = [p[2] for p in points_mock_ratio if cluster_cond(p)]
clustered_x_model = [p[0] for p in points_model_ratio if cluster_cond(p)]
clustered_y_model = [p[1] for p in points_model_ratio if cluster_cond(p)]
clustered_n_model = [p[2] for p in points_model_ratio if cluster_cond(p)]
xmin = min(clustered_x_mock + clustered_x_model)
ymin = min(clustered_y_mock + clustered_y_model)
xmax = max(clustered_x_mock + clustered_x_model)
ymax = max(clustered_y_mock + clustered_y_model)
xmargin = 0.1
ymargin = 0.1
width = xmax - xmin
height = ymax - ymin
box_start = (xmin - xmargin * width, ymin - ymargin * height)
box_width = width * (1 + 2*xmargin)
box_height = height * (1 + 2*ymargin)

mock_x = [p[0] for p in points_mock_ratio]
mock_y = [p[1] for p in points_mock_ratio]
mock_n = [p[2] if not cluster_cond(p) else None for p in points_mock_ratio ]
model_x = [p[0] for p in points_model_ratio]
model_y = [p[1] for p in points_model_ratio]
model_n = [p[2] if not cluster_cond(p) else None for p in points_model_ratio ]

fig = plt.figure()
ax = fig.add_subplot()

plt.xlabel("Total Loc / Benchmark LoC")
plt.ylabel("Toshokan Time / Mock or Model Time")

ax.scatter(mock_x, mock_y, marker="x")
ax.scatter(model_x, model_y, marker="o")
ax.legend(["Mock", "Model"], loc="upper left")

for i, txt in enumerate(mock_n):
    xytext = (-10, 5)
    arrow = None
    # Special annotate adaptions
    if txt == "set_match":
        xytext = (-20, -10)
    if txt == "stack_match":
        xytext = (-35, -10)
    if txt == "heap_sort":
        xytext = (-35, 5)
    if txt:
        ax.annotate(txt, (mock_x[i], mock_y[i]), xytext=xytext, textcoords="offset points", arrowprops=arrow)
for i, txt in enumerate(model_n):
    xytext = (-10, 5)
    arrow = None
    if txt == "primality_sqrt":
        xytext = (-35, -10)
    if txt == "set_match":
        xytext = (-30, 5)
    if txt == "arraylist_match":
        xytext = (-30, 5)
    if txt == "evalPoly_combined":
        xytext = (20, -5)
        arrow = {'arrowstyle':'-'}
    if txt:
        ax.annotate(txt, (model_x[i], model_y[i]), xytext=xytext, textcoords="offset points", arrowprops=arrow)

rect = patches.Rectangle(box_start, box_width, box_height, facecolor='none', edgecolor='black')
ax.add_patch(rect)
ax.annotate("Clustered Benchmarks", (box_start[0]+box_width+0.02, box_start[1]))

plt.savefig("scatter_ratio.pdf")

plt.clf()

fig = plt.figure()
ax = fig.add_subplot()

plt.xlabel("Total Loc / Benchmark LoC")
plt.ylabel("Toshokan Time / Mock or Model Time")

ax.scatter(clustered_x_mock, clustered_y_mock, marker="x")
ax.scatter(clustered_x_model, clustered_y_model, marker="o")
ax.legend(["Mock", "Model"], loc="upper left")

for i, txt in enumerate(clustered_n_mock):
    xytext = (-15, 5)
    arrow = None
    if txt == "powerroot_sqrt":
        xytext = (10, 10)
        arrow = {'arrowstyle': '-'}
    if txt == "gcd_n_numbers":
        xytext = (-15, -10)
    # Special annotate adaptions
    if txt:
        ax.annotate(txt, (clustered_x_mock[i], clustered_y_mock[i]), xytext=xytext, textcoords="offset points", arrowprops=arrow)
for i, txt in enumerate(clustered_n_model):
    xytext = (-15, 5)
    arrow = None
    if txt == "evalPoly_combined":
        xytext = (5, 0)
    if txt == "evalPoly_1":
        xytext = (-20, 5)
    if txt == "evalPoly_2":
        xytext = (-40, 5)
    # Special annotate adaptions
    if txt:
        ax.annotate(txt, (clustered_x_model[i], clustered_y_model[i]), xytext=xytext, textcoords="offset points", arrowprops=arrow)
# loc = list(range(0,29,3)) 
# lbl = [str(l) for l in loc]
# plt.xticks(loc, lbl)

plt.savefig("scatter_ratio_cluster.pdf")
