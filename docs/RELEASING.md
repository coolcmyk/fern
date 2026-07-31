# Release Fern

Fern publishes one launcher package and five platform packages to npm. The
platform packages are published first so the launcher's exact optional
dependencies are available when users install it.

## One-time setup

1. Create or obtain the npm scope `@coolcmyk`.
2. Create a granular npm token allowed to publish all `@coolcmyk/fern*`
   packages.
3. Add it to the GitHub repository as the `NPM_TOKEN` Actions secret.
4. Create the `npm` GitHub environment if release approvals are desired.

## Release

Keep the Cargo and npm versions aligned. The workflow verifies this before it
publishes anything.

```console
# Update Cargo.toml and Cargo.lock first.
bun scripts/set-package-version.ts 0.2.0
bun run test
git tag v0.2.0
git push origin v0.2.0
```

The tag workflow builds these native targets on matching GitHub-hosted
runners:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

It then verifies that every package contains its binary, publishes all native
packages, and publishes `@coolcmyk/fern` last.

## Local package verification

```console
cargo build --release --locked
cp target/release/fern npm/fern-linux-x64/bin/fern
chmod +x npm/fern-linux-x64/bin/fern
bun pm pack --cwd npm/fern-linux-x64 --destination /tmp
bun pm pack --cwd npm/fern --destination /tmp
```

Never commit staged binaries. Release binaries are produced from the tagged
commit by GitHub Actions.
