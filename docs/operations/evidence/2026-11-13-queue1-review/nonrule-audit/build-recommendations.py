"""Materialize the independently reviewed, item-specific static audit.

Run from noname-next. Writes only this evidence directory; never inventory.
The descriptions below are audit decisions, not automated equivalence claims.
"""
import collections
import json
from pathlib import Path

BASE = Path(__file__).resolve().parent
snapshot = json.loads((BASE / "inventory-snapshot.json").read_text())
items = snapshot["items"]
excerpts = {x["key"]: x for x in json.loads((BASE / "upstream-excerpts.json").read_text())}
rows = {}


def ref(path, line):
    return {"path": path, "line": line}


def add(src, cat, ident, upstream, native, difference, classification, native_refs, keep=False, extra_upstream=None):
    matches = [(k, v) for k, v in items.items() if (v["source_id"], v["category"], v["id"]) == (src, cat, ident)]
    assert len(matches) == 1, (src, cat, ident)
    key, old = matches[0]
    assert key not in rows, key
    status = old["status"] if keep else "简化近似"
    action = "保留原状态，仅澄清范围" if keep else ("建议下调" if old["status"] == "已复核等价" else "维持简化近似")
    e = excerpts.get(key)
    upstream_refs = [ref(e["path"], e["start"])] if e else [ref(old["upstream_ref"].rsplit(":", 1)[0], 1)]
    if extra_upstream:
        upstream_refs.extend(extra_upstream)
    scope_note = "本次仅静态核对，不是新的技能规则等价签署；上游与原生实际触发均未执行。"
    if classification == "实际规则，保留旧状态":
        scope_note = "该项实际改变规则/玩家选择，不能称无规则效果或惰性钩子；保留原状态不表示本次重新签署全部条件、分支和存读档。"
    elif classification == "上游注释，惰性登记":
        scope_note = "原始引用处为注释，不是本次可触发样本；原生仅保留惰性登记。保留旧状态仅表示此无活动行为范围，不作技能等价背书。"
    elif classification == "上游主动禁用，惰性登记":
        scope_note = "上游有效定义的 filter 恒 false，普通事件不会执行其 content；原生额外计数/cue 未形成实际播放。保留旧状态仅澄清惰性范围。"
    rows[key] = {
        "key": key, "source_id": src, "category": cat, "id": ident,
        "original_status": old["status"], "status": status, "status_action": action,
        "classification": classification, "upstream_behavior": upstream,
        "native_consumption_or_effect": native, "difference": difference, "diff": difference,
        "note": f"2026-11-13 独立非规则条目静态核对：{scope_note}",
        "upstream_refs": upstream_refs, "native_refs": native_refs,
        "runtime_validation": "未执行（静态源码审查）",
        "new_equivalence_signoff": False,
    }
    if e and not e["active_definition"]:
        rows[key]["upstream_comment"] = {"kind": e["disabled_by"], "start": e["comment_start"], "end": e["comment_end"]}
    if e and src == "L29":
        rows[key]["upstream_character_offsets"] = [e["start_offset"], e["end_offset"]]


L = "crates/content-native-loader/src/"
H = "crates/engine-native-content/src/content/host/step/"
K = "crates/engine-native-kernel/src/"
R = "apps/web-client/src/engine/extensionSurfaceRuntime.ts"
Z = "apps/web-client/src/engine/surfaces/zuanSurface.ts"
P = "apps/web-client/src/components/battle/ExtensionSurfacePanel.vue"
V = "apps/web-client/src/views/BattleView.vue"
B = "apps/web-client/src/components/battle/BattleBoard.vue"
U = "archive/upstream/noname-liuli/noname/extension/"

# L22: real rule configuration and presentation delay are different concerns.
add("L22", "configs", "eloPlayers", "开关段位制裁的人数/座位判定，影响 _eloMechanism、_eloDraw 等的启用及牌量调整。",
    "elo.rs 解析 eloPlayers；elo_apply_engine_config 可写 elo_players_enable，内核有计算消费；该应用 helper 在产品源码仅有定义/导出，未发现装包后调用。",
    "配置已解析并有可调用的规则辅助入口；未接通安装包配置到当前对局，不能由 helper/测试级接入宣称玩家切换已生效。", "规则配置未接通", [ref(L+"elo.rs", 111), ref(L+"elo.rs", 551), ref(K+"kernel/elo.rs", 685)])
add("L22", "configs", "eloDifficulty", "难度枚举及自动难度用于起手牌、摸牌、手牌上限/人数的段位调整。",
    "parse_elo_config、resolve_difficulty 及 elo_apply_engine_config 写入 elo_eh/elo_difficulty_val；产品没有调用配置应用入口。",
    "已有难度计算和内核消费能力；当前配置不是活跃对局的已接通开关，保留普通难度 helper 不能证明全部枚举生效。", "规则配置未接通", [ref(L+"elo.rs", 115), ref(L+"elo.rs", 551)])
add("L22", "configs", "timeKiller", "非本机角色的既有 chooseToUse/Respond/Discard 前随机等待，并可输出思考台词；不新增玩家选择。",
    "配置解析及内核 elo_on_choice_before 计算 delay_ms、记录台词索引；调用方丢弃返回值，未发现计时等待或聊天组件消费这些字段。",
    "无牌局规则效果；原生目前是延迟/台词参数登记，未实际等待或显示台词，配置应用入口也未接产品。", "表现配置未消费", [ref(L+"elo.rs", 120), ref(K+"kernel/elo.rs", 685), ref(K+"kernel/phase/play.rs", 4)])

# L10: upstream hand folding is not a card corner effect.
l10 = {
    "分割线01": ("设置页分隔图片 line.png；clear 项，不改变牌局。", "EpicConfig 没有分隔图字段，也无对应设置页组件。", "无规则效果；设置菜单分隔图未重建，不能把未登记字段记为配置等价。"),
    "skillEffects": ("启停限定/觉醒技能展示特效，默认 true。", "解析 skill_effects 并由 presentation() 返回布尔值；无产品调用方/渲染消费。", "无规则效果；仅展示开关登记，未驱动限定/觉醒特效。"),
    "skillEffectsScale": ("枚举 5/9/10/11/12/13/14/15，默认 11；特效使用处除以 10 得到缩放。", "解析器把原值当 0..10 浮点倍率，默认 1.0，且无渲染消费者。", "无规则效果；合法上游值 11–15 会被拒绝，9/10 也未按上游除以 10；不能宣称比例配置等价。"),
    "cardFold": ("默认 true；手牌过多时完整折叠并禁用滑动管理，点击后同时保存两套扩展配置。", "仅 card_fold 布尔登记，默认 false；没有牌区折叠消费者。", "无规则效果；默认值和手牌管理行为都不同，未实现完整手牌折叠。"),
    "cardFold_proportion": ("完整折叠关闭时设置手牌相互遮挡比例 30%–100%，默认 100%。", "EpicConfig 无此字段，手牌组件无该选项消费。", "无规则效果；既未建模也未应用折叠比例。"),
    "cardPhantom": ("默认 true；出牌幻影外观。", "仅 card_phantom 布尔登记，默认 false，无幻影渲染路径。", "无规则效果；默认值不符且未显示幻影。"),
    "cardEmoji": ("隐藏设置项，新菜篮子/卡牌表情开关，默认 false。", "仅 card_emoji 布尔登记；emoji helper 只写 cue 字符串，未接该配置。", "无规则效果；隐藏开关没有产品消费，不能将布尔解析当作表情功能。"),
    "shichangshi": ("默认 true；有十周年 UI/目标武将依赖时替换十常侍技能并换图。", "默认 false；shichangshi_plan 仅返回 Apply/缺依赖等计划，没有产品调用来改目录或换图。", "配置有实际角色/技能替换作用；原生仅依赖计划登记，已有 k_danggu/k_mowang 独立技能不能证明此开关应用。"),
    "meihuaskills": ("默认 true；优化定汉、蛊惑等部分技能的选牌显示（来自缘之空的显示改写）。", "仅 meihua_skills 布尔登记，默认 false，没有选牌显示消费。", "无规则效果；默认值不符，未接定汉/蛊惑等的选牌显示改写。"),
    "分割线02": ("设置页第二条 line.png 分隔图，不改变规则。", "没有分隔图字段或设置页对应项。", "无规则效果；该菜单外观未迁入产品。"),
}
for ident, (up, native, diff) in l10.items():
    cls = "规则配置未接通" if ident == "shichangshi" else "表现配置未消费"
    add("L10", "configs", ident, up, native, diff, cls, [ref(L+"epic_cards.rs", 76), ref(L+"epic_cards.rs", 114), ref(L+"epic_cards.rs", 128)])
