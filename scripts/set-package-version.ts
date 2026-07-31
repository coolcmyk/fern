import { readdir } from "node:fs/promises";
import { join, resolve } from "node:path";

const version = Bun.argv[2]?.replace(/^v/, "");
if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error("usage: bun scripts/set-package-version.ts <semver>");
}

const root = resolve(import.meta.dir, "..");
const cargoManifest = await Bun.file(join(root, "Cargo.toml")).text();
const cargoVersion = cargoManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (cargoVersion !== version) {
  throw new Error(
    `release version ${version} does not match Cargo.toml version ${cargoVersion}`,
  );
}

const packageRoot = join(root, "npm");
const directories = await readdir(packageRoot, { withFileTypes: true });

for (const directory of directories) {
  if (!directory.isDirectory()) continue;

  const manifestPath = join(packageRoot, directory.name, "package.json");
  const manifest = await Bun.file(manifestPath).json();
  manifest.version = version;

  if (manifest.optionalDependencies) {
    for (const dependency of Object.keys(manifest.optionalDependencies)) {
      if (dependency.startsWith("@coolcmyk/fern-")) {
        manifest.optionalDependencies[dependency] = version;
      }
    }
  }

  await Bun.write(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`${manifest.name} -> ${version}`);
}
