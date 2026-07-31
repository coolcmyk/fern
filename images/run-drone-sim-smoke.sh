#!/usr/bin/env bash
set -uo pipefail

mkdir -p "$OUTDIR"

/usr/local/bin/fern-lane-a-smoke 2>&1 | tee "$OUTDIR/smoke.log"
smoke_status=${PIPESTATUS[0]}
printf '%s\n' "$smoke_status" > "$OUTDIR/smoke.exit"

echo "### requesting Runpod Pod stop (smoke exit $smoke_status)"
stop_requested=0

if [[ -z "${RUNPOD_POD_ID:-}" ]]; then
  echo "WARN: Runpod did not inject RUNPOD_POD_ID"
else
  if command -v runpodctl >/dev/null 2>&1; then
    if timeout 30 runpodctl pod stop "$RUNPOD_POD_ID"; then
      stop_requested=1
      echo "### stop accepted through runpodctl"
    else
      echo "WARN: runpodctl did not accept the self-stop request; trying REST"
    fi
  else
    echo "WARN: runpodctl is unavailable; trying REST"
  fi

  if [[ "$stop_requested" = "0" && -n "${RUNPOD_API_KEY:-}" ]]; then
    if curl --fail --silent --show-error --retry 2 --connect-timeout 10 --max-time 30 \
      --request POST \
      --header "Authorization: Bearer ${RUNPOD_API_KEY}" \
      "https://rest.runpod.io/v1/pods/${RUNPOD_POD_ID}/stop" >/dev/null; then
      stop_requested=1
      echo "### stop accepted through the Runpod REST API"
    else
      echo "WARN: Runpod REST did not accept the self-stop request"
    fi
  elif [[ "$stop_requested" = "0" ]]; then
    echo "WARN: Runpod did not inject RUNPOD_API_KEY"
  fi
fi

if [[ "$stop_requested" = "0" ]]; then
  echo "WARN: automatic stop failed; idling to prevent a billable smoke-test restart loop"
else
  echo "### waiting for Runpod shutdown"
fi

exec sleep infinity
