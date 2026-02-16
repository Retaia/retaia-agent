#!/usr/bin/env node
import { execSync } from "node:child_process";

const run = (command) =>
  execSync(command, { stdio: ["ignore", "pipe", "pipe"], encoding: "utf-8" }).trim();

const eventName = process.env.GITHUB_EVENT_NAME;
const baseRef =
  eventName === "pull_request"
    ? process.env.GITHUB_BASE_REF
    : process.env.BASE_BRANCH || "master";
const headRef = process.env.GITHUB_HEAD_REF;

if (!baseRef) {
  console.error("Missing base branch reference.");
  process.exit(1);
}

try {
  execSync(`git fetch --no-tags origin ${baseRef}`, { stdio: "inherit" });
  if (eventName === "pull_request" && headRef) {
    execSync(`git fetch --no-tags origin ${headRef}`, { stdio: "inherit" });
  }

  const baseHead = run(`git rev-parse origin/${baseRef}`);
  const head =
    eventName === "pull_request" && headRef
      ? run(`git rev-parse origin/${headRef}`)
      : run("git rev-parse HEAD");
  const mergeBase = run(`git merge-base ${head} ${baseHead}`);

  if (mergeBase !== baseHead) {
    console.error(
      [
        `Branch is behind origin/${baseRef}.`,
        `Expected merge-base ${baseHead}, got ${mergeBase}.`,
        "Please rebase on the latest base branch.",
      ].join("\n"),
    );
    process.exit(1);
  }

  const mergeCommitsRaw = run(`git rev-list --merges origin/${baseRef}..${head} || true`);
  const mergeCommits = mergeCommitsRaw
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);

  if (mergeCommits.length > 0) {
    console.error("Linear history required: merge commits found in branch.");
    for (const sha of mergeCommits) {
      const subject = run(`git show -s --format=%s ${sha}`);
      console.error(`- ${sha} ${subject}`);
    }
    console.error("Please rebase and remove merge commits before pushing.");
    process.exit(1);
  }

  console.log(`Branch is up to date with origin/${baseRef} and has linear history.`);
} catch (error) {
  console.error("Failed to verify branch freshness and linear history.");
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
