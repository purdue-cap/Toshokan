#!/usr/bin/env python3
from multiprocessing.pool import ThreadPool
from multiprocessing import Lock
import optparse
import subprocess
import fcntl
import time
import errno

COMMAND="cargo run --example={}"
DATA_MOD="extract_record"
DATA_FUNC="extract"

def lock_file(fd):
    while True:
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            break
        except IOError as e:
            if e.errno != errno.EAGAIN:
                raise e
            else:
                time.sleep(0.1)

def unlock_file(fd):
    fcntl.flock(fd, fcntl.LOCK_UN)

print_lock = Lock()

def work(target, command, func, data_postfix, log_file):
    with print_lock:
        print("Running on {}".format(target))
    process = subprocess.Popen(command.format(target), shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    begin_wall = time.time()
    process.wait()
    with print_lock:
        print("Finished with {}".format(target))
    wall_time = time.time() - begin_wall
    stdout_str = process.stdout.read()
    stderr_str = process.stderr.read()
    data_line = func(target, stdout_str, stderr_str, wall_time)
    log_fd = open(log_file, "a")
    lock_file(log_fd)
    log_fd.write("Target:{}\n".format(target))
    log_fd.write("Stdout:\n{}\n".format(stdout_str))
    log_fd.write("Stderr:\n{}\n".format(stderr_str))
    unlock_file(log_fd)
    data_fd = open(target + data_postfix, "a")
    lock_file(data_fd)
    data_fd.write(data_line + "\n")
    with print_lock:
        print("Output for {} finished".format(target))

def main():
    parser = optparse.OptionParser("Usage: %prog [options] <target>")
    parser.add_option("-n", "--num_job", dest="num_jobs", default=1, type="int", help="Numbers of parallel jobs")
    parser.add_option("-r", "--repeat", dest="repeat", default=1, type="int", help="Repeat time of each job")
    parser.add_option("-c", "--command", dest="command", default=COMMAND, type="string", help="Command to run, subsitute target with '{}'")
    parser.add_option("-L", "--log_file", dest="log_file", default="run_batch.log", type="string", help="Log file")
    parser.add_option("-m", "--data_process_module", dest="data_process_module", default=DATA_MOD, type="string", help="Module to look data process function in")
    parser.add_option("-f", "--data_process_func", dest="data_process_func", default=DATA_FUNC, type="string",
                    help="Data process function name. Expecting signature to be: func(target_name, stdout, stderr, wall_time) -> data(string)")
    parser.add_option("-D", "--data_postfix", dest="data_postfix", default=".data.csv", type="string", help="Data file postfix")
    (options, args) = parser.parse_args()

    process_func = getattr(__import__(options.data_process_module), options.data_process_func)
    jobs = args * options.repeat
    pool = ThreadPool(options.num_jobs)
    for j in jobs:
        pool.apply_async(work, (j, options.command, process_func, options.data_postfix, options.log_file))
    pool.close()
    pool.join()


if __name__=="__main__":
    main()


