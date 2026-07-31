```
                                                               
eeee eeee eeeee  eeeee eeeee  eeeee eeeee  eeeee eeeee e  eeee 
8    8    8   8  8   8 8   8  8  88 8   8  8  88   8   8  8  8 
8eee 8eee 8eee8e 8e  8 8eee8e 8   8 8eee8e 8   8   8e  8e 8e   
88   88   88   8 88  8 88   8 8   8 88   8 8   8   88  88 88   
88   88ee 88   8 88  8 88   8 8eee8 88eee8 8eee8   88  88 88e8 
                                                               
```

skip the compute factor, run your robotics sim out in the cloud

## Install

Install the native Fern CLI through Bun:

```console
bun install --global @coolcmyk/fern
fern --help
```

Or run a temporary cached copy:

```console
bunx @coolcmyk/fern --help
```

See [the installation guide](docs/INSTALL.md) for supported platforms and
configuration. The package uses a platform-specific native Rust binary and
does not run a download script during installation.

## Status

Fern is in its initial scaffold phase. Its first target is Drone Sim Lane A on
Runpod Pods. See [the implementation plan](docs/PLAN.md) for the architecture,
constraints, milestones, and acceptance gates.

The current CLI provides read-only Runpod discovery:

```console
RUNPOD_API_KEY=... fern config check
RUNPOD_API_KEY=... fern pod list --compute cpu
RUNPOD_API_KEY=... fern pod get <pod-id>
```

`RUNPOD_ACCOUNT_API_KEY` takes precedence when both credential variables are
set. Fern never prints either credential.

## Develop

```console
bun install
bun run test
```

Rust-only checks remain available through Cargo:

```console
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```
