ARG BASE_IMAGE
FROM ${BASE_IMAGE}

COPY tests/lane-a-smoke.sh /usr/local/bin/fern-lane-a-smoke
RUN chmod +x /usr/local/bin/fern-lane-a-smoke

# Runpod does not expose Docker's --shm-size option. Keeping every process in one
# container and forcing Fast DDS onto loopback UDP avoids its shared-memory transport.
ENV DURATION=300 \
    FASTDDS_BUILTIN_TRANSPORTS=UDPv4 \
    OUTDIR=/workspace/fern/drone-sim

CMD ["bash", "-lc", "mkdir -p \"$OUTDIR\"; set -o pipefail; /usr/local/bin/fern-lane-a-smoke 2>&1 | tee \"$OUTDIR/smoke.log\"; status=${PIPESTATUS[0]}; printf '%s\\n' \"$status\" > \"$OUTDIR/smoke.exit\"; exit \"$status\""]
