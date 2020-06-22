#!/usr/bin/env python3
from multiprocessing.pool import ThreadPool
from multiprocessing import Lock
import optparse
import subprocess
import fcntl
import time
import errno
import select
import tempfile

COMMAND="target/debug/examples/{}"
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

def work(target, command, func, data_postfix, log_file_postfix):
    stdout_log = tempfile.NamedTemporaryFile(suffix=log_file_postfix, prefix="{}.stdout.".format(target), dir=".", delete=False)
    stderr_log = tempfile.NamedTemporaryFile(suffix=log_file_postfix, prefix="{}.stderr.".format(target), dir=".", delete=False)
    with print_lock:
        print("Running on {}".format(target))

    process = subprocess.Popen(command.format(target), shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    begin_wall = time.time()
    pollobj = select.epoll()
    
    stdout_fd = process.stdout.fileno()
    stderr_fd = process.stderr.fileno()

    pollobj.register(stdout_fd, select.EPOLLIN | select.EPOLLHUP)
    pollobj.register(stderr_fd, select.EPOLLIN)

    process_hup = False

    while True:
        for fd, flags in pollobj.poll():
            if fd == stdout_fd and (flags & select.EPOLLIN):
                content = process.stdout.readline()
                stdout_log.write(content)
                stdout_log.flush()
            if fd == stderr_fd and (flags & select.EPOLLIN):
                content = process.stderr.readline()
                stderr_log.write(content)
                stderr_log.flush()
            if fd == stdout_fd and (flags & select.EPOLLHUP):
                stdout_log.write(process.stdout.read())
                stdout_log.flush()
                stderr_log.write(process.stderr.read())
                stderr_log.flush()
                process_hup = True
        if process_hup:
            break

    stdout_log.close()
    stderr_log.close()
    pollobj.close()

    with print_lock:
        print("Finished with {}".format(target))
    wall_time = time.time() - begin_wall

    data_line = func(target, stdout_log.name, stderr_log.name, wall_time)
    data_fo = open(target + data_postfix, "a")
    lock_file(data_fo)
    data_fo.write(data_line + "\n")

    with print_lock:
        print("Output for {} finished".format(target))

def main():
    parser = optparse.OptionParser("Usage: %prog [options] <target>")
    parser.add_option("-n", "--num_job", dest="num_jobs", default=1, type="int", help="Numbers of parallel jobs")
    parser.add_option("-r", "--repeat", dest="repeat", default=1, type="int", help="Repeat time of each job")
    parser.add_option("-c", "--command", dest="command", default=COMMAND, type="string", help="Command to run, subsitute target with '{}'")
    parser.add_option("-L", "--log_file_postfix", dest="log_file_postfix", default=".log", type="string", help="Log file postfix")
    parser.add_option("-m", "--data_process_module", dest="data_process_module", default=DATA_MOD, type="string", help="Module to look data process function in")
    parser.add_option("-f", "--data_process_func", dest="data_process_func", default=DATA_FUNC, type="string",
                    help="Data process function name. Expecting signature to be: func(target_name, stdout_file_path, stderr_file_path, wall_time) -> data(string)")
    parser.add_option("-D", "--data_postfix", dest="data_postfix", default=".data.csv", type="string", help="Data file postfix")
    (options, args) = parser.parse_args()

    process_func = getattr(__import__(options.data_process_module), options.data_process_func)
    jobs = args * options.repeat
    pool = ThreadPool(options.num_jobs)
    for j in jobs:
        pool.apply_async(work, (j, options.command, process_func, options.data_postfix, options.log_file_postfix))
    pool.close()
    pool.join()


if __name__=="__main__":
    main()


