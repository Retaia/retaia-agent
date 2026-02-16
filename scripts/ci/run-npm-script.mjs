#!/usr/bin/env node
import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

const scriptName = process.argv[2];

if (!scriptName) {
  console.error("Usage: node scripts/ci/run-npm-script.mjs <script-name>");
  process.exit(1);
}

const packageJsonPath = path.resolve(process.cwd(), "package.json");
const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf-8"));
const scripts = packageJson.scripts || {};

if (!Object.prototype.hasOwnProperty.call(scripts, scriptName)) {
  console.error(
    [
      `Missing required npm script: "${scriptName}".`,
      "Define it in package.json.",
      "Expected suites:",
      '- "test:tdd": tests bases sur le fonctionnement du code.',
      '- "test:bdd": tests bases sur les scenarios derives des specs.',
      '- "test:e2e": tests end-to-end derives des specs.',
      '- "test:coverage": generation du rapport coverage Rust (coverage/llvm-cov-summary.json).',
    ].join("\n"),
  );
  process.exit(1);
}

try {
  execSync(`npm run ${scriptName}`, { stdio: "inherit" });
} catch (error) {
  process.exit(error && typeof error.status === "number" ? error.status : 1);
}
