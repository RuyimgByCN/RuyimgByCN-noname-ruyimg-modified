// Static evidence collector. Run from noname-next; reads original extension JS only.
// Does not read bundled Spine runtimes or write inventory/product files.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const outDir = __dirname;
const items = JSON.parse(fs.readFileSync(path.join(outDir, 'inventory-snapshot.json'), 'utf8')).items;
const parserDir = fs.readdirSync('apps/web-client/node_modules/.pnpm')
  .find(name => name.startsWith('@babel+parser@'));
if (!parserDir) throw new Error('Existing local Babel parser is required');
const parser = require(path.resolve('apps/web-client/node_modules/.pnpm', parserDir, 'node_modules/@babel/parser'));
const parsed = new Map();

function nameOf(node) {
  if (!node) return null;
  if (node.type === 'Identifier') return node.name;
  if (node.type === 'StringLiteral') return node.value;
  return null;
}

function readSource(rel) {
  if (parsed.has(rel)) return parsed.get(rel);
  const text = fs.readFileSync(path.resolve('..', rel), 'utf8');
  const ast = parser.parse(text, { sourceType: 'script', errorRecovery: true });
  const definitions = [];
  function walk(node) {
    if (!node || typeof node !== 'object') return;
    if (Array.isArray(node)) { node.forEach(walk); return; }
    if (node.type === 'ObjectProperty' || node.type === 'ObjectMethod') {
      definitions.push({ id: nameOf(node.key), node });
    } else if (node.type === 'AssignmentExpression' && node.left.type === 'MemberExpression') {
      definitions.push({ id: nameOf(node.left.property), node });
    }
    for (const [key, value] of Object.entries(node)) {
      if (!['loc', 'comments', 'leadingComments', 'trailingComments', 'innerComments', 'errors', 'tokens'].includes(key)) walk(value);
    }
  }
  walk(ast.program);
  const result = { text, lines: text.split(/\r?\n/), definitions, comments: ast.comments,
    errors: ast.errors.map(e => e.message), sha256: crypto.createHash('sha256').update(text).digest('hex') };
  parsed.set(rel, result);
  return result;
}

const results = [];
for (const [key, item] of Object.entries(items)) {
  if (item.category === 'surfaces') continue;
  let rel = item.upstream_ref.replace(/:\d+(?:-\d+)?$/, '');
  let line = Number(item.upstream_ref.match(/:(\d+)/)[1]);
  if (item.source_id === 'L29') {
    rel = 'archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js';
    line = 1;
  }
  const source = readSource(rel);
  const candidates = source.definitions.filter(d => d.id === item.id && d.node.loc.start.line === line);
  const candidate = candidates.sort((a, b) => a.node.end - a.node.start - (b.node.end - b.node.start))[0];
  const comment = source.comments.find(c => c.type === 'CommentBlock' && c.loc.start.line <= line && line <= c.loc.end.line)
    || source.comments.find(c => c.type === 'CommentLine' && c.loc.start.line === line && source.lines[line - 1].trimStart().startsWith('//'));
  if (candidate) {
    const n = candidate.node;
    const fragment = source.text.slice(n.start, n.end);
    results.push({ key, path: rel, start: n.loc.start.line, end: n.loc.end.line,
      start_offset: n.start, end_offset: n.end, active_definition: true,
      lines: fragment.split(/\r?\n/), source_sha256: source.sha256 });
  } else if (comment) {
    let commentEnd = comment.loc.end.line;
    if (comment.type === 'CommentLine') {
      while (commentEnd < source.lines.length) {
        const next = source.lines[commentEnd].trim();
        if (next && !next.startsWith('//')) break;
        commentEnd++;
      }
    }
    const end = Math.min(commentEnd, line + 64);
    results.push({ key, path: rel, start: line, end,
      active_definition: false, disabled_by: comment.type,
      comment_start: comment.loc.start.line, comment_end: commentEnd,
      excerpt_truncated: end < commentEnd,
      lines: source.lines.slice(line - 1, end), source_sha256: source.sha256 });
  } else {
    throw new Error(`Cannot locate exact source definition: ${key}`);
  }
}
fs.writeFileSync(path.join(outDir, 'upstream-excerpts.json'), JSON.stringify(results, null, 2) + '\n');
const summary = { parser: parserDir, files: [...parsed].map(([file, value]) => ({
  path: file, sha256: value.sha256, parse_errors: value.errors,
})), definitions: results.length,
  commented_items: results.filter(x => !x.active_definition).map(x => ({
    key: x.key, comment_start: x.comment_start, comment_end: x.comment_end, kind: x.disabled_by,
  })) };
fs.writeFileSync(path.join(outDir, 'upstream-parse-summary.json'), JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify({ files: summary.files.length, parse_errors: summary.files.reduce((n, x) => n + x.parse_errors.length, 0),
  definitions: results.length, commented_items: summary.commented_items.length }));
