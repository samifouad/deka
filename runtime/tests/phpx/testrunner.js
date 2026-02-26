#!/usr/bin/env bun
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

const repoRoot = path.resolve(process.cwd());
const envLockRoot = process.env.PHPX_MODULE_ROOT
  ? path.resolve(process.env.PHPX_MODULE_ROOT)
  : null;
function findLockRoot(startDir) {
  let current = startDir;
  while (true) {
    const candidate = path.join(current, "deka.lock");
    if (existsSync(candidate)) {
      return current;
    }
    const parent = path.dirname(current);
    if (parent === current) {
      break;
    }
    current = parent;
  }
  return null;
}
const lockRoot = envLockRoot ?? findLockRoot(repoRoot);
let suiteArg = "tests/phpx";
let skipArgs = [];

const rawArgs = process.argv.slice(2);
let suiteSpecified = false;
for (let i = 0; i < rawArgs.length; i += 1) {
  const arg = rawArgs[i];
  if (arg === "--skip") {
    if (i + 1 >= rawArgs.length) {
      throw new Error("--skip requires a value");
    }
    skipArgs.push(rawArgs[++i]);
  } else if (arg.startsWith("--skip=")) {
    skipArgs.push(arg.slice("--skip=".length));
  } else if (!suiteSpecified) {
    suiteArg = arg;
    suiteSpecified = true;
  } else {
    throw new Error(`Unexpected argument: ${arg}`);
  }
}

const suiteDir = path.resolve(repoRoot, suiteArg);
const phpBinaryCandidate = process.env.PHPX_BIN
  ? path.resolve(process.env.PHPX_BIN)
  : path.resolve(repoRoot, "target/release/cli");
const phpBinArgs = process.env.PHPX_BIN_ARGS
  ? process.env.PHPX_BIN_ARGS.split(" ").map((arg) => arg.trim()).filter(Boolean)
  : [];

const skipPatterns = [];
for (const raw of skipArgs.flatMap((arg) => arg.split(","))) {
  const trimmed = raw.trim();
  if (!trimmed) {
    continue;
  }
  skipPatterns.push(path.resolve(repoRoot, trimmed));
}
if (process.env.PHPX_DB_SMOKE !== "1") {
  skipPatterns.push(path.resolve(repoRoot, "tests/phpx/db"));
}

async function collectPhpxFiles(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  let files = [];
  for (const entry of entries) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name.startsWith("_")) {
        continue;
      }
      files = files.concat(await collectPhpxFiles(entryPath));
    } else if (entry.isFile() && entry.name.endsWith(".phpx")) {
      files.push(entryPath);
    }
  }
  return files.sort();
}

async function runBinary(binary, scriptPath) {
  const base = path.basename(binary).toLowerCase();
  const isCliRunner = base === "cli" || base === "deka";
  const runtimeArgs = isCliRunner
    ? ["run", ...phpBinArgs, scriptPath]
    : [...phpBinArgs, scriptPath];
  return new Promise((resolve, reject) => {
    const proc = spawn(binary, runtimeArgs, {
      env: { ...process.env },
      cwd: repoRoot,
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    proc.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });

    proc.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });

    proc.on("error", reject);
    proc.on("close", (code) => {
      resolve({ stdout, stderr, code: code ?? 0 });
    });
  });
}

function sanitizeStream(text) {
  const sanitized = text
    .split(/\r?\n/)
    .filter((line) => !line.startsWith("[PthreadsExtension]"))
    .filter((line) => !line.startsWith("    at ") && !line.startsWith("\tat "))
    .join("\n")
    .replace(/\n{3,}/g, "\n\n");
  const trimmedStart = sanitized.replace(/^\s*\n/, "");
  const trimmed = trimmedStart.trimEnd().concat(text.endsWith("\n") ? "\n" : "");
  const normalizedRepo = repoRoot.replace(/\\/g, "/");
  const normalizedLock = (lockRoot ?? repoRoot).replace(/\\/g, "/");
  return trimmed
    .replaceAll(normalizedRepo, "<repo>")
    .replaceAll(normalizedLock, "<repo>");
}

function indent(text) {
  if (text === "") {
    return "    (empty)";
  }
  return text
    .split(/\r?\n/)
    .map((line) => `    ${line}`)
    .join("\n");
}

function validateTestHeader(source, filePath) {
  const lines = source.split(/\r?\n/);
  let idx = 0;
  while (idx < lines.length && lines[idx].trim() === "") {
    idx += 1;
  }
  if (idx >= lines.length) {
    return `Missing header comment in ${filePath}`;
  }
  const firstLine = lines[idx].trim();
  if (firstLine === "---") {
    const head = lines.slice(idx + 1, Math.min(lines.length, idx + 20)).join("\n");
    if (!head.includes("TEST:")) {
      return `Header missing TEST: marker in ${filePath}`;
    }
    return null;
  }
  if (!firstLine.startsWith("/*") && !firstLine.startsWith("//")) {
    return `Missing header comment in ${filePath}`;
  }
  const head = lines.slice(idx, Math.min(lines.length, idx + 12)).join("\n");
  if (!head.includes("TEST:")) {
    return `Header missing TEST: marker in ${filePath}`;
  }
  return null;
}

