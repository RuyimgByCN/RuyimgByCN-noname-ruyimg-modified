python3 - <<'PY'
from pathlib import Path
import json,hashlib,struct
E=Path('/tmp/i02-main-typed-wire-functional-33d21ba5')
def masked(p):
 raw=p.read_bytes();b=bytearray(raw[:-20]);v,n=struct.unpack('>II',b[4:12]);assert v==2;off=12
 for i in range(n):
  start=off;b[start:start+8]=b'\0'*8;flags=struct.unpack('>H',b[start+60:start+62])[0];off=start+62+(2 if flags&0x4000 else 0);off=b.index(0,off)+1;off=start+((off-start+7)//8)*8
 return bytes(b)
a=masked(E/'before-index.raw');b=masked(E/'gate-entry-index.raw');assert a==b;f=E/'index-raw-field-diff.json';j=json.load(open(f));j['bodyEqualAfterMaskingOnlyEntryCtime']=True;j['maskedBodySha256']=hashlib.sha256(a).hexdigest();j['allOtherBytesIncludingHeaderFlagsPaddingAndExtensionsExact']=True;f.write_text(json.dumps(j,indent=2))
source=json.load(open(E/'integration-receipt.json'))['source30'];bundle=json.load(open(E/'ingress-bundle-sources.json'));critical=['apps/web-client/src/engine/client.ts','apps/web-client/src/engine/desktopEngineTransport.ts','apps/web-client/src/engine/networkClient.ts','apps/web-client/src/engine/presentationMetadata.ts','apps/web-client/src/engine/viewStore.ts'];R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');entry=E/'ingress-dist'/bundle['entry'];entrySha=hashlib.sha256(entry.read_bytes()).hexdigest();sentinels=['native_engine_open','native_engine_request','native_engine_close','public presentation metadata invalid'];assert all(s in entry.read_text() for s in sentinels);loaded={}
for mode in ['chrome','tauri']:
 http=json.load(open(E/f'ingress-{mode}-http-binding.json'));matches=[x for x in http['requests'] if x['path']=='/'+bundle['entry']];assert matches and all(x['sha256']==entrySha for x in matches);loaded[mode]={'root':http['root'],'url':http['url'],'actualEntryRequests':matches,'resultSha256':hashlib.sha256((E/f'ingress-{mode}-report.json').read_bytes()).hexdigest()}
mods={}
for p in critical:
 key=str(R/p);assert bundle['sources'][key]==source[p];mods[p]={'sourceSha256':source[p],'transformedIntoPrivateEntry':True}
(E/'actual-main-js-load-binding.json').write_text(json.dumps({'entry':bundle['entry'],'entrySha256':entrySha,'actualLoadedEntryChunks':loaded,'currentMainModules':mods,'requiredTransportSentinelsPresent':sentinels,'actualNativeIPCRequestsAndPositiveReplies':'Tauri report actualRequests811/positiveAbsent162/getTransportKind desktop-native assertion; actual compiled Main Rust binary identity in tauri-executed-identity.json','injectionBoundary':'Only successful real Tauri callback data is explicitly replaced for bad metadata fixtures; original invoke and native IPC dispatch remain in place. No Rust metadata producer claim.','defaultDist37Served':False,'qualification':'Current Main I02 JS is bundled into a private harness entry and actually fetched by both Chromium and hidden Tauri. The original current Main UI production bundle is separately validated by own-browser-final; no claim that private entry is the unchanged shipped desktop UI.'},indent=2));print('raw bytes ctime-only independently confirmed; actualMainJS entry loaded both runtimes',entrySha)
PY