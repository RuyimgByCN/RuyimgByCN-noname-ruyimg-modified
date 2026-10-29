from rebuild_common import *
for row in json.loads((SNAP/'snapshot-manifest.json').read_text()):
 p=row['path']
 if p.startswith(('apps/web-client/','crates/engine-native-wasm/')):copy(p)
evidence(3)