const COLOR = {
  reset: "\x1b[0m",
  green: "\x1b[32m",
  red: "\x1b[31m",
  yellow: "\x1b[33m",
};

function shouldSkip(filePath) {
  if (skipPatterns.length === 0) {
    return false;
  }
  const resolved = path.resolve(filePath);
  return skipPatterns.some((pattern) => resolved === pattern || resolved.startsWith(pattern + path.sep));
}

async function resolvePhpBinary() {
  if (await exists(phpBinaryCandidate)) {
    return phpBinaryCandidate;
  }
  const releasePhpFallback = path.resolve(repoRoot, "target/release/php");
  if (await exists(releasePhpFallback)) {
    return releasePhpFallback;
  }
  throw new Error(
    "phpx runner could not find a Deka runtime binary; build with 'cargo build -p cli --release' or set PHPX_BIN"
  );
}

async function exists(p) {
  try {
    await stat(p);
    return true;
  } catch {
    return false;
  }
}

async function loadExpectation(pathBase, extension) {
  const filePath = `${pathBase}.${extension}`;
  if (!(await exists(filePath))) {
    return null;
  }
  return readFile(filePath, "utf8");
}

function normalizeExpected(text) {
  return sanitizeStream(text ?? "");
}

function matchExpected(actual, expectedRaw) {
  const expected = normalizeExpected(expectedRaw);
  if (expected.startsWith("re:")) {
    const pattern = expected.slice(3).trim();
    const re = new RegExp(pattern, "s");
    return re.test(actual);
  }
  return actual === expected;
}

async function main() {
  console.log(`Running PHPX suite in ${suiteDir}`);
  const existsSuite = await exists(suiteDir);
  if (!existsSuite) {
    console.error(`${COLOR.red}Suite directory not found: ${suiteDir}${COLOR.reset}`);
    process.exit(1);
  }

  const suiteStat = await stat(suiteDir);
  let files = [];
  if (suiteStat.isFile()) {
    if (!suiteDir.endsWith(".phpx")) {
      console.error(`${COLOR.red}Suite file must be a .phpx file: ${suiteDir}${COLOR.reset}`);
      process.exit(1);
    }
    files = [suiteDir];
  } else {
    files = await collectPhpxFiles(suiteDir);
  }

  files = files.filter((filePath) => !shouldSkip(filePath));
  if (files.length === 0) {
    console.log("No PHPX files found.");
    return;
  }

  const phpBinaryPath = await resolvePhpBinary();
  let failures = 0;

  for (const scriptPath of files) {
    const relative = path.relative(repoRoot, scriptPath);
    process.stdout.write(`${relative} ... `);

    const source = await readFile(scriptPath, "utf8");
    const headerError = validateTestHeader(source, relative);
    if (headerError) {
      failures += 1;
      console.log(`${COLOR.red}FAILED${COLOR.reset}`);
      console.log(`  ${headerError}`);
      continue;
    }

    const res = await runBinary(phpBinaryPath, scriptPath);
    const stdout = sanitizeStream(res.stdout);
    const stderr = sanitizeStream(res.stderr);

    const base = scriptPath.replace(/\.phpx$/, "");
    const expectedOut = await loadExpectation(base, "out");
    const expectedErr = await loadExpectation(base, "err");
    const expectedCodeRaw = await loadExpectation(base, "code");
    const expectedCode = expectedCodeRaw === null ? null : Number(expectedCodeRaw.trim());

    let ok = true;
    let reason = "";

    if (expectedOut !== null && !matchExpected(stdout, expectedOut)) {
      ok = false;
      reason = "stdout mismatch";
    }
    if (expectedErr !== null && !matchExpected(stderr, expectedErr)) {
      ok = false;
      reason = reason || "stderr mismatch";
    }
    if (expectedCode !== null && res.code !== expectedCode) {
      ok = false;
      reason = reason || "exit code mismatch";
    }
    if (expectedOut === null && expectedErr === null && expectedCode === null && res.code !== 0) {
      ok = false;
      reason = "non-zero exit";
    }

    if (ok) {
      console.log(`${COLOR.green}ok${COLOR.reset}`);
      continue;
    }

    failures += 1;
    console.log(`${COLOR.red}FAILED${COLOR.reset}`);
    if (reason) {
      console.log(`  ${reason}`);
    }
    if (expectedOut !== null) {
      console.log("  stdout:");
      console.log("    expected:");
      console.log(indent(normalizeExpected(expectedOut)));
      console.log("    actual:");
      console.log(indent(stdout));
    }
    if (expectedErr !== null) {
      console.log("  stderr:");
      console.log("    expected:");
      console.log(indent(normalizeExpected(expectedErr)));
      console.log("    actual:");
      console.log(indent(stderr));
    }
    if (expectedCode !== null && res.code !== expectedCode) {
      console.log(`  exit code: expected=${expectedCode} actual=${res.code}`);
    }
  }

  const matched = files.length - failures;
  const total = files.length;
  const percent = total === 0 ? 0 : Math.round((matched / total) * 100);
  const summaryColor = failures > 0 ? COLOR.red : COLOR.green;
  console.log(`\n${summaryColor}Summary: ${matched}/${total} passed. ${percent}% overall.${COLOR.reset}`);
  if (failures > 0) {
    process.exit(1);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
