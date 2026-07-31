# Fern implementation plan

Status: initial scaffold  
Last updated: 2026-07-31

## Objective

Fern is a Rust-native control plane for running robotics simulations on rented
cloud compute. The first supported workload is
[Drone Sim](https://github.com/teapotlaboratories/drone-sim), and the first
provider is [Runpod Pods](https://docs.runpod.io/pods/overview).

Fern orchestrates the simulation; it does not rewrite PX4, ROS 2, Gazebo,
Unreal, or their plugins in Rust.

The first useful end-to-end result is:

```console
fern deploy --profile drone-sim-lane-a
fern wait --healthy
fern run smoke
fern download results/
fern destroy
```

## Scope

### Initial scope

- Runpod Pod discovery and lifecycle management.
- A reproducible, single-container Drone Sim Lane A image.
- CPU and GPU deployment specifications, with Lane A using CPU first.
- Persistent artifacts under `/workspace`.
- SSH and authenticated HTTP access.
- Health, readiness, simulation jobs, logs, and result discovery.
- Explicit cost and destructive-action guardrails.

### Non-goals for the first release

- Reimplementing simulation components in Rust.
- Running Docker Compose inside a Runpod Pod.
- Exposing MAVLink, DDS, or Gazebo UDP ports to the public internet.
- Claiming support for Drone Sim lanes that are not yet working upstream.
- Building a general-purpose Terraform replacement.
- Runpod Serverless support; simulations are stateful, multi-process jobs and
  fit Pods better.

## Evidence and reference implementations

Fern follows the current Runpod REST API rather than treating a third-party
SDK as the source of truth:

- [Create Pod REST API](https://docs.runpod.io/api-reference/pods/POST/pods)
- [List Pods REST API](https://docs.runpod.io/api-reference/pods/GET/pods)
- [Runpod networking constraints](https://docs.runpod.io/pods/configuration/expose-ports)
- [Runpod storage model](https://docs.runpod.io/pods/storage/types)

[Kioku](https://github.com/kioku-org/kioku) is the operational reference. Its
Runpod backend demonstrates several patterns Fern should retain:

- Call `https://rest.runpod.io/v1` with bearer authentication.
- Prefer `RUNPOD_ACCOUNT_API_KEY` over the pod-scoped `RUNPOD_API_KEY` when an
  orchestrator itself runs inside a Pod.
- Model CPU and GPU create payloads separately.
- Try an ordered list of GPU types when capacity is unavailable.
- Keep provider state and reconcile it with the provider API.
- Poll for exited Pods and remove orphaned resources.
- Use Runpod's HTTP proxy for HTTP services and direct TCP mapping for SSH.
- Validate published images on real Runpod infrastructure in CI.

The relevant Kioku implementation is in
`services/runtime-api/runtime_api/backends/runpod.py` and its deployment entry
point is `deployment/docker/scripts/runpod/deploy.sh`.

## Platform constraints

Runpod runs the workload container itself and does not support Docker Compose
inside Pods. It also does not expose UDP. Drone Sim currently uses multiple
processes, shared IPC, and local UDP for PX4, MAVLink, XRCE-DDS, and Gazebo.

The resulting invariant is:

> All simulation processes and all UDP transports live in one Pod and one
> container network namespace. Only authenticated HTTP and selected TCP ports
> cross the Pod boundary.

Drone Sim Lane A already has a single-container smoke-test path. Fern should
turn that path into a durable runtime image rather than translate the Compose
file into nested containers.

Fast-DDS requires at least 2 GiB of `/dev/shm`. The runtime must fail its
readiness check with a useful error when that requirement is not met.

## Architecture

```text
┌──────────────────────── developer machine ────────────────────────┐
│ fern CLI                                                          │
│  ├── manifest + profile resolution                                │
│  ├── provider API                                                 │
│  ├── local deployment index                                       │
│  └── SSH / HTTPS client                                           │
└───────────────────┬────────────────────────────────────────────────┘
                    │ Runpod REST API
                    ▼
┌────────────────────────── Runpod Pod ──────────────────────────────┐
│ custom Drone Sim OCI image                                        │
│                                                                   │
│ fern-agent (PID 1 or supervised service)                          │
│  ├── preflight: GPU, disk, /dev/shm, writable workspace           │
│  ├── PX4 SITL                                                     │
│  ├── Gazebo / later Unreal                                        │
│  ├── Micro-XRCE-DDS                                               │
│  ├── ROS 2                                                        │
│  ├── job and health API :8080/http                                │
│  └── optional noVNC :6080/http                                    │
│                                                                   │
│ local-only UDP: 14540, 14550, 8888                                │
│ persistent data: /workspace/runs/<run-id>/                        │
└───────────────────────────────────────────────────────────────────┘
```

### Rust layout

The repository starts as one package and should split into a workspace only
when the Pod-side agent is introduced:

```text
src/
  main.rs                 CLI entry point
  lib.rs                  reusable library surface
  config.rs               credentials and endpoint configuration
  error.rs                stable user-facing errors
  provider/
    mod.rs                provider-neutral domain types and trait
    runpod.rs             typed Runpod REST client
docs/
  PLAN.md
```

Expected later layout:

```text
crates/
  fern-cli/
  fern-core/
  fern-provider-runpod/
  fern-agent/
```

Do not split early merely for aesthetics. The provider boundary matters; the
number of Cargo packages does not.

## Configuration

Fern will resolve configuration in this order:

1. Command-line arguments.
2. Environment variables.
3. Project `fern.toml`.
4. User configuration.
5. Built-in safe defaults.

Credentials are never accepted in `fern.toml` and are never written to Fern's
deployment state. For account-level operations, the precedence is:

1. `RUNPOD_ACCOUNT_API_KEY`
2. `RUNPOD_API_KEY`

The separate account key avoids accidentally using the restricted key Runpod
injects into a running Pod.

Proposed project manifest:

```toml
[project]
name = "drone-sim"

[profiles.drone-sim-lane-a]
provider = "runpod"
image = "ghcr.io/teapotlaboratories/drone-sim:lane-a-v1.16.0"
compute = "cpu"
container_disk_gb = 40
volume_gb = 20
volume_mount_path = "/workspace"
ports = ["22/tcp", "8080/http"]

[profiles.drone-sim-lane-a.env]
FERN_PROFILE = "lane-a"
FERN_WORKSPACE = "/workspace"
```

Image tags in durable deployment manifests must be immutable release tags or
digests. `latest` is allowed only for local development.

## Provider model

Provider-neutral operations:

- `list(filter)`
- `get(id)`
- `deploy(spec)`
- `start(id)`
- `stop(id)`
- `destroy(id)`
- `wait(id, condition, timeout)`

Runpod-specific behavior belongs in the Runpod adapter:

- REST field names and status mapping.
- CPU flavor and GPU type selection.
- Ordered GPU-capacity fallback.
- Public IP and port-mapping discovery.
- Runpod HTTP proxy URL construction.
- Provider error parsing and retry classification.

Fern uses the provider's Pod ID as the durable identity. Names are for humans
and discovery, not uniqueness.

## State and reconciliation

Local state is a convenience index, not the authority. It records:

- Fern deployment name.
- Provider and provider resource ID.
- Image digest or immutable tag.
- Creation time and last observed state.
- Persistent volume ID, when used.
- Non-secret manifest fingerprint.

Every status-changing command reads the provider before acting. Missing local
state can be reconstructed by importing an explicit Pod ID. Reconciliation
must never delete a running Pod solely because local state is missing.

Automated cleanup is limited to resources carrying a Fern ownership marker or
an unambiguous Fern name prefix, and only after their provider state is exited
or terminated. Active resources are never reaped by name matching alone.

## Drone Sim image contract

The image used by Fern must:

- Be a single Linux OCI image supported by Runpod.
- Pin upstream repositories and verify their SHAs during the build.
- Use an init/supervision strategy that forwards signals and reaps children.
- Keep DDS, MAVLink, PX4, and Gazebo UDP listeners local to the Pod.
- Provide `GET /healthz` and `GET /readyz` on port 8080.
- Provide asynchronous job submission and status APIs; Runpod's HTTP proxy has
  a request-duration limit, so a simulation must not occupy one HTTP request.
- Write every job to `/workspace/runs/<run-id>/` using an atomic status file.
- Exit non-zero when the primary workload fails.
- Report the detected GPU, driver, disk, and `/dev/shm` capacity at startup.

Suggested run directory:

```text
/workspace/runs/<run-id>/
  request.json
  status.json
  logs/
  artifacts/
  metrics.json
```

## Security and cost controls

- Never log authorization headers or API keys.
- Mark authorization header values as sensitive in the HTTP client.
- Expose no unauthenticated control API.
- Bind simulation UDP ports to loopback or the container-local interface.
- Require confirmation before `destroy`, unless `--yes` is supplied.
- Make `deploy` print the requested compute, estimated hourly price when
  available, disk allocation, and interruption mode before creation.
- Support `--dry-run` for all resource-creating commands.
- Apply bounded timeouts to every provider request and wait loop.
- On Ctrl-C during creation, report the created Pod ID before exiting so it
  cannot become an invisible billable resource.

## Delivery phases and acceptance gates

### Phase 0 — foundation

- [x] Record architecture, constraints, and decisions.
- [x] Create a Rust package and provider boundary.
- [x] Add credential resolution without persisting secrets.
- [x] Add typed, read-only Runpod `list` and `get` operations.
- [x] Validate the read-only client against a real account.

Acceptance gate:

```console
cargo test
RUNPOD_API_KEY=... cargo run -- pod list
```

The command returns JSON by default and never prints the credential.

### Phase 1 — safe Pod lifecycle

- Add manifest parsing and validation.
- [x] Add create request generation and `--dry-run` output.
- [x] Add guarded `deploy` with explicit billable-operation confirmation.
- Add `start`, `stop`, and guarded `destroy`.
- Record a local deployment index using atomic writes.
- Add wait loops with deadlines and actionable provider errors.
- Add ordered GPU type fallback for capacity errors.

Acceptance gate: create, stop, start, and destroy a minimal public test image;
prove that interruption after create always reports and records the Pod ID.

### Phase 2 — Lane A runtime

- Produce the single-container Lane A image.
- Add a proper process supervisor and signal handling.
- Add the `/dev/shm >= 2 GiB` preflight.
- Add `fern-agent` health and asynchronous job endpoints.
- Persist smoke-test output to `/workspace`.
- Add `fern run smoke`, `fern logs`, and `fern download`.

Acceptance gate: a fresh CPU Pod passes Drone Sim's five-minute acceptance
test, reports real-time factor, and retains artifacts after a stop/start cycle.

### Phase 3 — CI and operations

- Build images in GitHub Actions and publish immutable SHA tags.
- Validate each candidate image on a real Runpod Pod.
- Always execute cleanup in CI, including after test failure.
- Add orphan detection that only removes exited Fern-managed Pods.
- Add structured diagnostics and support bundles.

Acceptance gate: one workflow builds, deploys, validates, collects artifacts,
and destroys the Pod without manual steps.

### Phase 4 — interactive visualization

- Add Xvfb and an optional noVNC service.
- Run QGroundControl inside the Pod when requested.
- Protect browser access with authentication.
- Evaluate an authenticated HTTP/WebSocket MAVLink bridge.

Acceptance gate: a user can observe Lane A in a browser without exposing UDP.

### Phase 5 — GPU lanes

- Add GPU capability and Vulkan preflight checks.
- Benchmark suitable Runpod GPU types.
- Integrate Lane C only after its upstream container is reproducible.
- Revisit Lane B only when Drone Sim resumes it.

Acceptance gate: a pinned GPU image passes an upstream-defined visual
simulation smoke test on at least two replaceable GPU types.

## Test strategy

### Unit tests

- Configuration precedence and missing credentials.
- Request serialization and response aliases.
- Provider status normalization.
- Capacity-error classification and GPU fallback order.
- Proxy URL and TCP port mapping construction.
- Secret redaction.

### Contract tests

Use a local mock HTTP server. Unit and client tests must not require real
credentials, Docker, or Runpod. Cover success, malformed JSON, timeouts, 401,
403, 404, 409, capacity exhaustion, and 5xx responses.

### Live integration tests

Live tests are opt-in and use a dedicated Runpod account/project. Every test
registers cleanup before creating a Pod. The workflow has an unconditional
final cleanup step and a maximum cost/runtime budget.

### Drone Sim acceptance tests

Retain Drone Sim's upstream assertions rather than replacing them with a Fern-
specific weaker test: populated ROS topics, no sensor timeouts, moving
telemetry, and the required aggregate real-time factor.

## Initial decisions

| Decision | Choice | Reason |
| --- | --- | --- |
| Runpod API | REST v1 | Current typed Pod lifecycle API and Kioku precedent |
| Workload product | Pods | Long-running, stateful, multi-process simulations |
| Runtime packaging | One OCI container | Runpod does not support Compose in Pods |
| External control | HTTPS/TCP | Runpod does not expose UDP |
| First workload | Drone Sim Lane A | It is currently working and CPU/headless |
| State authority | Runpod API | Local state can be lost or stale |
| Initial output | JSON | Stable for scripts and coding agents |
| Credentials | Environment only | Avoid secrets in repository and state files |

## Open questions

- Which Runpod CPU flavor sustains Lane A's required real-time factor at the
  lowest cost?
- Is Runpod's `/dev/shm` allocation consistently large enough across CPU and
  GPU Pod types, or does the image need a DDS transport fallback?
- Should persistent results use a Pod volume first or require a portable
  network volume from the start?
- Which authenticated browser protocol gives acceptable interactive latency
  for Gazebo and later Unreal?
- Does Lane C require host capabilities unavailable to Runpod custom
  containers?
- Should Fern build images locally, or only consume images produced by each
  simulation repository's CI?

These questions are resolved through measured acceptance tests, not assumptions
in the CLI.
