import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export const footerMarker = "<!-- CURSOR_AGENT_PR_BODY";
const cursorEmail = "cursoragent@cursor.com";
const fabricatedNoreply =
  "Co-authored-by: Andrejs Strazds <astrazds@users.noreply.github.com>";

export function bodyViolations(body) {
  if (!body || !body.includes(footerMarker)) return [];
  return [
    "Pull request body contains an agent footer comment. Remove the footer.",
  ];
}

export function commitMessageViolations(message, commit = "HEAD") {
  const violations = [];
  const cursorLine = message
    .split("\n")
    .find((entry) => entry.includes("Co-authored-by:") && entry.includes(cursorEmail));
  if (cursorLine) {
    violations.push(
      `${commit} contains "${cursorLine.trim()}". Commit as Andrejs Strazds and remove the Cursor agent trailer.`,
    );
  }
  if (message.includes(fabricatedNoreply)) {
    violations.push(
      `${commit} contains "${fabricatedNoreply}". Commit as Andrejs Strazds <35437548+astrazds@users.noreply.github.com> and remove this trailer.`,
    );
  }
  return violations;
}

export function commitRangeViolations(baseSha, root = process.cwd()) {
  const output = execFileSync(
    "git",
    ["-C", root, "log", `${baseSha}..HEAD`, "--format=%H%x1e%B%x1e"],
    { encoding: "utf8" },
  );
  const violations = [];
  const parts = output.split("\x1e");
  for (let index = 0; index + 1 < parts.length; index += 2) {
    const commit = parts[index].trim();
    if (!commit) continue;
    violations.push(...commitMessageViolations(parts[index + 1] ?? "", commit));
  }
  return violations;
}

function readBody(argv, env) {
  const fileFlag = argv.indexOf("--file");
  if (fileFlag === -1) return env.PR_BODY ?? "";
  const path = argv[fileFlag + 1];
  if (!path) throw new Error("Pass a file path after --file.");
  return readFileSync(path, "utf8");
}

function isDirectRun() {
  const entry = process.argv[1];
  return Boolean(entry) && import.meta.url === pathToFileURL(entry).href;
}

if (isDirectRun()) {
  const violations = bodyViolations(readBody(process.argv, process.env));
  if (process.env.BASE_SHA) {
    violations.push(
      ...commitRangeViolations(process.env.BASE_SHA, process.env.CHECK_ROOT ?? process.cwd()),
    );
  }
  if (violations.length > 0) {
    for (const violation of violations) console.error(violation);
    process.exit(1);
  }
}
