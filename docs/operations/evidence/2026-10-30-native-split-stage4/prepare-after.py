import pathlib,shutil,json,subprocess,os
base=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage4-9rhcfkw2')
E=pathlib.Path('docs/operations/evidence/2026-10-30-native-split-stage4')
assert not (base/'after').exists()
# Only remove this run's initial check cache, before the measured cold test build.
initial=base/'target-after'
size=subprocess.check_output(['du','-sk',str(initial)],text=True)
(E/'initial-check-cache-reclaimed.json').write_text(json.dumps({'path':str(initial),'du_before':size,'reason':'Fresh empty target required for after cold measurement; private initial cargo check artifacts only.'},indent=2)+'\n')
shutil.rmtree(initial)
shutil.copytree(pathlib.Path.cwd(),base/'after',symlinks=True,ignore=shutil.ignore_patterns('.git','target','node_modules','.scratch','.codegraph','native-pkg','pkg'))
# Upstream source snapshots live in a sibling archive, already present for before.
assert (base/'archive').exists()