add("L10", "ui_hooks", "_emojiRebate1", "强制在回复/濒死/获得牌/翻面等事件，根据来源关系和随机结果向其他角色丢花蛋等表情；不产生独立玩家选择。",
    "triggerL10EmojiRebate 仅写 theme.throwEmotionCue；未找到产品调用方，调试面板可显示 cue 文本。",
    "无规则效果；现为 cue 示意，未实现上游事件条件、随机对象与飞行表情，维持简化近似。", "表现钩子未实现可见效果", [ref(R, 1354), ref(P, 568)])
add("L10", "surfaces", "card.css", "定义卡牌字体及 .card.phantom 的幻影背景外观。", "原生卡牌组件有通用样式；未导入该 CSS，也无与 cardPhantom 配置连通的幻影类。",
    "无规则效果；通用卡面不覆盖上游 phantom 效果，保留待界面对照。", "CSS/显示面保留简化", [ref(B, 1)], keep=True)
add("L10", "surfaces", "extension.css", "表情容器、目标、花蛋/酒壶飞行动画及扩展菜单样式。", "仅调试面板展示 throwEmotionCue 文本，未接原表情动画 DOM/CSS。",
    "无规则效果；cue 文本是示意，尚无上游花蛋飞行动画与菜单外观对照。", "CSS/显示面保留简化", [ref(P, 568), ref(R, 1354)], keep=True)

# L28: parser/query helpers have no product application.
l28 = {
    "gameSpeed": ("小游戏中移动速度系数，默认 4；实际移动用 24+(值-4)*2。", "解析 game_speed，未接小游戏移动循环。", "无规则效果；速度参数仅解析，不能改变产品中的小游戏移动速度。"),
    "replaceCha": ("把目标基础武将实际技能列表替换为 th_* 技能。", "replacement_plan 只生成替换对；前端调试态固定两对，未应用上游完整替换名单到引擎目录。", "此项改变规则技能；仅替换计划/调试示意，未接实际角色目录替换。"),
    "hidePack": ("删除/隐藏扩展角色包的选择列表，同时保留角色技能供其他场景使用。", "解析 hide_pack 并有查询 getter；无选将目录消费者，调试文字不影响实际选将。", "属于目录可见性，不是无规则技能；开关未消费，包隐藏行为未接通。"),
    "UIItroduc": ("设置页操作项展示 UI 接口说明 HTML、示例和使用方法。", "仅 intro_enabled/标签等配置数据，没有上游帮助 HTML 面板调用。", "无规则效果；帮助操作仅登记，未提供该说明界面。"),
    "yuqiTheme": ("控制隅泣选牌对话框的主题背景/外观。", "resolve_themes 可返回主题名；没有牌桌选牌对话框消费。", "无规则效果；主题解析不等于对话框主题生效。"),
    "timeTheme": ("控制小游戏/技能进度条主题素材与样式。", "resolve_themes 返回主题名；无进度条组件消费。", "无规则效果；未驱动上游主题进度条。"),
    "standPath": ("选择立绘目录，供隅泣等界面读取人物立绘资源。", "stand_path 仅返回路径；无对应人物立绘加载调用。", "无规则效果；路径登记尚未连接到资源请求与对话框立绘。"),
}
for ident, (up, native, diff) in l28.items():
    add("L28", "configs", ident, up, native, diff, "规则配置未接通" if ident == "replaceCha" else "表现配置未消费", [ref(L+"thunder.rs", 142), ref(L+"thunder.rs", 177), ref(L+"thunder.rs", 201), ref(P, 525)])
add("L28", "surfaces", "thunder.css", "小游戏、选牌/立绘对话框、进度条与 th-* 动画样式。", "原生调试面板报告主题/两对替换状态，未连接上游对话框结构及主题 CSS。",
    "无规则效果；当前仅状态示意，小游戏与主题外观尚未对照。", "CSS/显示面保留简化", [ref(P, 525), ref(L+"thunder.rs", 201)], keep=True)

# L27 configuration is not the generic two-string workbench.
l27_configs = {
    "twotwo": ("2v2 模式专用选将改写，默认 true；原文不是任意模式的自由双将选择。", "仅解析 bool；selectL27DualGenerals 验证两个字符串并写 workbench，GeneralSelectPanel 仍 emit 单将 choose。", "选择流程/规则配置；调试双字符串未进入引擎双将分配，不能算 2v2 选将已实现。"),
    "gaibiaoji": ("部分武将技能/标记改写，默认 false。", "解析默认 true，未接对应技能/标记覆盖应用。", "不只是展示登记：既有默认值错误，又未消费开关来控制上游覆盖。"),
    "dianjibiaoji": ("点击标记打开牌库/展开标记牌展示。", "仅 dianjibiaoji 字段，无对应标记牌库界面消费。", "无规则效果；布尔登记未形成点击标记牌库的交互。"),
    "shoupaihuase": ("按花色分组显示清正等技能中可见的手牌。", "仅 shoupaihuase 字段；规则选择走通用 Choice，未读取此开关。", "无规则效果；花色牌组外观与开关未接牌桌显示。"),
    "feichuxuanxiang": ("改写废除装备栏的选项界面和提示。", "仅 feichuxuanxiang 字段，通用选项面板未读取。", "无规则效果；废除选项美化仅登记。"),
    "shuzianniu": ("把施法/数值技能改为带加减按钮的数字面板。", "仅 shuzianniu 字段，技能可用通用 Tag 选择，但不消费该选项。", "无规则效果；现有数字选择规则与此可开关美化不是同一件事，未实现配置驱动的数字按钮外观。"),
    "xuanzejineng": ("改写获得技能的选择窗口，展示技能信息与选择按钮。", "仅 xuanzejineng 字段，没有配置驱动的技能选择窗口。", "无规则效果；保留通用技能选择不代表此窗口美化已接通。"),
    "zediewugu": ("五谷丰登选牌框的折叠/布局样式。", "仅 zediewugu 字段，公共选牌面板无消费。", "无规则效果；五谷折叠框仍未实现此开关对应布局。"),
    "jinxuanbiaoyu": ("布尔开关，默认 true；不能选择时弹出文字。", "原生却要求字符串 1/2/3，并将其解释为红叉/锁链三种样式，默认 1。", "无规则效果；配置类型和含义均与原始快照不符，bool true 会被拒绝；也没有产品显示消费者。"),
    "jinengtexiaoxiugai": ("技能特效改写（如神华佗），默认 true。", "仅 jinengtexiaoxiugai 字段，无相关特效消费。", "无规则效果；特效开关惰性登记，不能用技能本身已存在替代显示效果。"),
}
for ident, (up, native, diff) in l27_configs.items():
    add("L27", "configs", ident, up, native, diff, "规则配置未接通" if ident in {"twotwo", "gaibiaoji"} else "表现配置未消费", [ref(L+"select_beautify.rs", 98), ref(R, 515), ref("apps/web-client/src/components/battle/GeneralSelectPanel.vue", 110)])

