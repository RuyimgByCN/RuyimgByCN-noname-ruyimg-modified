from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os
BASE=Path(__file__).resolve().parent
n=list(map(json.loads,(BASE/'native/09-formal.log').read_text().splitlines()))
b=list(map(json.loads,(BASE/'browser/case-results-twelfth.jsonl').read_text().splitlines()))
assert len(n)==len(b)==24
aliases={'archery':'wanjian','armor':'bagua','bing':'bingliang','chain':'tiesuo','dismantle':'guohe','duel':'juedou','horse_minus':'chitu','horse_plus':'jueying','le':'lebu','nullify':'wuxie','savage':'nanman','shandian_safe':'shandian','snatch':'shunshou','weapon2':'qinggang','weapon2b':'cixiong','weapon3':'qinglong','fire_sha':'sha','leisha':'sha'}
def mapped(name):
 if name in aliases:return aliases[name]
 if name.startswith('wd_'):return name[3:].rsplit('_',1)[0]
 return name
nature_ids={87:'LeiSha',88:'LeiSha',94:'FireSha',95:'FireSha',96:'FireSha',97:'FireSha'}
checks=[]
for a,z in zip(n,b):
 assert a['case']==z['case'] and z['success']
 native_faces={c['id']:c for c in a['initial']['deck_draw_order']}
 for p in a['initial']['players']:
  for zone in ['hand','equips','judge']:
   native_faces.update({c['id']:c for c in p[zone]})
 for c in z['setup']['initial']['deck_draw_order']+[c for p in z['setup']['initial']['players']for zone in ['hand','equips','judge']for c in p[zone]]:
  f=native_faces[c['id']];assert (c['name'],c['suit'],c['rank'])==(mapped(f['name']),f['suit'],f['rank']),(a['case'],c,f)
 moved=set()
 for state in [a['final'],z['result']['final']]:
  start=[c['id']for c in a['initial']['deck_draw_order']];end=[c['id']for c in state['deck_draw_order']]
  moved.update(set(start)^set(end))
  for p in state['players']:
   old=a['initial']['players'][p['seat']]
   for zone in ['hand','equips','judge']:moved.update(set(c['id']for c in old[zone])^set(c['id']for c in p[zone]))
  moved.update(c['id']for zone in ['discard','table']for c in state[zone])
 assert not moved.intersection(nature_ids),(a['case'],moved.intersection(nature_ids))
 checks.append({'case':a['case'],'initial_normalized_name_suit_rank_equal':True,'unmapped_or_foreign_final_cards':False,'elemental_six_moved_in_compared_run':False,'moved_ids':sorted(moved)})
