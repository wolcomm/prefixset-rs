#!/usr/bin/env python

import argparse
import ipaddress
import json
import sys

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("file")
    parser.add_argument("--ranges", action="store_true", default=False)
    args = parser.parse_args()
    with open(args.file) as f:
        data = json.load(f)
    s = set()
    for entry in data:
        p = ipaddress.ip_network(entry["prefix"])
        if entry["exact"]:
            l = u = p.prefixlen
        else:
            l = entry.get("greater-equal", p.prefixlen)
            u = entry["less-equal"]
        s.add((p, l, u))
    if args.ranges:
        for p, l, u in s:
            sys.stdout.write(f"{p},{l},{u}\n")
    else:
        for p, l, u in s:
            for k in range(l, u + 1):
                for q in p.subnets(new_prefix=k):
                    sys.stdout.write(f"{q}\n")


if __name__ == "__main__":
    sys.exit(main())
