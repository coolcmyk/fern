#!/usr/bin/env bun

import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const require = createRequire(import.meta.url);

const targets = {
  "linux-x64": ["@coolcmyk/fern-linux-x64", "fern"],
  "linux-arm64": ["@coolcmyk/fern-linux-arm64", "fern"],
  "darwin-x64": ["@coolcmyk/fern-darwin-x64", "fern"],
  "darwin-arm64": ["@coolcmyk/fern-darwin-arm64", "fern"],
  "win32-x64": ["@coolcmyk/fern-win32-x64", "fern.exe"],
};

function resolveBinary() {
  const override = process.env.FERN_BINARY_PATH;
  if (override) {
    const path = resolve(override);
    if (!existsSync(path)) {
      fail(`FERN_BINARY_PATH does not exist: ${path}`);
    }
    return path;
  }

  const target = `${process.platform}-${process.arch}`;
  const selection = targets[target];
  if (!selection) {
    fail(
      `Fern does not publish a binary for ${target}. ` +
        "Supported targets: linux-x64, linux-arm64, darwin-x64, " +
        "darwin-arm64, win32-x64.",
    );
  }

  const [packageName, executable] = selection;
  try {
    const manifest = require.resolve(`${packageName}/package.json`);
    const binary = join(dirname(manifest), "bin", executable);
    if (existsSync(binary)) {
      return binary;
    }
  } catch {
    // The actionable error below also covers optional dependencies omitted by
    // the package manager and incomplete local workspace installs.
  }

  const launcherDirectory = dirname(fileURLToPath(import.meta.url));
  const developmentBinary = resolve(
    launcherDirectory,
    "../../../target/release",
    process.platform === "win32" ? "fern.exe" : "fern",
  );
  if (existsSync(developmentBinary)) {
    return developmentBinary;
  }

  fail(
    `The platform package ${packageName} is missing. Reinstall with optional ` +
      "dependencies enabled:\n\n  bun install --global @coolcmyk/fern",
  );
}

function fail(message) {
  console.error(`fern: ${message}`);
  process.exit(1);
}

const child = spawnSync(resolveBinary(), process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
  shell: false,
});

if (child.error) {
  fail(child.error.message);
}

if (child.signal) {
  console.error(`fern: native process terminated by ${child.signal}`);
  process.exit(1);
}

process.exit(child.status ?? 1);
