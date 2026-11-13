import hashlib
import json
import pathlib
import re

BASE = pathlib.Path(__file__).resolve().parent
BROWSER = BASE / 'browser-round4'
NATIVE = BASE / 'native-round2'


def read(path):
    return json.loads(path.read_text())


def write(name, value):
    path = BASE / name
    assert not path.exists(), path
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


NOTES = {
    'g_sha_ally': ('projection_match', '桃45声明杀、三座均不质疑：目标1由4血降至3，载体进入弃牌；原生同投影。上游真实经过三次质疑询问，原生没有这些询问。'),
    'g_sha_question_fake': ('observed_difference', '座1质疑桃45声明的假杀：上游杀作废，座1保持4血并摸无中104；原生无质疑询问，直接命中座1至3血且未奖励摸牌。'),
    'g_true_question_discard': ('observed_difference', '杀8声明真杀、座2质疑后选择弃桃45：上游座1受伤、座2弃45并获rechanyuan；原生只结算座1受伤，座2仍持45且无缠怨。'),
    'g_true_question_losehp': ('observed_difference', '座2质疑真杀后取消弃牌：上游座2保留桃45、由4血降至3并获rechanyuan；原生没有惩罚窗，座2仍4血且无缠怨。'),
    'g_tao': ('projection_match', '座0以杀8声明桃且三座均不质疑：上游与原生均由2血回复至3，消耗杀8；只验证此声明与结算投影。'),
    'g_cancel_card': ('observed_difference', '上游在备份蛊惑的实体牌选择处点可见取消，保留桃45/杀8且未产生reguhuo_phase；原生取消reguhuo_card虽保留牌，却已记once，再ActivateSkill被InvalidChoice拒绝。'),
    'g_cancel_name': ('observed_difference', '上游取消声明牌名窗后回到chooseToUse，仍能看到reguhuo；原生取消reguhuo_name后已消耗once，再发动被InvalidChoice拒绝。'),
    'g_cancel_target': ('observed_difference', '上游选择载体后点取消可退出而不耗牌、不产生reguhuo_phase；原生reguhuo_targets为不可取消，空答复InvalidChoice且保持原目标询问。'),
    'g_no_hand': ('projection_match', '四座无手牌且仅座0有蛊惑：上游chooseToUse无reguhuo选项；原生ActivateSkill拒绝，无询问、无once、无人失牌失血。'),
    'g_no_skill': ('projection_match', '座0有桃45但没有蛊惑：上游无reguhuo选项；原生发动拒绝，手牌与血量不变。'),
    'g_response': ('observed_difference', '座1真实杀攻击只持桃45的座0：上游chooseToUse响应窗可发动蛊惑声明闪，三座不质疑后以45响应并保持4血；原生没有蛊惑响应入口，直接受伤至3血并保留45。'),
    'r_basic': ('observed_difference', '受1伤选获得牌/基本：双方先摸无中104；上游自顶检索桃45，原生自底检索酒85，实体及牌面不同。'),
    'r_trick': ('projection_match', '受1伤选获得牌/锦囊：先摸104后，余堆仅拆67符合；双方获得104和67。该场不能证明多张锦囊的检索顺序。'),
    'r_equip': ('observed_difference', '受1伤选获得牌/装备：双方先摸104；上游取自顶的青釭114，原生取自底的诸葛118。'),
    'r_move_A': ('projection_match', '座1先真实装备诸葛118再杀座0；座0受伤后选移动、源1目的0，双方均摸104且将118移至座0装备区。上游先选两座，原生先选实体再目的座。'),
    'r_move_B': ('projection_match', '相同受伤与场上118，改选目的座2：双方均摸104并把118移至座2装备区，座0没有该装备；与A形成不同结果。'),
    'r_cancel_choice': ('observed_difference', '上游点真实文字“取消”仍logSkill并摸104，再调用无牌可移的moveCard；原生选Tag2直接结束、不摸牌。保留上游result.control==cancel2与实际中文控件不符的行为。'),
    'r_cancel_type': ('observed_difference', '双方已摸104；上游类型chooseButton有取消，取消后技能结束；原生rangjie_type不允许取消，空答复InvalidChoice并保持pending，手牌相同不能掩盖续行差异。'),
    'r_no_skill': ('projection_match', '座0未授予让节，座1真实杀令其降至3血；两端均无让节询问且不摸牌。'),
    'r_other_target': ('projection_match', '仅座0有让节，杀改打座2：两端仅座2降至3血，座0未获让节询问、未摸牌。'),
    'r_double': ('observed_difference', '座1先真实使用酒再杀，座0受2伤并完成两次基本牌选择；上游获得104/45/114/10，原生获得104/86/45/43。两次伤害循环均执行，但取牌实体因检索方向不同而分歧。'),
    'k_A': ('upstream_blocked', '上游真实phaseBefore进入k_danggu step0，在dui.element.create抛dui未定义，未到伙伴选择；原生选Tag0后first=8、partner=0、active_mask=257，恢复续行成功。不能将原生结果当作上游A已执行。'),
    'k_B': ('upstream_blocked', '独立fresh场同样在k_danggu step0的dui依赖阻塞；原生改选Tag1后first=8、partner=1、active_mask=258，与A不同，未有上游B结算结果。'),
    'k_cancel': ('upstream_blocked', '独立fresh场同样未到上游伙伴选择，故没有实际上游取消结论；原生强制k_danggu_partner空答复被InvalidChoice拒绝且仍pending，挂起恢复成功。'),
    'k_after_initial_no_trigger': ('condition_only', '上游phaseNumber=1的真实phase()没有k_danggu触发；原生已置inited=1的下一Phase也不触发。仅比较初始化门控；上游随后正常摸2张至出牌窗，原生停在NextPhase后，后续牌区不作等价比较。'),
    'k_no_skill': ('condition_only', '均不授予党锢：上游真实phase()没有进入该技能，原生NextPhase没有伙伴问询。上游继续正常摸2张，与原生截点不同，牌区差异不归因党锢。'),
}


