from pathlib import Path
import importlib.util,json,hashlib
p=Path('docs/operations/evidence/2026-10-29-nc08-core-pack-statistics/scan_core.py').resolve()
spec=importlib.util.spec_from_file_location('reconstructed_core_statistics',p);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
module.HERE=Path(__file__).resolve().parent/'06'
module.main()
a=json.loads((module.HERE/'hits-core.json').read_text());b=json.loads((p.parent/'hits-core.json').read_text());assert a==b,'reconstructed statistics differ from frozen evidence'
print('RECONSTRUCTED_STATISTICS_MATCH_FROZEN_JSON', a['report']['totals'])
