from pathlib import Path
import datetime,json,hashlib
base=Path('docs/operations/evidence/2026-11-13-queue1-review'); ci=base/'publication/code-exact-ci-first'
d=json.loads((ci/'terminal-summary.json').read_text()); sha='1fa8ac6cc3a70446436d97ff49aa19ea4626e22d'; url='https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43468568765'
assert d['head_sha']==sha and d['job_success_count']==10 and d['conclusion']=='success' and d['run_attempt']==1
assert hashlib.sha256(Path('docs/migration/g0/inventory_status.json').read_bytes()).hexdigest()=='2658808bbe361ab248e53166157e25606c47155eb31fa19947cb601eb3f35c89'
stamp=datetime.datetime.now().astimezone().isoformat(timespec='seconds')
retry=[(j['name'],j['existing_tier_report_attempt_exits']) for j in d['jobs'] if j['existing_tier_report_attempt_exits']]
versions=sorted(set(v for j in d['jobs'] for v in j['setup_rustc_versions']))
assert versions == ['1.100.0'], versions
lines=[f'\n\n## {stamp} 同一精确代码CI终态','',f'代码 `{sha}` 已发布；[run43468568765]({url}) attempt1全部10作业success／0 failure／0 cancelled。没有由agent重启或补跑。记录时HEAD为该代码加文档/证据dirty，版本333／107／5／759／WS8；清单634／3790／0、新增等价0、八项0/37。','', '| 作业 | 结果 | Rust通过/失败/忽略/过滤 | 实际日志中的工具链 |','| --- | --- | --- | --- |']
for j in d['jobs']:
 c=j['rust_test_counts']; counts=f"{c['passed']}/{c['failed']}/{c['ignored']}/{c['filtered']}" if c else '无Rust测试计数（构建/预算/打包范围）'
 lines.append(f"| {j['name']} | {j['conclusion']} | {counts} | {', '.join(j['setup_rustc_versions']) or '见原始日志'} |")
