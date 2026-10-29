//! 触发体补全：具名技能结算器效果（effect resolve_*）。

use super::*;

pub(super) fn items() -> Vec<LspCompletionItem> {
    vec![
            LspCompletionItem {
                label: "effect resolve_gzjinfa".into(),
                kind: 15,
                detail: "Discard the selected card, then resolve equipment gift or target-card gain"
                    .into(),
                insert_text: "effect resolve_gzjinfa(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_wn_qifeng".into(),
                kind: 15,
                detail: "Use a virtual Sha whose damage equals cards lost this turn".into(),
                insert_text: "effect resolve_wn_qifeng(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcjue".into(),
                kind: 15,
                detail: "Resolve Jue's discarded-card-capped random virtual-card loop".into(),
                insert_text: "effect resolve_dcjue(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_ruanji".into(),
                kind: 15,
                detail: "Resolve Ruanji's Zhaowen or Jiudun branch".into(),
                insert_text:
                    "effect resolve_ruanji(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhengbi".into(),
                kind: 15,
                detail: "Resolve Cuimao's Zhengbi branch".into(),
                insert_text:
                    "effect resolve_zhengbi(${1:self}, ${2:event_source}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_fengying".into(),
                kind: 15,
                detail: "Resolve Cuimao's Fengying extra turn".into(),
                insert_text:
                    "effect resolve_fengying(${1:self}, ${2:597375372}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_qiaojian".into(),
                kind: 15,
                detail: "Resolve Fugan's Qiaojian simultaneous discard and next-turn branch".into(),
                insert_text:
                    "effect resolve_qiaojian(${1:self}, ${2:event_source}, ${3:0}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_xicha".into(),
                kind: 15,
                detail: "Resolve Fugan's Xicha hidden card watch".into(),
                insert_text:
                    "effect resolve_xicha(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_huzun".into(),
                kind: 15,
                detail: "Resolve Huzun's Zhantao or Anjing branch".into(),
                insert_text:
                    "effect resolve_huzun(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhugemengxue".into(),
                kind: 15,
                detail: "Resolve Zhugemengxue's Jichun or Hanying branch".into(),
                insert_text: "effect resolve_zhugemengxue(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_bailingyun".into(),
                kind: 15,
                detail: "Resolve Bailingyun's Linghui or Xiace branch".into(),
                insert_text: "effect resolve_bailingyun(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcshaowei".into(),
                kind: 15,
                detail: "Resolve Wei Guanyinping's Shaowei branches".into(),
                insert_text: "effect resolve_dcshaowei(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcdichou".into(),
                kind: 15,
                detail: "Resolve Wei Guanyinping's Dichou turn branches".into(),
                insert_text:
                    "effect resolve_dcdichou(${1:self}, ${2:0}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcdufeng".into(),
                kind: 15,
                detail: "Resolve Ling Cao's Dufeng costs and turn modifiers".into(),
                insert_text:
                    "effect resolve_dcdufeng(${1:self}, ${2:0}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcctjiuxian".into(),
                kind: 15,
                detail: "Resolve Chen Tai's Jiu Xian recast, Duel, and recovery".into(),
                insert_text:
                    "effect resolve_dcctjiuxian(${1:self}, ${2:self}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_bianyue".into(),
                kind: 15,
                detail: "Resolve Bian Yue's Bizu or Wuxie branch".into(),
                insert_text: "effect resolve_bianyue(${1:self}, ${2:self}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhupeilan".into(),
                kind: 15,
                detail: "Resolve Zhu Peilan's Cilv or Tongdao branch".into(),
                insert_text:
                    "effect resolve_zhupeilan(${1:self}, ${2:event_source}, ${3:event_card}, ${4:helper_skill_def}, ${5:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_zhangxiu".into(),
                kind: 15,
                detail: "Resolve DC SB Zhang Xiu's Fuxi or Haoyi branch".into(),
                insert_text:
                    "effect resolve_dc_sb_zhangxiu(${1:self}, ${2:event_player}, ${3:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_zhouyu".into(),
                kind: 15,
                detail: "Resolve DC SB Zhou Yu's Ronghuo or Yingmou branch".into(),
                insert_text:
                    "effect resolve_dc_sb_zhouyu(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_lusu".into(),
                kind: 15,
                detail: "Resolve DC SB Lu Su's Mingshi or Mengmou branch".into(),
                insert_text:
                    "effect resolve_dc_sb_lusu(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhangjian".into(),
                kind: 15,
                detail: "Resolve Zhangjian's damage rewrite or temporary skill grant".into(),
                insert_text:
                    "effect resolve_zhangjian(${1:self}, ${2:event_card}, ${3:grant_skill_def}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhugeruoxue".into(),
                kind: 15,
                detail: "Resolve Zhugeruoxue's Qiongying or Nuanhui sequence".into(),
                insert_text:
                    "effect resolve_zhugeruoxue(${1:self}, ${2:self}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbzhuguo".into(),
                kind: 15,
                detail: "Resolve capped hand adjustment and the optional follow-up Sha".into(),
                insert_text: "effect resolve_mbzhuguo(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbanxian".into(),
                kind: 15,
                detail: "Gain and use a Sha from the first hand-discard event".into(),
                insert_text: "effect resolve_mbanxian(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbjinzu".into(),
                kind: 15,
                detail: "Reveal selected hand cards and install the next-Sha helper".into(),
                insert_text:
                    "effect resolve_mbjinzu(${1:self}, ${2:event_source}, ${3:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_beige_configured".into(),
                kind: 15,
                detail:
                    "Resolve Beige with configurable Heart recovery and Diamond draw counts"
                        .into(),
                insert_text:
                    "effect resolve_beige_configured(${1:self}, ${2:event_player}, ${3:event_source}, ${4:event_amount}, ${5:3});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dingpan".into(),
                kind: 15,
                detail: "Draw, then resolve Dingpan's equipment branch and delayed damage".into(),
                insert_text: "effect resolve_dingpan(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_tanlu".into(),
                kind: 15,
                detail: "Resolve Tanlu's frozen HP-gap gift or damage branch".into(),
                insert_text: "effect resolve_tanlu(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dccunsi".into(),
                kind: 15,
                detail: "Grant Yongjue, draw for another target, then resolve Guixiu recovery"
                    .into(),
                insert_text:
                    "effect resolve_dccunsi(${1:self}, ${2:event_source}, ${3:guixiu_def}, ${4:yongjue_def}, 0);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_retieji".into(),
                kind: 15,
                detail: "Resolve 界铁骑's judgment, skill suppression, and gift branch".into(),
                insert_text: "effect resolve_retieji(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_new_reluoyi".into(),
                kind: 15,
                detail: "Resolve 界裸衣's top-three optional draw replacement".into(),
                insert_text: "effect resolve_new_reluoyi(self, ${1:buff_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_remoshi".into(),
                kind: 15,
                detail: "Place or retrieve 没矢 cards".into(),
                insert_text:
                    "effect resolve_remoshi(self, event_player, event_card, ${1:helper_skill_def}, ${2:action});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_xinyaoming".into(),
                kind: 15,
                detail: "Resolve 界邀名's per-turn target and action choices".into(),
                insert_text: "effect resolve_xinyaoming(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_rehuaiyi".into(),
                kind: 15,
                detail: "Resolve 界怀异's reveal, color discard, and target gains".into(),
                insert_text: "effect resolve_rehuaiyi(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_reanguo".into(),
                kind: 15,
                detail: "Resolve 界安国's minimum-metric branches and optional recast".into(),
                insert_text: "effect resolve_reanguo(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_beizhu".into(),
                kind: 15,
                detail: "Resolve 备诛's hand reveal, discard-or-Sha sequence".into(),
                insert_text: "effect resolve_beizhu(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbshanjia".into(),
                kind: 15,
                detail: "Resolve mobile 缮甲's draw, dynamic discard, and category rewards"
                    .into(),
                insert_text:
                    "effect resolve_mbshanjia(self, mark_count(self, ${1:main_skill_def}), ${2:distance_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_daoji".into(),
                kind: 15,
                detail: "Resolve mobile 盗戟's equipment gain, forced use, and weapon damage"
                    .into(),
                insert_text: "effect resolve_daoji(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_tw_daoji".into(),
                kind: 15,
                detail: "Resolve TW 盗戟's hand/equipment gain and category reward".into(),
                insert_text: "effect resolve_tw_daoji(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mobiledanshou".into(),
                kind: 15,
                detail: "Draw at zero targetings, otherwise discard that many to damage".into(),
                insert_text: "effect resolve_mobiledanshou(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_jsrgchiying".into(),
                kind: 15,
                detail: "Force in-range roles to discard; target conditionally gains basics"
                    .into(),
                insert_text: "effect resolve_jsrgchiying(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twfupan".into(),
                kind: 15,
                detail: "Draw by damage, give a card, then apply per-target Fupan state".into(),
                insert_text: "effect resolve_twfupan(self, event_amount);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twjiaohua".into(),
                kind: 15,
                detail: "Give the draw recipient an unused category from draw/discard".into(),
                insert_text: "effect resolve_twjiaohua(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twkujian".into(),
                kind: 15,
                detail: "Gift or settle tracked TW Kujian physical cards".into(),
                insert_text: "effect resolve_twkujian(self, event_source, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twruilian".into(),
                kind: 15,
                detail: "Track a role's round discard categories and gain matching cards".into(),
                insert_text: "effect resolve_twruilian(self, event_player, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twjianwei".into(),
                kind: 15,
                detail: "Start either direction of TW Jianwei's prepare-phase comparison".into(),
                insert_text: "effect resolve_twjianwei(self, event_player, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twtanfeng".into(),
                kind: 15,
                detail: "Resolve TW Tanfeng's discard, Sha, damage, and phase-skip branches"
                    .into(),
                insert_text: "effect resolve_twtanfeng(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzhibian".into(),
                kind: 15,
                detail: "Resolve TW Zhibian's comparison and win/loss branches".into(),
                insert_text: "effect resolve_twzhibian(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twyuyan".into(),
                kind: 15,
                detail: "Resolve TW Yuyan's gift-or-cancel current Sha target".into(),
                insert_text: "effect resolve_twyuyan(self, event_source, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twguoyi".into(),
                kind: 15,
                detail: "Resolve TW Guoyi's target branch and post-use replay".into(),
                insert_text:
                    "effect resolve_twguoyi(self, event_player, event_card, ${1:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_twchuhai".into(),
                kind: 15,
                detail: "Resolve TW Chuhai's mission success or gained-card discard".into(),
                insert_text: "effect resolve_twchuhai(self, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twyizhu".into(),
                kind: 15,
                detail: "Resolve TW Yizhu planting, use, or discard tracking".into(),
                insert_text:
                    "effect resolve_twyizhu(self, ${1:event_source}, event_card, ${2:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_twluanchou".into(),
                kind: 15,
                detail: "Reset Gonghuan holders or redirect current damage".into(),
                insert_text:
                    "effect resolve_twluanchou(self, ${1:helper_skill_def}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twxiawei".into(),
                kind: 15,
                detail: "Maintain Wei pile, Wangxing cost, or Qiongji damage".into(),
                insert_text:
                    "effect resolve_twxiawei(self, ${1:wangxing_skill_def}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twbianfuren".into(),
                kind: 15,
                detail: "Resolve TW Wanwei damage/end phase or Yuejian pile placement".into(),
                insert_text: "effect resolve_twbianfuren(self, ${1:event_player}, ${2:helper_skill_def}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twfeiyi".into(),
                kind: 15,
                detail: "Resolve TW Shengxi prepare/end or Kuanji transfer".into(),
                insert_text: "effect resolve_twfeiyi(self, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twchenzhen".into(),
                kind: 15,
                detail: "Resolve TW Muyue or Chayi state machines".into(),
                insert_text: "effect resolve_twchenzhen(self, ${1:event_player}, ${2:helper_skill_def}, ${3:record_skill_def}, ${4:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twqianju".into(),
                kind: 15,
                detail: "Equip a compatible shared-pile equipment card".into(),
                insert_text: "effect resolve_twqianju(self, ${1:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twqingxi".into(),
                kind: 15,
                detail: "Resolve TW Qingxi's first-Sha target choice".into(),
                insert_text: "effect resolve_twqingxi(self, ${1:event_player}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzaoli".into(),
                kind: 15,
                detail: "Resolve TW Zaoli discard, draw, and equipment refill".into(),
                insert_text: "effect resolve_twzaoli(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzhengjian".into(),
                kind: 15,
                detail: "Resolve TW Zhengjian setup, tribute, switching, and Zhongchi".into(),
                insert_text: "effect resolve_twzhengjian(self, ${1:event_player}, ${2:helper_skill_def}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_jiaohua".into(),
                kind: 15,
                detail: "Resolve mobile 教化's persistent category cycle and recipient gain"
                    .into(),
                insert_text: "effect resolve_jiaohua(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_luanqun".into(),
                kind: 15,
                detail: "Resolve mobile 乱群's private reveal and temporary Sha restrictions"
                    .into(),
                insert_text:
                    "effect resolve_luanqun(self, ${1:effect_skill_def}, ${2:direct_hit_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcqiaomeng".into(),
                kind: 15,
                detail:
                    "Choose a target card, discard it, then resolve 趫猛's equipment/trick branch"
                        .into(),
                insert_text:
                    "effect resolve_dcqiaomeng(self, event_card, ${1:helper_skill_def});".into(),
            },
    ]
}
