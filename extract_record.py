import json
import re
import os

# Returns (data_line(str), solved(bool))
def extract(target, rtn_code, stdout_fn, stderr_fn, wall_time, env):
    with open(stdout_fn) as f:
        stdout_str = f.read()
    res = re.search(r"Record File: (.*\.record\.json)$", stdout_str)
    record = res.group(1)
    with open(record, "r") as f:
        data = json.load(f)
    return ('{},{},{},{},{},"{}",{},{},{}'.format(data["solved"], data["total_wall_time"],
        data["total_synthesis_time"], data["total_verification_time"],
        data["total_iter"], str(env), record, os.path.basename(stdout_fn), os.path.basename(stderr_fn)), data["solved"])