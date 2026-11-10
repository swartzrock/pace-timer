import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

function fixture(t) {
  const cwd = mkdtempSync(join(tmpdir(), "pace-release-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  for (const file of ["Cargo.toml", "Cargo.lock"]) {
    const source = readFileSync(new URL(`../${file}`, import.meta.url), "utf8");
    writeFileSync(join(cwd, file), source.replace(/(name = "pace"\nversion = ")[^"]+/, "$10.3.0"));
  }
  writeFileSync(join(cwd, "package.json"), JSON.stringify({ name: "pace", version: "0.3.1" }));
  writeFileSync(join(cwd, "CHANGELOG.md"), "# pace\n\n## 0.3.1\n\n### Patch Changes\n\n- Timer fix.\n\n## 0.3.0\n\n- Older notes.\n");
  return cwd;
}

function run(script, cwd) {
  return spawnSync(process.execPath, [new URL(script, import.meta.url).pathname], { cwd, encoding: "utf8" });
}

test("version bump updates only the Rust package versions and is repeatable", (t) => {
  const cwd = fixture(t);
  for (let i = 0; i < 2; i++) {
    const result = run("version-bump.mjs", cwd);
    assert.equal(result.status, 0, result.stderr);
  }
  for (const file of ["Cargo.toml", "Cargo.lock"]) {
    const original = readFileSync(new URL(`../${file}`, import.meta.url), "utf8");
    assert.equal(readFileSync(join(cwd, file), "utf8"), original.replace(/(name = "pace"\nversion = ")[^"]+/, "$10.3.1"));
  }
});

test("release notes contain only the current version's changelog section", (t) => {
  const cwd = fixture(t);
  assert.equal(run("version-bump.mjs", cwd).status, 0);
  const result = run("prepare-release.mjs", cwd);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "### Patch Changes\n\n- Timer fix.\n");
});

test("release notes reject inconsistent Rust versions", (t) => {
  const result = run("prepare-release.mjs", fixture(t));
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /version mismatch/i);
});

test("release notes reject missing changelog entries", (t) => {
  const cwd = fixture(t);
  assert.equal(run("version-bump.mjs", cwd).status, 0);
  writeFileSync(join(cwd, "CHANGELOG.md"), "# pace\n\n## 0.3.0\n\n- Older notes.\n");
  const result = run("prepare-release.mjs", cwd);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Missing or empty/);
});
