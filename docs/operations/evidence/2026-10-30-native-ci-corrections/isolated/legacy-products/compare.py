import pathlib,subprocess,json,fcntl,time,hashlib,collections
p=pathlib.Path(__file__).resolve().parent
# Wait for both after products; each has a completed build before it is consumed.
while not (p/'harness-after-build.exit').exists():
    if (p/'validate-after-driver.log').read_text().strip(): raise RuntimeError((p/'validate-after-driver.log').read_text())
    time.sleep(1)
assert (p/'harness-after-build.exit').read_text().strip()=='0'
results=[]
with pathlib.Path('/tmp/noname-phase4-heavy-command.lock').open('a') as lock:
    fcntl.flock(lock,fcntl.LOCK_EX)
    for pair in range(7):
        for variant in (['before','after'] if pair%2==0 else ['after','before']):
            for scenario,args in [('tier',['tier-report','server-baseline']),('start',['start-hash']),('cost',[])]:
                binary=p/(variant+('-cost-harness' if scenario=='cost' else '-benchmark-runner'))
                output=p/f'pair-{pair}-{variant}-{scenario}.log'
                with output.open('w') as f:r=subprocess.run([str(binary)]+args,stdout=f,stderr=subprocess.STDOUT)
                results.append({'pair':pair,'variant':variant,'scenario':scenario,'argv':[str(binary)]+args,'exit':r.returncode,'log':output.name,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()})
(p/'comparison-results.json').write_text(json.dumps(results,indent=2))
summary={}
for variant in ['before','after']:
    phases=collections.defaultdict(list)
    for pair in range(7):
        for line in (p/f'pair-{pair}-{variant}-cost.log').read_text().splitlines():
            row=json.loads(line); phases[row['phase']].append(row)
    summary[variant]={}
    for phase,rows in phases.items():
        summary[variant][phase]={'samples':len(rows)}
        for field in ['ns','allocs','bytes']:
            values=sorted(r[field] for r in rows)
            summary[variant][phase][field]={f'p{q}':values[min(int(len(values)*q/100),len(values)-1)] for q in [50,95,99]}
(p/'comparison-cost-summary.json').write_text(json.dumps(summary,indent=2))
print(json.dumps(summary,indent=2))
