from pathlib import Path
import json,subprocess,shutil,datetime,os
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path(__file__).resolve().parent;P=Path(json.loads((E/'product-root.json').read_text())['path']);TARGET=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target')
active=json.loads((E/'owned-process-check.json').read_text())['activeOwned'];assert not active,active
profiles=[]
for directory in [P/'chrome-fast',P/'chrome-slow']:
 for profile in directory.glob('chrome-*'):
  assert profile.is_dir() and not profile.is_symlink() and profile.resolve().is_relative_to(P.resolve())
  profiles.append({'path':str(profile),'bytes':sum(f.stat().st_size for f in profile.rglob('*') if f.is_file())});shutil.rmtree(profile)
incDirs=[d for d in TARGET.rglob('incremental') if d.is_dir()];incBytes=sum(f.stat().st_size for d in incDirs for f in d.rglob('*') if f.is_file());assert incBytes==0
assert not (ROOT/'apps/web-client/src/worker/.native-pkg-tmp').exists();assert not (ROOT/'apps/web-client/src/.wasm-pkg-tmp').exists()
cleanups={str(f.relative_to(P)):json.loads(f.read_text()) for f in P.rglob('*cleanup.json') if f.is_file()}
record={'time':datetime.datetime.now().isoformat(),'ownedProcessesStillActive':active,'allOwnedServicesStoppedAndWaited':all(x.get('waited') for x in cleanups.values()),'cleanupRecords':cleanups,'removedOwnedUnusedChromeProfiles':profiles,'mainGeneratedTmpAbsent':True,'incrementalBytesAllTargets':incBytes,'incrementalDirectories':list(map(str,incDirs)),'stage3Retained':True,'stage3Purpose':'Parent current330 acceptance, independent final preflight21 and integration revalidation; retain deps/executables/WASM caches; worker has no further heavy use pending','freeDiskBytes':shutil.disk_usage(ROOT).free,'user7700AndOtherChromeServicesUntouched':True,'mainDefaultTargetWritesDenied':True}
assert record['allOwnedServicesStoppedAndWaited'];(E/'owned-cleanup.json').write_text(json.dumps(record,indent=2)+'\n');print('owned0, allwait true, profiles removed',len(profiles),'incremental0; freeGiB',round(record['freeDiskBytes']/1024**3,2))
