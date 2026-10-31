import sys,json,os
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from verify import run,O,R,ENV
commands=[
('native-wasm-build',['node','scripts/build-native.mjs'],R/'apps/web-client',{'NODE_OPTIONS':'--require='+str(O/'target-preload.cjs')}),
('worker-smoke',['pnpm','smoke:native'],R/'apps/web-client',{}),
('frontend-type',['pnpm','typecheck'],R/'apps/web-client',{}),
('frontend-tests',['pnpm','test'],R/'apps/web-client',{}),
('vite-bundle',['node',str(O/'vite-build.mjs')],R/'apps/web-client',{}),
('own-controls',['cargo','test','--offline','-p','engine-native','--test','nc_rounds','nc10_public_own_plan','--','--nocapture','--test-threads=1'],R,{}),
('semantic-seed',['cargo','test','--offline','-p','engine-native','--test','misc','policy_v2_drives_all_seats_without_answer_routing_panic','--','--nocapture','--test-threads=1'],R,{}),
('shan-control',['cargo','test','--offline','-p','engine-native','--test','misc','plan_enumerates_single_choice_fallbacks','--','--nocapture','--test-threads=1'],R,{}),
('native-probe',['cargo','run','--offline','--manifest-path',str(O/'probe/Cargo.toml'),'--',str(O)],R,{}),
('wasm-current',['node','--import',str(R/'apps/web-client/scripts/ts-register.mjs'),str(O/'wasm-current.mjs'),str(O),str(R)],R,{}),
('canonical-snapshots',['cargo','run','--offline','--manifest-path',str(O/'probe/Cargo.toml'),'--',str(O),'compare'],R,{}),
('browser',['node',str(O/'browser.mjs'),str(O/'frontend-dist'),'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',str(O)],R,{}),
('formal-trajectories',['python3','-B','scripts/check-native-wasm-cross-target.py'],R,{}),
('conformance',['cargo','run','--offline','-p','conformance'],R,{}),
('legacy-crosshash',['python3','-B',str(O/'legacy-crosshash.py')],R,{})]
for name,argv,cwd,extra in commands:
 if run(name,argv,cwd,extra):sys.exit(1)
print('ALL FUNCTIONAL PRODUCT COMMANDS PASS',flush=True)
