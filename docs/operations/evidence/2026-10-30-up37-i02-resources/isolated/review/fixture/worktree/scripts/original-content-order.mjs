import { readdir } from "node:fs/promises";
import path from "node:path";

const collator = new Intl.Collator("en-US");

export function compareFileNames(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

export function compareContentNames(a, b) {
  return collator.compare(a, b) || compareFileNames(a, b);
}

export async function listFiles(root) {
  const files = [];
  async function visit(directory, prefix = "") {
    const entries = await readdir(directory, { withFileTypes: true });
    entries.sort((a, b) => compareFileNames(a.name, b.name));
    for (const entry of entries) {
      const relative = path.join(prefix, entry.name);
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) await visit(absolute, relative);
      else if (entry.isFile()) files.push(relative);
    }
  }
  await visit(root);
  return files;
}