l27_rules = {
    "mbcmqingzheng": ("出牌阶段开始可选自己一种手牌花色，再选其他角色及其一种花色；双方弃所选花色，原文伤害条件直接比较两数组。", "原生提供花色/目标选择和弃牌，伤害用双方张数比较。", "实际规则项；原文 :4259 比较 discardMeCards/discardHimCards 数组并要求后者 >0，原生按 len 比较，不能默认为同一条件。自定义清正牌组界面也未由配置驱动。", H+"skills_damage.rs", 453, False),
    "sbqingzheng": ("出牌阶段开始选择 3-治世 数量的自身花色与目标花色，双方弃牌；随后有独立的可选获得治世分支。", "原生花色选择后先弃自己牌，再选目标；只在自身弃牌数更多且造成伤害的分支询问治世。", "实际规则项；上游 :4827–4832 的治世可选分支不以造成伤害为前提，原生 :790 附着在 self_cnt>target_cnt 分支；弃牌顺序/数组伤害判据也不同。", H+"l27_batch_b_tw.rs", 710, False),
    "spyanjiao": ("出牌阶段一次，选择手牌花色，交出该花色所有牌给所选角色并造成 1 伤害，下回合开始按牌数摸牌。", "step_spyanjiao 有花色/目标 ask、给牌、伤害与 pending 摸牌消费。", "实际规则而非惰性 UI：已有交牌、伤害、延迟摸牌及玩家选择路径；本轮只澄清分类，未重新核验完整规则。自定义花色面板的外观仍由简化 CSS 项承接。", H+"l27_batch_b_tw.rs", 815, True),
    "twxiafeng": ("出牌阶段开始可选消耗至多 3 点暴虐值，给接下来若干张牌添加黠凤修饰；回合结束清理。", "step_twxiafeng 询问消耗数并写暴虐值/剩余次数，修饰器读取对应状态。", "实际规则项：消耗暴虐值与牌的后续修饰均有执行落点，不是无规则效果的数字 UI 登记；本次不重新签署全部修饰条件。", H+"l27_batch_b_tw.rs", 22, True),
    "twxingzhui": ("出牌阶段一次，选择 1–3 回合施法并失去体力；倒计时结束亮 2X 张牌，选目标获得黑牌，足量则受 X 雷伤。", "step_twxingzhui 有回合数/目标 ask、失去体力、亮牌区、黑牌转移和雷伤。", "实际规则项：施法数值与目标选择进入状态机，并非纯显示钩子；保留旧规则状态，本轮不把通用 Choice 当作自定义数值面板对照通过。", H+"l27_batch_b_tw.rs", 56, True),
    "xingchong": ("轮开始选择摸牌数并展示若干手牌；展示牌带公开标记，失去后按数量摸两倍牌。", "step_xingchong 选择牌后只设置 xingchong_card flags，没有对应 showCards/公开展示输出。", "实际规则项；牌选择及补牌有实现，但所选牌的公开展示/标记没有可见消费，不能以内部 flags 代表上游公开信息展示。", H+"l27_batch_b_tw.rs", 144, False),
    "dcxiaoxi": ("出牌阶段开始强制选择失去 1–2 上限，选目标及夺取牌/视为使用相同数量【杀】。", "有数量、目标、模式 ask；夺牌固定取前 X 张，杀分支直接发 Damage。", "实际规则差异：原文 :6882 调用 useCard(sha)，原生 :260–265 直接伤害，跳过使用杀、响应和卡牌触发；并非 UI 简化。", H+"l27_batch_b_tw.rs", 217, False),
    "twfengji": ("选一张牌置为扩展牌与 1–3 回合施法；完成时按原牌名字从牌堆取得至多 X 张同名牌，不足不补。", "按 CardKind 匹配，先牌堆再弃牌堆取，不足又从牌堆普通摸牌补足。", "实际规则差异：原文 :7016–7023 按 card.name 查找且不足停止，原生 :288–325 改为类型匹配/弃牌堆/补足，结果并非仅外观区别。", H+"l27_batch_b_tw.rs", 273, False),
    "twharvestinori": ("出牌阶段一次，弃一黑色手牌并选 1–3 回合施法；每个回合结束倒计时，完成摸 2X 张。", "有选黑牌/数值 ask、弃牌、phaseEnd 倒计时及 draw_direct(2X)。", "实际规则项：成本、施法数值与延迟摸牌均有状态机，不应写为惰性显示登记；保留旧状态不代表本次重新复核所有取消/恢复分支。", H+"l27_batch_b_tw.rs", 366, True),
    "twzhouhu": ("出牌阶段一次，弃一红色手牌并选施法回合，完成回复 X 体力。", "有选牌/数值 ask、弃牌、倒计时及 recover(X)。", "实际规则项：回复并非 UI 色调；有规则执行落点，本轮仅澄清条目性质。", H+"l27_batch_b_tw.rs", 408, True),
    "twzuhuo": ("弃一张非基本手牌/装备牌施法，完成获得 X 次阻祸标记，每次伤害前取消一次伤害并减标记。", "有非基本牌/施法 ask 与 shields 状态，伤害修饰消费阻祸次数。", "实际规则项：有付成本、倒计时和伤害抵消，不是计数式 UI 钩子；不在本次静态分类审查中重新签署所有伤害分支。", H+"l27_batch_b_tw.rs", 450, True),
    "twzhouzu": ("选择他人和施法回合；完成时由目标自己选弃 X 张牌，然后受 1 雷伤。", "选施法者目标/回合数，但结算直接 take(X) 目标手牌/装备并弃置，再伤害。", "实际规则差异：原文 :7542 为目标 chooseToDiscard，原生 :510–519 固定弃前 X 张且范围含装备，未给目标上游弃牌选择。", H+"l27_batch_b_tw.rs", 496, False),
}
for ident, (up, native, diff, file, line, keep) in l27_rules.items():
    add("L27", "ui_hooks", ident, up, native, diff, "实际规则，保留旧状态" if keep else "UI分类内实际规则差异", [ref(file, line)], keep=keep)
add("L27", "surfaces", "e.css", "2v2 选将、清正花色牌组、数字面板、施法标记等定制窗口样式。", "applyL27SelectBeautifyTheme 只返回 l27-select-beautify 类名且无产品调用/对应样式；调试面板可写双将草稿。",
    "无规则效果；通用 Choice 可承载规则选择，但没有该 CSS 对应的定制外观，维持简化近似。", "CSS/显示面保留简化", [ref(R, 1494), ref(P, 364)], keep=True)

add("L19", "configs", "zuan_brawl", "控制接入祖安乱斗入口：off 关闭，standard 三项、random 随机三项、all 全部；daily 跨日保存随机活动与新标记，需祖安扩展开启。", "doomsday.rs 仅解析五枚举，无产品消费者据此注册/过滤乱斗入口或保存每日活动。",
    "属于玩法目录/活动选择配置，不是武将禁用；仅枚举解析，未实现入口选择、祖安依赖与 daily 持久化。", "目录/活动配置未消费", [ref(L+"doomsday.rs", 225)], extra_upstream=[ref(U+"末日乱斗/extension.js", 40), ref(U+"末日乱斗/extension.js", 2034)])

# L29 locations are the original minified file, line 1 plus character offsets.
add("L29", "configs", "shengjian", "默认 true；条件注册 _CMeffects，为手杀曹髦创建剑、道心进度和技能区域特效。", "longxue.rs 仅解析 bool；前端初始 shengjian 固定 true，按钮可手动修改；没有从安装包配置读值。",
    "无规则效果；该开关仅登记且未控制实际剑特效。BattleBoard 只有 active 类绑定，未发现对应 CSS 剑/进度条，不能以默认状态示意等同启停特效。", "表现配置未消费", [ref(L+"longxue.rs", 40), ref(R, 870), ref(B, 344)])
add("L29", "ui_hooks", "_CMeffects", "gameStart/dieAfter 更新 cmarea/jian/技能区 DOM，展示手杀曹髦圣剑与道心变化。", "triggerLongxueHolySwordEffect 只返回 cue/修改 bool，未找到产品调用方；调试面板显示道心文本。",
    "无规则效果；现为道心/布尔状态示意，未实现圣剑及技能区动画，维持简化近似。", "表现钩子未实现可见效果", [ref(R, 1511), ref(P, 525), ref(B, 344)])
add("L29", "surfaces", "main1", "技能控制插件覆盖 skills/skills2/skills3，创建主动、触发、特殊主动/触发四区，随加减技能/觉醒/使命更新按钮状态。", "当前牌桌有通用技能按钮；未接该四区插件结构和动态更新协议。",
    "无规则效果；通用技能入口不代表四区技能控制、限定/使命按钮状态已对照，维持简化近似。", "CSS/显示面保留简化", [ref(B, 1)], keep=True)
add("L29", "surfaces", "main1.css", "四区技能按钮布局/渐变及 mbjuejin 等技能的选中、禁用视觉状态。", "未移植技能控制插件 DOM，通用按钮没有该选择器结构。",
    "无规则效果；尚无专属技能按钮外观与状态对照，维持简化近似。", "CSS/显示面保留简化", [ref(B, 1)], keep=True)

# L20 configuration: rule toggles do not automatically enable the bound skills.
l20_config = {
    "jinengxiugai": ("重启后修改部分技能和描述；实际分支包括 wengua5 等技能覆盖。", "技能覆盖开关只解析为字段，未控制原生技能目录/绑定。", "规则配置未接通"),
    "biaojixiugai": ("修改部分角色和国战标记显示/介绍。", "仅字段登记；没有标记替换/显示消费者。", "表现配置未消费"),
    "zhijiechifa": ("调整苦肉与重铸时机/内容，包含真实规则覆盖。", "仅字段登记；kurou、_chongzhu 有独立实现，但不由此 bool 开关应用覆盖。", "规则配置未接通"),
    "doudizhu_guanqiuxiaoguo": ("斗地主光球、入场官阶/国标与标题效果。", "仅字段登记，相关钩子只置 UI 标记。", "表现配置未消费"),
    "huajiatexiao": ("整肃、护甲特效与音效；原始 _shiqihujia_ 本身是注释块。", "仅字段登记，无护甲音效/动画消费者。", "表现配置未消费"),
    "guozhanmeihua": ("国战建国、军令、标记与 HP 样式的美化改写。", "仅字段登记；国战规则钩子独立绑定，未按此配置应用 UI。", "表现配置未消费"),
    "shoushajishatexiao": ("击杀连杀、癫狂无双等全局动画/音效。", "仅字段登记，连杀只留计数，没有动画/音频调用。", "表现配置未消费"),
    "shiyongkapaitixiao": ("杀闪桃酒等卡牌使用的特效。", "仅字段登记，相应钩子只有 flag。", "表现配置未消费"),
    "youxishengfu": ("隐藏设置项：关闭/十周年身份结算/手杀结算界面，默认关闭。", "解析枚举，无结算面板主题消费。", "表现配置未消费"),
    "daojianfu": ("刀剑斧动画风格 shousha/4.5/off，默认 4.5。", "解析枚举，无刀剑斧动画组件消费。", "表现配置未消费"),
    "daojianfu_type": ("随机/无/刀/剑/斧指定动画类型，默认 random。", "解析枚举，不驱动随机/指定动画。", "表现配置未消费"),
    "xinshadoudizhu": ("启用兵临城下斗地主界面美化，默认 false。", "仅字段登记，未切换实际斗地主界面。", "表现配置未消费"),
    "xlfbmoshi": ("启用 gameStart 时体力上限加当前体力并回复同量。", "已有 XiguaXueliangFanbei 规则实现，但配置解析器没有产品应用方来启停技能。", "规则配置未接通"),
    "mpfbmoshi": ("启用摸牌阶段额外摸 2 张（原文不是按任意摸牌数乘 2）。", "已有抽牌 +2 消费，受技能启用状态控制，不读此配置。", "规则配置未接通"),
    "zhujizhongwang": ("身份模式中主公死亡时由忠臣/明忠继承身份。", "已有 ZhuWanzhongJi 规则实现，但此开关未接应用/模式过滤。", "规则配置未接通"),
}
for ident, (up, native, cls) in l20_config.items():
    diff = ("有真实规则作用；" if cls == "规则配置未接通" else "无规则效果；") + native + "安装包配置到产品对局的应用入口未接通，不能把类型解析当作开关已生效。"
    add("L20", "configs", ident, up, native, diff, cls, [ref(L+"mark_supplement.rs", 234), ref(H+"l20_batch_a.rs", 525)])

