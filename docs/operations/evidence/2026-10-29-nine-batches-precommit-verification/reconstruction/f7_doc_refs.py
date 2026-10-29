from rebuild_common import *
rx=re.compile(r'(?:[A-Za-z0-9_./-]+/)?intrinsic\.rs:\d+(?:[/–-]\d+)*')
changed=[]
for row in json.loads((SNAP/'snapshot-manifest.json').read_text()):
 p=Path(row['path']);f=SNAP/p
 if p.suffix!='.md' or not p.exists() or not f.exists():continue
 s=p.read_text();a=rx.findall(s);b=rx.findall(f.read_text())
 if a and a!=b:
  assert len(a)==len(b),(str(p),a,b)
  replacements=iter(b);new=rx.sub(lambda _:next(replacements),s);p.write_text(new);changed.append(str(p))
print(json.dumps(changed,indent=2))
