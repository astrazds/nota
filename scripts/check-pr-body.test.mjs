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

test("a7092a0 fabricated noreply fails without a Cursor trailer", () => {
  const line = "Co-authored-by: Andrejs Strazds <astrazds@users.noreply.github.com>";
  const violations = commitMessageViolations(`chore\n\n${line}\n`, "a7092a0");
  assert.equal(violations.length, 1);
  assert.match(
    violations[0],
    /a7092a0 contains "Co-authored-by: Andrejs Strazds <astrazds@users\.noreply\.github\.com>"/,
  );
  assert.match(
    violations[0],
    /Commit as Andrejs Strazds <35437548\+astrazds@users\.noreply\.github\.com>/,
  );
});

test("the real GitHub noreply co-author passes", () => {
  const message =
    "Co-authored-by: Andrejs Strazds <35437548+astrazds@users.noreply.github.com>\n";
  assert.deepEqual(commitMessageViolations(message, "36fc64b"), []);
});

test("a dependabot trailer is not a Cursor agent signature", () => {
  const message = "Co-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\n";
  assert.deepEqual(commitMessageViolations(message, "dependabot"), []);
});
