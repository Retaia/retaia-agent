#!/usr/bin/env node
import { readFileSync } from "node:fs";
import path from "node:path";

const args = process.argv.slice(2);
let file = "coverage/coverage-summary.json";
let min = 80;

for (let i = 0; i < args.length; i += 1) {
  if (args[i] === "--file") {
    file = args[i + 1];
    i += 1;
    continue;
  }
  if (args[i] === "--min") {
    min = Number(args[i + 1]);
    i += 1;
  }
}

if (!Number.isFinite(min)) {
  console.error("Invalid --min value.");
  process.exit(1);
}

const filePath = path.resolve(process.cwd(), file);
let summary;

try {
  summary = JSON.parse(readFileSync(filePath, "utf-8"));
} catch (error) {
  console.error(`Unable to read coverage summary at ${filePath}.`);
  console.error("Ensure test:coverage generates coverage/coverage-summary.json.");
  process.exit(1);
}

const coverageCandidates = [
  summary?.total?.lines?.pct,
  summary?.totals?.lines?.percent,
  summary?.data?.[0]?.totals?.lines?.percent,
];
const coverage = coverageCandidates.find((value) => Number.isFinite(value));

if (!Number.isFinite(coverage)) {
  console.error("Invalid coverage summary format.");
  console.error(
    "Expected one of JSON paths: total.lines.pct, totals.lines.percent, data[0].totals.lines.percent",
  );
  process.exit(1);
}

if (coverage < min) {
  console.error(`Coverage check failed: ${coverage}% < ${min}% (minimum required).`);
  process.exit(1);
}

console.log(`Coverage check passed: ${coverage}% >= ${min}%.`);
