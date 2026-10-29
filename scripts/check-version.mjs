import { readFileSync } from 'node:fs';
const read = (path) =>
  readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const version = JSON.parse(read('package.json')).version;
const tauri = JSON.parse(read('src-tauri/tauri.conf.json')).version;
const cargo = read('src-tauri/Cargo.toml').match(
  /^version\s*=\s*"([^"]+)"/m,
)?.[1];
if (version !== tauri || version !== cargo)
  throw new Error(
    `Version mismatch: package=${version}, Tauri=${tauri}, Cargo=${cargo}`,
  );
const tag = process.argv[2];
if (tag && (!/^v\d+\.\d+\.\d+$/.test(tag) || tag !== `v${version}`))
  throw new Error(`Release tag ${tag} must equal v${version}`);
console.log(
  `RVNFlow ${version}: application versions agree${tag ? ` with ${tag}` : ''}.`,
);
