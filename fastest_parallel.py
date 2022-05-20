#!/usr/bin/env python3
from argparse import ArgumentParser
import asyncio
import os, re, json

async def run(job_name, job_count):
    exec_file = f"target/release/examples/{job_name}"
    if not os.path.isfile(exec_file):
        exec_file = f"target/debug/examples/{job_name}"
        if not os.path.isfile(exec_file):
            print(f"Executable not found for {job_name}")
            return
    print(f"Running {exec_file}, for fastest out of {job_count} parallel runs") 
    sub_procs = [
        asyncio.create_subprocess_shell(exec_file,
            stdin=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE)
        for _ in range(job_count)]
    sub_procs = await asyncio.gather(*sub_procs)
    tasks = [asyncio.create_task(proc.communicate()) for proc in sub_procs]
    first_finished = await next(asyncio.as_completed(tasks))
    print("Fastest finished, killing all running processes")
    for proc in sub_procs:
        proc.kill()
    stdout, stderr = first_finished

    print(f"STDOUT:\n{stdout}")
    print(f"STDERR:\n{stderr}")

    res = re.search(r"Record File: (.*\.record\.json)$", stdout)
    record = res.group(1)
    with open(record, "r") as f:
        data = json.load(f)
    
    with open("result.jsonl", "a") as f:
        f.write(json.dumps({
            "name": job_name,
            "record_file": record,
            "solved": data["solved"],
            "total_iter": data["total_iter"]+1,
            "total_synthesis_time": data["total_synthesis_time"],
            "total_verification_time": data["total_verification_time"],
            "total_wall_time": data["total_wall_time"]
        }))


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("-j", "--jobs", default=10, help="Number of parallel jobs to run")
    parser.add_argument("job_name", nargs="+", help="Name of job to run")

    args = parser.parse_args()

    for job_name in args.job_name:
        asyncio.run(run(job_name, args.jobs))
