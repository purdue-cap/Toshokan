#!/usr/bin/env python3
from multiprocessing.pool import ThreadPool
from multiprocessing import Event, Semaphore
import os, signal, sys
from os import kill
import optparse
from select import poll
import subprocess
import fcntl
from threading import Thread
import time
import errno
import select
import tempfile

COMMAND="target/debug/examples/{}"
DATA_MOD="extract_record"
DATA_FUNC="extract"
POLL_INTERNAL=0.5 # Unit in seconds

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


def work(target, command, func, data_postfix, log_file_postfix, timeout, finish_event, running_sem, ignore_solved):
    stdout_log = tempfile.NamedTemporaryFile(suffix=log_file_postfix, prefix="{}.stdout.".format(target), dir=".", delete=False)
    stderr_log = tempfile.NamedTemporaryFile(suffix=log_file_postfix, prefix="{}.stderr.".format(target), dir=".", delete=False)
    print("Running on {}".format(target))

    process = subprocess.Popen(command.format(target), preexec_fn=lambda: os.setpgid(0, 0), shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    begin_wall = time.time()
    elapsed = 0
    pollobj = select.epoll()
    
    stdout_fd = process.stdout.fileno()
    stderr_fd = process.stderr.fileno()

    pollobj.register(stdout_fd, select.EPOLLIN | select.EPOLLHUP)
    pollobj.register(stderr_fd, select.EPOLLIN)

    process_hup = False
    timeouted = False

    while True:
        elapsed = time.time() - begin_wall
        if finish_event is not None and finish_event.is_set():
            os.killpg(os.getpgid(process.pid), signal.SIGTERM)
            os.killpg(os.getpgid(process.pid), signal.SIGKILL)
            process_hup = True
        if timeout > 0 and elapsed > timeout:
            timeouted = True
            os.killpg(os.getpgid(process.pid), signal.SIGTERM)
            os.killpg(os.getpgid(process.pid), signal.SIGKILL)
            process_hup = True
        for fd, flags in pollobj.poll(POLL_INTERNAL):
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
            rtn_code = process.poll()
            break

    stdout_log.close()
    stderr_log.close()
    pollobj.close()

    if timeouted:
        print("Timeouted with {} after {} seconds".format(target, elapsed))
        running_sem.release()
        return
    if finish_event.is_set():
        print("Terminated with {} after {} seconds due to a parallel process succeeding".format(target, elapsed))
        running_sem.release()
        return

    print("Finished with {}".format(target))
    wall_time = time.time() - begin_wall

    data_line, solved = func(target, rtn_code, stdout_log.name, stderr_log.name, wall_time)
    data_fo = open(target + data_postfix, "a")
    lock_file(data_fo)
    data_fo.write(data_line + "\n")
    unlock_file(data_fo)

    print("Output for {} finished".format(target))
    
    if finish_event is not None:
        if ignore_solved or solved:
            print("Setting completion flag")
            finish_event.set()
    running_sem.release()

# Kills every processes in the current session other than the current one
def kill_all_other(kill_current_pg = False):
    session_id = os.getsid(0)
    current_pid = os.getpid()
    current_pgid = os.getpgid(0)
    pids = [int(pid) for pid in os.listdir('/proc') if pid.isdigit()]
    for pid in pids:
        if pid == current_pid:
            continue
        try:
            if os.getsid(pid) != session_id:
                continue
            if os.getpgid(pid) == current_pgid and kill_current_pg:
                os.kill(pid, signal.SIGTERM)
                os.kill(pid, signal.SIGKILL)
            else:
                os.killpg(os.getpgid(pid), signal.SIGTERM)
                os.killpg(os.getpgid(pid), signal.SIGKILL)
        except ProcessLookupError:
            continue

def main():
    parser = optparse.OptionParser("Usage: %prog [options] <target>")
    parser.add_option("-n", "--num_job", dest="num_jobs", default=1, type="int", help="Numbers of parallel jobs")
    parser.add_option("-r", "--repeat", dest="repeat", default=1, type="int", help="Repeat time of each job")
    parser.add_option("-F", "--fastest", dest="fastest", default=False, action="store_true", help="Return when we have results, yielding just the fastest results, ignores --repeat")
    parser.add_option("-t", "--timeout", dest="timeout", default=0, type="int", help="Timeout when waiting for result, in seconds")
    parser.add_option("-c", "--command", dest="command", default=COMMAND, type="string", help="Command to run, subsitute target with '{}'")
    parser.add_option("-L", "--log_file_postfix", dest="log_file_postfix", default=".log", type="string", help="Log file postfix")
    parser.add_option("-m", "--data_process_module", dest="data_process_module", default=DATA_MOD, type="string", help="Module to look data process function in")
    parser.add_option("-f", "--data_process_func", dest="data_process_func", default=DATA_FUNC, type="string",
                    help="Data process function name. Expecting signature to be: func(target_name, stdout_file_path, stderr_file_path, wall_time) -> data(string)")
    parser.add_option("-D", "--data_postfix", dest="data_postfix", default=".data.csv", type="string", help="Data file postfix")
    parser.add_option("-S", "--ignore_solved", dest="ignore_solved", action="store_true", default=False, help="Ignores `solved` flag, terminating all jobs once a single job returned no matter the outcome.")
    (options, args) = parser.parse_args()

    pid = os.fork()
    if pid != 0:
        signal.signal(signal.SIGINT, lambda _sig, _frame: os.kill(pid, signal.SIGINT))
        signal.signal(signal.SIGTERM, lambda _sig, _frame: os.kill(pid, signal.SIGTERM))
        os.waitpid(pid, 0)
        return
    os.setsid()
    def clean_exit(_sig, _frame):
        print("Cleanup before exit prematruely")
        kill_all_other(True)
        sys.exit(1)
    signal.signal(signal.SIGINT, clean_exit)
    signal.signal(signal.SIGTERM, clean_exit)

    try:
        process_func = getattr(__import__(options.data_process_module), options.data_process_func)
        if options.fastest:
            finish_event = Event()
            for job in args:
                print("Running job {} for fastest".format(job))
                finish_event.clear()
                running_sem = Semaphore(0)
                pool = ThreadPool(options.num_jobs)
                for _ in range(options.num_jobs):
                    pool.apply_async(work, (job, options.command, process_func, options.data_postfix, options.log_file_postfix, 0, finish_event, running_sem, options.ignore_solved))
                print("Waiting for one thread to finish")
                timeout = None if options.timeout <= 0 else options.timeout
                for _ in range(options.num_jobs):
                    if not running_sem.acquire(timeout=timeout):
                        print("Job {} timeout".format(job))
                        print("Setting completion flag")
                        finish_event.set()
                        break
                    if finish_event.is_set():
                        break
                pool.close()
                print("Killing all remaining processes")
                kill_all_other()
                pool.join()
        else:
            jobs = args * options.repeat
            pool = ThreadPool(options.num_jobs)
            for j in jobs:
                pool.apply_async(work, (j, options.command, process_func, options.data_postfix, options.log_file_postfix, options.timeout, None))
            pool.close()
            pool.join()
    finally:
        print("Cleaning up any unkilled processes")
        kill_all_other(True)


if __name__=="__main__":
    main()


