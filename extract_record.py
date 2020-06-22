import json
import re

def extract(target, stdout_str, stderr_str, wall_time):
    res = re.search(r"Record File: (.*\.record\.json)$", stdout_str)
    with open(res.group(1), "r") as f:
        data = json.load(f)
    return "{},{},{}".format(data["solved"], data["wall_time"], data["total_iter"])