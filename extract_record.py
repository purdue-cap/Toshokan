import json
import re
import os

def extract(target, stdout_fn, stderr_fn, wall_time):
    with open(stdout_fn) as f:
        stdout_str = f.read()
    res = re.search(r"Record File: (.*\.record\.json)$", stdout_str)
    record = res.group(1)
    with open(record, "r") as f:
        data = json.load(f)
    return "{},{},{},{},{},{},{},{},{}".format(data["solved"], data["total_wall_time"],
        data["total_synthesis_time"], data["total_verification_time"], data["total_trace_time"],
        data["total_iter"], record, os.path.basename(stdout_fn), os.path.basename(stderr_fn))