l20_presentation = {
    "_xigua_feiyanbahu_": "进入游戏/gameStart 时显示飞扬跋扈、官阶/国标标题与光球。",
    "_guozhanChooseControlCHG": "本机国战选择/弃牌/用牌控制窗口前后调整背景遮罩透明度。",
    "_guozhan_diebiaoji_": "国战角色阵亡时删除其标记 UI 节点与背景。",
    "_guozhan_duiweibiaoji_gua_": "国战势力/对位标记的初始化和位置布局。",
    "_guozhan_duiweibiaoji_gua_gua_": "国战对位标记随身份/角色变化刷新布局。",
    "_guozhanhalfchghp": "国战开局临时调用 HP 样式点击处理器，保留原配置以兼容半血显示。",
    "_shoushajisha_": "记录击杀者连杀，按击杀数播放首杀/双杀等声音和动画。",
    "_shoushajisha_delete_": "角色死亡时清除该角色的连杀计数。",
    "_diankuangtulu_gua": "source damageBegin4 且伤害恰为 3 时显示癫狂屠戮表演及语音。",
    "_wanjunqushou_gua": "source damageBegin4 且伤害至少 4 时显示万军取首动画/语音。",
    "_o_o_recovertrigger": "recoverEnd 记录自身回合自回累计 3 点、或为他人回复达到其当前体力的累计次数；达阈值发自定义表现事件并定时清理。",
    "_o_o_miaoshouhuichun": "每轮救助其他角色达到三次的 o_omiaoshouhuichun 事件播放动画与语音。",
    "_o_o_yishugaochao": "自身回合自回累计三点触发 o_oyishugaochao，播放动画与语音。",
    "_losehp_": "失去体力事件播放对应音效。",
    "_mianshang_": "伤害被防止/降为零时显示免伤提示特效。",
    "_jiubuff_": "酒生效后给角色添加酒覆盖层。",
    "_jiubuff_xiaoshi_": "酒加成失效/结算后移除酒覆盖层。",
    "_player_game_guagua": "回合/阶段变化时在角色处展示阶段标签与动态节点。",
    "_guozhan_quanjubiaoji_guagua": "国战全局标记变动时更新对应 UI/提示音。",
    "_shiyongkapaitexiao_": "按所用杀闪桃酒等牌创建专属动画并播放语音。",
    "_shiyongkapaitexiao_xi_gua_": "用牌指定目标时按牌种/对象绘制专属指示线与命中特效。",
    "_shanghaipaishiyongtexiao_": "伤害牌使用时播放特定卡牌展示效果。",
    "_player_daojianfu_xxgg_fuzi_": "按刀剑斧风格、类型和伤害事件显示命中动画及声音。",
    "_binglintu": "斗地主兵临城下开局创建角色周边和中央界面的背景/按钮层。",
    "_dysp": "开局调用手杀特效播放视频/动态表演。",
}
for i, letter in enumerate("ABCDEFGH", 1):
    condition = f"event.num > {i} 且 <= {i+1}"
    if letter == "F":
        condition = "原文字面 event > 6 且 event.num <=7（不是 event.num >6）"
    l20_presentation["_player_baoji"+letter] = f"{condition} 时显示对应暴击字样/音效；保留原文条件，不推定更高伤害档。"
for ident, up in l20_presentation.items():
    native = "统一 handle_l20_batch_a 分支仅设 l20_ui_hook:<id>/invoked；没有对应 DOM、动画、音频事件。"
    if ident == "_shoushajisha_":
        native = "原生额外递增 _shoushajisha_count；其余只置 UI flag，没有连杀语音/动画。"
    if ident == "_shoushajisha_delete_":
        # This specific behavior is only counter cleanup in both versions.
        add("L20", "ui_hooks", ident, up, "原生将 _shoushajisha_count 清零并留惰性 UI flag。",
            "无规则效果；仅清理连杀展示计数，原生存在同类清零落点。此条不代表 _shoushajisha_ 的连杀动画/语音已实现。", "无规则效果，计数消费", [ref(H+"l20_batch_a.rs", 538)], keep=True)
        continue
    add("L20", "ui_hooks", ident, up, native, "无规则效果；" + native + "上游的可见/可听表现与条件尚未接通。", "表现钩子未实现可见效果", [ref(H+"l20_batch_a.rs", 525)])

l20_real = {
    "diaohulishan": ("临时离场期间取消伤害/失血/回复并禁止用牌、救援、被指定，另创建离场背景。", "原生有伤害与用牌/目标修饰及清理落点；可见背景属于仍简化的 CSS 显示面。", H+"l20_batch_a.rs", 682),
    "_zhulianbihe_mark_tao": ("消耗珠联璧合/野心家标记，作为桃用于合法使用/救援。", "有确认 ask、标记消耗与恢复目标执行。", H+"l20_batch_a.rs", 834),
    "_yinyang_mark_add": ("弃牌阶段开始可消耗阴阳鱼/野心家标记，临时增加 2 手牌上限并在回合结束清理。", "有确认 ask、标记消耗、上限加成状态和清理。", H+"l20_batch_a.rs", 853),
    "_aozhan_judge": ("国战满足人数、非同盟/配置条件时进入鏖战并授予 aozhan；另有提示 UI。", "已有鏖战判定及技能启用消费；特效提示仍由未完成显示面承接。", H+"l20_batch_a.rs", 870),
    "_xigua_xueliangfanbei_": ("gameStart 上限增加当前体力，并回复当前体力。", "HostBehavior::XiguaXueliangFanbei 实际改 max_hp 并 recover。", H+"skills_recovery.rs", 40),
    "_xigua_mopaifanbei_": ("phaseDrawBegin2 为摸牌数增加 2。", "抽牌内核读取该启用技能并给阶段摸牌 +2。", K+"kernel/draw.rs", 307),
    "_xigua_zhuwanzhongji_": ("主公 dieBegin 时忠臣/明忠继承主公，更新身份与 zhuUpdate。", "ZhuWanzhongJi 有身份继承落点。", H+"skills_status.rs", 31),
    "kuanshi2": ("宽释保护目标大额伤害前防止一次，给保护者留下跳过摸牌限制并定时清理。", "handle_kuanshi2 防止伤害、清理保护、设置 skip_next_draw。", H+"l20_batch_b_ext.rs", 748),
    "mbmowang": ("十常侍休整/重返与死亡窗口、弃区域牌、恢复状态及组合轮换。", "有休整/死亡取消、弃区域牌、回复及组合状态机；不是纯角色头像效果。", H+"l20_batch_b_mouyi.rs", 745),
}
for ident, (up, native, file, line) in l20_real.items():
    add("L20", "ui_hooks", ident, up, native, "实际规则项；" + native + "本轮仅区分规则执行与显示，不重新签署完整条件/选择/存读档。配置是否启停另见对应 configs 行。", "实际规则，保留旧状态", [ref(file, line)], keep=True)
add("L20", "ui_hooks", "_guozhan_marks", "玩家选择消耗先驱/珠联璧合/阴阳鱼等标记，执行摸牌、回复或查看他人隐藏武将的不同分支。", "摸牌/回复和选择有实现；先驱查看分支仅写 xianqu_viewed_main/deputy 等 flags，没有观察者查看结果输出。",
    "实际规则/公开信息差异；不是纯 UI。上游先驱分支调用查看武将界面，原生标记未形成可消费的查看结果，不能用 invoked/viewed flag 证明玩家已看到武将。", "UI分类内实际规则差异", [ref(H+"l20_batch_a.rs", 720), ref(H+"l20_batch_a.rs", 773)])
