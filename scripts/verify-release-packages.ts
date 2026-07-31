import { existsSync } from "node:fs";
import { join, resolve } from "node:path";

const version = Bun.argv[2]?.replace(/^v/, "");
if (!version) throw new Error("release version is required");

const root = resolve(import.meta.dir, "..");
const packages = [
  ["fern-linux-x64", "fern"],
  ["fern-linux-arm64", "fern"],
  ["fern-darwin-x64", "fern"],
  ["fern-darwin-arm64", "fern"],
  ["fern-win32-x64", "fern.exe"],
];

for (const [directory, executable] of packages) {
  const packageRoot = join(root, "npm", directory);
  const manifest = await Bun.file(join(packageRoot, "package.json")).json();
  if (manifest.version !== version) {
    throw new Error(`${manifest.name} has version ${manifest.version}, expected ${version}`);
  }

  const binary = join(packageRoot, "bin", executable);
  if (!existsSync(binary)) {
    throw new Error(`${manifest.name} is missing ${binary}`);
  }
}

const launcher = await Bun.file(join(root, "npm", "fern", "package.json")).json();
if (launcher.version !== version) {
  throw new Error(`launcher has version ${launcher.version}, expected ${version}`);
}
for (const [dependency, dependencyVersion] of Object.entries(
  launcher.optionalDependencies,
)) {
  if (dependencyVersion !== version) {
    throw new Error(`${dependency} points to ${dependencyVersion}, expected ${version}`);
  }
}

console.log(`release packages are complete for ${version}`);
