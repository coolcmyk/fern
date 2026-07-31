ARG BASE_IMAGE
FROM ${BASE_IMAGE}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

COPY tests/lane-a-smoke.sh /usr/local/bin/fern-lane-a-smoke
RUN sed -i \
      -e '/^sleep 10$/c\
echo "### waiting for daemonless ROS 2 graph discovery"\
for i in $(seq 1 30); do\
  N_READY=$(ros2 topic list --no-daemon --spin-time 2 2>/dev/null | grep -c "^/fmu/out/")\
  [ "$N_READY" -ge 24 ] && break\
  sleep 2\
done' \
      -e 's|ros2 topic list 2>/dev/null|ros2 topic list --no-daemon --spin-time 2 2>/dev/null|' \
      /usr/local/bin/fern-lane-a-smoke \
    && chmod +x /usr/local/bin/fern-lane-a-smoke \
    && grep -q 'topic list --no-daemon' /usr/local/bin/fern-lane-a-smoke

# Runpod does not expose Docker's --shm-size option. Keeping every process in one
# container and forcing Fast DDS onto loopback UDP avoids its shared-memory transport.
ENV DURATION=300 \
    FASTDDS_BUILTIN_TRANSPORTS=UDPv4 \
    OUTDIR=/workspace/fern/drone-sim

# Runpod restarts a container that exits while the Pod's desired state is RUNNING.
# Stop the Pod through its injected, Pod-scoped credential after recording the result.
CMD ["bash", "-lc", "mkdir -p \"$OUTDIR\"; set -o pipefail; /usr/local/bin/fern-lane-a-smoke 2>&1 | tee \"$OUTDIR/smoke.log\"; status=${PIPESTATUS[0]}; printf '%s\\n' \"$status\" > \"$OUTDIR/smoke.exit\"; if [[ -n \"${RUNPOD_POD_ID:-}\" && -n \"${RUNPOD_API_KEY:-}\" ]] && curl --fail --silent --show-error --retry 5 --request POST --header \"Authorization: Bearer ${RUNPOD_API_KEY}\" \"https://rest.runpod.io/v1/pods/${RUNPOD_POD_ID}/stop\" >/dev/null; then sleep infinity; fi; exit \"$status\""]
