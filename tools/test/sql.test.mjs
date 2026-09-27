import { test } from "node:test";
import assert from "node:assert/strict";
import { dbNameFor, normalizeOutput, diffLines, planRun, setupScript } from "../sql.mjs";

test("dbNameFor turns a slug into a database name", () => {
  assert.equal(dbNameFor("crud-in-sql"), "crud_in_sql");
  assert.equal(dbNameFor("a1-build-library-schema"), "a1_build_library_schema");
  assert.throws(() => dbNameFor("bad; DROP DATABASE x"));
});

test("normalizeOutput strips trailing spaces and blank lines, keeps one final newline", () => {
  assert.equal(normalizeOutput(" id | name \r\n----+------\n  1 | Ada  \n\n\n"), " id | name\n----+------\n  1 | Ada\n");
  assert.equal(normalizeOutput(""), "");
});

test("diffLines reports changed, added and removed lines (1-based)", () => {
  assert.deepEqual(diffLines("a\nb\n", "a\nb\n"), []);
  assert.deepEqual(diffLines("a\nb\n", "a\nc\n"), [{ line: 2, expected: "b", actual: "c" }]);
  assert.deepEqual(diffLines("a\n", "a\nb\n"), [{ line: 2, expected: undefined, actual: "b" }]);
});

test("planRun sorts .sql files and finds orphan .out files", () => {
  const r = planRun(["02-select.sql", "01-create.sql", "01-create.out", "09-old.out", "README.md"]);
  assert.deepEqual(r.sql, ["01-create.sql", "02-select.sql"]);
  assert.deepEqual(r.orphans, ["09-old.out"]);
  assert.throws(() => planRun(["select.sql"]), /NN-/);
});

test("setupScript sets ON_ERROR_STOP so a failed DROP/CREATE actually fails, and only for setup", () => {
  assert.equal(
    setupScript("crud_in_sql"),
    "\\set ON_ERROR_STOP 1\nDROP DATABASE IF EXISTS crud_in_sql WITH (FORCE);\nCREATE DATABASE crud_in_sql;\n"
  );
});