add("L20", "ui_hooks", "_chongzhu", "玩家选择可重铸牌；普通牌摸一张；stone deck 和 spell_gold/silver 分支创建/替换对应牌类。", "当前仅取手牌弃置并 draw_direct(1)，没有 stone/spell 分支。",
    "实际规则项；普通重铸已有选择/弃牌/摸牌，但上游 :6263–6296 的 stone/spell 特殊分支未迁入，不能作为无规则效果钩子签署。", "UI分类内实际规则差异", [ref(H+"l20_batch_b_asks.rs", 740)])
for ident, up, native in [
    ("_shiqihujia_", "原始 :690–728 整段注释的失去护甲声音/覆盖动画，未注册活动钩子。", "映射保留，但实际处理只置 UI 标记。"),
    ("zuoxing", "原始 :6155–6204 逐行 //，拟议的佐幸锦囊选择与减上限覆盖未启用。", "L20 HostBehavior 分派明确不覆盖 zuoxing；仅旧来源列表/通用触发与 prompt 登记仍有名称，没有 L20 活动规则覆盖。"),
]:
    refs = [ref(H+"l20_batch_a.rs", 525)] if ident != "zuoxing" else [ref(L+"host_skill_bindings.rs", 568), ref(L+"decision34_pack9_data.rs", 283), ref(L+"decision34_pack9_triggers.rs", 261)]
    add("L20", "ui_hooks", ident, up, native, "无活动规则/显示效果；上游引用是注释，原生仅惰性登记。不能把注释内拟议行为计作已经实际触发的等价技能。", "上游注释，惰性登记", refs, keep=True)
for ident, up in {
    "extension.css": "入场、暴击、连杀、国战与各种角色覆盖/动画样式。",
    "group.css": "选将选中角色的特定势力/data-nature 光效与边框。",
    "mark.css": "酒状态、选牌对话框、牌名与 decadeUI-canvas 层级的样式。",
}.items():
    add("L20", "surfaces", ident, up, "applyL20MarkCssTheme 仅生成 CSS 字符串/theme 状态，无产品调用方；调试按钮只切换 patch 标志。",
        "无规则效果；未把样式注入实际牌桌或建立相应 DOM，调试 patch 标志不是界面对照证据。", "CSS/显示面保留简化", [ref(R, 1453), ref(P, 208)], keep=True)

# L25 menu/configuration: 68 original inventory keys, including two misclassified skills.
menu = {
    "informknow": "设置页长按说明与文字彩蛋菜单。", "about1": "设置页装饰分隔线及长按彩蛋。",
    "about2": "设置页祖安标题图片。", "blank1": "设置页留白项。", "blank2": "设置页第二留白项。",
    "Myfriend": "设置页好友/感谢图片。", "about4": "扩展首页末尾分隔线/文字菜单。",
    "falseedit": "菜单里的假编辑项，只显示不能编辑的文本彩蛋。", "free": "设置页动态表情标题。",
    "aboutZ": "设置页上端分隔线。", "settext": "祖安设置标题 HTML。", "aboutX": "祖安设置标题下分隔线。",
    "linea": "模式设置分组标题。", "lineb": "特效设置分组标题。", "linec": "美化设置分组标题。",
    "lined": "互动设置分组标题。", "linef": "增强设置分组标题。", "lineg": "音效设置分组标题。",
    "lineh": "更多设置分组标题。", "aboutH": "更多设置中的分隔线/长按彩蛋。", "about0": "开发工具下分隔线/文字彩蛋。",
}
for ident, up in menu.items():
    add("L25", "configs", ident, up, "原生 ZuanConfig 只建模四个替代选项，没有该菜单项/对应设置页消费。",
        "无规则效果；此项是菜单内容，原生未显示该标题/留白/图片/文字交互。不能用空解析或不存在规则影响称 UI 等价。", "菜单显示未实现", [ref(L+"zuan.rs", 16), ref(L+"zuan.rs", 73)])

l25_cfg = {
    "mode": ("八种身份玩法模式选择，改变游戏初始化、阵营/胜负与各模式技能。", "规则配置未接通", "不能用通用 mode 字段命中代替祖安八模式配置消费。"),
    "fun": ("关闭/大将英魂/群侠论剑/桃花落尽/同将之争/暗夜鬼城/元气等娱乐玩法选择。", "规则配置未接通", "部分相关技能独立存在，但选项未驱动玩法启停或初始化。"),
    "goodluck": ("只为本机玩家开启摸好牌/判定偏好的天选模式，实际会移动牌堆顶牌。", "规则配置未接通", "这是真实抽牌/判定规则，_haoyunlai 当前仅计数/光效 cue，不能称纯显示。"),
    "washhandcard": ("关闭/自动/手动手牌整理，按类型、花色、点数与属性排序。", "表现配置未消费", "通用手牌排序不读此选项，未还原自动/手动三个分支。"),
    "levelstrong": ("关闭/旧/新等级奖励；改变上限、使用次数、距离、抽牌等奖励。", "规则配置未接通", "等级/经验实现不能代替此三态奖励配置的消费。"),
    "upspeed": ("0 关闭等级系统，否则按 30%–200% 缩放经验获取。", "规则配置未接通", "未消费此比例与关闭值来控制升级路径。"),
    "relife": ("控制复活间隔的缩放，0 表示固定三回合，非零为比例。", "规则配置未接通", "没有把该参数接入死亡倒计时；共享 L19 复活处理是另一套行为。"),
    "balance": ("切换各玩法的平衡规则，涉及势力、奖励、模式伤害/恢复等分支。", "规则配置未接通", "通用 balance 标识不能证明祖安各玩法分支应用。"),
    "newover": ("切换祖安玩法的新胜负结算逻辑。", "规则配置未接通", "未连接各玩法的 game.over 判定分支。"),
    "bingletexiao": ("菜单多选开关组合写 ZUANbingletexiaoSET，控制兵乐、天气、濒死与模式覆盖效果。", "表现配置未消费", "只查同名 bool 会漏掉实际 localStorage 多选集合；原生没有该集合消费。"),
    "texiao": ("启停动态 GIF/全屏画面特效。", "表现配置未消费", "zuanUi 的若干 cue/flag 没有动画组件，也不读总开关。"),
    "zuancoveropacity": ("把 0–10 设置转换为覆盖图 opacity（乘 0.1）。", "表现配置未消费", "没有覆盖图组件读取透明度，也没有该归一化参数。"),
    "huihe": ("武将回合效果强度/显示参数，进入回合覆盖图创建流程。", "表现配置未消费", "没有用该设置绘制/调节回合覆盖层。"),
    "huiheopa": ("调节回合效果的尺寸/速度等表现参数。", "表现配置未消费", "没有消费该数值的回合动画循环。"),
    "newhuihe": ("选择旧版/新版不同回合亮化或闪光效果。", "表现配置未消费", "没有版本分支或回合效果渲染。"),
    "imgtime": ("回合图像特效持续时间，调用时除以 10。", "表现配置未消费", "没有特效计时器消费此值。"),
    "imgfps": ("图像特效帧率，传入 zuanhideBackground 动画。", "表现配置未消费", "没有该帧率驱动的动画消费。"),
    "fujiaset": ("依赖 coin 插件切换雪花特效并保存 snowFall，缺依赖则关闭。", "表现配置未消费", "没有雪花、coin 依赖判断及 snowFall 应用。"),
    "newui": ("选择不同 UI 立绘/覆盖图风格与尺寸，影响诸多图片素材路径。", "表现配置未消费", "通用 newui 文本命中不是祖安立绘风格消费者。"),
    "helasisy": ("关闭开局动画；false 时显示开局 splash。", "表现配置未消费", "前端 helper 固定 skipOpeningSplash=true，不读取此开关，且无 splash 消费组件。"),
    "perspective": ("角色立绘透视缩放。", "表现配置未消费", "普通 CSS perspective 不是祖安此布尔开关消费。"),
    "bizhi": ("菜单界面壁纸模式选择与保存。", "表现配置未消费", "没有菜单壁纸切换与保存该选项的界面。"),
    "wallpaper": ("对局场景壁纸模式选择，影响动态/角色相关背景。", "表现配置未消费", "activeWallpaper 仅数据状态，未读取原始选项，也无 Vue 背景消费者。"),
    "paperchoose": ("壁纸按指定角色/主公等依据选择。", "表现配置未消费", "没有按该依据解析角色并请求背景资源。"),
    "exwallpaper": ("特殊玩法采用专用壁纸。", "表现配置未消费", "没有各模式专用壁纸切换消费。"),
    "qiankuang": ("手牌边框的随机/指定样式。", "表现配置未消费", "没有手牌边框组件或样式选择消费。"),
    "backgroundttop": ("选择并保存开始界面背景/Logo 美化。", "表现配置未消费", "没有入口页 backgroundlogo 的消费路径。"),
    "zhishixian": ("选择多种攻击指示特效，保存配置并同步十周年 UI/替换 game.linexy。", "表现配置未消费", "前端 attackLineStyle 固定 guanjie 且未被 Vue 使用，未实现选项与指示线同步。"),
    "zhishiopi": ("攻击指示线透明度除以 10 后写入线条 DOM。", "表现配置未消费", "没有该透明度作用的线条 DOM 消费。"),
    "zuantype": ("选择新/旧等对话台词版本，控制对应事件钩子分支。", "表现配置未消费", "cue 字符串没有台词版本选择或真实播报。"),
    "ransay": ("0–100 概率控制互动发言/成为龙王的随机分支。", "表现配置未消费", "没有该概率控制的台词随机与聊天输出。"),
    "levelsay": ("升级/获得经验时发聊天播报。", "表现配置未消费", "没有读取该开关的聊天输出。"),
    "girlspeak": ("女角色发言时增添卖萌文本。", "表现配置未消费", "没有基于性别/开关追加文本的聊天消费。"),
    "zuanemotion": ("在特定关系/事件与概率条件下丢花送蛋等互动表情。", "表现配置未消费", "未实现配置驱动的随机对象、表情弹道与可见动画。"),
    "shenst": ("启用神将增强，实际覆盖 baonu、wumou 等的触发/选择和效果。", "规则配置未接通", "同名神将技能存在不代表此配置触发覆盖，也不能称无规则效果。"),
    "showboss": ("启用诸神降临相关将包/注册入口，影响角色可用列表。", "目录/活动配置未消费", "没有按该开关引入对应将包/角色入口。"),
    "hidessr": ("把祖安武将加 unseen/minskin 并从 characterPack 删除隐藏其选将包。", "目录/活动配置未消费", "正式包可安装不代表按此设置隐藏角色/将包。"),
    "delkey": ("给 key_/Key 势力动漫武将加 unseen 和 forbidai 标记。", "目录/活动配置未消费", "未应用该配置过滤玩家/AI 的将池。"),
    "hideide": ("身份局内隐藏身份显示，按 none/game/player 等选项作用。", "表现配置未消费", "没有读取此设置的身份标记隐藏逻辑；普通私有视图规则不是此 UI 配置。"),
    "audio_draw": ("屏蔽获得/失去/使用牌的纸牌音效，多个钩子的 filter 读取该值。", "表现配置未消费", "mutePaperAudio 是独立默认字段，未从该选项赋值，且 lastAudioCue 无真实播放消费者。"),
    "audio_start": ("选择 none 或几种开局声音，enterGame/gameDrawBefore 播放对应音频。", "表现配置未消费", "helper 固定 dump_start，未处理枚举/none，没有实际音频播放消费。"),
    "isPC": ("界面比例/PC 适配，点击调整 ui_zoom、保存 zuanpc 并影响多处布局分支。", "表现配置未消费", "没有该配置的 zoom/布局应用；通用响应式不等于选项生效。"),
    "windowHigh": ("桌面窗口宽高比例，执行 moveTo/resizeTo 并保存窗口配置。", "表现配置未消费", "没有桌面窗口命令或此选项到窗口大小的消费。"),
    "text": ("启用游戏自定义/调试工具，可换将、换位、改手牌并查看身份。", "规则/开发工具配置未接通", "enable_editor/offline_debug_only 是替代参数，调试 workbench 草稿没有将原文全场编辑操作应用到引擎。"),
}
for ident, (up, cls, specific) in l25_cfg.items():
    native = "ZuanConfig 只建模 enable_editor/rule_variant/offline_debug_only/taunt_pack；未发现该原始选项的原生配置字段/产品消费者。"
    prefix = "有真实规则/目录作用；" if cls.startswith(("规则", "目录")) else "无牌局规则效果；"
    add("L25", "configs", ident, up, native, prefix + specific + "原始配置目前未接通。", cls, [ref(L+"zuan.rs", 16), ref(L+"zuan.rs", 73), ref(Z, 73)])
