#!/usr/bin/env python3

import sys

def main():
    path = sys.argv[1]

    out = ""
    icount = 0

    with open(path, "r") as f:
        for line in f:
            if "}" in line:
                icount -= 1
            line = "\t"*icount + line.strip()
            if "{" in line:
                icount += 1
    
            out += line
            out += "\n"

    with open(path, "w") as f:
        f.write(out)

if __name__ == '__main__':
    main()