import assert from "node:assert/strict";
import test from "node:test";
import { bodyViolations, commitMessageViolations, footerMarker } from "./check-pr-body.mjs";

test("a pull request body with the agent footer comment fails", () => {
  const violations = bodyViolations(`Summary\n${footerMarker}_BEGIN -->\n`);
  assert.equal(violations.length, 1);
  assert.match(violations[0], /agent footer comment/);
  assert.match(violations[0], /Remove the footer/);
});

test("an ordinary pull request body passes", () => {
  assert.deepEqual(bodyViolations("Explain the change and the check you ran."), []);
});

test("prose that names the footer marker passes", () => {
  assert.deepEqual(
    bodyViolations("The check rejects a CURSOR_AGENT_PR_BODY comment."),
    [],
  );
});

test("99f9198 Cursor trailer fails", () => {
  const line = "Co-authored-by: Cursor <cursoragent@cursor.com>";
  const violations = commitMessageViolations(`feat\n\n${line}\n`, "99f9198");
  assert.equal(violations.length, 1);
  assert.match(violations[0], /99f9198 contains "Co-authored-by: Cursor <cursoragent@cursor.com>"/);
  assert.match(violations[0], /Commit as Andrejs Strazds and remove the Cursor agent trailer/);
});

test("678ed76 Cursor Agent trailer fails", () => {
  const line = "Co-authored-by: Cursor Agent <cursoragent@cursor.com>";
  const violations = commitMessageViolations(`docs\n\n${line}\n`, "678ed76");
  assert.equal(violations.length, 1);
  assert.match(violations[0], /678ed76 contains "Co-authored-by: Cursor Agent <cursoragent@cursor.com>"/);
});

test("a dependabot trailer is not a Cursor agent signature", () => {
  const message = "Co-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\n";
  assert.deepEqual(commitMessageViolations(message, "dependabot"), []);
});
