from pathlib import Path
import subprocess,sys
p=Path('crates/engine-native-content/src/content/standard.rs')
original=p.read_bytes()
try:
 p.write_bytes(original.replace(b'STANDARD_RULESET_VERSION: u64 = 330',b'STANDARD_RULESET_VERSION: u64 = 329'))
 result=subprocess.run(['cargo','test','--offline','-p','engine-native','--test','kernel','replay_scenarios::replay_scenarios_match_golden','--','--exact','--nocapture'])
finally:
 p.write_bytes(original)
sys.exit(result.returncode)
