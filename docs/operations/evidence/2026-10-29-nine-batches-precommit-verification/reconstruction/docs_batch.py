from rebuild_common import *
import sys
n=int(sys.argv[1])
ledger='docs/architecture/native-remaining-ledger.md';s=Path(ledger).read_text();f=(SNAP/ledger).read_text()
starts={2:['| NC04 VM'],3:['| NC02 安装','| NC05 可独立','| NC09 原生'],4:['| NC07 状态'],5:['| PA12 单一'],6:['| NC08 规则']}.get(n,[])
for start in starts:
 old=next(l for l in s.splitlines() if l.startswith(start));new=next(l for l in f.splitlines() if l.startswith(start));s=s.replace(old,new)
marker={2:'**NC04 `cards_given` 单帧修正 v326',3:'**GT-014 导入期深层校验（2026-10-29',4:'**NC07 字段批 v327',5:'**ADR 0786 J1 提交期选择窗挂起 v328',6:'**NC08 核心武将包统计与 filter'}.get(n)
if marker:
 para=next(l for l in f.splitlines() if l.startswith(marker));s=s.rstrip()+'\n\n'+para+'\n'
Path(ledger).write_text(s)
if n in [2,4,5]:
 p='docs/architecture/native-rules-baseline.md';s=Path(p).read_text();f=(SNAP/p).read_text();v={2:326,4:327,5:328}[n];prefix={2:'> **NC04（2026-10-29）',4:'> **NC07（2026-10-29）',5:'> **ADR 0786 J1（2026-10-29）'}[n];para=next(l for l in f.splitlines() if l.startswith(prefix));key='> `STANDARD_RULESET_VERSION';i=s.index(key);s=s[:i]+para+'\n'+s[i:];s=re.sub(r'> `STANDARD_RULESET_VERSION = \d+`[^\n]*',f'> `STANDARD_RULESET_VERSION = {v}`（本次按冻结最终快照正向重建的中间态；不是历史源码原件）、',s);s=re.sub(r'> ## ⚠ \*\*当前版本[^\n]*',f'> ## ⚠ **重建中间态：规则 v{v}（本次重建验证，非历史原件）**',s);Path(p).write_text(s)