def face(card, native):
    return (card['id'], card.get('upstream_name') if native else card['name'], card['suit'], card['rank'], card.get('nature'))


def project(state, native, mirrored_ids, case):
    players = []
    for p in state['players']:
        players.append({
            'seat': p['seat'], 'hp': p['hp'], 'max_hp': p['max_hp'], 'face_down': p['face_down'],
            **{zone: sorted(face(c, native) for c in p[zone]) for zone in ['hand', 'equips', 'judge']},
            'skills_excluding_internal_reguhuo_phase': sorted(s['name'] for s in p['skills'] if s['name'] != 'reguhuo_phase'),
        })
    result = {'players': players,
              'deck_top_first': [face(c, native) for c in state['deck_top_first' if native else 'deck']],
              'mapped_discard_ids_sorted': sorted(c['id'] for c in state['discard'] if c.get('id') in mirrored_ids),
              'table_or_ordering_ids_sorted': sorted(c['id'] for c in state['table' if native else 'ordering'])}
    if case.startswith('g_'):
        result['reguhuo_used_this_turn'] = '(0, "reguhuo", 1)' in state['once'] if native else state['players'][0]['reguhuo_phase']
    return result


browser = read(BROWSER / 'results.json')
natives = {r['case']: r for r in read(NATIVE / 'native-results.json')}
assert set(NOTES) == set(natives) == {r['case'] for r in browser}
rows = []
for source in browser:
    name = source['case']; native = natives[name]['direct']
    assert len(source['fixture_checks']) == 4
    assert all(all(v for k, v in p.items() if k != 'seat') for p in source['fixture_checks'])
    assert not source['final']['untaggedDeck']
    assert not source.get('comparison_excluded')
    mirrored_ids = {c['id'] for c in source['initial']['discard'] + source['initial']['deck']}
    for p in source['initial']['players']:
        mirrored_ids.update(c['id'] for zone in ['hand', 'equips', 'judge'] for c in p[zone])
    uproj = project(source['final'], False, mirrored_ids, name)
    nproj = project(native['final'], True, mirrored_ids, name)
    kind, note = NOTES[name]
    if kind == 'projection_match':
        assert uproj == nproj, name
    assert source['upstream_completed'] == (kind != 'upstream_blocked'), name
    checks = natives[name]['direct']['pending_archive_checks'] + natives[name]['restored']['pending_archive_checks']
    rows.append({'case': name, 'skill': 'reguhuo' if name.startswith('g_') else 'rangjie' if name.startswith('r_') else 'k_danggu',
                 'outcome': kind, 'observed': note,
                 'upstream_branch_observed': source['upstream_completed'], 'upstream_root_done': source['final']['done'],
                 'upstream_stop_event': source['final']['choice'].get('name'),
                 'upstream_projection': uproj, 'native_projection': nproj,
                 'projection_equal': uproj == nproj,
                 'pending_semantics_compared_separately': name in ['g_cancel_target', 'r_cancel_type', 'k_cancel'],
                 'native_final_choice': native['final']['choice'],
                 'native_pending_archives': len(checks), 'native_pending_restore_success': sum(x['error'] is None for x in checks),
                 'initial_mirrored_entities': len(mirrored_ids),
                 'excluded_initial_discard_entities': len(source['initial']['skippedInitialDiscard']),
                 'late_untagged_deck_entities': source['untagged_deck_count_final'],
                 'upstream_evidence': f'browser-round4/upstream-{name}.json',
                 'native_evidence': f'native-round2/native-{name}.json'})

