# Install Fern

## Bun

Fern is distributed as a small Bun launcher plus a platform-specific native
Rust executable. Installation does not run a lifecycle script and does not
download executables from an arbitrary URL during installation.

Install globally:

```console
bun install --global @coolcmyk/fern
fern --help
```

Run a temporary cached copy:

```console
bunx @coolcmyk/fern --help
```

Supported release targets:

- Linux x64 with glibc
- Linux arm64 with glibc
- macOS x64
- macOS arm64
- Windows x64

The matching binary is selected using npm's `os`, `cpu`, and, on Linux,
`libc` package metadata. Optional dependencies must remain enabled.

## Configuration

Fern reads credentials from the process environment and from a `.env` file in
the current directory. Process variables take precedence.

```dotenv
RUNPOD_API_KEY=...
```

When Fern itself runs inside a Runpod Pod, use an account credential for
account-level orchestration:

```dotenv
RUNPOD_ACCOUNT_API_KEY=...
```

Fern does not include credential values in `config check` output.

## From source

```console
cargo install --path .
```

For package-development checks:

```console
bun install
bun run test
```

Set `FERN_BINARY_PATH` only when testing the Bun launcher against a locally
built Fern executable. It is not required for published packages.