lines.extend(['', '各作业计数有重叠，不能相加为唯一测试数。build三平台矩阵为workspace `--lib`，完整集成门禁在Linux；native-web中四项新增聚贤回归通过。CI实际stable为'+','.join(versions)+'，本地严格检查1.99.1、WASM1.95.0另列[工具链](../../toolchain-observation.json)，不称同一工具链。', '', '性能步骤原有最多3次内部尝试，按原工作流执行；所有实际尝试保留：`'+json.dumps(retry,ensure_ascii=False)+'`。这与workflow run_attempt或agent补跑分开记录，不把内部首失败隐藏。', '', '[终态原始run](run-final.json)、[全部作业/步骤](jobs-final.json)、[安装器artifact元数据](artifacts-final.json)、[全部日志ZIP](logs-first.zip)、[逐作业解析/命令/指纹](terminal-summary.json)。三个安装器上传只证明构建和上传，不代表本机安装、签名/公证、真机或UI验收。', '', 'CLI首次读取单作业日志因控制转义输出保护而exit1，保留[job-log-download-first-failure.json](job-log-download-first-failure.json)；加`--allow-escape-sequences`后仅将原始字节写文件，不向终端执行。这不是CI作业失败或重新运行。', '', '队列1仍部分，实际5技能13分支24次仅为指定投影；剩余10来源30候选、L22自动规则及其他队列不据绿色CI关闭。代码CI结果与发布证据将形成纯文档后续提交；当前记录时该文档提交尚未发生。'])
p=ci/'README.md'; old=p.read_text(); assert '同一精确代码CI终态' not in old; p.write_text(old+'\n'.join(lines)+'\n')
common=f'代码 `{sha}` 已按裁决25正常发布；[精确SHA CI]({url}) attempt1全部10作业success／0失败／0取消，未由agent补跑。记录时HEAD为该代码加文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。'
entries={
'docs/architecture/native-progress.md':f'''**队列1部分增量发布及精确代码CI通过（{stamp}）**：{common} 本地完整三crate2446通过／0失败／6忽略、workspace7842／0／8、前端489／0／0及第五节其余命令exit0，完整preflight复跑21项通过；[命令/首次失败/忽略边界](../operations/evidence/2026-11-13-queue1-review/final-local-gates.md)。CI实际stable1.100.0、与本机1.99.1严格检查及1.95.0 WASM分开；[逐作业计数、原有性能内部尝试与原始日志](../operations/evidence/2026-11-13-queue1-review/publication/code-exact-ci-first/README.md)。清单[634／3790／0](../operations/evidence/2026-11-13-queue1-review/source-counts-reviewed.md)、本批撤回215旧等价、新增等价0、八项0/37。上游[5技能13分支24次指定投影](../operations/evidence/2026-11-13-queue1-review/runtime-comparison.md)相同，非全轨迹；27次本轮挂起恢复。队列0完成、队列1仍部分，其他10来源30候选与L22自动规则、已知差异后续修复待做，不进入队列2。UI截图／Spine逐特性与净室及用户验证／真机签名／第六阶段保持待；[剩余范围](../operations/evidence/2026-11-13-queue1-review/remaining-scope.md)。''',
'docs/architecture/native-remaining-ledger.md':f'''## {stamp} 队列1部分增量已发布、代码CI通过

{common} 完整本地必跑命令通过，首失败/忽略项保留[门禁表](../operations/evidence/2026-11-13-queue1-review/final-local-gates.md)；[CI原始及逐作业范围](../operations/evidence/2026-11-13-queue1-review/publication/code-exact-ci-first/README.md)。634／3790／0、新增等价0、八项0/37；队列1尚余10来源30交互候选、L22自动规则及已登记差异后续修复，不开始队列2。全自动/固定选择清单未完成37来源穷举、尚无证明必须升号项；UI、Spine、设备/签名、第六阶段仍按[剩余范围](../operations/evidence/2026-11-13-queue1-review/remaining-scope.md)。''',
'docs/architecture/native-rules-baseline.md':f'''## {stamp} 聚贤部分修复发布与代码CI

{common} [完整原始门禁与CI](../operations/evidence/2026-11-13-queue1-review/publication/code-exact-ci-first/README.md)。14标准golden及一份旧NGSA107核验不变；聚贤J01–J05仍保留，整技能为简化近似。634／3790／0、新增等价0；没有因CI或5技能指定投影相同签整源规则等价。''',
'docs/migration/g0/README.md':f'''## {stamp} 队列1部分增量发布与CI

{common} [本地及精确代码CI记录](../../operations/evidence/2026-11-13-queue1-review/publication/code-exact-ci-first/README.md)。[37来源清单](../../operations/evidence/2026-11-13-queue1-review/source-counts-reviewed.md)634／3790／0、本批撤回215、新增等价0；八项0/37、队列1部分。旧状态沿用不作本轮重新认证，保留下方全部历史表。''',
'docs/migration/upstream-extension-reconstruction.md':f'''## {stamp} 队列1部分增量发布及精确代码CI

{common} [逐作业与本地验证](../operations/evidence/2026-11-13-queue1-review/publication/code-exact-ci-first/README.md)。第①项按[634／3790／0](../operations/evidence/2026-11-13-queue1-review/source-counts-reviewed.md)，新增等价0；第⑧项继续保留所有已知差异。八项整包0/37，界面/Spine/平台等不据CI上调；队列1部分、队列2及以后未开始。''',
'docs/operations/evidence/2026-10-19-source-signoff-32/README.md':f'''## {stamp} 队列1部分增量发布及CI核验

{common} [完整门禁/作业/原始日志](../2026-11-13-queue1-review/publication/code-exact-ci-first/README.md)。第①项按[634／3790／0](../2026-11-13-queue1-review/source-counts-reviewed.md)、新增等价0；第⑧项保留聚贤及全部已知差异，八项整包仍0/37。队列1部分，平台/组合/性能及规则签署不因测试绿色关闭。'''
}
for name,section in entries.items():
 p=Path(name); old=p.read_text(); assert stamp not in old
 if p.name=='native-progress.md':
  marker='## 当前交付与带日期核验记录\n\n'; assert old.count(marker)==1; new=old.replace(marker,marker+section+'\n\n',1)
 else:
  first,rest=old.split('\n',1); new=first+'\n\n'+section+'\n'+rest
 p.write_text(new)
for name,title,text in [
 ('README.md','精确代码CI通过',common+' [全部终态及日志](publication/code-exact-ci-first/README.md)。本批发布与验证事实已确定；队列1仍部分，不上调0/37或634／3790／0，不开始队列2。后续纯文档提交只补发布记录，不更改产品输入。'),
 ('final-local-gates.md','发布与精确代码CI补充',common+' 见[逐作业计数/工具链/内部尝试/原始日志](publication/code-exact-ci-first/README.md)。本地门禁表仍为提交前的原日期记录；额外全暂存空白检查exit2及保留原证据的范围不改写为通过。'),
 ('remaining-scope.md','本批发布与剩余范围补充',common+' [完整CI证据](publication/code-exact-ci-first/README.md)。本批部分增量已经提交并发布，原文“尚未提交/发布”保留当时日期；队列1仍未完成、队列2未开始，634／3790／0、新增等价0、八项0/37及上述全部未验门槛不变。')]:
 p=base/name; old=p.read_text(); p.write_text(old+f'\n\n## {stamp} {title}\n\n{text}\n')
print('recorded exact code CI terminal status and additive public records at',stamp)