add("L25", "configs", "ssr_luoyi", "这不是配置：裸衣锁定技禁用防具，锦囊/延时锦囊视为决斗，杀/决斗让对方额外响应一次。", "step_ssr_luoyi 只设 double_req/dmg_boost flags；伤害修饰直接令该技能的杀/决斗（及无牌伤害）+1，没有双响应消费。",
    "实际规则且清单分类错误；原生 +1 伤害不是上游双响应，禁防具与锦囊视为决斗也未见对应消费，建议下调后进入技能修复复核。", "配置误分类，实际规则差异", [ref(H+"l25_batch_d_ext.rs", 552), ref(K+"rvm_bridge/modifiers/d34_l25_mods.rs", 411)])
add("L25", "configs", "ssr_lieren", "这不是配置：烈刃出牌可将武器放到他人武将上；开局创建专属武器，回合/濒死回收并给对方闪响应。原文没有 usable:1。", "native 无武器时允许任意 he 牌，强制每回合一次；开局从牌堆取武器/装备代替创建专属武器，另有放牌/回收/响应状态机。",
    "实际规则且清单分类错误；牌型过滤、次数和开局专属武器产生方式不同，不能当作配置解析等价。", "配置误分类，实际规则差异", [ref(H+"l25_batch_e.rs", 192), ref(H+"l25_batch_e.rs", 499), ref(H+"l25_batch_e_ext.rs", 588)])
add("L25", "configs", "falsedelete", "整项位于 :51421–51445 块注释中，假删除彩蛋菜单未启用。", "没有活动删除功能；也没有重建此被注释菜单。",
    "无活动规则/显示效果；上游是假删除项的注释文本，原生没有删除行为。保留键用于历史定位，不能算实际配置功能或浏览器样本。", "上游注释，惰性登记", [ref(L+"zuan.rs", 16)], keep=True)

l25_sound = {
    "_extraphaseskill": "phaseBefore、audio>=4 时按手杀/十周年版本播放 phase.mp3。",
    "_extradieskill": "die、audio>=4 时播放对应版本 die.mp3。",
    "_extraturnoverskill": "原文 trigger 字面为 trunOverBefore（不是 turnOverBefore），audio>=4 播 turnover.mp3；不能静默修正该拼写。",
    "_extraloseskill": "loseAfter/addCardToStorage/loseAsyncAfter 满足存储条件与 audio>=4 时播放 common.mp3。",
    "_extragainskill": "gainBefore 排除入储存并检查 audio_draw/audio>=4 后播放 draw.mp3。",
    "_extraloseCardskill": "loseBefore/discardBefore 排除储存并检查 audio_draw/audio>=4 后播放 lose.mp3。",
    "_extracompareskill": "chooseToCompareBefore、audio>=4 按版本播放 compare.mp3。",
    "_extrauseskill": "useCardBefore/respondBefore，检查 audio_draw/audio>=4 后播放 use.mp3。",
    "_extrajudgeskill": "judgeBefore、audio>=4 播对应版本 judge.mp3。",
    "_extralinkskill": "linkBefore、audio>=4 按当前连环状态选择 link 音频。",
    "_comparesayskill": "chooseToCompareBefore、audio>=2 播 compare.ogg。",
    "_equipuseskill": "使用装备牌且 audio>=3 时播装备语音。",
    "_damsayskill": "damageBegin 且 audio>=3 时播受伤语音。",
    "_saveskill": "dying 且 audio>=3 时播濒死语音（不是回复事件）。",
    "_dieskill": "die 时原文还要求 get.type(event.card)=='equip' 与 audio>=3 才播死亡语音。",
    "_shanskill": "respond 时原文检查 event.card.name=='equip'（并非 shan），audio>=3 才播放对应语音。",
    "_useCardsayskill": "useCard、audio>=2 播通用音效；>=3 按杀/拆/无懈/万箭/桃/借刀等牌名与随机值追加语音。",
    "_phasebeginskill": "phaseBegin、audio>=2，按语音等级播放回合开始语音。",
    "_phaseendskill": "phaseEnd、audio>=2，按等级播放回合结束语音。",
    "_judgeskill": "judgeBegin、audio>=2 播 judge.ogg。",
    "_linkskill": "linkAfter、audio>=2 播 link.ogg。",
}
for ident, up in l25_sound.items():
    native = "后端统一钩子只计数；前端 dispatcher 若调用只写 lastAudioCue，没有 Vue/音频播放消费者。"
    if ident in {"_saveskill", "_dieskill", "_extrauseskill"}:
        native += "当前 envelope 只在通用回复/死亡/用牌事件分别分派三种 cue，没有上游完整 filter。"
    add("L25", "ui_hooks", ident, up, native, "无牌局规则效果；" + native + "原始条件/语音选取未接通，不能称音效已实现。", "表现钩子未实现可见效果", [ref(H+"l25_batch_a_ext.rs", 333), ref(Z, 93), ref(R, 1042)])
