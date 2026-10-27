import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

test("content traversal and index ordering ignore the process locale", (t) => {
  const root = mkdtempSync(path.join(tmpdir(), "original-content-order-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(path.join(root, "nested"));
  for (const file of ["guānning.jpg", "guanning.jpg", "nested/赵.jpg", "nested/张.jpg"]) {
    writeFileSync(path.join(root, file), "");
  }

  const moduleUrl = new URL("./original-content-order.mjs", import.meta.url).href;
  const code = `
    import { compareContentNames, listFiles } from ${JSON.stringify(moduleUrl)};
    const names = ["赵", "张", "guānning", "guanning", "é", "e\\u0301"];
    console.log(JSON.stringify({
      locale: Intl.DateTimeFormat().resolvedOptions().locale,
      files: await listFiles(process.argv[1]),
      names: names.sort(compareContentNames),
      equivalent: compareContentNames("e\\u0301", "é"),
    }));
  `;
  const results = ["en_US", "zh_CN", "tr_TR"].map((locale) => JSON.parse(execFileSync(
    process.execPath,
    ["--input-type=module", "-e", code, root],
    { encoding: "utf8", env: { ...process.env, LANG: `${locale}.UTF-8`, LC_ALL: `${locale}.UTF-8` } },
  )));
  assert.deepEqual(results.map((result) => result.locale), ["en-US", "zh-CN", "tr-TR"]);
  for (const result of results) {
    assert.deepEqual(result.files, results[0].files);
    assert.deepEqual(result.names, results[0].names);
    assert.ok(result.files.indexOf("guanning.jpg") < result.files.indexOf("guānning.jpg"));
    assert.ok(result.files.includes(path.join("nested", "赵.jpg")));
    assert.ok(result.files.includes(path.join("nested", "张.jpg")));
    assert.ok(result.equivalent < 0);
  }
});
