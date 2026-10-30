from pathlib import Path
import json
import re

ROOT = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/evidence/phase2')
OUT = Path('/tmp/parent-c331-golden-review')
OUT.mkdir(exist_ok=True)
TOKEN = re.compile(r'\s*("(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|-?(?:\d+\.\d+(?:[eE][+-]?\d+)?|\d+)|[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*|[{}\[\](),:])')

class DebugParser:
    def __init__(self, text):
        self.tokens = []
        pos = 0
        while pos < len(text):
            match = TOKEN.match(text, pos)
            if not match:
                assert not text[pos:].strip(), text[pos:pos+100]
                break
            self.tokens.append(match.group(1))
            pos = match.end()
        self.pos = 0
    def take(self):
        token = self.tokens[self.pos]
        self.pos += 1
        return token
    def peek(self):
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None
    def expect(self, token):
        assert self.take() == token
    def sequence(self, closing):
        values = []
        while self.peek() != closing:
            values.append(self.value())
            if self.peek() == ',':
                self.take()
            else:
                assert self.peek() == closing
        self.take()
        return values
    def value(self):
        token = self.take()
        if token == '[':
            return {'$list': self.sequence(']')}
        if token == '(':
            return {'$tuple': self.sequence(')')}
        if token == '{':
            pairs = []
            while self.peek() != '}':
                key = self.value()
                self.expect(':')
                pairs.append([key, self.value()])
                if self.peek() == ',':
                    self.take()
                else:
                    assert self.peek() == '}'
            self.take()
            return {'$map': pairs}
        if self.peek() == '{':
            self.take()
            fields = {}
            while self.peek() != '}':
                name = self.take()
                assert name not in fields
                self.expect(':')
                fields[name] = self.value()
                if self.peek() == ',':
                    self.take()
                else:
                    assert self.peek() == '}'
            self.take()
            return {'$type': token, '$fields': fields}
        if self.peek() == '(':
            self.take()
            return {'$type': token, '$args': self.sequence(')')}
        return {'$atom': token}
    def parse(self):
        result = self.value()
        assert self.pos == len(self.tokens), self.tokens[self.pos:self.pos+10]
        return result

def parse(path):
    text = path.read_text()
    if path.name.endswith('.choices.txt'):
        assert text.count('\nreferee=') == 1
        players, referee = text.split('\nreferee=')
        return {'players': DebugParser(players).parse(), 'referee': DebugParser(referee).parse()}
    return DebugParser(text).parse()
def differences(left, right, path='$'):
    if type(left) is not type(right):
        return [{'path': path, 'old': left, 'new': right}]
    if isinstance(left, dict):
        if left.keys() != right.keys():
            return [{'path': path, 'old': left, 'new': right}]
        return [item for key in left for item in differences(left[key], right[key], path + '.' + key)]
    if isinstance(left, list):
        if len(left) != len(right):
            return [{'path': path, 'old': left, 'new': right}]
        return [item for index, (a, b) in enumerate(zip(left, right)) for item in differences(a, b, f'{path}[{index}]')]
    return [] if left == right else [{'path': path, 'old': left, 'new': right}]

assert DebugParser('X { a: Some([1, -2,]), b: {(1, 2): None,}, }').parse()['$type'] == 'X'
assert DebugParser('[]').parse() == {'$list': []}
assert DebugParser('"a\\\"b"').parse() == {'$atom': '"a\\\"b"'}
rows = []
for old in sorted((ROOT / 'golden330').glob('*.snapshot.txt')):
    scenario = old.name.removesuffix('.snapshot.txt')
    before = parse(old)
    after = parse(ROOT / 'golden331' / old.name)
    record = {'scenario': scenario, 'snapshot_diff': differences(before, after)}
    for kind in ['events', 'choices', 'views']:
        a = ROOT / 'golden330' / f'{scenario}.{kind}.txt'
        b = ROOT / 'golden331' / f'{scenario}.{kind}.txt'
        record[kind + '_byte_equal'] = a.read_bytes() == b.read_bytes()
        record[kind + '_diff'] = differences(parse(a), parse(b))
        if kind != 'views':
            assert not record[kind + '_diff'] and record[kind + '_byte_equal'], (scenario, kind)
    record['core_field_names'] = list(before['$fields']['core']['$fields'])
    players_count = len(before['$fields']['core']['$fields']['players']['$list'])
    before_views = parse(ROOT / 'golden330' / f'{scenario}.views.txt')['$list']
    after_views = parse(ROOT / 'golden331' / f'{scenario}.views.txt')['$list']
    assert len(before_views) == len(after_views) == players_count
    record['seat_views_count'] = players_count
    assert len(record['views_diff']) == players_count
    assert all(item['path'].endswith('.$fields.ruleset_version.$atom') and item['old'] == '330' and item['new'] == '331' for item in record['views_diff'])
    version_paths = {'$.$fields.ruleset_version.$atom', '$.$fields.core.$fields.ruleset_version.$atom'}
    for item in record['snapshot_diff']:
        if item['path'] in version_paths:
            assert (item['old'], item['new']) == ('330', '331')
        elif scenario == 'suidun_window_matrix':
            assert item['path'] == '$.$fields.resolution.$fields.budget.$atom' and (item['old'], item['new']) == ('4075', '4076')
        else:
            assert scenario == 'rollback_invariants' and item['path'] == '$.$fields.resolution.$fields.parks.$list'
            old_parks, new_parks = item['old'], item['new']
            assert len(new_parks) == len(old_parks) + 1
            possible = [index for index in range(len(new_parks)) if new_parks[:index] + new_parks[index+1:] == old_parks]
            assert len(possible) == 1
            index = possible[0]
            extra = new_parks[index]
            assert extra['$fields']['machine']['$type'] == 'TriggerOrder'
            record['rollback_new_window'] = {'index': index, 'complete_park': extra, 'all_other_parks_exact': True}
    rng_keys = [key for key in record['core_field_names'] if 'rng' in key.lower() or 'random' in key.lower()]
    record['rng_full_fields'] = {key: before['$fields']['core']['$fields'][key] for key in rng_keys}
    assert all(before['$fields']['core']['$fields'][key] == after['$fields']['core']['$fields'][key] for key in rng_keys)
    rows.append(record)
assert len(rows) == 14
report = {'method': 'Full token consumption recursive Rust Debug AST preserving named type, each struct field, enum/tuple/list shape, ordered map pairs and exact scalar/string tokens; rejects unknown syntax and duplicate fields. No field exclusions.', 'parser_assertions': 3, 'scenarios': rows, 'pairs': 14, 'no_binary_probe_execution_or_golden_update': True}
(OUT / 'object-differences.json').write_text(json.dumps(report, indent=2) + '\n')
print('Parsed14pairs of all four complete Debug objects; differences at', OUT / 'object-differences.json')
