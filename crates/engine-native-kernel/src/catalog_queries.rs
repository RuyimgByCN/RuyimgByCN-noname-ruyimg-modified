//! State-dependent card/skill/target queries on [`Catalog`].

use super::{CardDef, CardId, CardKind, Catalog, EventKind, SkillDefId, Suit, TriggerMeta};
use crate::ids::PlayerId;
use crate::state::GameStateCore;

impl Catalog {
    /// L20 标记补充 (`xinhongyan` / `hongyan`, `extension.js:1105-1135`, `shenhua.js:6887`):
    /// `mod.suit` (`Spade -> Heart` when `owner` holds non-blocked `xinhongyan` or `hongyan`).
    pub fn effective_card_suit(
        &self,
        core: &crate::state::GameStateCore,
        owner: crate::ids::PlayerId,
        card: CardId,
    ) -> Suit {
        let raw = self.resolved_card_suit(core, card);
        if raw == Suit::Spade
            && !self.is_chanyuan_sealed(core, owner)
            && ["xinhongyan", "hongyan", "红颜"]
                .iter()
                .any(|name| self.skill_by_name(name).is_some_and(|def| core.has_skill(owner, def)))
        {
            Suit::Heart
        } else {
            raw
        }
    }

    /// 缠怨（于吉 `chanyuan` / `rechanyuan`, `shenhua.js:7693-7724`, `refresh.js:10760-10783`）：
    /// 查询玩家是否持有 `chanyuan` / `rechanyuan`（或对应旗标）。
    pub fn has_rechanyuan(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.vm_aux
            .flags
            .get(&(p, "l10_rechanyuan".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || ["chanyuan", "rechanyuan"]
                .iter()
                .any(|name| self.skill_by_name(name).is_some_and(|def| core.has_skill(p, def)))
    }

    /// 缠怨生效门控：`hp <= 1` 时封锁自身所有其他非 `charlotte` 技能（含锁定技）。
    pub fn is_chanyuan_sealed(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.player(p).hp <= 1 && self.has_rechanyuan(core, p)
    }

    /// L10/L11 `chanyuan` / `rechanyuan` 封技门控（`refresh.js:10770`, `shenhua.js:7693-7724`）：
    /// `hp <= 1` 时除 `chanyuan`/`rechanyuan` 与 `charlotte` 外所有技能（含锁定技）均失效。
    pub fn is_rechanyuan_sealed_def(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
        def: SkillDefId,
    ) -> bool {
        let Some(skill) = self.skills.get(def as usize) else {
            return false;
        };
        self.is_chanyuan_sealed(core, p)
            && !matches!(skill.name, "chanyuan" | "rechanyuan")
            && !self.is_charlotte_skill(def)
    }

    /// [`Self::is_rechanyuan_sealed_def`] 的 `chanyuan` 别名。
    pub fn is_skill_chanyuan_blocked(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
        def: SkillDefId,
    ) -> bool {
        self.is_rechanyuan_sealed_def(core, p, def)
    }

    /// 按技能名查询是否被 `chanyuan` / `rechanyuan` 封印。
    pub fn is_rechanyuan_sealed_name(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
        name: &str,
    ) -> bool {
        if !self.is_chanyuan_sealed(core, p) || matches!(name, "chanyuan" | "rechanyuan") {
            return false;
        }
        if let Some(def) = self.skill_by_name(name) {
            !self.is_charlotte_skill(def)
        } else {
            !name.starts_with('_')
        }
    }

    /// Whether a skill is a locked skill (`锁定技` / `forced: true` / `charlotte: true`) exempt from `tieji` seal.
    pub fn is_locked_skill_name(name: &str, eff_meta: Option<&TriggerMeta>) -> bool {
        if matches!(
            name,
            "zhenyi_spade"
                | "xinfu_langxi"
                | "fz_new_longdan"
                | "fz_new_tieji"
                | "fz_liegong"
                | "fz_xinkuanggu"
                | "rekuanshi"
                | "kuanshi"
                | "kuanshi2"
                | "xinfu_lingren"
                | "sbtieji"
                | "sbduanliang"
                | "shouye"
                | "mbmowang"
                | "xianqu_mark"
                | "zhulianbihe_mark"
                | "yinyangyu_mark"
                | "_guozhan_marks"
                | "_chongzhu"
                | "taoluan"
                | "shouxi"
                | "dinghan"
                | "quanji"
                | "zili"
                | "paiyi"
                | "qimou"
                | "reqimou"
                | "zhaxiang"
                | "potzhuangshi"
                | "twxiafeng"
                | "twzuhuo"
                | "buff_txhj_gusoejueyan"
                | "_buff_txhj_guosejueyan"
                | "_buff_txhj_wuweizhile"
                | "_buff_txhj_jiezizhidao"
                | "hshuanling"
                | "hstianqi"
                | "lieqi"
                | "zhuilie"
                | "szbianshen"
                | "kekao"
                | "kqizhou"
                | "fuwen"
                | "mengye"
                | "kuixin"
                | "zhongji"
                | "jinzhou"
                | "xingluo"
                | "lianjin"
                | "yingxi"
                | "peiyu"
                | "peiyu2"
                | "mobao"
                | "xianji"
                | "xueren"
                | "xunbao2"
                | "xunying"
                | "liguang"
                | "xiepan"
                | "fenglue"
                | "husha"
                | "husha_count"
                | "danqing"
                | "danqing_count"
                | "zhangmu"
                | "zhangmu_ai"
                | "leiyu"
                | "leiyu2"
                | "leiyu4"
                | "shuiyun"
                | "shuiyun5"
                | "wangyou"
                | "wangyou2"
                | "changnian"
                | "duci"
                | "xshuangren"
                | "xshuangren2"
                | "qijian"
                | "qianfang"
                | "poyun"
                | "zhuyue"
                | "zhuyue2"
                | "runxin"
                | "xjzhimeng2"
                | "xjzhimeng3"
                | "qixia"
                | "yanshi2"
                | "tianwu"
                | "tianwu2"
                | "yujia"
                | "qiongguang"
                | "txianqu"
                | "minsha"
                | "lingdi"
                | "huanlei"
                | "anwugu"
                | "anwugu2"
                | "xtanxi"
                | "linghuo"
                | "guijin"
                | "guijin2"
                | "chengxin"
                | "shiying"
                | "xjzongyu"
                | "dujiang"
                | "sheying"
                | "huahu"
                | "binxin"
                | "longhuo"
                | "yanzhan"
                | "feixia"
                | "lueying"
                | "tianjian"
                | "huxi"
                | "xuanmo"
                | "sheling"
                | "zhaoyao"
                | "feizhua"
                | "guijiang"
                | "diewu"
                | "lingyu"
                | "duxinshu"
                | "feixu"
                | "luanjian"
                | "ctianfu"
                | "sajin"
                | "jtjubao"
                | "shenmu"
                | "guanri"
                | "tannang"
                | "tuoqiao"
                | "huimeng"
                | "yuexing"
                | "lanzhi"
                | "lingxian"
                | "tiebi"
                | "huanxing"
                | "lingyue"
                | "duanliang"
                | "jiuchi"
                | "new_rewusheng"
                | "xiaoxi_hansui"
                | "zymoucheng"
                | "lizhan"
                | "shelie"
                | "twkujian"
                | "retieji"
                | "gnjinfan_gain"
                | "reshanxi2"
                | "ranshang2"
        ) {
            return false;
        }
        if matches!(
            name,
            "qingyu"
                | "mibei"
                | "tspowei"
                | "rechuhai"
                | "xinhongyan"
                | "fz_gzpaoxiao"
                | "zhengsu"
                | "aozhan"
                | "aozhan_shan"
                | "diaohulishan"
                | "txmanjia"
                | "txmanjia1"
                | "txmanjia2"
                | "txfusha"
                | "txxiebi"
                | "txshixin"
                | "txmazhan"
                | "txhuangfu"
                | "txleizhen"
                | "txlianbi"
                | "txbaobian"
                | "diychunxiao"
                | "diyzhongdu"
                | "cxyYangWu"
                | "cxyJingQi"
                | "cxyRuiQi"
                | "cxyJiaoXia"
                | "cxyTunJun"
                | "cxyFengYing"
                | "cxyPoLu"
                | "txhj_ruiSkill1"
                | "txhj_dundunSkill2"
                | "txhj_jiuweiSkill1"
                | "txhj_tengsheSkill1"
                | "txhj_tengsheSkill3"
                | "txhj_yayaSkill2"
                | "txhj_qilinSkill3"
                | "txhj_chiyanzhenhunqin"
                | "txhj_lingbaoxianhu"
                | "txhj_chongyingshenfu"
                | "txminguangkai"
                | "txminguangkai_cancel"
                | "txminguangkai_link"
                | "txhj_feilong_skill_blank"
                | "txhj_guofengyupao"
                | "txhj_xuwangzhimian"
                | "txhj_aleSkill1"
                | "txhj_aleSkill2"
                | "txhj_dundunSkill1"
                | "txhj_datongSkill1"
                | "txhj_luluSkill1"
                | "txhj_luluSkill2"
                | "txhj_ruiSkill2"
                | "txhj_ruiSkill3"
                | "txhj_yueerSkill1"
                | "txhj_yueerSkill2"
                | "txhj_yanSkill1"
                | "txhj_yanSkill2"
                | "txhj_yanSkill3"
                | "txhj_liuliSkill1"
                | "txhj_liuliSkill2"
                | "txhj_manmanSkill1"
                | "txhj_manmanSkill2"
                | "txhj_xiaoxiaoSkill1"
                | "txhj_xiaoxiaoSkill2"
                | "txhj_xuerenSkill1"
                | "txhj_xuanwuSkill1"
                | "txhj_xuanwuSkill2"
                | "txhj_xuanwuSkill3"
                | "txhj_aHaoSkill2"
                | "txhj_aHaoSkill3"
                | "oldfuman2"
                | "txhuao"
                | "hgkurou_effect"
                | "txtanbei_effect1"
                | "txtanbei_effect2"
                | "txtanbei_effect3"
                | "ljxuechi"
                | "txzhuxin"
                | "txnianrui"
                | "txmengtai"
                | "txshouyi"
                | "txrenxing"
                | "txyinli"
                | "txzhulian"
                | "txkeji"
                | "txkeji_add"
                | "mdqianxi72"
                | "zglkongcheng"
                | "txjuxiang"
                | "txhj_huangmen"
                | "txhj_suzhi"
                | "txmanzhen"
                | "txleizhou"
                | "txyongguan"
                | "txyongguan_fate"
                | "txbenji"
                | "txhj_oldhuxiao"
                | "twwushen"
                | "shizhi"
                | "paoxiao"
                | "mashu"
                | "wushuang"
                | "roulin"
                | "weimu"
                | "qianxun"
                | "hongyan"
                | "wushen"
                | "wansha"
                | "rewansha"
                | "完杀"
                | "benghuai"
                | "kuangcai"
                | "txhj_kapaikeshi"
                | "txhj_wenhejibei"
                | "txhj_yueyingdezhihui"
                | "txhj_changbingguangren"
                | "txhj_xiongziyingfa"
                | "txhj_wuxiangdexinxue"
                | "txhj_wuweizhile"
                | "txhj_yuanduandilue"
                | "hlongyi"
                | "qingzun"
                | "shouwang2"
                | "ysjqisha_yi"
                | "ysjqisha_ao"
                | "hsbaowu_huangjinyuanhou"
                | "lieyang2"
                | "xuanyan"
                | "xjzhimeng"
                | "xiaoyue2"
                | "guijiang2"
                | "guijiang3"
                | "yanzhan2"
                | "yanzhan3"
                | "yanzhan4"
                | "yuexing2"
                | "liefeng"
                | "xiaoyue"
                | "yanshi"
                | "tanhua"
                | "ywuhun"
                | "xfenxin"
                | "xfenxin2"
                | "xuanyan2"
                | "xuanyan3"
                | "tianshe"
                | "tianshe2"
                | "ctianfu2"
                | "ctianfu3"
                | "luanjian2"
                | "changnian2"
                | "chengxin2"
                | "diesha"
                | "lingxue"
                | "lingquan"
                | "shenwu"
                | "fanling"
                | "jianzhen"
                | "fenshi"
                | "feng"
                | "ya"
                | "song"
                | "leiyu3"
                | "fenxing"
                | "ningbin"
                | "poyun2"
                | "qianfang2"
                | "longxi"
                | "tianxian"
                | "xjyufeng"
                | "xjyingfeng"
                | "dcnutao"
                | "guju"
                | "jili"
                | "jsrgzhenqiao"
                | "lixia"
                | "mjmouzhi"
                | "olzhuiji"
                | "reqizhou"
                | "fulin"
                | "nzry_longnu"
                | "nzry_longnu_fire"
                | "nzry_longnu_thunder"
                | "jiezi"
                | "kunfen"
                | "manji"
                | "jsrgbahu"
                | "drlt_qianjie"
                | "zhuiji"
                | "anjian"
                | "baonu"
                | "jintao"
                | "juejing"
                | "new_repaoxiao"
                | "new_repaoxiao2"
                | "nsyice"
                | "nzry_cunmu"
                | "nzry_jieying"
                | "ranshang"
                | "redangxian"
                | "rekuangcai"
                | "shibei"
                | "xiandeng"
                | "xinanjian"
                | "xswuyan"
                | "yizhong"
                | "xinfu_zhanji"
        ) {
            return true;
        }
        eff_meta.is_some_and(|m| !m.optional)
    }

    /// 某事件类别的候选触发定义（静态顺序，确定）。
    pub fn candidates(&self, kind: EventKind) -> &[SkillDefId] {
        self.dispatch.get(&kind).map(Vec::as_slice).unwrap_or(&[])
    }

    /// L20 标记补充 (`diaohulishan`, `extension.js:1432`): whether `p` is currently under `diaohulishan`.
    pub fn has_diaohulishan(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        if core
            .vm_aux
            .flags
            .get(&(p, "diaohulishan_active".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "mbmowang_resting".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
        {
            return true;
        }
        self.skill_by_name("diaohulishan")
            .is_some_and(|def| core.has_skill(p, def))
    }

    /// 查找持有指定宿主行为且已启用的技能定义 ID。
    pub fn skill_by_behavior(
        &self,
        behavior: crate::vocabulary::host::HostBehavior,
    ) -> Option<crate::ids::SkillDefId> {
        self.skills.iter().enumerate().find_map(|(id, def)| {
            (def.behavior.as_host_behavior() == Some(behavior))
                .then_some(id as crate::ids::SkillDefId)
        })
    }

    /// 玩家 `p` 是否拥有启用中的指定宿主行为。
    pub fn has_host_behavior(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
        behavior: crate::vocabulary::host::HostBehavior,
    ) -> bool {
        core.skills.range((p, u16::MIN)..).take_while(|((o, _), _)| *o == p).any(
            |((_, def), inst)| {
                inst.enabled
                    && self.skills[*def as usize].behavior.as_host_behavior() == Some(behavior)
            },
        )
    }

    /// 对决出局标记（`duijue2`）：不计距离、不能使用牌/救援、不能成为牌目标。
    pub fn has_duijue_out(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.vm_aux
            .flags
            .get(&(p, "duijue_out_active".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || self.has_host_behavior(
                core,
                p,
                crate::vocabulary::host::HostBehavior::SwdDuijueOut,
            )
    }

    /// 不计入距离计算且作为无效目标（调虎离山或对决出局）。
    pub fn has_undist(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        self.has_diaohulishan(core, p) || self.has_duijue_out(core, p)
    }

    /// 出牌封锁查询（swd 批 134 冰封）：玩家任一启用实例的行为声明封锁。
    pub fn blocks_all_cards(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        if self.has_undist(core, p)
            || core
                .vm_aux
                .flags
                .get(&(p, "chixueqingfeng2_count".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "l11_noda_axe2_turn".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "hs_qianfu2_active".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "u05_mod:qianfu2".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || self
                .skill_by_name("txhj_noda_axe2")
                .is_some_and(|def| core.has_skill(p, def))
            || self
                .skill_by_name("qianfu2")
                .is_some_and(|d| core.skills.get(&(p, d)).is_some_and(|inst| inst.enabled))
        {
            return true;
        }
        core.skills.iter().any(|((o, def), inst)| {
            *o == p
                && inst.enabled
                && self.skills[*def as usize]
                    .behavior
                    .blocks_all_cards_when(self, core, p)
        })
    }

    /// 枚举 `player` 装备区中可用于宿主 view-as 出牌的卡牌 ID（`huanxia`/`poxiao`；`yeying`/`guiying`/`fengze` 经 ActivateSkill 通道处理）。
    pub fn host_view_as_extra_cards(
        &self,
        core: &crate::state::GameStateCore,
        player: crate::ids::PlayerId,
    ) -> Vec<CardId> {
        let sealed = self.is_tieji_sealed(core, player)
            || (core.player(player).hp <= 1 && self.has_rechanyuan(core, player));
        if sealed || self.blocks_all_cards(core, player) {
            return Vec::new();
        }
        let has_ready = |name: &str, check_once: bool| {
            self.skill_by_name(name).is_some_and(|d| {
                core.skills
                    .get(&(player, d))
                    .is_some_and(|i| i.enabled && (!check_once || !i.once_used))
            })
        };
        let huanxia = has_ready("huanxia", false);
        let poxiao = has_ready("poxiao", false);
        if !(huanxia || poxiao) {
            return Vec::new();
        }
        core.player(player)
            .equips
            .iter()
            .copied()
            .filter(|&c| {
                let suit = self.effective_card_suit(core, player, c);
                let red = matches!(suit, Suit::Heart | Suit::Diamond);
                (huanxia && red) || (poxiao && self.card_kind(c).is_equip())
            })
            .collect()
    }

    /// G1 目标过滤原语：`p` 是否持有「延时锦囊目标免疫」类技能（墨守：乐/兵不能指定其为目标）。
    pub fn blocks_delayed_trick(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        if core
            .vm_aux
            .flags
            .get(&(p, "l11_judge_slot_disabled".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
        {
            return true;
        }
        core.skills
            .keys()
            .any(|(o, _)| *o == p)
            .then(|| {
                core.skills.iter().any(|((o, def), inst)| {
                    *o == p
                        && inst.enabled
                        && self.skills[*def as usize]
                            .behavior
                            .blocks_delayed_trick_target()
                })
            })
            .unwrap_or(false)
    }

    /// Whether a skill has `charlotte: true` (global `_` / runtime helper / trait):
    /// excluded from `xuelunyang` theft and `mbcmfangzhu` skill-disable.
    pub fn is_charlotte_skill(&self, def: crate::ids::SkillDefId) -> bool {
        let Some(skill) = self.skills.get(def as usize) else {
            return false;
        };
        skill.behavior.is_charlotte()
            || skill.name.starts_with('_')
            || skill.name.contains("#r05:")
            || matches!(
                skill.name,
                crate::vocabulary::names::WANG_ZUN_MARK
                    | "wuxing_shengke"
                    | "diaohulishan"
                    | "chanyuan"
                    | "rechanyuan"
                    | "gw_xianzumaijiu"
                    | "gwmaoxian_hengsaite_sha"
            )
    }

    /// Whether a skill has `persevereSkill / forever: true` (`持恒技`):
    /// exempt from skill-disable effects such as `mbcmfangzhu`.
    /// `yange` 等临时牌面克隆标志 `gj_yange_clone:{card}` 指向的原模板牌号。
    pub fn resolved_base_card_id(
        &self,
        core: &GameStateCore,
        card: crate::ids::CardId,
    ) -> crate::ids::CardId {
        if let Some(&v) = core.vm_aux.flags.get(&(0, format!("gj_yange_clone:{card}"))) {
            if let Some(id) = v.checked_sub(1) {
                if (id as usize) < self.cards.len() {
                    return id;
                }
            }
        }
        card
    }

    pub fn resolved_card_def(
        &self,
        core: &GameStateCore,
        card: crate::ids::CardId,
    ) -> Option<&CardDef> {
        self.cards
            .get(self.resolved_base_card_id(core, card) as usize)
    }

    pub fn resolved_card_kind(&self, core: &GameStateCore, card: crate::ids::CardId) -> CardKind {
        core.declared_kinds.get(&card).copied().unwrap_or_else(|| {
            self.resolved_card_def(core, card)
                .map(|d| d.kind)
                .unwrap_or(CardKind::Sha)
        })
    }

    pub fn resolved_card_suit(&self, core: &GameStateCore, card: crate::ids::CardId) -> Suit {
        self.resolved_card_def(core, card)
            .map_or(Suit::Club, |d| d.suit)
    }

    pub fn yanjia_merged_card_id(
        &self,
        core: &GameStateCore,
        card: crate::ids::CardId,
    ) -> Option<crate::ids::CardId> {
        let raw = core
            .vm_aux
            .flags
            .get(&(0, format!("gj_yanjia_merged:{card}")))
            .and_then(|v| v.checked_sub(1))?;
        let base = self.resolved_base_card_id(core, raw);
        ((base as usize) < self.cards.len()).then_some(base)
    }

    pub fn equip_card_has_kind(
        &self,
        core: &GameStateCore,
        card: crate::ids::CardId,
        want: CardKind,
    ) -> bool {
        if self.resolved_card_kind(core, card) == want {
            return true;
        }
        self.yanjia_merged_card_id(core, card)
            .is_some_and(|c2| self.resolved_card_kind(core, c2) == want)
    }

    pub fn player_has_equip_kind(
        &self,
        core: &GameStateCore,
        player: PlayerId,
        want: CardKind,
    ) -> bool {
        core.player(player)
            .equips
            .iter()
            .any(|&c| self.equip_card_has_kind(core, c, want))
    }

    pub fn equip_card_skills(
        &self,
        core: &GameStateCore,
        card: crate::ids::CardId,
    ) -> Vec<&'static str> {
        let mut out = Vec::new();
        if let Some(def) = self.resolved_card_def(core, card) {
            out.extend_from_slice(&def.equip_skills);
        }
        if let Some(c2) = self.yanjia_merged_card_id(core, card) {
            if let Some(def2) = self.cards.get(c2 as usize) {
                out.extend_from_slice(&def2.equip_skills);
            }
        }
        out
    }

    pub fn equip_card_links(&self, core: &GameStateCore, card: crate::ids::CardId) -> bool {
        self.resolved_card_def(core, card)
            .is_some_and(|d| d.link_on_equip)
            || self
                .yanjia_merged_card_id(core, card)
                .and_then(|c2| self.cards.get(c2 as usize))
                .is_some_and(|d| d.link_on_equip)
    }

    pub fn is_forever_skill(&self, def: crate::ids::SkillDefId) -> bool {
        self.skills
            .get(def as usize)
            .is_some_and(|skill| skill.behavior.is_forever())
    }

    /// PA12 批⑮：查找 `player` 当前启用的 VM 技能中首个匹配 `card` 的单牌转化规则。
    pub fn find_player_vm_view_as(
        &self,
        core: &crate::state::GameStateCore,
        player: crate::ids::PlayerId,
        card: CardId,
        in_play: bool,
        target_kind: Option<CardKind>,
    ) -> Option<&crate::rvm_bridge::VmViewAsSpec> {
        let sealed = self.is_tieji_sealed(core, player);
        core.skills
            .range((player, u16::MIN)..)
            .take_while(|((p, _), _)| *p == player)
            .filter(|((_, _), inst)| inst.enabled)
            .find_map(|((_, def), _)| {
                let s = self.skills.get(*def as usize)?;
                if (sealed && !Self::is_locked_skill_name(s.name, s.trigger.as_ref()))
                    || self.is_rechanyuan_sealed_def(core, player, *def)
                {
                    return None;
                }
                s.behavior.vm_view_as().iter().find(|spec| {
                    spec.matches_card(core, self, player, card, in_play, target_kind)
                })
            })
            .or_else(|| {
                crate::rvm_bridge::modifiers::d34_flags::find_d34_flag_view_as(
                    core,
                    self,
                    player,
                    card,
                    in_play,
                    target_kind,
                )
            })
    }

    /// PA12 批⑰：返回 `player` 转化技在 `from none` 模式下使用的确定性虚拟令牌卡牌 ID（不在手牌与装备区中）。
    pub fn vm_view_as_none_token(
        &self,
        core: &crate::state::GameStateCore,
        player: crate::ids::PlayerId,
    ) -> CardId {
        let p = core.player(player);
        (0..self.cards.len() as CardId)
            .find(|c| !p.hand.contains(c) && !p.equips.contains(c))
            .unwrap_or(0)
    }

    /// PA12 批⑰：枚举 `player` 非手牌区（装备区、扩展牌堆及 `from none` 虚拟令牌）可参与 VM 转化技的卡牌 ID。
    pub fn player_vm_view_as_extra_cards(
        &self,
        core: &crate::state::GameStateCore,
        player: crate::ids::PlayerId,
        in_play: bool,
        target_kind: Option<CardKind>,
    ) -> Vec<CardId> {
        let mut out = Vec::new();
        let p = core.player(player);
        for &eq in &p.equips {
            if self
                .find_player_vm_view_as(core, player, eq, in_play, target_kind)
                .is_some()
                && !out.contains(&eq)
            {
                out.push(eq);
            }
        }
        for ((po, _), pile) in &core.vm_aux.piles {
            if *po == player {
                for &c in pile {
                    if self
                        .find_player_vm_view_as(core, player, c, in_play, target_kind)
                        .is_some()
                        && !p.hand.contains(&c)
                        && !out.contains(&c)
                    {
                        out.push(c);
                    }
                }
            }
        }
        let none_tok = self.vm_view_as_none_token(core, player);
        if self
            .find_player_vm_view_as(core, player, none_tok, in_play, target_kind)
            .is_some()
            && !p.hand.contains(&none_tok)
            && !out.contains(&none_tok)
        {
            out.push(none_tok);
        }
        out
    }

    /// Whether `player` holds an unblocked `L11VmFixA` `wushen` (only `chanyuan` HP==1 seals).
    pub fn has_active_l11_wushen(&self, core: &GameStateCore, player: PlayerId) -> bool {
        self.skill_by_name("wushen").is_some_and(|def| {
            self.skills[def as usize].behavior.as_host_behavior()
                == Some(crate::vocabulary::host::HostBehavior::L11VmFixA)
                && core.has_skill(player, def)
                && !self.is_skill_chanyuan_blocked(core, player, def)
        })
    }

    /// Card kind seen by `player` after mandatory `mod.cardname` (`L11VmFixA` `wushen`:
    /// Heart non-equip cards act strictly as `CardKind::Sha`).
    pub fn player_card_kind(
        &self,
        core: &GameStateCore,
        player: PlayerId,
        card: CardId,
    ) -> CardKind {
        if !core.player(player).equips.contains(&card)
            && self.has_active_l11_wushen(core, player)
            && self.effective_card_suit(core, player, card) == crate::catalog::Suit::Heart
        {
            CardKind::Sha
        } else {
            self.resolved_card_kind(core, card)
        }
    }
}
