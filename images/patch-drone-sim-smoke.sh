#!/usr/bin/env bash
set -euo pipefail

target=${1:?usage: patch-drone-sim-smoke.sh PATH}
temporary=$(mktemp)

awk '
  $0 == "sleep 10" {
    print "echo \"### waiting for daemonless ROS 2 graph discovery\""
    print "for i in $(seq 1 30); do"
    print "  N_READY=$(ros2 topic list --no-daemon --spin-time 2 2>/dev/null | grep -c \"^/fmu/out/\")"
    print "  [ \"$N_READY\" -ge 24 ] && break"
    print "  sleep 2"
    print "done"
    next
  }
  {
    gsub("ros2 topic list 2>/dev/null", "ros2 topic list --no-daemon --spin-time 2 2>/dev/null")
    print
  }
' "$target" > "$temporary"

install -m 0755 "$temporary" "$target"
rm -f "$temporary"

grep -q 'topic list --no-daemon --spin-time 2' "$target"
bash -n "$target"
