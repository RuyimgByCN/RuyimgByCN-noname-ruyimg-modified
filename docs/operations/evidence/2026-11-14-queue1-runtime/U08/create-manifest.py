import datetime
import hashlib
import json
import pathlib
import re
import subprocess

BASE = pathlib.Path(__file__).resolve().parent
excluded = {'manifest.json', 'manifest.sha256', 'manifest.raw.txt'}
# Validate every Markdown link local to this evidence directory before freezing.
links = []
for p in BASE.glob('*.md'):
    text = re.sub(r'```.*?```', '', p.read_text(), flags=re.S)
    for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)', text):
        target = target.split('#', 1)[0]
        if target and '://' not in target:
            assert (p.parent / target).exists(), (p.name, target)
            links.append({'file': p.name, 'target': target})
freeze = {'date': '2026-11-14', 'frozen_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
          'owner': '/root/fix_dddanzhi',
          'directory_frozen': True, 'product_source_modified_by_owner': False,
          'shared_target_cleaned': False, 'private_target_created': False,
          'local_markdown_link_checks': links,
          'scope': 'Evidence collection complete. Independent review pending; no equivalence signoff. Only manifest metadata/log are excluded to avoid recursive file hashes.',
          'manifest_excluded': sorted(excluded), 'temporary_browser_cleanup': 'cleanup.json',
          'retained_probe': 'preserved-probe.json'}
(BASE / 'freeze.json').write_text(json.dumps(freeze, ensure_ascii=False, indent=2) + '\n')
files = []
for p in sorted(BASE.rglob('*')):
    if p.is_file() and p.name not in excluded:
        files.append({'path': str(p.relative_to(BASE)), 'bytes': p.stat().st_size,
                      'sha256': hashlib.sha256(p.read_bytes()).hexdigest()})
manifest = {'created_at': freeze['frozen_at'], 'head': freeze['head'], 'files': files,
            'excluded_paths': sorted(excluded), 'file_count': len(files),
            'total_bytes': sum(x['bytes'] for x in files)}
(BASE / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
(BASE / 'manifest.sha256').write_text(''.join(x['sha256'] + '  ' + x['path'] + '\n' for x in files))
for x in files:
    assert hashlib.sha256((BASE / x['path']).read_bytes()).hexdigest() == x['sha256'], x['path']
print(json.dumps({'frozen_at': freeze['frozen_at'], 'head': freeze['head'], 'verified_files': len(files),
                  'total_bytes': manifest['total_bytes'], 'local_md_links': len(links),
                  'manifest_sha256': hashlib.sha256((BASE / 'manifest.json').read_bytes()).hexdigest(),
                  'manifest_sha256_file_sha256': hashlib.sha256((BASE / 'manifest.sha256').read_bytes()).hexdigest(),
                  'review_input_manifest_sha256': hashlib.sha256((BASE / 'review-input-manifest.json').read_bytes()).hexdigest(),
                  'raw_index_sha256': hashlib.sha256((BASE / 'raw-transcript-index.json').read_bytes()).hexdigest()}))
