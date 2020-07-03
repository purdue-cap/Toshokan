#!/usr/bin/env python3
import re, sys, os

def main():
    if len(sys.argv) < 2:
        print("Usage: {} sk_file".format(sys.argv[0]))
        sys.exit(1)
    input_file = sys.argv[1]
    included_files = set()
    with open(input_file) as f:
        while True:
            line = f.readline()
            if not line:
                break
            res = re.match(r'^include +"(.*)";$', line)
            if res != None:
                inc_file = res.group(1)
                if not inc_file in included_files:
                    included_files.add(inc_file)
                    inc_file = os.path.join(os.path.split(input_file)[0], inc_file)
                    with open(inc_file) as i_f:
                        sys.stdout.write(i_f.read())
            else:
                sys.stdout.write(line)

if __name__ == "__main__":
    main()