counts = {k: sum(r['outcome'] == k for r in rows) for k in ['projection_match', 'observed_difference', 'condition_only', 'upstream_blocked']}
assert counts == {'projection_match': 9, 'observed_difference': 12, 'condition_only': 2, 'upstream_blocked': 3}
write('comparison.json', {'source': 'L10', 'head': read(BASE / 'preflight.json')['head'], 'versions': read(BASE / 'preflight.json')['versions'],
                        'new_equivalence_signoff': False,
                        'counts': {'cases': len(rows), 'upstream_branch_observed': 23, 'blocked': 3, 'initial_seat_checks': 104, 'late_untagged_deck_entities': 0, **counts},
                        'projection_scope': ['四座HP/maxHP/翻面、按原生实体ID和牌名花色点数属性匹配的手牌/装备/判定区集合',
                                             '完整保留的7或8张牌堆及其顶到底顺序', '双方已映射初始牌及后续消费牌的弃牌集合，不比较弃牌顺序',
                                             'reguhuo成功/取消时的本回合已用标记，以及实际rechanyuan授予',
                                             '目标选择/取消是否结束或保持pending另列，不以牌区相同代替交互一致'],
                        'not_compared': ['未映射初始discard的装备/其他牌面；明列每场数量与完整牌面，未声称完整初态相同',
                                         '上游DOM手牌排序、弃牌堆顺序、动画、UI样式、隐藏信息显示和AI评估',
                                         '跨引擎完整事件树、RNG流/游标、全历史和全局phase状态；原生内部直接/恢复则比较完整快照',
                                         'k_danggu随机伙伴候选、真实UI的A/B/取消以及伙伴技能后续效果：原始入口阻塞后仅逐行比较',
                                         'k两个反例的后续摸牌阶段：上游继续到phaseUse，原生仅推进一个Phase，截点不同'],
                        'cases': rows})

markdown = ['| case | 实际结果 | 判定范围 |', '|---|---|---|']
labels = {'projection_match': '限定投影相同', 'observed_difference': '实际差异', 'condition_only': '仅反例门控', 'upstream_blocked': '上游实际阻塞'}
for row in rows:
    markdown.append(f"| [{row['case']}]({row['upstream_evidence']}) | {row['observed']} | {labels[row['outcome']]} |")
(BASE / 'case-table.md').write_text('\n'.join(markdown) + '\n')
print(json.dumps({'cases': len(rows), **counts}, ensure_ascii=False))
