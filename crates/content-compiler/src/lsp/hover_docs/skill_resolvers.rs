//! LSP 悬停文档链：具名技能结算器（resolve_*）说明。

pub(in super::super) fn documentation(word: &str) -> Option<&'static str> {
    Some(match word {
            "resolve_pothaoshi" => {
                "Effect: action 0 optionally authorizes another living player to use/respond with the owner's current hand until action 1 clears all grants at the owner's next turn start. The first two borrowed uses that empty the owner's hand draw it to three. Requires Rule VM ABI 536.\n\n```\neffect resolve_pothaoshi(self, 0);\n```"
            }
            "resolve_spmiaolve" => {
                "Effect: action 0 creates up to two globally suit-distinct Mantianguohai cards in the owner's hand; action 1 lets the owner cancel, draw two, or gain one canonical zhinang. Requires Rule VM ABI 539.\n\n```\neffect resolve_spmiaolve(self, 0);\n```"
            }
            "resolve_pottuntian" => {
                "势屯田 effect: gain charge after a qualifying loss, grow the maximum and draw when full at turn start, or spend charge to distribute heart cards and trigger 势凿险 threshold rewards. Requires Rule VM ABI 543."
            }
            "resolve_potjixi" => {
                "势急袭 effect: reset/record current-turn card targets, then discard one card from the turn player and use an ignore-distance virtual Shunshou on selected recorded targets. Requires Rule VM ABI 543."
            }
            "resolve_potceduan" => {
                "势策断 effect: collect one hidden hand-card choice from every player in the selected target's attack range, reveal them together, and use all owner hand cards of every tied majority color as an unlimited physical Sha. Requires Rule VM ABI 544."
            }
            "resolve_pottaishici" => {
                "势太史慈 effect: reset/collect 战烈 marks, resolve its enhanced virtual Sha, draw-to-X then duel for 酣战, and choose the limited 振锋 recovery or X-source rewrite. Requires Rule VM ABI 548."
            }
            "resolve_mbzhonghui" => {
                "势钟会 effect: spend/settle 俟资 charge, grow 挟志 limits, awaken 狱难, and upgrade 克昌. Requires Rule VM ABI 553."
            }
            "resolve_potjiejie" => {
                "势辛宪英诫节 effect: privately reveal the current actor hand, choose a suit, apply the matching discard/unlimited-use or draw-pile gain branch, and track the twice-per-round unique-maximum 清识 bonus. Requires Rule VM ABI 554."
            }
            "resolve_potqingshi" => {
                "势辛宪英清识 effect: choose any living player; allied participants each draw one, otherwise both discard one hand/equipment card. Requires Rule VM ABI 554."
            }
            "resolve_potweiyan" => {
                "势魏延 effect: resolve 忠傲 mission state, 壮誓 first-N-card helpers, 饮战 Sha follow-up, upgraded 狂骨, or 困奋. Requires Rule VM ABI 558."
            }
            "resolve_potzhouyu" => {
                "势周瑜 effect: resolve 炽沄 gifts, 焰洄 reveal history, 焚涛 linked fire-damage choices, or 雄姿 branch locking. Requires Rule VM ABI 561."
            }
            "resolve_potlvyi" => {
                "势吕壹 effect: resolve 隳律 tracked-card settlement, 颂谳 Wuzhong response conversion, or 势施 exact-card response locks. Requires Rule VM ABI 564."
            }
            "resolve_potcaoshuang" => {
                "势曹爽 effect: resolve 典易/奢权 timing assignment, persistent card tags, 渐专 awakening, and 溺尊 pile use. Requires Rule VM ABI 568."
            }
            "resolve_potyuji" => {
                "势于吉 effect: resolve 道转 basic-card conversion costs and 符济 tagged-card rewards. Requires Rule VM ABI 570."
            }
            "resolve_rehengjiang" => {
                "SP 横江 effect: stack the current turn player's temporary hand-limit penalty and settle the owner's draw at turn end. Requires Rule VM ABI 572."
            }
            "resolve_spyingjia" => {
                "Effect: actions 0/1 reset and record same-name trick uses; action 2 may discard one hand card and queue an extra turn for any living target, granting the supplied draw helper. Requires Rule VM ABI 539.\n\n```\neffect resolve_spyingjia(self, 2, 3612952432);\n```"
            }
            "resolve_nzry_longnu" => {
                "Effect: toggles Longnu, pays the matching HP/max-HP cost, draws one card, and grants the matching helper. Requires Rule VM ABI 213."
            }
            "resolve_drlt_poxi" => {
                "Effect: privately exposes the selected target hand, chooses exactly four distinct-suit cards across both hands, discards them, and resolves Poxi by the owner's selected-card count. Requires Rule VM ABI 214."
            }
            "resolve_drlt_jieying" => {
                "Effect: resolves Jieying's ensure-mark, optional transfer, or other-player turn-end hand collection mode. Requires Rule VM ABI 215."
            }
            "resolve_shen_simayi" => {
                "Effect: resolves Shen Simayi's discard-phase Renjie count, Lianpo extra-turn queue, or a Jilue Wansha/Zhiheng/Jizhi branch selected by mode. Requires Rule VM ABI 218."
            }
            "resolve_dcqiaomeng" => {
                "Effect: choose one other target of the current black card, then discard one of that target's hand/equipment cards. Gain discarded equipment; discarded trick cards make the current card use unrespondable to all players. Available only in the matching `card_targeted` trigger and requires Rule VM ABI 313.\n\n```\neffect resolve_dcqiaomeng(self, event_card, helper_skill_def);\n```"
            }
            "resolve_xinfu_guolun" => {
                "Active effect: the owner privately selects and publicly reveals one target hand card, then may cancel or publicly reveal one own hand/equipment card. A successful choice atomically exchanges both cards into the opposite hands; the original owner of the lower physical rank draws one card, while a tie draws none. Requires Rule VM ABI 294.\n\n```\neffect resolve_xinfu_guolun(self, event_source);\n```"
            }
            "resolve_twtianyu" => {
                "TW Tianyu resolver: Zhenxi discards up to distance hand cards and/or moves one target field card; Yangshi adds a permanent attack-range mark unless all others are already in range, then gains a Sha. Requires Rule VM ABI 382."
            }
            "resolve_twwujing" => {
                "TW Wujing resolver: Fenghan chooses up to the triggering card's target count to draw one each; Congji chooses red event cards still in discard and gives them to one other player. Requires Rule VM ABI 384."
            }
            "resolve_qianlong" => {
                "Effect: authoritative machine for 潜龙 — after damage, view the top three deck cards and gain up to lost-HP count, the rest go to deck bottom.\n\n```\neffect resolve_qianlong(self, self_skill);\n```"
            }
            "resolve_jsrgzuozhan" => {
                "Effect: authoritative machine for 坐瞻 — at game start choose up to two other players; gain an attack-range mark equal to min(their highest HP, 5) and mark them as 坐瞻 allies.\n\n```\neffect resolve_jsrgzuozhan(self, self_skill);\n```"
            }
            "resolve_jsrgzuozhan_gain" => {
                "Effect: authoritative machine for the 坐瞻 group face — after a marked ally dies, have one surviving 坐瞻 ally gain up to X basic cards of distinct names from the discard pile.\n\n```\neffect resolve_jsrgzuozhan_gain(self, self_skill, event_player);\n```"
            }
            "resolve_twmouzhi" => {
                "Effect: authoritative machine for 谋制 — mode 0 (damage_before): if the damage card's color matches the recorded last damage color, prevent the damage; mode 1 (damage): record the event card's color (red/black, cleared when colorless).\n\n```\neffect resolve_twmouzhi(self, self_skill, mode, event_card);\n```"
            }
            "resolve_twdaigui" => {
                "Effect: authoritative machine for 待归 (slice 1) — at play-phase end with a uniform-colored hand, choose up to hand-count players and reveal that many cards from the deck bottom (per-target picks follow in a later slice).\n\n```\neffect resolve_twdaigui(self, self_skill, stage, count);\n```"
            }
            "resolve_twmitu" => {
                "Effect: authoritative machine for 迷途 (slice 1) — at your prepare phase choose up to three players; each draws two cards tagged 迷途 and shows them publicly (compare rounds follow in a later slice).\n\n```\neffect resolve_twmitu(self, self_skill, stage, count);\n```"
            }
            "resolve_qiantun" => {
                "Effect: authoritative machine for 谦吞 — once per play phase, have another player show any number of hand cards (their only compare cards), then compare: if you win, take the shown cards; otherwise take their other hand cards.\n\n```\neffect resolve_qiantun(self, self_skill);\n```"
            }
            "resolve_yangming" => {
                "Effect: authoritative machine for 养名 — once per play phase, compare with one player; if the target wins, they draw once per prior loss and you recover 1; otherwise count a loss and you may repeat.\n\n```\neffect resolve_yangming(self, self_skill);\n```"
            }
            "resolve_canyun" => {
                "Effect: authoritative machine for 残殒 — discard one card and pick 1-2 other players; compare equipment counts: if you have more they recover 1, if equal they draw 1, if they have more they lose 1 HP; you draw 1 when a target is at 1 HP.\n\n```\neffect resolve_canyun(self, self_skill);\n```"
            }
            "resolve_hengwu" => {
                "Effect: authoritative machine for 横骛 — after using or playing a card whose suit is absent from your hand, draw one card per equipment of that suit on the field.\n\n```\neffect resolve_hengwu(self, self_skill, event_card);\n```"
            }
            "resolve_biaozhao3" => {
                "Effect: authoritative machine for 表召·归 — at your prepare phase with a 表 card present: discard it, then optionally have one player draw up to the field's largest hand count (max 5) and recover 1.\n\n```\neffect resolve_biaozhao3(self, self_skill);\n```"
            }
            "resolve_chuanyun" => {
                "Effect: authoritative machine for 穿云 — after your Sha targets a player with equipment, randomly discard one of their equipped cards.\n\n```\neffect resolve_chuanyun(self, self_skill, event_player, event_card);\n```"
            }
            "resolve_biaozhao2" => {
                "Effect: authoritative machine for 表召·应 — when a discard matches the 表 card's suit and rank, give the 表 card to the discarder; you lose 1 HP.\n\n```\neffect resolve_biaozhao2(self, self_skill, event_player);\n```"
            }
            "resolve_dcniji" => {
                "Effect: authoritative machine for 逆击 — mode 0 (targeted by a non-equipment card): draw a card and tag it 逆击; mode 1 (your end phase): discard all tagged cards from your hand.\n\n```\neffect resolve_dcniji(self, self_skill, mode, event_card);\n```"
            }
            "resolve_dcmoukui" => {
                "Effect: authoritative machine for 谋溃 — after your Sha targets a player, choose one or both: draw a card, or discard one of the target's hand cards.\n\n```\neffect resolve_dcmoukui(self, self_skill, stage, target);\n```"
            }
            "resolve_zhongyan" => {
                "Effect: authoritative machine for 忠言 — once per play phase, reveal the top three deck cards; a chosen player with hand cards exchanges one of them for one revealed card.\n\n```\neffect resolve_zhongyan(self, self_skill, stage);\n```"
            }
            "resolve_chumei" => {
                "Effect: authoritative machine for 狐魅 — during your play phase, choose a player whose HP is at most the damage you dealt this turn; they draw a card, give you a card, or recover 1 HP.\n\n```\neffect resolve_chumei(self, self_skill, stage);\n```"
            }
            "resolve_qiangyong" => {
                "Effect: authoritative machine for 羌勇 — when your Sha deals damage, discard X cards from the wounded player (X = Sha uses this turn); if they then have no hand cards this damage is increased by one.\n\n```\neffect resolve_qiangyong(self, event_player);\n```"
            }
            "resolve_twxiongjun" => {
                "Effect: authoritative machine for 雄军 — after the owner deals damage, every living player owning 雄军 draws one card.\n\n```\neffect resolve_twxiongjun(self);\n```"
            }
            "resolve_dcqiaolve" => {
                "Effect: authoritative machine for 巧略 — discard one of your cards first, then take a random hand card from the event target (cost-added completion).\n\n```\neffect resolve_dcqiaolve(self);\n```"
            }
            "resolve_yixiang" => {
                "Effect: authoritative machine for 义襄 — randomly gain a basic card from the deck that is not already in your hand (once per role turn when targeted).\n\n```\neffect resolve_yixiang(self);\n```"
            }
            "resolve_xinfu_qiai" => {
                "Effect: authoritative machine for 求援 — once per game, while dying, every other player in seat order hands you one hand/equip card (players with none are skipped).\n\n```\neffect resolve_xinfu_qiai(self);\n```"
            }
            "resolve_olwenren" => {
                "Effect: authoritative machine for 温仁 — once per play phase, pick any number of targets; each draws 1 if they have no hand cards and 1 more if their hand count is not greater than yours.\n\n```\neffect resolve_olwenren(self);\n```"
            }
            "resolve_spolaige" => {
                "Effect: authoritative machine for 霸业 — after the second dying window of the match, exchange 西向 for 朔北, draw up to your max HP, and recover up to your max HP.\n\n```\neffect resolve_spolaige(self);\n```"
            }
            "resolve_olzhijie" => {
                "Effect: authoritative machine for 直谏 — at any player\'s end phase, choose a player within your attack range and deal 1 damage to them.\n\n```\neffect resolve_olzhijie(self);\n```"
            }
            "resolve_olsuyi" => {
                "Effect: authoritative machine for 俗义 — once per play phase, search the deck for equipment the target can use, have them equip it, then use a virtual Sha against them.\n\n```\neffect resolve_olsuyi(self, event_source);\n```"
            }
            "resolve_olqiaozhi" => {
                "Effect: authoritative machine for 巧织 — discard one card, reveal the top two deck cards, and gain one of them (tagged 巧织; while you hold one you cannot activate again).\n\n```\neffect resolve_olqiaozhi(self, stage);\n```"
            }
            "resolve_dczhee" => {
                "Effect: authoritative machine for 拓跋? — on entry/game start search the draw pile, and at every end phase search the discard pile, for a Lightning to hang as your judge card.\n\n```\neffect resolve_dczhee(self, mode);\n```"
            }
            "resolve_dchaoxian" => {
                "Effect: authoritative machine for 朝献 — once per game, shuffle every rank-3 card from the discard pile back into the deck bottom and collect all rank-3 hand cards from other players.\n\n```\neffect resolve_dchaoxian(self);\n```"
            }
            "resolve_xiangshu" => {
                "Effect: authoritative machine for 襄戍 — once per game, a wounded target recovers X hp and draws X cards (X = min(5, damage you dealt this turn)).\n\n```\neffect resolve_xiangshu(self, event_target);\n```"
            }
            "resolve_nhlundao" => {
                "Effect: authoritative machine for 论道 — after taking damage, if the source is alive with a different hand count: draw one when your hand is larger, otherwise discard one of the source\'s cards.\n\n```\neffect resolve_nhlundao(self, event_source);\n```"
            }
            "resolve_sanshou" => {
                "Effect: authoritative machine for 三首 — reveal the top three deck cards; if any revealed card is of a type never used this match, the incoming damage is cancelled.\n\n```\neffect resolve_sanshou(self);\n```"
            }
            "resolve_starcuxia_weiming" => {
                "Effect: authoritative machine for 粗遐/未命 — the card user randomly discards a hand card; the owner draws one if the dominance condition still holds.\n\n```\neffect resolve_starcuxia_weiming(self, event_source, variant);\n```"
            }
            "resolve_zhongjie" => {
                "Effect: authoritative machine for 张介 — when you die, choose another player: they gain 1 max hp, recover 1 hp, and draw a card.\n\n```\neffect resolve_zhongjie(self);\n```"
            }
            "resolve_dingce" => {
                "Effect: authoritative machine for 定策 — after taking damage, discard one of your hand cards, then the damage source discards one (you choose it); same-color bonus is deferred (card absent from the pool).\n\n```\neffect resolve_dingce(self, event_source);\n```"
            }
            "resolve_meiyong" => {
                "Effect: authoritative machine for 姝勇 — when you use or play a Sha, take one card from another player\'s hand/equip/judge area, then they draw one.\n\n```\neffect resolve_meiyong(self);\n```"
            }
            "resolve_mbzhenann" => {
                "Effect: authoritative machine for 镇南 — when targeted by a trick whose target count exceeds max(1, the user\'s hp), discard one card and deal 1 damage to any character.\n\n```\neffect resolve_mbzhenann(self);\n```"
            }
            "resolve_friendbiwei" => {
                "Effect: authoritative machine for 鄙位 — discard the highest-rank hand card, then the target discards every hand card of rank >= it; if they discarded nothing the skill\'s use is refunded.\n\n```\neffect resolve_friendbiwei(self, event_source);\n```"
            }
            "resolve_chulao" => {
                "Effect: authoritative machine for 黜老 — discard one card, then for each selected target of a distinct group the owner picks one of their cards to discard; spade discards draw.\n\n```\neffect resolve_chulao(self);\n```"
            }
            "resolve_dcanxu" => {
                "Effect: authoritative machine for 暗蓄 — the player with fewer hand cards takes one card from the other; non-spade gain draws owner one, equalized hands recover owner one.\n\n```\neffect resolve_dcanxu(self);\n```"
            }
            "resolve_duoduan" => {
                "Effect: authoritative machine for 度断 — once per turn when targeted by a Sha, recast one card, then choose: the Sha user draws two and the Sha has no effect on you, or the user discards a card and you cannot respond.\n\n```\neffect resolve_duoduan(self, event_source, event_card);\n```"
            }
            "resolve_gaoyuan" => {
                "Effect: authoritative machine for 告援 — when targeted by a Sha, discard one card to redirect the Sha to a legal target holding a 诤 mark.\n\n```\neffect resolve_gaoyuan(self, event_source, event_card);\n```"
            }
            "resolve_mbbeiming" => {
                "Effect: authoritative machine for 孛明 — at game start choose up to two players; each gains a weapon from the deck whose attack range equals the number of suits in their hand.\n\n```\neffect resolve_mbbeiming(self, self_skill, stage);\n```"
            }
            "resolve_shuaiyan" => {
                "Effect: authoritative machine for 率言 — at your discard phase with more than one hand card, choose another player; they give you one card from their hand or equipment.\n\n```\neffect resolve_shuaiyan(self, self_skill, stage);\n```"
            }
            "resolve_zhengjian" => {
                "Effect: authoritative machine for 诤荐 — mode 0: at your end phase, grant a 诤 mark to one player; mode 1/2: a marked player's card use or play increments their mark; mode 3: at your turn begin, each marked player draws min(5, max HP, marks) and marks clear.\n\n```\neffect resolve_zhengjian(self, self_skill, stage, event_player);\n```"
            }
            "resolve_qinmian" => {
                "Effect: authoritative machine for 勤勉 — mode 0: at your prepare phase or after damage, draw that many cards tagged 勤勉; mode 1: at an end phase, optionally discard all tagged cards to damage the current turn player equal to their count (or 1 if your whole hand is tagged).\n\n```\neffect resolve_qinmian(self, self_skill, mode, amount);\n```"
            }
            "resolve_jsrgdaoren" => {
                "Effect: authoritative machine for 蹈仁 — give one hand card to another player, then deal 1 damage to every player inside both attack ranges.\n\n```\neffect resolve_jsrgdaoren(self, self_skill);\n```"
            }
            "resolve_duwang" => {
                "Effect: authoritative machine for 度望 — at game start, place the first five non-Sha cards from the deck top onto the own skill pile.\n\n```\neffect resolve_duwang(self, self_skill);\n```"
            }
            "resolve_quanjia" => {
                "Effect: authoritative machine for 拳甲 — randomly view (2+marks) cards of the damage source's hand; use a virtual Sha on them if any, else gain a mark.\n\n```\neffect resolve_quanjia(self, self_skill, event_source, event_player);\n```"
            }
            "resolve_pxqixian" => {
                "Effect: authoritative machine for 祁县 — recast one equip card as a virtual Sha or Guohe used on another player.\n\n```\neffect resolve_pxqixian(self, self_skill, 0, 0);\n```"
            }
            "resolve_dcbingji" => {
                "Effect: authoritative machine for 秉纪 — validates hand suit uniformity and per-turn suit history, then uses a virtual Sha or Tao.\n\n```\neffect resolve_dcbingji(self, self_skill, 0, 0);\n```"
            }
            _ => return super::skill_owner_filters::documentation(word),
    })
}