for ident, up in {
    "_extradamageskill": "damageBefore 定义仍在，但 filter 无条件 return false，普通对局不播放该重复伤害音效。",
    "_extradlosehpskill": "loseHpBefore 定义仍在，但 filter 无条件 return false，普通对局不播放流失音效。",
}.items():
    add("L25", "ui_hooks", ident, up, "后端仅计数；dispatcher 可写未消费的 sfx cue；_extradamageskill 还被通用伤害 envelope 额外分派。",
        "无规则/实际音频效果；上游主动禁用，原生仅惰性计数/未播放 cue。应保留关闭语义，不能以人工调用 dispatcher 的状态变化冒充上游可触发音效。", "上游主动禁用，惰性登记", [ref(H+"l25_batch_a_ext.rs", 333), ref(Z, 114), ref(R, 1042)], keep=True)
add("L25", "ui_hooks", "_zuanattacklinecheck", "本机 enterGame/gameDrawBefore 时同步十周年攻击线选项，替换 game.linexy 或恢复默认线。", "后端只计数；前端固定 attackLineStyle='guanjie'，无 Vue 消费，配置 zhishixian 未接通。",
    "无规则效果；仅固定状态示意，没有上游指示线函数替换/选项同步/实际绘制。", "表现钩子未实现可见效果", [ref(Z, 101), ref(H+"l25_batch_a_ext.rs", 333)])
add("L25", "ui_hooks", "_zuanboss", "boss 模式主公 dieBegin 且 exwallpaper 开启时替换特殊壁纸，依 boss 类型分支。", "dispatcher 只设 bossDefeatedBanner=true，未接包配置、Boss 条件或任何背景/banner Vue 消费。",
    "无规则效果；原文是特殊壁纸更新，当前一个未渲染 banner flag 不是相同显示行为。", "表现钩子未实现可见效果", [ref(Z, 135)])
add("L25", "ui_hooks", "_helasisyskill", "仅本机 enterGame/gameDrawBefore 按 audio_start 播声音，helasisy 未开启时创建开局动画。", "dispatcher 固定 skipOpeningSplash=true、lastAudioCue='dump_start'；无实际音频/splash 消费。",
    "无规则效果；未按两项配置和本机条件选择声音/开局动画，当前仅固定标志登记。", "表现钩子未实现可见效果", [ref(Z, 138)])
for ident, up, native, diff, file, line in [
    ("_anshaskill", "身份局主公 phaseEnd 处理暗杀倒计时、忠臣死亡/刺客暴露和胜负。", "共享 L19 处理在 Dying/Death 窗口重写杀主公者身份；前端只删隐藏座位标志。", "实际模式规则；上游主公回合结束的暗杀死亡/胜负并非死亡窗口身份继承，不能归类纯隐藏 UI。", H+"l19_batch_b_asks.rs", 319),
    ("_seadreammodeskill", "身份局单将主公 phaseBegin，按 balance 抽牌/骰数弃牌，特定结果暂时封禁技能/调整位置。", "共享 L19 在 Dying/Death 给敌人分阶段加上限/技能；前端只切换 seaDreamPhaseTint。", "实际模式规则；骰数、弃牌与封技时机不同，色调状态不能代替上游玩法。", H+"l19_batch_b_asks.rs", 290),
    ("_relifeskill", "身份局主公 roundStart 遍历死者，倒计时归零按 maxHP 复活，按等级向上取整摸牌。", "共享 L19 在 Dying/Death 消耗 lives，重置等级，4/4 复活摸四；前端只记 reviveCountdown。", "实际复活规则；roundStart 死者倒计时与死亡窗口固定 4/4、摸四不一致，不能称单纯 HUD。", H+"l19_batch_b_asks.rs", 219),
    ("_haoyunlai", "drawBegin 播音效；本机开启 goodluck 时从前最多 20 张牌挑高价值牌移到牌堆顶。", "被 is_l25_ui_hook 兜底为计数 handler；前端只有 luckyDrawShimmer/音效 cue，不改牌堆。", "实际抽牌规则被当 UI；缺少 goodluck 牌堆操作及条件，不能把 shimmer 标志当等价。", H+"l25_batch_a_ext.rs", 333),
]:
    add("L25", "ui_hooks", ident, up, native, diff, "UI分类内实际规则差异", [ref(file, line), ref(Z, 141), ref(L+"host_skill_bindings.rs", 545)])
add("L25", "ui_hooks", "_bingleBigScreen", "judgeBegin、texiao 开启且判定牌为指定延时锦囊时播放该牌名对应大屏 GIF/音效。", "后端计数；dispatcher 把所有此调用固定记为 shandian 覆盖/cue，没有渲染/播放消费者。",
    "无规则效果；没有实际大屏效果，且固定闪电不能覆盖原文按延时牌名分派。", "表现钩子未实现可见效果", [ref(Z, 162), ref(H+"l25_batch_a_ext.rs", 333)])
commented_hooks = {
    "_newBackgroundUse": "原始 :41317–41343 整段注释的用牌后动态壁纸刷新（注释内 filter 也恒 false）。",
    "_bingletexiaoshandian": "原始 :47983–48948 注释块中的闪电覆盖/动画旧实现。",
    "_bingletexiaohuoshan": "同一注释块内的火山覆盖/动画旧实现。",
    "_bingletexiaohongshui": "同一注释块内的洪水覆盖/动画旧实现。",
    "_bingletexiaofulei": "同一注释块内的浮雷覆盖/动画旧实现。",
    "_bingletexiaole": "同一注释块内的乐不思蜀覆盖旧实现。",
    "_bingletexiaobingliang": "同一注释块内的兵粮寸断覆盖旧实现。",
    "_thundertexiao": "同一注释块内的 phaseDrawBefore 判定区覆盖刷新旧实现。",
    "_thundertexiao2": "同一注释块内的 judgeBegin 天气 GIF/音效旧实现。",
}
for ident, up in commented_hooks.items():
    add("L25", "ui_hooks", ident, up, "后端仅计数，前端 dispatcher 只改无 Vue 消费的壁纸/覆盖/cue 数据；没有活动显示。",
        "上游原始定义不启用；原生只是惰性登记/可人工调用的未渲染状态。保留原状态仅说明无活动行为范围，不能算已迁移的上游显示效果。", "上游注释，惰性登记", [ref(H+"l25_batch_a_ext.rs", 333), ref(Z, 132)], keep=True)

