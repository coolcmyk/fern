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

```bash
bun install --global @coolcmyk/fern
fern --help
```

Or run a temporary cached copy:

```bash
bunx @coolcmyk/fern --help
```

See [the installation guide](docs/INSTALL.md) for supported platforms and
configuration. The package uses a platform-specific native Rust binary and
does not run a download script during installation.

## Status

Fern is in its initial scaffold phase. Its first target is Drone Sim Lane A on
Runpod Pods. See [the implementation plan](docs/PLAN.md) for the architecture,
constraints, milestones, and acceptance gates.

## Configure Runpod

Set the API key once in your shell startup file so every Fern invocation can
use it. Add this line to `~/.bashrc` when using Bash or `~/.zshrc` when using
Zsh:

```bash
export RUNPOD_API_KEY="your-runpod-api-key"
```

Reload the matching shell configuration:

```bash
# Bash
source ~/.bashrc

# Zsh
source ~/.zshrc
```

Keep that file private and never commit it to a repository. Fern also supports
`RUNPOD_ACCOUNT_API_KEY`, which takes precedence when both variables are set.

The current CLI provides read-only Runpod discovery:

```bash
fern config check
fern pod list --compute cpu
fern pod get <pod-id>
```

Fern never prints either credential.

## Develop

```bash
bun install
bun run test
```

Rust-only checks remain available through Cargo:

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```
