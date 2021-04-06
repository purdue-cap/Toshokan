#!/usr/bin/env python3
import json
import argparse
import itertools

parser = argparse.ArgumentParser()
parser.add_argument("-H", "--hole", default=[], action="append", help="Hole values to output, use 'all' for all holes")
parser.add_argument("-s", "--stateful_config", default=[], action="append", help="Stateful function configuration, format: [FuncName]/[StateArgIdx]")
parser.add_argument("-p", "--prefix", help="Strip this prefix when outputing function names")
parser.add_argument("-x", "--suffix", help="Strip this suffix when outputing function names")

sub_parsers = parser.add_subparsers(dest="analysis", required=True)
parser.add_argument("record_file", help="Record file to parse")

glb_parser = sub_parsers.add_parser("global")
glb_parser.add_argument("-S", "--stateful_trace", default=[], action="append", help="Stateful function configurations to count trace for")

stf_parser = sub_parsers.add_parser("stateful")
stf_parser.add_argument("stateful_trace", help="Stateful function configurations to count trace for")


args = parser.parse_args()

with open(args.record_file) as f:
    record = json.load(f)
entries = record["entries"]
header = ["iter #"]
items = []
for e in entries:
    items.append([str(e['iter_nth'])])

if args.analysis == "global":
    header += ["C.E.#", "Raw Traces", "Synth Time", "Veri Time", "Trace Time"]
    c_e = set()
    total_raw_traces = 0
    for e, i in zip(entries, items):
        if 'new_c_e_s' in e:
            c_e.add(tuple(e['new_c_e_s']))
        i.append(str(len(c_e)))
        if 'new_traces' in e:
            total_raw_traces += len(e['new_traces'])
        i.append(str(total_raw_traces))
        i.append(str(e.get("synthesis_wall_time", "N/A")))
        i.append(str(e.get("verification_wall_time", "N/A")))
        i.append(str(e.get("trace_wall_time", "N/A")))
        
if "all" in args.hole:
    args.hole = []
    for e in entries:
        for h in e.get("holes", {}):
            if not h in args.hole:
                args.hole.append(h)
header.extend(args.hole)
for e, i in zip(entries, items):
    i.extend(v if isinstance(v:=e.get("holes", {}).get(h, "(Unchanged)"), str) else str(v) for h in args.hole)

st_funcs = [i.split("/") for i in args.stateful_config]
st_funcs = dict((i[0], int(i[1])) for i in st_funcs) # {func_name -> state_arg_idx}

st_traces = dict((k, set()) for k in st_funcs.keys())

def convert_value(value):
    if isinstance(value, dict) and value.get("@placeholder") == "UNSUPPORTED":
        return "@"
    else:
        return str(value)

def strip_fixes(func):
    if args.prefix and func.startswith(args.prefix):
        func = func[len(args.prefix):]
    if args.suffix and func.endswith(args.suffix):
        func = func[:-len(args.suffix)]
    return func

def format_log(log):
    func = strip_fixes(log[0])
    return "{}({})={}".format(func, ",".join(log[1]), log[2])

if args.analysis == "global":
    header.extend("{} #traces".format(strip_fixes(f)) for f in args.stateful_trace)
elif args.analysis == "stateful":
    header.append("new traces for {} ---->".format(strip_fixes(args.stateful_trace)))

for e, i in zip(entries, items):
    if not "new_traces" in e:
        if args.analysis == "global":
            for f in args.stateful_trace:
                i.append("(Unchanged)")
        elif args.analysis == "stateful":
            i.append([["(Unchanged)"]])
        continue
    raw_traces = e["new_traces"]
    cur_state = {}
    new_st_traces = dict((k, set()) for k in st_funcs.keys())
    for trace in raw_traces:
        if trace["meta"] == "TestStart":
            cur_state = {}
            continue
        if trace["meta"] == "FuncCall":
            func = trace["func"]
            idx = st_funcs[func]
            if idx >= 0: 
                addr = trace["args"][idx]["@address"]
            else: # if idx < 0, treat it as init function and look for addr in return value
                addr = trace["rtn"]["@address"]
            if not addr in cur_state:
                cur_state[addr] = []
            call_args = tuple(map(convert_value, trace["args"]))
            cur_state[addr].append((
                func, call_args, convert_value(trace["rtn"])
            )) #(func, args, rtn)
        if trace["meta"] == "TestEnd":
            for state in cur_state.values():
                hist = []
                for last_hist in state:
                    hist.append(last_hist)
                    func = last_hist[0]
                    new_st_traces[func].add(tuple(hist))
    for func in st_traces:
        new_st_traces[func] = new_st_traces[func].difference(st_traces[func])
        st_traces[func] = st_traces[func].union(new_st_traces[func])
    if args.analysis == "global":
        for f in args.stateful_trace:
            i.append(str(len(st_traces[f])))
    elif args.analysis == "stateful":
        l = list(list(map(format_log, hist)) for hist in new_st_traces[args.stateful_trace])
        l.sort(key=lambda x: len(x))
        if l:
            i.append(l)
        else:
            i.append([["(Unchanged)"]])

if args.analysis == "stateful":
    items = [[item[:-1]+log for log in item[-1]] for item in items]
    items = list(itertools.chain(*items))

print(",".join(header))
print("\n".join(",".join('"{}"'.format(e) for e in i) for i in items))