config_chains = {
    "L10": {"asset": "packages/extensions/epic-cards-upstream-liuli/assets/config.json", "parser": "parse_epic_config", "file": L+"epic_cards.rs", "line": 76,
            "chain": "config.json -> parse_epic_config -> EpicConfig -> presentation()/shichangshi_plan() 返回值；产品未发现解析/计划调用；throwEmotionCue 仅调试文本。"},
    "L19": {"asset": "packages/extensions/doomsday-brawl-upstream-liuli/assets/config.json", "parser": "parse_zuan_brawl", "file": L+"doomsday.rs", "line": 225,
            "chain": "config.json/输入字符串 -> parse_zuan_brawl -> ZuanBrawlMode 枚举；未发现产品按枚举注册/筛选 brawl 或保存 daily。"},
    "L20": {"asset": "packages/extensions/mark-supplement-upstream-liuli/assets/config.json", "parser": "load_mark_supplement_config", "file": L+"mark_supplement.rs", "line": 234,
            "chain": "config.json -> load_mark_supplement_config -> MarkSupplementConfig；产品未发现应用此对象的调用；技能由 host_skill_bindings 独立绑定；applyL20MarkCssTheme 只生成未应用的样式。"},
    "L22": {"asset": "packages/extensions/elo-sanction-upstream-liuli/assets/config.json", "parser": "parse_elo_config", "file": L+"elo.rs", "line": 104,
            "chain": "config.json -> parse_elo_config -> EloConfig；elo_apply_engine_config 可写对局 scalar，但产品仅定义/导出；elo_on_choice_before 返回的等待时长未进入计时/聊天消费者。"},
    "L25": {"asset": "packages/extensions/zuan-upstream-liuli/assets/config.json", "parser": "parse_zuan_config", "file": L+"zuan.rs", "line": 73,
            "chain": "替代 config.json -> parse_zuan_config -> 4 个替代字段；原文 68 项中多数无模型；frontend zuanUi 未由原始选项赋值，也没有 Vue 消费。misclassified ssr_* 单独检查真实规则。"},
    "L27": {"asset": "packages/extensions/select-beautify-upstream-liuli/assets/config.json", "parser": "parse_select_beautify_config", "file": L+"select_beautify.rs", "line": 98,
            "chain": "config.json -> parse_select_beautify_config -> SelectBeautifyConfig；产品未发现对象应用；selectL27DualGenerals 仅 workbench，实际 GeneralSelectPanel 仍选单将。"},
    "L28": {"asset": "packages/extensions/thunder-upstream-liuli/assets/config.json", "parser": "parse_thunder_config", "file": L+"thunder.rs", "line": 142,
            "chain": "config.json -> parse_thunder_config -> ThunderConfig -> replacement_plan/resolve_themes/stand_path 返回数据；产品未发现应用调用；调试面板为固定简化状态。"},
    "L29": {"asset": "packages/extensions/longxue-upstream-liuli/assets/config.json", "parser": "parse_longxue_config", "file": L+"longxue.rs", "line": 40,
            "chain": "config.json -> parse_longxue_config -> LongxueConfig.shengjian；前端独立固定初值/手动调试值 -> BattleBoard active 类；无该类 CSS 圣剑/进度条。"},
}
consumer_sources = json.loads((BASE / "upstream-config-consumers.json").read_text())
consumer_sources = {r["key"]: r for r in consumer_sources}
for key, row in rows.items():
    row["recommendation_only"] = True
    row["observed_runtime_failure"] = False
    cls = row["classification"]
    if cls in {"UI分类内实际规则差异", "配置误分类，实际规则差异"}:
        row["difference_kind"] = "具体规则/公开信息分支差异"
        row["evidence_level"] = "静态已定位代码差异，尚未运行复现；须定向反例确认影响"
    elif cls in {"上游注释，惰性登记", "上游主动禁用，惰性登记"}:
        row["difference_kind"] = "上游不启用，原生惰性登记"
        row["evidence_level"] = "原始语法/过滤条件已确认；不声称执行了被禁用行为"
    elif cls == "实际规则，保留旧状态":
        row["difference_kind"] = "真实规则有消费，保留旧声明不重签"
        row["evidence_level"] = "已定位真实规则执行点；未完成新的逐分支等价复核"
    elif cls == "无规则效果，计数消费":
        row["difference_kind"] = "仅显示计数清理，已有消费"
        row["evidence_level"] = "静态清零落点匹配，不为相关特效背书"
    elif cls == "CSS/显示面保留简化":
        row["difference_kind"] = "CSS/显示面仍待对照"
        row["evidence_level"] = "静态显示链核查；无新的并排截图/运行复现"
    elif cls == "菜单显示未实现":
        row["difference_kind"] = "菜单模型和可见交互未接入"
        row["evidence_level"] = "原生模型与实际 UI 入口静态核查，尚未浏览器复现"
    elif cls.startswith(("规则", "目录")):
        row["difference_kind"] = "规则/目录配置应用链未闭合"
        row["evidence_level"] = "静态未发现配置应用链，作为待验证的下调建议，不断言运行时缺失"
    elif cls == "表现配置未消费":
        row["difference_kind"] = "表现配置仅解析/未驱动显示"
        row["evidence_level"] = "静态未发现产品消费，作为待验证的下调建议，不断言运行时缺失"
    else:
        row["difference_kind"] = "只有计数/cue/flag，未接可见/可听效果"
        row["evidence_level"] = "已读具体 handler/dispatcher 与 UI 入口；尚未浏览器复现"
    if row["category"] == "configs" and cls != "配置误分类，实际规则差异":
        row["consumer_chain"] = config_chains[row["source_id"]]["chain"]
        row["consumer_search_tokens"] = [row["id"], config_chains[row["source_id"]]["parser"]]
        asset = config_chains[row["source_id"]]["asset"]
        row["native_refs"].append(ref(asset, 1))
    elif row["source_id"] == "L25" and row["category"] == "ui_hooks":
        row["consumer_chain"] = "host_skill_bindings/zuan_data -> HostBehavior；常规 UI 分支 l25_batch_a_ext 只增计数 -> frontend dispatcher 写 zuanUi -> Vue 消费 0；共享模式规则已单独沿 L19 handler 检查。"
        row["consumer_search_tokens"] = [row["id"], "zuanUi", "lastAudioCue", "judgmentOverlay", "reviveCountdownBySeat"]
    elif row["source_id"] == "L20" and row["category"] == "ui_hooks":
        row["consumer_chain"] = "host_skill_bindings -> l20_batch_a/b 或独立规则 HostBehavior；真实规则与只置 l20_ui_hook flag 已分开；applyL20MarkCssTheme 无产品调用。"
        row["consumer_search_tokens"] = [row["id"], "applyL20MarkCssTheme"]
    else:
        row["consumer_chain"] = "见本项 native_refs 的具体实现/组件；原始配置消费另列 configs 行。"
        row["consumer_search_tokens"] = [row["id"]]
    row["consumer_search_evidence"] = "consumer-probes.json（581 个产品源码文件；排除测试/Spine；命中含定义/导出，不能单靠命中计为调用）"
    for hit in consumer_sources.get(key, {}).get("matches", []):
        if hit["text"].lstrip().startswith(("//", "/*", "*")):
            continue
        if row["source_id"] == "L25" and row["id"] == "mode" and "extension_祖安设置_mode" not in hit["text"]:
            continue
        candidate = ref(consumer_sources[key]["path"], hit["line"])
        if candidate not in row["upstream_refs"]:
            row["upstream_refs"].append(candidate)
        if len(row["upstream_refs"]) >= 3:
            break

assert set(rows) == set(items), {"missing": sorted(set(items)-set(rows)), "unexpected": sorted(set(rows)-set(items))}
result = {
    "audit_date": "2026-11-13", "baseline_head": snapshot["head"], "inventory_sha256": snapshot["inventory_sha256"],
    "source_versions": {"rules": 333, "NGSA": 107, "content_schema": 5, "ABI": 759, "WS": 8},
    "scope": "13 来源的 configs/ui_hooks/surfaces 224 项；静态源码/消费者审查，未实际触发；不修改公共清单或旧证据。",
    "status_policy": "有具体未消费配置、未实现显示或规则差异才建议下调；保留原状态不是新的等价签署。注释与 filter=false 单独澄清，不因未跑浏览器一律降级。",
    "consumer_chains": config_chains,
    "items": [rows[key] for key in items],
}
(BASE / "recommendations.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
(BASE / "findings.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")

for src in ["L10", "L19", "L20", "L22", "L25", "L27", "L28", "L29"]:
    selected = [r for r in result["items"] if r["source_id"] == src]
    counts = collections.Counter(r["status_action"] for r in selected)
    lines = [f"# {src} 非规则分类条目逐项静态审查", "", f"日期 2026-11-13；源码基线 `{snapshot['head']}`；规则/NGSA/schema/ABI/WS：`333/107/5/759/8`。", "",
             "本表仅为静态源码证据，未在上游或原生实际触发。保留旧状态的行不是新的等价签署；JSON 保留完整唯一 key。", "",
             "动作计数：" + "；".join(f"{k} {v}" for k, v in counts.items()) + "。", "",
             "| 类别/key 后缀 | 上游实际行为 | 原生消费/效果 | 差别建议 | 状态建议 | 源码依据 |",
             "| --- | --- | --- | --- | --- | --- |"]
    def cell(text):
        return text.replace("|", "\\|").replace("\n", " ")
    for row in selected:
        refs = row["upstream_refs"] + row["native_refs"]
        evidence = "；".join(f"`{r['path']}:{r['line']}`" for r in refs)
        lines.append("| " + " | ".join(cell(t) for t in [f"{row['category']} / {row['id']}", row["upstream_behavior"], row["native_consumption_or_effect"], row["difference"], row["status"] + "（" + row["status_action"] + "）", evidence]) + " |")
    (BASE / f"{src}.md").write_text("\n".join(lines) + "\n")

counts = {"items": len(rows), "original_status": dict(collections.Counter(v["original_status"] for v in rows.values())),
          "recommended_status": dict(collections.Counter(v["status"] for v in rows.values())),
          "actions": dict(collections.Counter(v["status_action"] for v in rows.values())),
          "classifications": dict(collections.Counter(v["classification"] for v in rows.values()))}
(BASE / "recommendation-counts.json").write_text(json.dumps(counts, ensure_ascii=False, indent=2) + "\n")
grouped = {}
for kind in sorted({v["difference_kind"] for v in rows.values()}):
    selected = [v for v in rows.values() if v["difference_kind"] == kind]
    grouped[kind] = {"all_items": len(selected), "downgrade_recommendations": sum(v["status_action"] == "建议下调" for v in selected),
                     "keys": [v["key"] for v in selected]}
source_counts = {}
for source in ["U06", "U07", "L06", "L22", "L29", "L10", "U01", "L28", "L27", "L19", "L20", "U08", "L25"]:
    selected = [v for v in rows.values() if v["source_id"] == source]
    source_counts[source] = {"audited_nonrule_category_items": len(selected),
                            "original_status": dict(collections.Counter(v["original_status"] for v in selected)),
                            "recommended_status": dict(collections.Counter(v["status"] for v in selected)),
                            "actions": dict(collections.Counter(v["status_action"] for v in selected))}
(BASE / "findings-summary.json").write_text(json.dumps({"counts": counts, "by_difference_kind": grouped, "by_source": source_counts}, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(counts, ensure_ascii=False))
