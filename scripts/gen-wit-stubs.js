#!/usr/bin/env node
/* eslint-disable no-console */

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const args = process.argv.slice(2);

let roots = [];
let dryRun = false;
let verbose = false;
let defaultRecords = null;
let defaultWorld = null;
let interfacePrefixOverride = null;

function usage() {
  console.log(
    'Usage: node scripts/gen-wit-stubs.js [--root dir] [--dry-run] [--verbose] [--records struct|object] [--world name] [--no-interface-prefix]'
  );
}

for (let i = 0; i < args.length; i += 1) {
  const arg = args[i];
  if (arg === '--help' || arg === '-h') {
    usage();
    process.exit(0);
  }
  if (arg === '--root') {
    const value = args[i + 1];
    if (!value) {
      console.error('--root expects a directory');
      process.exit(1);
    }
    roots.push(value);
    i += 1;
    continue;
  }
  if (arg === '--dry-run') {
    dryRun = true;
    continue;
  }
  if (arg === '--verbose') {
    verbose = true;
    continue;
  }
  if (arg === '--records') {
    const value = args[i + 1];
    if (!value || (value !== 'struct' && value !== 'object')) {
      console.error('--records expects struct|object');
      process.exit(1);
    }
    defaultRecords = value;
    i += 1;
    continue;
  }
  if (arg === '--world') {
    const value = args[i + 1];
    if (!value) {
      console.error('--world expects a name');
      process.exit(1);
    }
    defaultWorld = value;
    i += 1;
    continue;
  }
  if (arg === '--no-interface-prefix') {
    interfacePrefixOverride = false;
    continue;
  }
  if (arg === '--interface-prefix') {
    interfacePrefixOverride = true;
    continue;
  }

  console.error(`Unknown argument: ${arg}`);
  usage();
  process.exit(1);
}

if (roots.length === 0) {
  roots = ['php_modules'];
}

const skipDirs = new Set(['.git', 'node_modules', 'target', 'dist']);

function walk(dir, out) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (skipDirs.has(entry.name)) {
        continue;
      }
      walk(fullPath, out);
      continue;
    }
    if (entry.isFile() && entry.name.endsWith('.wit')) {
      out.push(fullPath);
    }
  }
}

function readJson(filePath) {
  try {
    const raw = fs.readFileSync(filePath, 'utf8');
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

function resolveStubPath(dir, manifest) {
  if (!manifest) {
    return path.join(dir, 'module.d.phpx');
  }
  const stubSpec = manifest.stubs || manifest.stub;
  if (typeof stubSpec === 'string' && stubSpec.length > 0) {
    if (path.isAbsolute(stubSpec)) {
      return stubSpec;
    }
    return path.join(dir, stubSpec);
  }
  return path.join(dir, 'module.d.phpx');
}

function shouldRegenerate(outPath, inputs) {
  if (!fs.existsSync(outPath)) {
    return true;
  }
  const outStat = fs.statSync(outPath);
  for (const input of inputs) {
    if (!input || !fs.existsSync(input)) {
      continue;
    }
    const stat = fs.statSync(input);
    if (stat.mtimeMs > outStat.mtimeMs) {
      return true;
    }
  }
  return false;
}

let total = 0;
let generated = 0;

for (const root of roots) {
  const absRoot = path.resolve(root);
  if (!fs.existsSync(absRoot)) {
    console.warn(`Skipping missing root: ${absRoot}`);
    continue;
  }
  const witFiles = [];
  walk(absRoot, witFiles);

  for (const witPath of witFiles) {
    total += 1;
    const dir = path.dirname(witPath);
    const dekaJsonPath = path.join(dir, 'deka.json');
    const dekaJson = readJson(dekaJsonPath) || {};
    const stubPath = resolveStubPath(dir, dekaJson);

    const records = defaultRecords || dekaJson.records || 'struct';
    const world = defaultWorld || dekaJson.world || null;

    let interfacePrefix = true;
    if (typeof dekaJson.interfacePrefix === 'boolean') {
      interfacePrefix = dekaJson.interfacePrefix;
    }
    if (typeof dekaJson.interface_prefix === 'boolean') {
      interfacePrefix = dekaJson.interface_prefix;
    }
    if (interfacePrefixOverride !== null) {
      interfacePrefix = interfacePrefixOverride;
    }

    if (!shouldRegenerate(stubPath, [witPath, dekaJsonPath])) {
      if (verbose) {
        console.log(`Up to date: ${stubPath}`);
      }
      continue;
    }

    const cmd = ['cargo', 'run', '-p', 'wit-phpx', '--', witPath, '--out', stubPath, '--records', records];
    if (world) {
      cmd.push('--world', world);
    }
    if (!interfacePrefix) {
      cmd.push('--no-interface-prefix');
    }

    if (dryRun) {
      console.log(`[dry-run] ${cmd.join(' ')}`);
      continue;
    }

    if (verbose) {
      console.log(cmd.join(' '));
    }

    const result = spawnSync(cmd[0], cmd.slice(1), { stdio: 'inherit' });
    if (result.status !== 0) {
      process.exit(result.status || 1);
    }
    generated += 1;
  }
}

console.log(`WIT stubs: ${generated} generated, ${total} total`);
