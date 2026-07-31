ARG BASE_IMAGE
FROM ${BASE_IMAGE}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

COPY upstream/tests/lane-a-smoke.sh /usr/local/bin/fern-lane-a-smoke
COPY images/patch-drone-sim-smoke.sh /tmp/patch-drone-sim-smoke.sh
RUN bash /tmp/patch-drone-sim-smoke.sh /usr/local/bin/fern-lane-a-smoke \
    && rm -f /tmp/patch-drone-sim-smoke.sh

# Runpod does not expose Docker's --shm-size option. Keeping every process in one
# container and forcing Fast DDS onto loopback UDP avoids its shared-memory transport.
ENV DURATION=300 \
    FASTDDS_BUILTIN_TRANSPORTS=UDPv4 \
    OUTDIR=/workspace/fern/drone-sim

# Runpod restarts a container that exits while the Pod's desired state is RUNNING.
# Stop the Pod through its injected, Pod-scoped credential after recording the result.
CMD ["bash", "-lc", "mkdir -p \"$OUTDIR\"; set -o pipefail; /usr/local/bin/fern-lane-a-smoke 2>&1 | tee \"$OUTDIR/smoke.log\"; status=${PIPESTATUS[0]}; printf '%s\\n' \"$status\" > \"$OUTDIR/smoke.exit\"; echo \"### requesting Runpod Pod stop (smoke exit $status)\"; if [[ -z \"${RUNPOD_POD_ID:-}\" || -z \"${RUNPOD_API_KEY:-}\" ]]; then echo \"WARN: Runpod did not inject the Pod ID or Pod-scoped API key\"; exit \"$status\"; fi; if curl --fail --silent --show-error --retry 5 --request POST --header \"Authorization: Bearer ${RUNPOD_API_KEY}\" \"https://rest.runpod.io/v1/pods/${RUNPOD_POD_ID}/stop\" >/dev/null; then echo \"### stop accepted; waiting for Runpod shutdown\"; sleep infinity; fi; echo \"WARN: Runpod rejected the self-stop request\"; exit \"$status\""]
