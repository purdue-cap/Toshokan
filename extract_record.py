import json
import re

def extract(target, stdout_str, stderr_str, wall_time):
    res = re.search(r"Record File: (.*\.record\.json)$", stdout_str)
    record = res.group(1)
    with open(record, "r") as f:
        data = json.load(f)
    return "{},{},{},{}".format(data["solved"], data["wall_time"], data["total_iter"], record)