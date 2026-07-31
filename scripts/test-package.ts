import { existsSync } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const executable = process.platform === "win32" ? "fern.exe" : "fern";
const binary = join(root, "target", "debug", executable);
const launcher = join(root, "npm", "fern", "bin", "fern.js");

if (!existsSync(binary)) {
  const build = Bun.spawnSync(["cargo", "build"], {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  });
  if (build.exitCode !== 0) {
    throw new Error(`cargo build failed with exit code ${build.exitCode}`);
  }
}

const baseEnvironment = {
  ...process.env,
  FERN_BINARY_PATH: binary,
};

const version = Bun.spawnSync(["bun", launcher, "--version"], {
  cwd: root,
  env: baseEnvironment,
  stdout: "pipe",
  stderr: "pipe",
});

assertSuccess(version, "launcher version check");
const versionOutput = version.stdout.toString().trim();
if (!/^fern \d+\.\d+\.\d+/.test(versionOutput)) {
  throw new Error(`unexpected version output: ${JSON.stringify(versionOutput)}`);
}

const config = Bun.spawnSync(["bun", launcher, "config", "check"], {
  cwd: root,
  env: {
    ...baseEnvironment,
    RUNPOD_ACCOUNT_API_KEY: "package-test-account-key",
    RUNPOD_API_KEY: "package-test-fallback-key",
  },
  stdout: "pipe",
  stderr: "pipe",
});

assertSuccess(config, "launcher environment forwarding check");
const summary = JSON.parse(config.stdout.toString());
if (summary.credential_source !== "RUNPOD_ACCOUNT_API_KEY") {
  throw new Error(`unexpected credential source: ${summary.credential_source}`);
}

console.log(`Bun launcher OK (${versionOutput})`);

function assertSuccess(result: Bun.SyncSubprocess, label: string) {
  if (result.exitCode !== 0) {
    throw new Error(
      `${label} failed (${result.exitCode}): ${result.stderr.toString().trim()}`,
    );
  }
}
