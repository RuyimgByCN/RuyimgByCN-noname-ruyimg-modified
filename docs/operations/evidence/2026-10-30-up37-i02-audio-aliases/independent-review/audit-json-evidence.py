from pathlib import Path
from urllib.parse import urlsplit, unquote
import collections
import json
import math

MAIN = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
ROOT = MAIN / 'docs/operations/evidence/2026-10-30-up37-i02-audio-aliases/main-acceptance'
OUT = Path('/tmp/parent-audio-alias-main-review')
def load(name):
    return json.loads((ROOT / name).read_text())
def asset_path(url):
    path = unquote(urlsplit(url).path)
    return path.split('/original-content/assets/', 1)[1]

happy = load('happy/browser-result.json')
errors = load('errors/browser-result.json')
http = load('real-http.json')
allowlist = load('prior-passed-sample-allowlist.json')
owners = load('all79-owner-audit.json')
identities = load('all-catalog-identities.json')
mappings = load('all-mappings.json')
byte_rows = load('alias-byte-evidence.json')
manifest = json.loads((MAIN / 'content/original/asset-manifest.json').read_text())
assets = {row['path']: row for row in manifest['assets']}
assert len(assets) == len(manifest['assets'])
roles = happy['matrix']
assert len(roles) == 15
page = [(role['id'], audio) for role in roles for audio in role['audio']]
plays = [(role['id'], play) for role in roles for play in role['clickedPlayback']]
assert len(page) == 121 and len(plays) == 14
known_five = {f'audio/skill/tsumugi_mugyu{index}.mp3' for index in range(1, 6)}
page_records = []
for owner, audio in page:
    path = asset_path(audio['src'])
    asset = assets[path]
    source = asset['source_path']
    assert asset['source_category'] == 'core' and source not in known_five
    assert path.endswith('.mp3') and audio['sha256'] == asset['sha256']
    assert audio['byteLength'] == asset['size']
    assert math.isfinite(audio['duration']) and audio['duration'] > 0
    assert audio['channels'] > 0 and audio['sampleRate'] > 0
    prior = allowlist[source]
    assert prior['ok'] is True and prior['decoderOK'] is True
    assert prior['sha256'] == audio['sha256'] and prior['bytes'] == audio['byteLength']
    page_records.append({'owner': owner, 'path': path, 'source_path': source, 'sha256': audio['sha256'], 'bytes': audio['byteLength'], 'prior_index': prior['index']})
for owner, play in plays:
    assert play['time'] > 0 and play['playingEvents'] > 0 and play['playedRanges'] > 0
    assert play['error'] is None
assert len(errors['matrix']) == 9 and errors['passed'] is True and not errors['exceptions']
requests = http['requests']
assert len(requests) == 214 and all(row['status'] == 200 for row in requests)
assert len([row for row in requests if not row['path'].startswith('assets/')]) == 2
request_assets = [row for row in requests if row['path'].startswith('assets/')]
for row in request_assets:
    asset = assets[row['path'].removeprefix('assets/')]
    assert row['bytes'] == asset['size'] and row['sha256'] == asset['sha256']
    assert row['source_path'] == asset['source_path'] and row['source_path'] not in known_five
page_paths = [row['path'] for row in page_records]
assert collections.Counter(row['path'].removeprefix('assets/') for row in requests[2:123]) == collections.Counter(page_paths)
assert len(requests[123:]) == 91
assert len(owners) == 79
cross = [row for row in owners if row['cross_pack']]
assert len(cross) == 8 and all(row['aliases'] and row['expected_sources'] for row in cross)
assert sum(len(row['aliases']) for row in cross) == 91
assert sum(len(row['expected_sources']) for row in owners) == 571
cross_by_owner = {}
for row in cross:
    cross_by_owner.setdefault(row['owner'], []).append({'form': row['substitute'], 'aliases': len(row['aliases']), 'sources': len(row['expected_sources'])})
assert len(cross_by_owner) == 4
cross_paths = [asset['path'] for row in cross for asset in row['aliases']]
assert collections.Counter(row['path'].removeprefix('assets/') for row in requests[123:]) == collections.Counter(cross_paths)
assert len(identities) == 2582
for row in identities:
    pack, name = row['id'].split(':', 1)
    prefixes = (f'audio-declared/{pack}/die/{name}/', f'audio-declared/{pack}/character/{name}/skill/') + tuple(
        f'audio-declared/{pack}/character/{name}/die/form-{form["substitute"]}/'
        for form in owners if form['owner'] == row['id']
    )
    assert all(asset['path'].startswith(prefixes) for asset in row['selected']), row['id']
assert len(byte_rows) == 23393 and len({row['path'] for row in byte_rows}) == 23393
assert all(row['same_inode_chain'] and row['upstream_backing_target_sha_equal'] for row in byte_rows)
assert len({row['source_path'] for row in byte_rows}) == 8824
for row in byte_rows:
    asset = assets[row['path']]
    assert (row['bytes'], row['sha256'], row['source_path']) == (asset['size'], asset['sha256'], asset['source_path'])
assert len(mappings['mappings']) == 25880 and len(mappings['assets']) == 23393
assert len(mappings['missing']) == 2487 and len(mappings['dynamic']) == 234
wanted = {row['prior_index']: row for row in page_records}
found = {}
with Path('/tmp/noname-i02-all-codecs.LXnAxI/chrome.jsonl').open() as stream:
    for line in stream:
        row = json.loads(line)
        if row.get('event') != 'result' or row.get('index') not in wanted:
            continue
        expected = wanted[row['index']]
        assert row['ok'] is True and row['decoderOK'] is True
        assert (row['source_path'], row['sha256'], row['bytes']) == (expected['source_path'], expected['sha256'], expected['bytes'])
        found[row['index']] = {'index': row['index'], 'source_path': row['source_path'], 'sha256': row['sha256'], 'bytes': row['bytes'], 'ok': True, 'decoderOK': True}
assert set(found) == set(wanted)
report = {'all_assertions_passed': True, 'roles': 15, 'decoded_page_entries': 121, 'controls_playbacks': [{'owner': owner, **play} for owner, play in plays], 'error_scenes': 9, 'http': {'all': 214, 'json': 2, 'page_audio': 121, 'cross_alias': 91, 'all_page_manifest_sha_size_exact': True}, 'owners': 79, 'cross_by_owner': cross_by_owner, 'cross_forms': 8, 'available_substitute_sources': 571, 'cross_alias_rows': 91, 'catalog_identities': 2582, 'selected_prefixes_own_identity': True, 'alias_byte_rows': 23393, 'distinct_source_paths': 8824, 'inode_and_hash_assertions_all_true': True, 'mappings': 25880, 'missing': 2487, 'dynamic': 234, 'page_records': page_records, 'prior_codec_actual_rows': list(found.values()), 'known5_or_compat_in_page': False, 'scope': 'Existing JSON only; metadata assertions cross-checked, no fresh media read/decode/test/build/driver or protected tree hash.'}
(OUT / 'browser-and-mapping-results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({key: report[key] for key in ['all_assertions_passed', 'roles', 'decoded_page_entries', 'http', 'cross_by_owner', 'alias_byte_rows', 'distinct_source_paths', 'mappings', 'missing', 'dynamic']}, indent=2))
