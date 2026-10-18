import { readFileSync } from "node:fs";
const tag = process.argv[2];
const pkg = JSON.parse(readFileSync("package.json", "utf8"));
const tauri = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const cargo = readFileSync("Cargo.toml", "utf8").split("[workspace.package]")[1]?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag || "") ||
    [pkg.version, tauri.version, cargo].some(version => `v${version}` !== tag)) {
  console.error("Release requires a v-prefixed version matching package.json, Cargo.toml and tauri.conf.json.");
  process.exit(1);
}
console.log(`Verified release ${tag}`);
