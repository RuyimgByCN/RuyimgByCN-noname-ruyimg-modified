import collections
import hashlib
import json
import pathlib
import re

BASE = pathlib.Path(__file__).resolve().parent
prefix = 'session-u08-1794594672096'
requests = [json.loads(s) for s in (BASE / (prefix + '-requests.jsonl')).read_text().splitlines()]
replies = [json.loads(s) for s in (BASE / (prefix + '-replies.jsonl')).read_text().splitlines()]
reply_by_time = collections.defaultdict(list)
for line, row in enumerate(replies, 1):
    reply_by_time[row['at']].append(line)
starts = []
reloads = []
for line, row in enumerate(requests, 1):
    expression = row['expression']
    if expression.startswith('window.u08ReloadPending=true;'):
        reloads.append(line)
    m = re.match(r'window\.u08Case=("[^"]+");window\.u08Fixture=', expression)
    if m:
        starts.append({'case': json.loads(m.group(1)), 'assignment_line': line,
                       'request_first_line': max(x for x in reloads if x < line)})
counts = [4, 4, 31, 31]
assert len(starts) == sum(counts), len(starts)
cohort_names = [f'browser-round{round}' for round, count in enumerate(counts, 1) for _ in range(count)]
segments = []
for i, item in enumerate(starts):
    last = starts[i + 1]['request_first_line'] - 1 if i + 1 < len(starts) else len(requests)
    rq = requests[item['request_first_line'] - 1:last]
    lines = sorted({line for row in rq for line in reply_by_time[row['at']]})
    segments.append({**item, 'cohort': cohort_names[i], 'request_last_line': last,
                     'request_lines': len(rq), 'reply_first_line': min(lines), 'reply_last_line': max(lines),
                     'reply_lines': lines, 'first_request_at': rq[0]['at'], 'last_request_at': rq[-1]['at'],
                     'fixture_assignment_request_line': item['assignment_line'],
                     'upstream_result_file': f"{cohort_names[i]}/upstream-{item['case']}.json"})
files = []
for suffix in ['requests', 'replies', 'network', 'events']:
    p = BASE / f'{prefix}-{suffix}.jsonl'
    files.append({'path': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'bytes': p.stat().st_size,
                  'lines': len(p.read_text().splitlines())})
report = {'session': 'active-session-u08.json', 'browser_environment': 'browser-environment.json',
          'request_pairing': 'Control server stamps each request/reply with the same at value; reply line indices are retained rather than assuming fixed line offset.',
          'prelude_request_last_line': starts[0]['request_first_line'] - 1,
          'raw_files': files, 'cohort_counts': dict(zip(['browser-round1', 'browser-round2', 'browser-round3', 'browser-round4'], counts)),
          'cases': segments,
          'runtime_exception_details_replies': [{'reply_line': i, 'at': r['at'], 'exception': r['result']['exceptionDetails']} for i, r in enumerate(replies, 1) if 'exceptionDetails' in r.get('result', {})],
          'spine_network_requests': [r for s in (BASE / (prefix + '-network.jsonl')).read_text().splitlines() if 'spine' in (r := json.loads(s)).get('url', '').lower()]}
(BASE / 'raw-transcript-index.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'cases': len(segments), 'cohorts': report['cohort_counts'], 'request_lines': len(requests), 'reply_lines': len(replies), 'spine_requests': len(report['spine_network_requests'])}))
