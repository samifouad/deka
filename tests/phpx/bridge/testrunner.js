#!/usr/bin/env bun
import { spawn } from "node:child_process";
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

const repoRoot = path.resolve(process.cwd());
let suiteArg = "tests/phpx/bridge";
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
  : ["run"];

const skipPatterns = [];
for (const raw of skipArgs.flatMap((arg) => arg.split(","))) {
  const trimmed = raw.trim();
  if (!trimmed) {
    continue;
  }
  skipPatterns.push(path.resolve(repoRoot, trimmed));
}

async function collectPhpFiles(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  let files = [];
  for (const entry of entries) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name.startsWith("_")) {
        continue;
      }
      files = files.concat(await collectPhpFiles(entryPath));
    } else if (entry.isFile() && entry.name.endsWith(".php")) {
      files.push(entryPath);
    }
  }
  return files.sort();
}

async function runBinary(binary, scriptPath) {
    return new Promise((resolve, reject) => {
    const stdlibOverride = process.env.DEKA_PHPX_STDLIB;
    const skipDekaOverride = process.env.DEKA_PHPX_SKIP_DEKA;
    const env = {
      ...process.env,
      ...(stdlibOverride ? {} : { DEKA_PHPX_STDLIB: "core/option,array/array_key_exists" }),
      ...(skipDekaOverride ? {} : { DEKA_PHPX_SKIP_DEKA: "1" }),
    };
    const proc = spawn(binary, [...phpBinArgs, scriptPath], {
      env,
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
  const normalizedRoot = repoRoot.replace(/\\/g, "/");
  return trimmed.replaceAll(normalizedRoot, "<repo>");
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
  const fallback = path.resolve(repoRoot, "target/debug/cli");
  if (await exists(fallback)) {
    return fallback;
  }
  throw new Error(
    "bridge runner could not find the Deka CLI; build with 'cargo build --release -p cli' or set PHPX_BIN"
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

async function main() {
  console.log(`Running PHPX bridge suite in ${suiteDir}`);
  const existsSuite = await exists(suiteDir);
  if (!existsSuite) {
    console.error(`${COLOR.red}Suite directory not found: ${suiteDir}${COLOR.reset}`);
    process.exit(1);
  }

  const files = (await collectPhpFiles(suiteDir)).filter((filePath) => !shouldSkip(filePath));
  if (files.length === 0) {
    console.log("No PHP files found.");
    return;
  }

  const phpBinaryPath = await resolvePhpBinary();
  let failures = 0;

  for (const scriptPath of files) {
    const relative = path.relative(repoRoot, scriptPath);
    process.stdout.write(`${relative} ... `);

    const res = await runBinary(phpBinaryPath, scriptPath);
    const stdout = sanitizeStream(res.stdout);
    const stderr = sanitizeStream(res.stderr);

    const base = scriptPath.replace(/\.php$/, "");
    const expectedOut = await loadExpectation(base, "out");
    const expectedErr = await loadExpectation(base, "err");
    const expectedCodeRaw = await loadExpectation(base, "code");
    const expectedCode = expectedCodeRaw === null ? null : Number(expectedCodeRaw.trim());

    let ok = true;
    let reason = "";

    if (expectedOut !== null && stdout !== normalizeExpected(expectedOut)) {
      ok = false;
      reason = "stdout mismatch";
    }
    if (expectedErr !== null && stderr !== normalizeExpected(expectedErr)) {
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
      console.log(`${COLOR.yellow}expected stdout:${COLOR.reset}`);
      console.log(expectedOut === "" ? "    (empty)" : expectedOut.replace(/^/gm, "    "));
    }
    console.log(`${COLOR.yellow}actual stdout:${COLOR.reset}`);
    console.log(stdout === "" ? "    (empty)" : stdout.replace(/^/gm, "    "));

    if (expectedErr !== null) {
      console.log(`${COLOR.yellow}expected stderr:${COLOR.reset}`);
      console.log(expectedErr === "" ? "    (empty)" : expectedErr.replace(/^/gm, "    "));
    }
    console.log(`${COLOR.yellow}actual stderr:${COLOR.reset}`);
    console.log(stderr === "" ? "    (empty)" : stderr.replace(/^/gm, "    "));
    if (expectedCode !== null) {
      console.log(`${COLOR.yellow}expected code:${COLOR.reset} ${expectedCode}`);
    }
    console.log(`${COLOR.yellow}actual code:${COLOR.reset} ${res.code}`);
  }

  if (failures > 0) {
    console.error(`${COLOR.red}\n${failures} test(s) failed.${COLOR.reset}`);
    process.exit(1);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
