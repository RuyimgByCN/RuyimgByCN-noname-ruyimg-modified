#!/usr/bin/env python3
"""Reproduce checkout byte drift with original 777 blobs; writes only a new /tmp fixture."""
import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

HEAD = '777bf33969e5ecc04b63971adab26be32f12fd28'
TREE = '7f2d41e257dfc44072bd6a5ef68dad4cb6c70496'
PIN = '1bd98a3e5d7ab22f6749b132d041f409e95a0dbf2f7ed9a603ec256422cc5313'
PROVENANCE = 'content/original-audio-compat/provenance.json'
RULE = b'content/original-audio-compat/provenance.json -text\n'
WHY = b'# Preserve the exact provenance bytes pinned by validateAudioCompat.\n'
SCRIPTS = ['original-audio-resources.mjs', 'original-content-order.mjs',
           'original-source-ast.mjs', 'sync-original-content.mjs',
           'test_original_content_order.mjs', 'test_original_audio_resources.mjs',
           'test_original_resources.mjs']


def sha(data):
    return hashlib.sha256(data).hexdigest()


def description(data):
    return {'bytes': len(data), 'sha256': sha(data), 'crlf': data.count(b'\r\n'),
            'lf': data.count(b'\n'), 'bare_lf': data.count(b'\n') - data.count(b'\r\n')}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True, type=Path)
    parser.add_argument('--core', required=True, type=Path)
    parser.add_argument('--publication', required=True, type=Path)
    parser.add_argument('--node', default='node')
    args = parser.parse_args()
    started = time.monotonic()
    output = Path(tempfile.mkdtemp(prefix='noname-ci-777-checkout-selfcheck-', dir='/tmp'))
    print(output, flush=True)
    logs = output / 'commands'
    logs.mkdir()
    fixture = output / 'checkout'
    fixture.mkdir()
    source = output / 'source'
    core = output / 'core'
    index = output / 'private-index'
    env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
    env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_SYSTEM='/dev/null',
               GIT_CONFIG_NOSYSTEM='1', GIT_ATTR_NOSYSTEM='1',
               TMPDIR=str(output))
    records = []

    def run(argv, cwd=None, data=None, expected=0, extra_env=None, name='command'):
        result = subprocess.run([str(x) for x in argv], cwd=cwd, input=data,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                env={**env, **(extra_env or {})})
        stem = f'{len(records):03d}-{name}'
        (logs / f'{stem}.stdout').write_bytes(result.stdout)
        (logs / f'{stem}.stderr').write_bytes(result.stderr)
        record = {'argv': [str(x) for x in argv], 'cwd': str(cwd) if cwd else None,
                  'exit': result.returncode, 'expected_exit': expected,
                  'stdin_bytes': len(data) if data is not None else None,
                  'stdin_sha256': sha(data) if data is not None else None,
                  'stdout': f'commands/{stem}.stdout', 'stderr': f'commands/{stem}.stderr'}
        records.append(record)
        (output / 'commands.json').write_text(json.dumps(records, indent=2) + '\n')
        assert result.returncode == expected, (record, result.stderr.decode(errors='replace'))
        return result.stdout

    def original(*argv):
        return run(['git', '-C', args.repo, *argv], name='original-read')

    private_env = {'GIT_INDEX_FILE': str(index)}

    def git(*argv, config=()):
        return run(['git', '-C', fixture, *config, *argv], extra_env=private_env, name='fixture-git')

    def blob(name):
        return original('cat-file', 'blob', f'{HEAD}:{name}')

    index_path = Path(original('rev-parse', '--path-format=absolute', '--git-path', 'index').decode().strip())
    product_index_before = description(index_path.read_bytes())
    assert original('rev-parse', HEAD + '^{tree}').decode().strip() == TREE
    published = json.loads((args.publication / 'push-stage/result.json').read_text())['committed_source14_sha256']
    old_artifacts = ['copy-manifest.json', 'push-stage/result.json',
                     'ci-stage/job-124265455086-raw-log.stdout.gz']
    old_artifacts_before = {p: description((args.publication / p).read_bytes()) for p in old_artifacts}
    admission_before = {p: sha(blob(p)) for p in published}
    assert admission_before == published
    product_before = {p: description((args.repo / p).read_bytes()) for p in published}
    product_before['.gitattributes'] = description((args.repo / '.gitattributes').read_bytes())
    canonical = blob(PROVENANCE)
    assert sha(canonical) == PIN
    proof = json.loads(canonical)
    original_attr = blob('.gitattributes')
    assert RULE not in original_attr and original_attr.endswith(b'\n')
    candidate_attr = original_attr + WHY + RULE
    candidate = output / 'candidate'
    candidate.mkdir()
    (candidate / '.gitattributes').write_bytes(candidate_attr)
    (output / 'candidate.patch').write_text(''.join(difflib.unified_diff(
        original_attr.decode().splitlines(keepends=True), candidate_attr.decode().splitlines(keepends=True),
        fromfile='a/.gitattributes', tofile='b/.gitattributes')))

    # Only seven reviewed small files enter the standalone Git index.
    git('-c', 'init.defaultBranch=fixture', 'init', '.')
    checkout_paths = ['.gitattributes', PROVENANCE] + [
        'content/original-audio-compat/' + row['derivative_path'] for row in proof['bindings']]
    indexed = {}
    for p in checkout_paths:
        data = original_attr if p == '.gitattributes' else blob(p)
        oid = run(
            ['git', '-C', fixture, 'hash-object', '-w', '--stdin'], data=data,
            extra_env=private_env, name='fixture-blob').decode().strip()
        assert oid == original('rev-parse', f'{HEAD}:{p}').decode().strip()
        git('update-index', '--add', '--cacheinfo', f'100644,{oid},{p}')
        indexed[p] = oid
    git('ls-files', '--stage')

    # Original source modules, copied canonical bytes, plus existing dependencies used read-only.
    for name in SCRIPTS:
        p = source / 'scripts' / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(blob('scripts/' + name))
        p.chmod(0o444)
    manifest = source / 'content/original/asset-manifest.json'
    manifest.parent.mkdir(parents=True, exist_ok=True)
    manifest.write_bytes(blob('content/original/asset-manifest.json'))
    manifest.chmod(0o444)
    deps = source / 'apps/web-client/node_modules'
    deps.parent.mkdir(parents=True)
    deps.symlink_to((args.repo / 'apps/web-client/node_modules').resolve(), target_is_directory=True)
    source_rows = []
    for row in proof['bindings']:
        raw = (args.core / row['source_path']).read_bytes()
        assert sha(raw) == row['source_sha256']
        target = core / row['source_path']
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
        target.chmod(0o444)
        source_rows.append({'path': row['source_path'], **description(raw)})

    runner = output / 'validate.mjs'
    runner.write_text('''import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
const [module, core, compat, outcome] = process.argv.slice(2);
const { validateAudioCompat } = await import(pathToFileURL(module));
const calls = [];
const sha256 = async file => {
  const bytes = await readFile(file);
  const digest = createHash('sha256').update(bytes).digest('hex');
  calls.push({ file, bytes: bytes.length, sha256: digest });
  return digest;
};
try {
  const bindings = await validateAudioCompat(core, compat, sha256);
  assert.equal(outcome, 'accept');
  assert.equal(bindings.length, 5);
  assert.equal(calls.length, 10);
  console.log(JSON.stringify({ accepted: true, bindings, calls }, null, 2));
} catch (error) {
  console.error(error.stack);
  console.log(JSON.stringify({ accepted: false, error: error.message, calls }, null, 2));
  if (outcome === 'reject') {
    assert.equal(error.message, 'audio compat fixed provenance/recipe/tool binding differs');
    assert.equal(calls.length, 10);
    process.exitCode = 1;
  } else throw error;
}
''')
    win = ('-c', 'core.autocrlf=true', '-c', 'core.eol=crlf', '-c', 'core.attributesFile=/dev/null')
    lf = ('-c', 'core.autocrlf=false', '-c', 'core.eol=lf', '-c', 'core.attributesFile=/dev/null')
    module = source / 'scripts/original-audio-resources.mjs'
    compat = fixture / 'content/original-audio-compat'
    cases = {}

    def validate(label, outcome):
        before = {p: sha((fixture / p).read_bytes()) for p in checkout_paths}
        before.update({'core/' + r['source_path']: sha((core / r['source_path']).read_bytes()) for r in proof['bindings']})
        result = run([args.node, runner, module, core, compat, outcome],
                     expected=1 if outcome == 'reject' else 0, name=label)
        after = {p: sha((fixture / p).read_bytes()) for p in checkout_paths}
        after.update({'core/' + r['source_path']: sha((core / r['source_path']).read_bytes()) for r in proof['bindings']})
        assert before == after
        cases[label] = {'provenance': description((fixture / PROVENANCE).read_bytes()),
                        'json_equals_original': json.loads((fixture / PROVENANCE).read_bytes()) == proof,
                        'no_input_writes': True, 'validator': json.loads(result)}

    for config in [win, lf]:
        git('config', '--show-origin', '--list', config=config)
    git('checkout-index', '--force', '--', '.gitattributes', config=win)
    git('check-attr', 'text', 'eol', '--', PROVENANCE, config=win)
    git('checkout-index', '--force', '--', *checkout_paths[1:], config=win)
    red = (fixture / PROVENANCE).read_bytes()
    assert red.count(b'\r\n') == canonical.count(b'\n') > 0
    assert sha(red) != PIN and json.loads(red) == proof
    validate('windows-before-red', 'reject')
    # Repeat the same unpatched checkout under the LF configuration.
    git('checkout-index', '--force', '--', PROVENANCE, config=lf)
    assert (fixture / PROVENANCE).read_bytes() == canonical
    validate('lf-before-green', 'accept')
    # Change only the indexed attributes blob, then let Git re-check out both files.
    candidate_oid = run(['git', '-C', fixture, 'hash-object', '-w', '--stdin'],
                        data=candidate_attr, extra_env=private_env, name='candidate-attribute-blob').decode().strip()
    git('update-index', '--cacheinfo', f'100644,{candidate_oid},.gitattributes')
    git('checkout-index', '--force', '--', '.gitattributes', config=win)
    git('check-attr', 'text', 'eol', '--', PROVENANCE,
        'docs/operations/evidence/2026-10-30-native-ci-preparation/first-publication/run41053688083-desktop-windows-raw.log',
        'content/unrelated.json', config=win)
    git('checkout-index', '--force', '--', PROVENANCE, config=win)
    assert (fixture / PROVENANCE).read_bytes() == canonical
    validate('windows-after-green', 'accept')
    git('checkout-index', '--force', '--', PROVENANCE, config=lf)
    assert (fixture / PROVENANCE).read_bytes() == canonical
    validate('lf-after-green', 'accept')
    git('ls-files', '--stage')
    # The published source14 still has identical index object IDs for all loaded inputs.
    for p in checkout_paths[1:]:
        assert git('ls-files', '--format=%(objectname)', '--', p).decode().strip() == indexed[p]
    shutil.copytree(compat, source / 'content/original-audio-compat')
    tests = ['scripts/test_original_content_order.mjs', 'scripts/test_original_audio_resources.mjs',
             'scripts/test_original_resources.mjs']
    test_output = run([args.node, '--test', '--test-reporter=tap', *tests], cwd=source,
                      extra_env={'NONAME_ORIGINAL_ROOT': str(core)}, name='original-resource17')
    assert b'# tests 17' in test_output and b'# pass 17' in test_output and b'# fail 0' in test_output
    admission_after = {p: sha(blob(p)) for p in published}
    assert admission_after == admission_before
    product_after = {p: description((args.repo / p).read_bytes()) for p in product_before}
    assert product_after == product_before
    assert description(index_path.read_bytes()) == product_index_before
    assert {p: description((args.publication / p).read_bytes()) for p in old_artifacts} == old_artifacts_before
    # Private staging is only a test input; leave no index or staged files behind.
    index.unlink()
    git('diff', '--cached', '--name-only')
    run([args.node, '--version'], name='node-version')
    git('--version')
    summary = {'head': HEAD, 'tree': TREE, 'output': str(output),
               'source14_before': admission_before, 'source14_after': admission_after,
               'product_bytes_before': product_before, 'product_bytes_after': product_after,
               'product_index_unchanged': product_index_before,
               'old_evidence_unchanged': old_artifacts_before,
               'candidate': {'changed_paths': ['.gitattributes'],
                             'before': description(original_attr), 'after': description(candidate_attr)},
               'canonical_provenance': description(canonical), 'readonly_core5': source_rows,
               'cases': cases, 'tests': {'pass': 17, 'fail': 0},
               'environment': {k: env[k] for k in ['GIT_CONFIG_GLOBAL', 'GIT_CONFIG_SYSTEM',
                                                 'GIT_CONFIG_NOSYSTEM', 'GIT_ATTR_NOSYSTEM', 'TMPDIR']},
               'private_index_removed': not index.exists(), 'elapsed_seconds': time.monotonic() - started}
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'output': str(output), 'cases': list(cases),
                      'tests_passed': 17, 'elapsed_seconds': summary['elapsed_seconds']}), flush=True)


if __name__ == '__main__':
    main()
