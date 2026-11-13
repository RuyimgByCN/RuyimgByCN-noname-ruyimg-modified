from pathlib import Path
import hashlib,importlib.util,json,re
BASE=Path(__file__).resolve().parent
ROOT=BASE.parents[4]
spec=importlib.util.spec_from_file_location('links',ROOT/'scripts/check_local_links.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
mds=list(BASE.rglob('*.md'));links=0;anchors=0
for p in mds:
 bad=mod.check(p);assert not bad,(p,bad)
 for href in mod.LINK.findall(p.read_text()):
  links+=1
  m=re.match(r'^([^#]+)#L(\d+)$',href)
  if m:
   file=(p.parent/m[1]).resolve();line=int(m[2]);assert file.exists()and 0<line<=len(file.read_text().splitlines()),(p,href);anchors+=1
sources=json.loads((BASE/'source-map-final.json').read_text())['refs']
for r in sources:
 p=(BASE/r['path']).resolve();assert p.exists()and 0<r['line']<=len(p.read_text().splitlines());assert hashlib.sha256(p.read_bytes()).hexdigest()==r['sha256']
idx=json.loads((BASE/'final-input-index.json').read_text())
for rel,want in idx['sha256'].items():assert hashlib.sha256((BASE/rel).read_bytes()).hexdigest()==want,rel
arc=json.loads((BASE/'native/archive-manifest-formal.json').read_text())
assert arc['count']==43==len(arc['archives'])
for r in arc['archives']:
 p=BASE/r['path'];assert p.stat().st_size==r['bytes']and hashlib.sha256(p.read_bytes()).hexdigest()==r['sha256']
for rel in ['native/09-formal.log','browser/case-results-twelfth.jsonl']:
 records=list(map(json.loads,(BASE/rel).read_text().splitlines()));assert len(records)==len({r['case']for r in records})==24
print(f'本目录Markdown {len(mds)} 份，链接 {links} 个，坏链 0，#L越界 0（检查 {anchors} 个锚点）。')
print(f'源码引用 {len(sources)} 条均存在、行号有效且哈希未变。最终索引 {len(idx["sha256"])} 个文件哈希匹配。')
print('24 native + 24 browser raw行均唯一；43份NGSA字节数和SHA256匹配。未运行完整preflight/产品门禁。')
