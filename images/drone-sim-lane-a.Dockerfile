ARG BASE_IMAGE
FROM ${BASE_IMAGE}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

COPY upstream/tests/lane-a-smoke.sh /usr/local/bin/fern-lane-a-smoke
COPY images/patch-drone-sim-smoke.sh /tmp/patch-drone-sim-smoke.sh
COPY images/run-drone-sim-smoke.sh /usr/local/bin/fern-run-lane-a
RUN bash /tmp/patch-drone-sim-smoke.sh /usr/local/bin/fern-lane-a-smoke \
    && chmod +x /usr/local/bin/fern-run-lane-a \
    && rm -f /tmp/patch-drone-sim-smoke.sh

# Runpod does not expose Docker's --shm-size option. Keeping every process in one
# container and forcing Fast DDS onto loopback UDP avoids its shared-memory transport.
ENV DURATION=300 \
    FASTDDS_BUILTIN_TRANSPORTS=UDPv4 \
    OUTDIR=/workspace/fern/drone-sim

# Runpod restarts a container that exits while the Pod's desired state is RUNNING.
# The wrapper records the result, requests a stop, and never reruns the smoke test.
CMD ["/usr/local/bin/fern-run-lane-a"]