history=[]
for p in sorted((BASE/'browser').glob('case-results-*.jsonl')):
 rows=list(map(json.loads,p.read_text().splitlines()));history.append({'file':os.path.relpath(p,BASE),'rows':len(rows),'unique_cases':len({x['case']for x in rows}),'success_records':sum(bool(x['success'])for x in rows),'failure_records':sum(not x['success']for x in rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
scope={'at':datetime.now(timezone.utc).isoformat(),'head':'605201f3f49e10cea61c31d52a61bbb66c85599d','rules':333,'ngsa':107,'schema':5,'abi':759,'ws':8,'final_cohort':'native09/formal + browser twelfth','unique_cases':24,'ngsa_native_windows':43,'compared':['player seat, hp/max_hp, alive, linked, face_down, hujia','player hand/equip/judge ID multisets; normalized skill keys and fuchou source marks','initial native ID → upstream normalized name/suit/rank metadata (not nature)','deck ID order; discard/table ID multisets; no foreign card append after fixture'],'not_compared':['complete initial state identity across engines','elemental-card nature: leisha/fire_sha mapped to sha without nature; IDs87/88/94–97 untouched','semantic equality of unused deck suffix, generic equipment/mount aliases','initial general/identity/AI values','full cross-engine event/continuation/RNG history','hand and discard physical ordering','independent install/package lifecycle or normal product UI acceptance','unexecuted skill branches'],'new_equivalence_signoff':False,'initial_face_checks':checks,'browser_attempt_history':history,'old_attempt_binary_limit':'native01–04 retained source/stderr, but no per-attempt binary fingerprint was captured; these are setup failures, not final reproducible-version acceptance','final_binary_sha256':hashlib.sha256((BASE.parents[4]/'target/debug/queue1-u07-runtime-probe').read_bytes()).hexdigest()}
with (BASE/'final-scope.json').open('x')as out:out.write(json.dumps(scope,ensure_ascii=False,indent=2)+'\n')
source=json.loads((BASE/'source-map.json').read_text());p=BASE.parents[5]/'archive/upstream/noname/apps/core/extension/玩点论杀/card/list.js'
source['refs'].append({'path':os.path.relpath(p,BASE),'line':25,'meaning':'upstream physical Diaobing cards heart4/diamond4 exist in list; native formal catalog has none','sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
with (BASE/'source-map-final.json').open('x')as out:out.write(json.dumps(source,ensure_ascii=False,indent=2)+'\n')
with (BASE/'comparison-formal.md').open('a')as out:out.write('\n范围补充（2026-11-14）：这里的初态/终态相同仅指所列 ID、映射后的 name/suit/rank、玩家区域及 HP 等投影。未比较整副牌性质；原生 87/88 雷杀、94–97 火杀在浏览器后缀映射中未携带 nature，已核对这6张牌未进入本次已执行路径。不得据此称完整初态或完整牌堆同义。详见 [最终范围与逐案元数据核验](final-scope.json)。\n')
readme='''# U07 三技能真实运行样本（2026-11-14）

最终输入为 native09 与 browser twelfth：24行对应24个唯一case，原生和上游各实际运行24案，43个原生待答窗口均保存原始NGSA、编码/解码恢复，并核对同答后的state_hash和新增日志一致。24案的所选初态投影通过；终态投影12案相同、12案不同。本轮不签新增规则等价。

HEAD为 `605201f3f49e10cea61c31d52a61bbb66c85599d`，dirty；规则333／NGSA107／schema5／ABI759／WS8。执行期间主agent有酒诗恢复定义、L29测试及公共文档/构建产物变动，完整dirty输出保留在输入快照；所捕获478个原生/loader/上游输入文件在最终原生运行前后、浏览器运行前后均无哈希变化。原生编译时第479项probe二进制变化符合预期，浏览器期间该二进制也未变。

[逐case对照表](comparison-formal.md)、[逐字段比较](comparison-formal.json)、[精确原始输入索引](final-input-index.json)是最终入口。索引逐案列出native/browser JSONL行号、原始CDP夹具请求及结算请求/回复行号、源文件和日志哈希，没有将重复尝试静默去重。[最终范围](final-scope.json)保留全部浏览器轮次的成功/失败记录数。

| 技能 | 实际覆盖 | 当前实际差别 |
| --- | --- | --- |
| 筹度 | 目标1/2、手牌/已装备成本、首张/异面展示牌、放回牌堆顶、选成本/选目标取消、无he牌 | 上游询问全体有手牌角色并允许选入展示牌、最后放回牌堆顶；原生只问所选角色换出牌，固定拿展示首牌，余牌全弃。全部后续都取消时可得到相同终态投影，选择空间仍不同。 |
| 复仇/复仇2 | 交手牌/已装备牌、取消交牌、无he牌；phaseAfter初手无杀及初手有备用杀的实际拒绝/使用 | 交牌取消原杀后，上游原杀弃置，原生额外返还原杀。上游回合末需实际出杀或自身失HP；原生自动虚拟杀。无装备的末尾隔离案中，已有备用杀的拒绝/使用都实际出现chooseToUse。 |
| 诱战 | 手牌装备/已装备成本、取消、距离2、无装备、反击杀命中、反击杀被闪后再拒绝 | 支付成本后上游取消原杀；原生仍让原目标额外承受原杀伤害。反击被闪后，上游再次询问出杀，原生没有相应循环。 |

正式包没有调兵遣将实体牌：[无占位目录输出](native/formal-catalog-without-placeholder.json)记录catalog共138张、匹配实体ID为空、未增加任何卡。上游[原始牌表](../../../../../../archive/upstream/noname/apps/core/extension/玩点论杀/card/list.js#L25)确有红桃4/方块4两张。native09的筹度宿主路径能执行，不证明该实体已注册或可真实使用；因为没有合法实体ID，未捏造一次PlayCard尝试。native01–08手工加占位的轮次原样保留，不作正式包产品可用性的证明。

比较范围是玩家HP/maxHP、存活/横置/翻面/护甲、区域牌ID集、规范化技能名及复仇标记，另比较牌堆ID顺序和弃牌/处理中牌多重集；初态还逐ID核对映射后的name/suit/rank。没有比较完整初态、身份/武将/AI值、跨引擎完整事件/RNG/续行，也没有比较整副牌的性质与语义。原生6张性质杀（87/88雷、94–97火）被浏览器映射为sha而未携带nature；已核对6张都未进入本次实际路径。后缀通用装备/坐骑的映射仅为夹具载入；armor→bagua只作为手牌装备成本，没有防具效果背书。

上游共享服务只读，使用独立Chrome157 profile/CDP/control。每案刷新，等8玩家、sha定义及节点就绪，等待官方onfree/cardsAsync完成后重建夹具；逐个移除虚拟装备/判定，再核对实际区域、技能、状态与牌堆ID。记录中24案都无额外无ID牌追加。实际调用原始技能函数和原始选择事件，只包装choose方法切换手动控制座位，决策通过原UI点击。启动可停在选将，因此[末案截图](browser/final-table.png)仍有原选将对话框及旧UI牌数提示；截图不证明正常产品UI已验收，实际区域/牌堆以原始快照为准。

原生命令和源码：[run09命令](native/run-09-command.json)、[probe09](native/probe-09.rs)、[Cargo清单](native/Cargo.toml)、[锁文件](native/Cargo.lock)、[stdout24行](native/09-formal.log)、[stderr](native/09-formal.stderr)、[43份NGSA及SHA-256](native/archive-manifest-formal.json)。浏览器实际命令为 `U07_NATIVE_LOG=09-formal.log U07_RUN_TAG=twelfth U07_CONTINUE=1 python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/U07/browser/run-cases.py`；[24行原始结果](browser/case-results-twelfth.jsonl)、[运行stdout](browser/twelfth-run.log)、[运行stderr](browser/twelfth-run.stderr)。源码落点与哈希见[最终源码索引](source-map-final.json)。

首次失败和更正均保留。早期native有目录名不匹配、丢失牌ID导致NGSA拒绝、空答不等于Pass等夹具错误；早期browser有序列化循环对象、将动画暂停误当选择、未提供模块全局绑定等失败。第一份汇总把onfree追加尾牌算进牌堆差别，并按case名字误称执行出杀选择；第二份汇总的cancel后缀误取消初次交牌，且末尾装备造成额外效果。两份中间表均不是最终依据，修正见[启动/调度更正](correction-07-initial-review.json)及[分支/目录更正](correction-08-final-fixture.json)。native01–04没有逐次二进制指纹，不作最终版本重放证明；最终native09与browser twelfth有前后源码与二进制指纹。

未跑产品全量门禁、本轮未改产品/清单/公共文档/Git index/refs。未覆盖多复仇标记、来源死亡、筹度同回合二次发动/其他cardEnabled条件、诱战全部距离和其他转化杀等分支。全局门禁及发布由主agent负责；本目录仍属未提交证据。

已停止自己的Chrome并清理约543MB profile及24KB外部probe crate；未创建私有Cargo target、未清共享target，Vite/FS和U06会话未动，详见[清理记录](cleanup.json)。原始最终输入已发独立复核agent，复核结果由其独立记录。
'''
with (BASE/'README.md').open('x')as out:out.write(readme)
print(json.dumps({'cases':24,'ngsa':43,'initial_faces_checked':24,'elemental_six_untouched_all_cases':True,'history_files':len(history)},ensure_ascii=False))
