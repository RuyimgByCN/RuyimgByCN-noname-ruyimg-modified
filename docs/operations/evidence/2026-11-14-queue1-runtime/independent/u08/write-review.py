#!/usr/bin/env python3
"""Write this review from independently derived records and actual source anchors."""
import hashlib
import json
import os
from pathlib import Path
import re

BASE = Path(__file__).resolve().parent
REPO = BASE.parents[5]
ORIGINAL = BASE.parents[1] / 'U08'
UPSTREAM = REPO.parent / 'archive/upstream/noname'
cases = json.loads((BASE / 'case-recalculation-second.json').read_text())
provenance = {r['case']: r for r in json.loads((BASE / 'cdp-provenance-second.json').read_text())['cases']}
candidates = {r['case']: r for r in json.loads((BASE / 'candidate-recalculation-second.json').read_text())}
keys = json.loads((BASE / 'inventory-keys-at-start.json').read_text())
key_by_id = {v['id']: k for k, v in keys.items()}
anchors = {}


def anchor(key, path, pattern, label, occurrence=0):
    path = Path(path)
    lines = path.read_text().splitlines()
    found = [i + 1 for i, line in enumerate(lines) if re.search(pattern, line)]
    line = found[occurrence]
    anchors[key] = {'path': str(path), 'line': line, 'source_text': lines[line - 1], 'label': label,
                    'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def link(path, label, line=None):
    relative = os.path.relpath(path, BASE)
    return f'[{label}]({relative}' + (f'#L{line}' if line else '') + ')'


def source(key):
    a = anchors[key]
    return link(a['path'], a['label'], a['line'])


def output(name, data, json_data=False):
    with (BASE / name).open('x') as stream:
        if json_data:
            json.dump(data, stream, ensure_ascii=False, indent=2)
            stream.write('\n')
        else:
            stream.write(data)


us = UPSTREAM / 'apps/core/extension/英雄杀/character/skill.js'
na = REPO / 'crates/engine-native-content/src/content/host/step/u08_batch_a_asks.rs'
naa = REPO / 'crates/engine-native-content/src/content/host/step/u08_batch_a.rs'
nd = REPO / 'crates/engine-native-content/src/content/host/step/u08_batch_d.rs'
nda = REPO / 'crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs'
nde = REPO / 'crates/engine-native-content/src/content/host/step/u08_batch_d_ext.rs'
anchor('uy', us, r'^\s*yixin: \{', '上游医心')
anchor('uycost', us, r'return player.countCards\("he"\) > 2;', '医心费用门槛')
anchor('uyrecover', us, r'player.awakenSkill\(event.name\)', '医心限定标记与回复')
anchor('ub', us, r'^\s*bolehuiyan: \{', '上游慧眼')
anchor('ubcontrol', us, r'if \(event.target.isUnderControl\(\)\)', '慧眼受控目标过滤')
anchor('ubcancel', us, r'if \(result.control !== "cancel"\)', '慧眼取消判断')
anchor('ub2', us, r'^\s*bolehuiyan2: \{', '上游慧眼未命中结算')
anchor('ub3', us, r'^\s*bolehuiyan3: \{', '上游慧眼命中结算')
anchor('ug', us, r'^\s*guifu: \{', '上游鬼斧')
anchor('ny', na, r'^\s*"yixin" => match m.stage', '原生医心步骤')
anchor('nyrefund', na, r'refund_once_used\(ctx, owner, "yixin"\)', '医心取消返还次数')
anchor('nytarget', na, r'ASK_YIXIN_TARGET =>', '医心目标候选')
anchor('nygate', naa, r'^\s*"yixin" => \{', '医心激活条件')
anchor('nbmeta', nd, r'"bolehuiyan" => Some\(meta', '慧眼触发元数据')
anchor('nbfilter', nd, r'"bolehuiyan" => match event', '慧眼过滤')
anchor('nbdamage', nd, r'let pred = flag\(ctx.state, owner, "bolehuiyan_pred"\)', '慧眼普通 Damage 过滤')
anchor('nbresolve', nda, r'pub\(super\) fn resolve_bolehuiyan_outcome', '慧眼结算')
anchor('nbstep', nda, r'"bolehuiyan" => match m.stage', '慧眼取消与次数记录')
anchor('nbdiscard', nda, r'ASK_BOLEHUIYAN_DISCARD =>', '慧眼强制弃牌候选')
anchor('ng', nde, r'"guifu" => match m.stage', '鬼斧步骤')
anchor('ngdraw', nde, r'^fn guifu_draw_both', '鬼斧摸牌座次')
anchor('ngask', nda, r'ASK_GUIFU_EQUIP =>', '鬼斧多装备必选询问')
anchor('gate', REPO / 'crates/engine-native/src/engine/apply_command.rs', r'GameAction::ActivateSkill \{ skill \} =>', '统一回合与出牌阶段门控')
anchor('formal', REPO / 'crates/content-native-loader/src/decision34_pack9.rs', r'\("upstream-noname", "hero-kill"\) =>', '正式英雄杀注册')
anchor('locked', REPO / 'crates/content-native-loader/src/locked_content.rs', r'attach_formal_package_upstream_rules', '锁定内容正式装载调用')
anchor('control', UPSTREAM / 'apps/core/noname/ui/click/index.js', r'else if \(this.link == "cancel"\)', '真实 control 点击分发')
anchor('cancel', UPSTREAM / 'apps/core/noname/ui/click/index.js', r'^\s*cancel\(node\)', '真实取消返回值')
anchor('under', UPSTREAM / 'apps/core/noname/library/element/player.js', r'^\s*isUnderControl\(self, me\)', '受控关系计算')
anchor('shaemit', REPO / 'crates/engine-native-kernel/src/kernel/sha.rs', r'^\s*fn emit_sha_damage', '原生杀发射 Damage 后推进')
anchor('shaadvance', REPO / 'crates/engine-native-kernel/src/kernel/sha.rs', r'^\s*fn advance_target', '原生杀移入弃牌堆')
anchor('ushadamage', UPSTREAM / 'apps/core/card/standard.js', r'await event.trigger\("shaDamage"\)', '上游杀等待 shaDamage 子事件')
anchor('ushend', UPSTREAM / 'apps/core/noname/library/element/gameEvent.ts', r'await trigger\("End", 3\)', '上游通用事件 End 窗口')
anchor('probeformal', ORIGINAL / 'native-probe/src/main.rs', r'attach_formal_package_upstream_rules', '采集 probe 正式包路径')
anchor('probesetup', ORIGINAL / 'native-probe/src/main.rs', r'^fn setup\(', '原生实际夹具')
anchor('setupreset', ORIGINAL / 'browser-round4-upstream-setup.js', r'p.removeVirtualEquip', '上游实际虚拟区清理')
anchor('setupcontrol', ORIGINAL / 'browser-round4-upstream-setup.js', r"if \(name === 'b_under_control'\)", '受控条件实际设置')
anchor('setupentry', ORIGINAL / 'browser-round4-upstream-setup.js', r'root.setContent', '上游真实 useCard/chooseToUse 入口')
anchor('runnercontrol', ORIGINAL / 'browser-round4-browser-runner.py', r'^def control', '真实 DOM 控件点击')
anchor('runnertargetcancel', ORIGINAL / 'browser-round4-browser-runner.py', r'^def cancel_visible', '取消与可见结束按钮分支')
output('source-anchors.json', anchors, True)

notes = {
    'y_target_A': '实际选弃牌9/42、目标1；损失2体力→回复2、摸106/49，目标HP=4。费用候选与受伤目标候选相同。',
    'y_target_B': '实际选弃牌9/42、目标2；其6上限/1体力→回复4至5，不摸牌；不是回复满至6。',
    'y_cost_B': '改选弃牌42/46，剩手牌9；目标1仍回复2并摸106/49，费用A/B产生不同剩手牌。',
    'y_self': '目标实际为自己；HP 2→4，弃9/42后摸106/49；目标候选含自己。',
    'y_cancel_cards': '真实取消选技能后退回，重新打开再次取消；原生两次空答均接受、未弃牌/回复/摸牌/消耗限定标记。',
    'y_cancel_target': '上游选满两张牌后没有取消按钮，实际点“结束回合”并结束phaseUse；原生空目标答后可在原Play阶段重开。费用和限定标记均未消耗，但取消动作与阶段端点不同。',
    'y_two_cards': '恰有2张he；上游候选不含医心，原生ActivateSkill拒绝；没有付费或结算。',
    'y_no_wound': '无人受伤；上游仍可点技能但无合法目标/确认键，取消退回；原生ActivateSkill直接拒绝。均未结算，不把候选UI差别写成规则效果差异。',
    'y_reuse': '首次选9/42与目标1成功；第二次上游候选不含医心、原生命令拒绝。本案只测同一阶段再次发动，未跑下一回合。',
    'y_no_skill': '未授医心；上游候选不含，原生NoSuchSkill；牌区和HP未变。',
    'g_target_A': '实际目标1，弃其青釭114；依次给座0无中106、座1桃46，另一座2连弩118保留。',
    'g_target_B': '实际目标2，弃其连弩118；座0摸106、座2摸46，座1青釭114保留。目标A/B确有不同装备及获牌者。',
    'g_cancel_target': '真实取消后重开又取消；原生空目标答两次接受并返还次数；双方未弃装备、未摸牌。',
    'g_no_equip': '全桌无装备；上游可点技能但无合法目标，原生直接拒绝；双方未弃牌/摸牌。',
    'g_only_own': '仅自己有青釭114；上游无合法目标、原生拒绝；自己装备保留，不可对自己发动。',
    'g_reuse': '首次弃目标1青釭并各摸1；第二次上游候选不含鬼斧、原生SkillAlreadyUsed。',
    'g_no_skill': '未授鬼斧；上游候选不含，原生NoSuchSkill；他人装备未动。',
    'b_hit': '真实点击“能命中”/原生Tag1；目标未出闪，HP 4→3，慧眼主摸106并消耗本回合次数。',
    'b_miss': '真实点击“不能命中”/Tag2，目标真实使用闪42；目标HP=4，慧眼主摸106。原生RespondCard、上游chooseToUse分列。',
    'b_wrong_A': '预测能命中后目标真实使用闪42；失败强制弃主手牌4，剩46，不额外摸牌；实际失败弃牌窗口已触发。',
    'b_wrong_B': '同一预测失败分支改弃46，剩4；弃牌A/B实际选择不同实体。',
    'b_wrong_miss': '预测不能命中、目标未出闪；HP 4→3，慧眼主强制弃4、剩46；没有预测失败摸牌。',
    'b_cancel_predict': '真实DOM点击cancel返回{confirm:cancel,bool:false}，没有control。上游因此加bolehuiyan4，原生空答不设used_turn；HP和牌区相同仍不能签取消语义相同。',
    'b_cancel_discard': '预测不能命中后实际受伤，停于强制弃牌；上游无取消按钮、未尝试隐藏空答；原生空答被拒且仍pending。杀9此时上游ordering、原生discard。该案未完成结算。',
    'b_empty_target': '目标初始无手牌，双方均无预测；杀使其HP 4→3。上游仍打开无闪chooseToUse并被真实取消，原生直接结算。',
    'b_owner_attacker': '慧眼主自己作为来源出杀，双方均不预测；目标1 HP 4→3。',
    'b_owner_target': '慧眼主自己成为目标，双方均不预测；其HP 4→3。',
    'b_under_control': '上游目标2._trueMe=座0且isUnderControl=true，未出现预测；原生缺该条件仍问Tag1并额外摸106。控制关系不在双方共同初态投影内。',
    'b_repeat': '同回合两张普通杀只实际预测一次，目标HP=2、慧眼主只摸106一次。上游出杀ID顺序10→9、原生9→10，不能称完整轨迹同一。',
    'b_cancel_then_retry': '第一次真实cancel已使上游加bolehuiyan4；第二杀不上预测、未摸106。原生第一次空答后第二杀再次Tag1并摸106。出杀实体先后同样不同，取消后的次数差异在第二杀前已直接可见。',
    'b_no_skill': '未授慧眼；双方均无预测，目标HP 4→3，慧眼主未摸106。',
}
assessments = []
table = ['# U08 逐案独立重算（2026-11-14）', '',
         'HEAD `5c73d1d9158145e33f98f2018013914667d13964`；产品受跟踪源码 clean，工作区含并行未提交证据。规则333 / NGSA107 / schema5 / ABI759 / WS8。每行仅评价指定投影；初始弃牌缺21实体、手牌顺序及控制上下文的限制见 [范围](README.md)。新增等价签署0。', '',
         '| case | 实际选择、条件与结果 | 挂起存档（直接/恢复） | 原始输入 |',
         '| --- | --- | --- | --- |']
for row in cases:
    name = row['case']
    skill = 'yixin' if name.startswith('y_') else 'guifu' if name.startswith('g_') else 'bolehuiyan'
    category = '指定终态投影与用次标记相同；不代表完整规则等价'
    if name in ('b_cancel_predict', 'b_cancel_then_retry'):
        category = '双端实际取消行为差异'
    elif name == 'b_under_control':
        category = '双端实际受控条件差异；控制关系不属于共同初态投影'
    elif name == 'b_cancel_discard':
        category = '双端均待答；强制待答时使用牌所在区域不同'
    elif name == 'y_cancel_target':
        category = '指定牌区/HP/标记相同；实际取消动作与阶段端点不同'
    rec = {'case': name, 'inventory_key': key_by_id[skill], 'assessment': category,
           'observed_behavior': notes[name], 'native_raw': str(ORIGINAL / f'native-round2/native-{name}.json'),
           'upstream_raw': str(ORIGINAL / f'browser-round4/upstream-{name}.json'),
           'cdp_assignment_line': provenance[name]['assignment_line'], 'cdp_final_snapshot_line': provenance[name]['final_snapshot_reply_line'],
           'initial_player_projection_matches': not row['initial']['player_projection_differences'],
           'initial_deck_projection_matches': row['initial']['deck_ids_names_suits_ranks_natures_order_equal'],
           'full_initial_state_compared_or_equal': False,
           'specified_final_projection_differences': row['final_specified_projection_differences'],
           'semantic_usage_flag_differences': row['semantic_usage_flag_differences'],
           'pending_counts': row['pending_counts'], 'source_skill_anchor': anchors['uy' if skill == 'yixin' else 'ug' if skill == 'guifu' else 'ub'],
           'new_equivalence_signoff': False}
    assessments.append(rec)
    raw = link(rec['upstream_raw'], '上游') + ' / ' + link(rec['native_raw'], '原生')
    table.append(f'| `{name}` | {notes[name]} | {row["pending_counts"]["direct"]}/{row["pending_counts"]["restored"]} | {raw} |')
output('case-assessments.json', assessments, True)
output('case-table.md', '\n'.join(table) + '\n')

findings = [
    {'id': 'U08-R1', 'title': '受控目标过滤缺失', 'inventory_keys': [key_by_id['bolehuiyan']],
     'cases': ['b_under_control'], 'evidence_level': '上游真实触发负例+原生真实命令+独立原生复跑+源码过滤反例',
     'native_observed': '仍问Tag1，预测命中后座0多获得实体106，牌堆少106、used_turn=true。',
     'upstream_observed': '初始目标2._trueMe=座0、isUnderControl=true；无chooseControl、无106、bolehuiyan4=false。',
     'boundary': '共同初态只含座位HP/牌面等；控制关系是上游附加条件，原生没有对应过滤，不称全初态同一。',
     'source_anchors': ['ubcontrol', 'under', 'nbfilter', 'setupcontrol'],
     'recommendation': '保留/维持简化近似并补该实际反例；不新增等价。'},
    {'id': 'U08-R2', 'title': '真实cancel返回值导致上游消耗本回合慧眼次数', 'inventory_keys': [key_by_id['bolehuiyan']],
     'cases': ['b_cancel_predict', 'b_cancel_then_retry'], 'evidence_level': '原始DOM点击结果+上游实际标记与后续杀+原生空答与后续杀+独立原生复跑',
     'upstream_observed': 'DOM link=cancel实际走ui.click.cancel，返回confirm=cancel/bool=false而无control；源result.control!==cancel为真，加bolehuiyan4但storage未形成有效预测。第二杀不再提供预测。',
     'native_observed': '空答不设置bolehuiyan_used_turn；第二杀可再问Tag1并摸106。',
     'boundary': '取消字符串的意图不能替换真实返回字段；两杀案例实体顺序不同，但上游取消后的bolehuiyan4与原生未用标记在第二杀前已不同。',
     'source_anchors': ['control', 'cancel', 'ubcancel', 'nbstep'],
     'recommendation': '差别栏应记录当前上游真实取消行为；不擅自修正上游或把原生意图写成等价。'},
    {'id': 'U08-R3', 'title': '强制弃牌挂起时使用中的杀区域不同', 'inventory_keys': [key_by_id['bolehuiyan'], key_by_id['bolehuiyan3']],
     'cases': ['b_cancel_discard'], 'evidence_level': '双方实际已受伤并停于强制弃牌，原始牌区快照+源码时序',
     'native_observed': '空答被Rejected(InvalidChoice)，仍待bolehuiyan:discard；杀9已经进入discard，table空。',
     'upstream_observed': 'chooseToDiscard forced、无取消键；杀9仍在ordering，未进入discard，root.done=false。没有注入隐藏取消答案。',
     'source_anchors': ['shaemit', 'shaadvance', 'ushadamage', 'nbdiscard'],
     'recommendation': '保留挂起时点/实体区域差异；不能按最终未弃费用、HP相同写全状态或完整结算通过。此案不是上游失败，也不是强制取消成功。'},
    {'id': 'U08-S1', 'title': '慧眼触发和结算事件范围不同，31案未覆盖其全部反例',
     'inventory_keys': [key_by_id['bolehuiyan'], key_by_id['bolehuiyan2'], key_by_id['bolehuiyan3']],
     'cases': [], 'evidence_level': '仅本轮独立源码确认；没有新增对应上游运行场景',
     'upstream_observed': '主技能逐目标shaBegin/priority11且过滤受控；辅助分别shaEnd、shaDamage。',
     'native_observed': '主技能整牌CardUsed/priority1只取首目标且targets含自己就整体拒绝；未命中辅助为ShaDodged/TurnEnded，命中辅助为普通Damage，过滤检查来源/目标但不核对本次杀的card/kind。',
     'source_anchors': ['ub', 'ub2', 'ub3', 'nbmeta', 'nbfilter', 'nbdamage', 'ushend'],
     'recommendation': '多目标、第一目标无手牌而后续有手牌、免伤/无伤shaEnd、其他嵌套非杀伤害及优先级组合继续待验。不得拿本31案签两辅助或整个来源全分支等价。'},
]
boundaries = [
    {'id': 'B1', 'description': '31案每案均重载页面，并实查八座HP/maxHP/face_down、he/j实体id/name/suit/rank/nature、主技能，以及上游虚拟装备/判定与实体对应、无旧预测/觉醒残留。原生牌堆保留7或8张，逐案顺序与牌面严格相同。'},
    {'id': 'B2', 'description': '每案21张未映射的原生初始discard实体未在上游创建；另77/78闪电的native导出nature=null、上游固有thunder。本轮不归一后称完整初态相同；这些初始discard在双方各自执行中保持原样。'},
    {'id': 'B3', 'description': '上游directgain形成的手牌顺序与原生不同，比较he/j是实体集合与牌面，不包含手牌排列；b_repeat/b_cancel_then_retry因此普通杀实体出牌顺序相反。'},
    {'id': 'B4', 'description': 'y_cancel_target上游实际点结束回合，原生空目标取消仍在Play，不能当相同取消协议/阶段轨迹。y_no_wound/g_no_equip/g_only_own上游可点技能再发现无目标，原生直接拒绝；均无实际费用或效果。'},
    {'id': 'B5', 'description': '鬼斧每个目标均仅一件装备，实际没有打开guifu:choose_equip；多装备选择A/B/强制选牌挂起恢复未覆盖。医心费用只选手牌，没有装备费用/下一回合限定标记重用/其他技能组合。'},
    {'id': 'B6', 'description': '原生使用正式hero-kill注册，109host/0VM/0新增card；夹具通过debug_grant赋三技和自设回合/牌区，不是武将自然装配、服务器/Worker/产品UI或整局验收。上游手建phase/phaseUse root，但真实调用原版useCard/chooseToUse及DOM控件，不替换技能结算函数。'},
    {'id': 'B7', 'description': '恢复证明是原生内每份原始NGSA的完整快照、继续命令、日志与摘要；只允许auth_generation变化。它不证明上游事件/RNG/存档与原生完整同一。强制待答案仍pending，不写31案全部完成结算。'},
    {'id': 'B8', 'description': '未重新启动上游浏览器；逐stage/最终快照、夹具赋值和真实点击均回溯最终原始CDP。未运行完整preflight、产品三crate/全工作区门禁、其他平台或资源/界面验收。未读/加载/复制禁止Spine运行库；原网络日志38215请求中无Spine URL。'},
]
output('findings.json', {'review_date': '2026-11-14', 'head': '5c73d1d9158145e33f98f2018013914667d13964',
                        'reviewer': '/root/repair_doc_refs', 'scope': 'U08三交互技31案独立复核',
                        'batch_result': '退回；慧眼有已实际确认差异，不新增规则等价签署。',
                        'runtime_difference_cases': ['b_under_control', 'b_cancel_predict', 'b_cancel_then_retry'],
                        'pending_region_difference_cases': ['b_cancel_discard'],
                        'specified_projection_match_count': 28, 'specified_projection_different_count': 3,
                        'specified_projection_and_usage_flags_match_count': 27,
                        'new_equivalence_signoffs': 0, 'findings': findings, 'boundaries': boundaries,
                        'inventory_modified': False, 'original_source_evidence_modified': False,
                        'collector_report_or_comparator_read': False}, True)

source_review = f'''# U08 源码独立复核（2026-11-14）

本表只根据实际上游/原生源码、冻结 probe/runner 和原始运行记录写成。未读采集者本批 README、结论或比较器。CodeGraph 工具未提供，使用指定文件的字面定位与直接读取。源码及锚点哈希见 [source-anchors.json](source-anchors.json)。

| 技能 | 上游条件、玩家选择与结算 | 原生消费与本轮结果 |
| --- | --- | --- |
| 医心 | {source('uy')}：phaseUse、limited；he必须大于2，可选两张he和任一已受伤角色；执行后觉醒限定标记，回复min(4,损失体力)，再摸4-X。 | {source('nygate')}与{source('gate')}落实回合/阶段/大于2/未限定用/存在受伤目标；{source('ny')}分两次提问、取消返还once，选目标后才弃费用并写持久yixin_used。10案的手牌费用、目标A/B/自己、条件和同阶段重用结果已核；选满两牌后上游实际只有结束阶段按钮，与原生空目标答不同。 |
| 慧眼 | {source('ub')}：每个shaBegin、priority11、direct；排除来源/目标为自己、目标无手牌、受控目标和本回合已用。可点能命中/不能命中/cancel，{source('ubcancel')}依result.control记录预测和tempSkill；{source('ub2')}在shaEnd作未命中结算，{source('ub3')}在shaDamage作命中结算；正确摸1、错误必弃1。 | {source('nbmeta')}是整牌CardUsed/priority1，{source('nbfilter')}只看首目标且排除targets中任何自己，缺受控过滤；{source('nbstep')}空答直接完成且不写used_turn；{source('nbresolve')}正确摸1，错误开不可取消的he弃1。14案包含两个实际规则差异类型与强制待答牌区差异，不能签等价。 |
| 鬼斧 | {source('ug')}：phaseUse、usable1、目标不为自己且有装备；玩家强制选弃目标装备，再按sortBySeat顺序双方各摸1。 | {source('gate')}统一回合/出牌阶段门控，{source('ng')}先问目标且取消返还once，单装备直接弃、多装备再强制问牌；{source('ngdraw')}从当前回合玩家按座次摸牌。本轮7案目标A/B分别只有单装备，实核座0先摸106、被选座摸46；{source('ngask')}多装备询问从未触发。 |

慧眼的“取消”必须按实际 UI 返回值解释：{source('control')}对link=cancel调用{source('cancel')}，返回confirm/bool而非control。原始CDP第954、1050行的真实HTMLElement.click回包均为该字段集；上游随后实际存在bolehuiyan4。原生空答不消耗次数。上游源码意图或本地把字符串写成cancel，均不能替代这些运行记录。

原生强制弃牌待答时杀9已discard：{source('shaemit')}发射Damage后调用{source('shaadvance')}，最后目标结束时将牌放入弃牌堆。上游{source('ushadamage')}等待shaDamage子事件，原始挂起快照中该牌仍在ordering。双方HP相同不代表此阶段区域相同。

正式路径已确认：{source('locked')}调用{source('formal')}。冻结{source('probeformal')}及独立 [archive-probe](archive-probe/src/main.rs)均走同一注册；[正式注册原始输出](archive-replay02/formal-registration.json)为109 host、0 VM、0包内新增牌，绑定锁9187435935481107005、绑定规则摘要18410807972335883120。没有额外占位卡/技能。资产声明的upstream_commit为8485dbb29ff16f9b258a766b1a17ef75f883aade；实际运行来源checkout为d9d0d28a06c88a3b556e914cf5abb6cfe90e206a，技能文件sha256为3c07c24d79b7abf6fb07584d89b57a6584ccc0c85388c2b747d528f0113d4f98，两者分列。

仍未采样：慧眼多目标/首目标无手牌而后续有手牌、无伤shaEnd、其他嵌套非杀Damage、priority组合；医心装备费用与跨回合重用；鬼斧单个目标多装备的玩家选牌和挂起恢复。原生事件范围差异是源码发现，不写成这些上游运行反例已执行。

实际夹具入口为{source('probesetup')}、{source('setupreset')}、{source('setupentry')}。上游保留现代ESM/async调度，逐场刷新并等onfree/cardsAsync静止；八座和虚拟区实拍初态都能追溯原CDP。为玩家选择切换控制属于夹具操作，{source('setupcontrol')}是b_under_control额外条件；不把该关系当双方共同状态字段。未加载禁止Spine运行库，也未用stub替代技能或其运行依赖。
'''
output('source-review.md', source_review)

readme = '''# U08 三交互技独立复核（2026-11-14）

本批退回：慧眼存在实际触发差异；新增“已复核等价”签署0。医心10案、鬼斧7案、慧眼14案，共31唯一case，逐案见 [对照表](case-table.md) 与 [结构化判断](case-assessments.json)。产品、测试、inventory、公共文档、原U08及L10目录均未修改，也未操作Git index/refs。

31案的八座HP/maxHP/face_down、手/装备/判定实体集合及牌面、主技能和实际7/8张牌堆顺序匹配；上游虚拟装备/判定及旧技能/预测/觉醒残留已逐案排除。28案的指定终态投影相同、3案不同；加上已用次数标记后27案相同。这里的“相同”只指列出的字段，不签完整初态、完整事件/RNG轨迹或技能规则等价。

确认的差异与限制：

- `b_under_control`：上游目标实际由慧眼主控制，过滤后不预测；原生缺该条件并额外摸实体106。控制关系未包含在共同初态投影中。
- `b_cancel_predict` / `b_cancel_then_retry`：上游真实DOM取消返回confirm/bool而无control，实际消耗本回合bolehuiyan4；原生空答不消耗，第二次预测多摸106。只有HP相同的取消单案仍有用次状态差异。
- `b_cancel_discard`：双方均已触发强制弃牌并仍待答；上游无取消按钮，未提交隐藏取消；原生空答被拒。此时杀9在上游ordering而原生discard，不写完成结算或全牌区相同。
- 每案21张无映射的初始discard实体未复制；77/78闪电的native导出nature=null、上游固有thunder。原始差别保留，不靠归一化签全初态。手牌排列亦不同，两杀案例实际出牌ID次序相反；共同比较的手牌为集合。
- 医心选满两张费用牌后的取消在上游实际点击“结束回合”，原生为空目标答，阶段端点不同。鬼斧目标均仅一件装备，多装备选牌/挂起未覆盖。其余未执行分支见 [源码复核](source-review.md) 和 [findings.json](findings.json)。

独立核验执行了：

1. 冻结输入核hash：[输入验证](input-validation-first.json) 对原raw manifest384文件/77,845,519B、完整manifest404文件/79,684,128B、29份源副本以及11个点名文件/保留二进制均匹配。只哈希作者报告字节，未读取其README、结论或比较器。原始完整manifest的SHA256为7934ad115c63fd0ace23d3b3800d1e2703e621554e9db6391035f09a1185205c。
2. 原始CDP重算：[来源记录](cdp-provenance-second.json) 匹配1072对请求/回包，最终31案的赋值从行588开始、最后快照行1072。每个保存stage/最终快照、fixture赋值、真实控件点击均对到原始回包。共70组赋值含旧失败/探索轮；没有把旧轮当最终案或按预期去重。30案root.done=true，强制弃牌1案仍pending。旧SyntaxError及所有采集轮保持原样。
3. 原生保留二进制重跑：[验证记录](native-replay01-validation.json) 的238个输出文件与原native-round2逐字节一致。二进制SHA256为6a08977ed231a8c71229dc50c650ee8b4fd02973d96389dc38d60239c4d3dffd；实际运行输出在 [native-replay01](native-replay01/native-results.json)。
4. 独立NGSA probe：[76份pending原档及后续命令](archive-replay02/pending-suffix-results.json) 全部解码/重编码字节匹配；恢复后的完整Snapshot仅auth_generation增加；询问规格一致。对每份pending单独重放后续全部命令，共134次命令的请求、实际回复、增量日志、状态摘要及最终完整Snapshot（只归一auth_generation）均匹配原记录。38个直接、38个逐答恢复原档均覆盖；31对直接/恢复final完整Debug亦只差auth_generation。[补跑原始日志](archive-replay02.raw.log) 通过76/76，失败0；不是原始NGSA字节相同。[首次编译失败](archive-replay01.raw.log)与[失败源码](archive-probe-first-source.rs)保留，原因是自有probe的json!表达式括号，不是产品失败。

独立重算首轮把每座编号输出中的seat字段误写成布尔比较值；[首轮源码](recompute-first-source.py)/first输出保留，second输出改为seat编号与seat_matches两个字段。31案投影/差异计数和CDP匹配结果未变。后续引用统一使用second结果。

版本和环境：实际采集在605201f3f49e10cea61c31d52a61bbb66c85599d及当时dirty产品快照执行，收口与本独立运行HEAD为5c73d1d9158145e33f98f2018013914667d13964。冻结源副本29份与当前字节相同，本次重跑前后489个相关Rust源码文件未变。产品受跟踪路径clean，工作区含并行未提交证据。规则333 / NGSA107 / content schema5 / ABI759 / WS8；当前rustc/cargo均1.99.1。正式hero-kill装载109host/0VM、无占位实体，实际锁与绑定规则摘要见 [注册输出](archive-replay02/formal-registration.json)。

原上游Chrome为157.0.8103.52、CDP 50185、control 50210、独立profile；使用5173现代临时ESM源码服务，FS51215归主agent。原始工具和路径见 [browser-environment](../../U08/browser-environment.json)、[源载入指纹](../../U08/source-load-verification.json)、[原始转录索引](../../U08/raw-transcript-index.json)。本轮只读转录，没有控制他人Chrome或重新启动浏览器；网络日志38,215请求中无Spine URL，未读/加载/复制禁止Spine运行库。手建fixture和root调用的原版技能不等于自然整局或原生产品UI验收。

未运行完整preflight、产品三crate/全工作区测试、其他平台或界面验收。没有私有Cargo target；独立probe使用共享target正常编译且未清理。命令、二进制/源指纹、磁盘、文档链接与冻结复核见 [commands](commands.md)、[收口记录](freeze-context.json)、[文档验证](validation-final.json)、[冻结文件清单](frozen-file-manifest.json)。
'''
output('README.md', readme)
output('commands.md', '''# 独立复核命令（2026-11-14）

工作目录：`/Users/ruyimg/Documents/Claude/sanguosha/noname-next`。脚本只向本独立目录写入；均未使用Git add/commit/push、Cargo clean、产品生成器或完整preflight。

```sh
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/capture-inputs.py > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/capture-inputs-first.log
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/validate-inputs.py > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/input-validation-first.log
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/replay-native.py > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/native-replay01-validation.log
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute.py first > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute-first.log 2>&1
CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target U08_ORIGINAL=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/U08 U08_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next U08_REVIEW_OUT=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay01 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-probe/Cargo.toml > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay01.raw.log 2>&1
CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target U08_ORIGINAL=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/U08 U08_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next U08_REVIEW_OUT=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay02 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-probe/Cargo.toml > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay02.raw.log 2>&1
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute.py second > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute-second.log 2>&1
```

`archive-replay01`编译失败，未创建其运行输出目录；失败源码与日志保留。`archive-replay02`才是76份存档的独立解码/后续命令重放结果。首轮重算metadata修正在README说明，原始first输出未覆盖。
''')
print(json.dumps({'case_assessments': len(assessments), 'findings': len(findings), 'boundaries': len(boundaries), 'source_anchors': len(anchors)}, ensure_ascii=False))
