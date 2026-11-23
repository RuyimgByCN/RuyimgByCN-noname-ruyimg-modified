use super::*;

mod common;
mod xianjian;
mod wudian;
mod swd_artifacts;
mod swd_skills;

impl HostSkillBehavior {
    pub(super) fn activate_behavior(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            // 枕流·临月面（武将面批 83）：有红色手牌且有其他存活角色
            HostBehavior::SwdZhenjiu => self.activate_swd_skills(ctx, owner),
            // 麻醉·临月面（武将面批 83）：有黑色手牌且有其他未挂附存活角色
            HostBehavior::SwdLmazui => self.activate_swd_skills(ctx, owner),
            // 剑击·少微面（武将面批 91）：有装备牌且射程内有目标
            HostBehavior::SwdJianji => self.activate_swd_skills(ctx, owner),
            // 奥剑（deferred 消解批）：已损失体力≥1 且有手牌
            HostBehavior::SwdAojian => self.activate_swd_skills(ctx, owner),
            // 化金（deferred 消解批）：he 有牌
            HostBehavior::SwdHuajin => self.activate_swd_skills(ctx, owner),
            // 拼点（deferred 消解批）：手牌有牌
            HostBehavior::SwdNlianji => self.activate_swd_skills(ctx, owner),
            // 蛇魂（deferred 消解批）：手牌有牌
            HostBehavior::SwdShehun => self.activate_swd_skills(ctx, owner),
            // 聚悉（deferred 消解批）：计数 >= 存活数
            HostBehavior::SwdJuxi => self.activate_swd_skills(ctx, owner),
            // 破舟·封印（deferred 消解批）：计数 >0 且存在未封印的其他角色
            HostBehavior::SwdPozhou2 => self.activate_swd_skills(ctx, owner),
            // 陷阱（deferred 消解批）：有花色未在陷阱表中的手牌
            HostBehavior::SwdSxianjing => self.activate_swd_skills(ctx, owner),
            // 锦鳞（deferred 消解批 114）：limited 一次性——已发动则无机器
            //（技能实例仍在，门在 activate；上游 awakenSkill 同口径）
            HostBehavior::SwdJinlin => self.activate_swd_skills(ctx, owner),
            // 轩辕（swd 批 146）：结束阶段触发（无主动面）
            HostBehavior::Xuanyuan => self.activate_common(ctx, owner),
            // 苦肉（L20 批 2）：出牌阶段主动
            HostBehavior::Kurou => self.activate_common(ctx, owner),
            // 集舍（L27 批 5）：出牌阶段主动（手牌上限 > 0）
            HostBehavior::Jishe => self.activate_common(ctx, owner),
            // 蛊惑（史诗卡牌批 1）：出牌阶段主动（每回合一次；手牌非空）
            HostBehavior::Reguhuo => self.activate_common(ctx, owner),
            // 攻心（swd 批 155）：出牌阶段主动（每回合一次；存在有手牌的其他角色）
            HostBehavior::Gongxin => self.activate_common(ctx, owner),
            // 魔焰（swd 批 151）：出牌阶段主动（每回合一次；体力未满且有红牌）
            HostBehavior::Moyan => self.activate_common(ctx, owner),
            // 天书（swd 批 142）：出牌阶段主动（限定——实例启用时）
            HostBehavior::SwdTianshu => self.activate_swd_skills(ctx, owner),
            // 极略（swd 批 136）：出牌阶段主动（每阶段 3 次）
            HostBehavior::Jilve => self.activate_common(ctx, owner),
            // 革新印（swd 批 135）：出牌阶段主动（手牌非空）
            HostBehavior::Gxianyin => self.activate_common(ctx, owner),
            // 冰封（swd 批 134）：出牌阶段主动（限定——实例启用时）
            HostBehavior::Bingfeng => self.activate_common(ctx, owner),
            // 魂殉志（swd 批 132）：出牌阶段主动（限定——实例启用时）
            HostBehavior::Hxunzhi => self.activate_common(ctx, owner),
            // 颂词（swd 批 129）：出牌阶段主动——选未记录角色（含自己）
            HostBehavior::Songci => self.activate_common(ctx, owner),
            // 林韵（swd 批 127）：出牌阶段主动——两张牌（手/装备）当杀
            HostBehavior::SwdLinyun => self.activate_swd_skills(ctx, owner),
            // 涅槃（T11 Boss 庞统族批 118）：出牌阶段主动面——限定技未发动
            // 时可用（濒死面走触发路径；两面共用收束）。
            HostBehavior::Niepan => self.activate_common(ctx, owner),
            // 孤行（deferred 消解批）：手牌数 <= max(1, 已损失体力) 且 >0
            HostBehavior::SwdGuxing => self.activate_swd_skills(ctx, owner),
            // 玄咒（deferred 消解批）：手牌有普通锦囊
            HostBehavior::SwdXuanzhou => self.activate_swd_skills(ctx, owner),
            // 孵鸟（deferred 消解批）：有手牌
            HostBehavior::SwdFuniao => self.activate_swd_skills(ctx, owner),
            // 黑火（deferred 消解批）：he 有装备牌
            HostBehavior::SwdHeihuo => self.activate_swd_skills(ctx, owner),
            // 头弹·麦星面（武将面批 91）：有黑桃 he 且有其他存活
            HostBehavior::SwdToudan => self.activate_swd_skills(ctx, owner),
            // 玄影·端蒙面（武将面批 90）：未横置且有存活其他角色（sha 面）
            HostBehavior::SwdXuanying => self.activate_swd_skills(ctx, owner),
            // 命符·姜子牙面（武将面批 88）：有梅花手牌
            HostBehavior::SwdMingfu => self.activate_swd_skills(ctx, owner),
            // 义绝（武将面批 86）：自己有手牌且有手牌的其他角色
            HostBehavior::SwdYijue => self.activate_swd_skills(ctx, owner),
            // 活弹·舒莞儿面（武将面批 85）：有红色手牌且存活其他角色≥1
            HostBehavior::SwdHuodan => self.activate_swd_skills(ctx, owner),
            // 愚钝·福面（武将面批 82）：两张锦囊手牌且在攻击范围内有目标
            HostBehavior::SwdYudun => self.activate_swd_skills(ctx, owner),
            // 占星·肯迪面（武将面批 82）：有手牌
            HostBehavior::SwdZhanxing => self.activate_swd_skills(ctx, owner),
            // 乱击·多鹏面（武将面批 81）：手牌存在同花色两卡
            HostBehavior::SwdLuanji => self.activate_swd_skills(ctx, owner),
            // 运筹·姬亮面（武将面批㊳）：自己有手牌
            HostBehavior::SwdYunchou => self.activate_swd_skills(ctx, owner),
            // 天火·秦壬面（武将面批㊲）：任一其他角色判定区有牌
            HostBehavior::SwdTianhuo => self.activate_swd_skills(ctx, owner),
            // 焰羽·少微面（武将面批㊱）：红色牌 ≥2
            HostBehavior::SwdHuangyu => self.activate_swd_skills(ctx, owner),
            // 摄引（武将面批㊱）：任一其他角色判定区有牌
            HostBehavior::SwdXianyin => self.activate_swd_skills(ctx, owner),
            // 星坠·慕月面（武将面批㉟）：he 有牌
            HostBehavior::SwdXingzhui => self.activate_swd_skills(ctx, owner),
            // 冰箭（武将面批㉞ 海大胖）：手牌有黑色杀
            HostBehavior::SwdBingjian => self.activate_swd_skills(ctx, owner),
            // 神鼎·麦星面（武将面批㉞）：恒可发动（判定分支自带无效果面）
            HostBehavior::SwdShending => self.activate_swd_skills(ctx, owner),
            // 鬼眼（武将面批㉞ 冯天凌）：任一其他角色手牌有梅花
            HostBehavior::SwdGuiyan => self.activate_swd_skills(ctx, owner),
            // 鸣凤（武将面批㉝ 胡月）：有装备牌可弃
            HostBehavior::SwdFengming => self.activate_swd_skills(ctx, owner),
            // 断义（武将面批㉘ 慕容诗）：手牌中杀多于 1（上游 countCards
            // "sha" > 1 同口径）
            HostBehavior::SwdDuanyi => self.activate_swd_skills(ctx, owner),
            // 森罗（武将面批㉖ 莫耶）：自己手牌数为全场严格最少
            HostBehavior::SwdSenluo => self.activate_swd_skills(ctx, owner),
            // 湛卢（武将面批㉕ 屠维）：有黑桃牌且场上有已受伤角色
            HostBehavior::SwdZhanlu => self.activate_swd_skills(ctx, owner),
            // 相惠（武将面批㉕ 夏柔）：有红色手牌且体力最少者已受伤
            HostBehavior::SwdXianghui => self.activate_swd_skills(ctx, owner),
            // 对诗（武将面批㉒ 王思月）：手牌非空即发动（usable:1 走
            // limits；上游 duishi2/3 临时技面由限一次收敛）
            HostBehavior::SwdDuishi => self.activate_swd_skills(ctx, owner),
            // 鼠槌（equip5 窗口机批）：手中有杀且本回合未用过杀（上游
            // filter cardUsable 同口径——限击数提升时收窄登记差异）
            HostBehavior::SwdShuchuiUse => self.activate_swd_artifacts(ctx, owner),
            // 分舵：存在手牌多于自己的存活角色才可发动
            HostBehavior::SwdJiefen => self.activate_swd_skills(ctx, owner),
            // 五行盘：出牌阶段限一次（阶段门控在命令边界），需手牌非空
            HostBehavior::ChangeCardNature => self.activate_common(ctx, owner),
            HostBehavior::GujianFoodLuyugeng => self.activate_common(ctx, owner),
            HostBehavior::GujianYuhengEquipSkill => self.activate_common(ctx, owner),
            HostBehavior::SwdLangeguaiyiEquipSkill => self.activate_swd_artifacts(ctx, owner),
            HostBehavior::SwdYufuluEquipSkill => self.activate_swd_artifacts(ctx, owner),
            HostBehavior::SwdShentouEquipSkill => self.activate_swd_artifacts(ctx, owner),
            // 断月：有装备牌（手/装备区）且存在其他存活角色
            HostBehavior::SwdDuanyue => self.activate_swd_skills(ctx, owner),
            // 星点：有手牌且存在其他有牌角色
            HostBehavior::SwdXingdian => self.activate_swd_skills(ctx, owner),
            // 雪煌：limited 未发动、手牌全红且非空
            HostBehavior::SwdXuehuang => self.activate_swd_skills(ctx, owner),
            // 收纳：有手牌且存在其他有手牌角色（装备炼妖壶在 ActivateSkill
            // 命令层由 equip 技能实例承载——实例随 grant_equip_skills 授予）
            HostBehavior::SwdLianyaohuShouna => self.activate_swd_artifacts(ctx, owner),
            // 炼化：壶仓 ≥2
            HostBehavior::SwdLianyaohuLianhua => self.activate_swd_artifacts(ctx, owner),
            // 昆仑镜：牌堆 ≥3 且有手牌
            HostBehavior::SwdKunlunjingcCard => self.activate_swd_artifacts(ctx, owner),
            // 四非珍面：存在其他有手牌角色
            HostBehavior::SwdSifeizhenmianCard => self.activate_swd_artifacts(ctx, owner),
            // 逆攻发动：标记 ≥1；lv≥2 打伤需选目标，lv==1 仅摸 1（直发）
            HostBehavior::SwdNigongStrike => self.activate_swd_artifacts(ctx, owner),
            // 神农鼎：受伤且手牌 ≥2
            HostBehavior::SwdShennongdingCard => self.activate_swd_artifacts(ctx, owner),
            // 悲歌：确认门由 optional 承担（activate 恒供机）
            HostBehavior::SwdTiebi => self.activate_swd_skills(ctx, owner),
            HostBehavior::SwdDatong => self.activate_swd_skills(ctx, owner),
            HostBehavior::YunchouBeige => self.activate_common(ctx, owner),
            // 屯田·DIY：有装备牌（手/场）才可发动
            HostBehavior::WudianDiyjuntun => self.activate_wudian(ctx, owner),
            // 隙焰：有红手牌才可发动
            HostBehavior::YunchouJieyan => self.activate_common(ctx, owner),
            // 野升：有黑手牌才可发动（自选多张）
            HostBehavior::YunchouYiesheng => self.activate_common(ctx, owner),
            HostBehavior::ZhangbaSha => self.activate_common(ctx, owner),
            HostBehavior::WudianPaotai => self.activate_wudian(ctx, owner),
            HostBehavior::WudianShouhu => self.activate_wudian(ctx, owner),
            HostBehavior::WudianBaoleiSwitch => self.activate_wudian(ctx, owner),
            HostBehavior::XianjianXtanxi => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianHuahu => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianLuanjian => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianCtianfu => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianTianjian => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianSajin => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianMinsha => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianZhuyue => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianGuanri => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianDiewu => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianGuiyuan => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianGuijin => self.activate_xianjian(ctx, owner),
            // 玄墨（U05 xianjian 夏侯瑾轩，规则 v174）：手牌中有三类牌
            //（usable:1 走命令层 once_used）
            HostBehavior::XianjianXuanmo => self.activate_xianjian(ctx, owner),
            HostBehavior::XianjianYuexing => self.activate_xianjian(ctx, owner),
            HostBehavior::WudianKangyin => self.activate_wudian(ctx, owner),
            HostBehavior::WudianJuedao => self.activate_wudian(ctx, owner),
            HostBehavior::WudianMujing => self.activate_wudian(ctx, owner),
            HostBehavior::WudianGuangshu => self.activate_wudian(ctx, owner),
            HostBehavior::WudianZiyu => self.activate_wudian(ctx, owner),
            HostBehavior::WudianKuoyin => self.activate_wudian(ctx, owner),
            HostBehavior::WudianHuhuan => self.activate_wudian(ctx, owner),
            HostBehavior::WudianBaoxue => self.activate_wudian(ctx, owner),
            HostBehavior::WudianLiangou => self.activate_wudian(ctx, owner),
            HostBehavior::WudianYinbo => self.activate_wudian(ctx, owner),
            HostBehavior::WudianQianggu => self.activate_wudian(ctx, owner),
            HostBehavior::WudianFeitiao => self.activate_wudian(ctx, owner),
            HostBehavior::WudianChoudu => self.activate_wudian(ctx, owner),
            HostBehavior::MtgYixialanRecast => self.activate_common(ctx, owner),
            HostBehavior::MtgYoulinDiscard => self.activate_common(ctx, owner),
            HostBehavior::MtgLinzhongjiantaStrike => self.activate_common(ctx, owner),
            HostBehavior::DddXiaxing
            | HostBehavior::MbcmJiushi
            | HostBehavior::MbcmFangzhu
            | HostBehavior::MbJuejin => self.activate_common(ctx, owner),
            HostBehavior::U01DddBatchA => super::step::u01_fixes_1::activate_u01_fix(self, ctx, owner)
                .unwrap_or_else(|| super::step::u01_batch_a::activate_u01_batch_a(self, ctx, owner)),
            HostBehavior::U01DddBatchB => super::step::u01_fixes_1::activate_u01_fix(self, ctx, owner)
                .or_else(|| super::step::u01_fixes_2::activate_u01_fix2(self, ctx, owner))
                .unwrap_or_else(|| super::step::u01_batch_b::activate_u01_batch_b(self, ctx, owner)),
            HostBehavior::U01DddBatchC => super::step::u01_fixes_1::activate_u01_fix(self, ctx, owner)
                .or_else(|| super::step::u01_fixes_2::activate_u01_fix2(self, ctx, owner))
                .or_else(|| super::step::u01_fixes_3::activate_u01_fix3(self, ctx, owner))
                .unwrap_or_else(|| super::step::u01_batch_c::activate_u01_batch_c(self, ctx, owner)),
            HostBehavior::U01DddBatchD => super::step::u01_fixes_1::activate_u01_fix(self, ctx, owner)
                .or_else(|| super::step::u01_fixes_2::activate_u01_fix2(self, ctx, owner))
                .or_else(|| super::step::u01_fixes_3::activate_u01_fix3(self, ctx, owner))
                .unwrap_or_else(|| super::step::u01_batch_d::activate_u01_batch_d(self, ctx, owner)),
            HostBehavior::L28ThunderBatchA => {
                super::step::l28_batch_a::activate_l28_batch_a(self, ctx, owner)
            }
            HostBehavior::L28ThunderBatchB => {
                super::step::l28_batch_b::activate_l28_batch_b(self, ctx, owner)
            }
            HostBehavior::L27BeautifyBatchA => {
                super::step::l27_batch_a::activate_l27_batch_a(self, ctx, owner)
            }
            HostBehavior::L27BeautifyBatchB => {
                super::step::l27_batch_b::activate_l27_batch_b(self, ctx, owner)
            }
            HostBehavior::L19DoomsdayBatchA => {
                super::step::l19_batch_a::activate_l19_batch_a(self, ctx, owner)
            }
            HostBehavior::L19DoomsdayBatchB => {
                super::step::l19_batch_b::activate_l19_batch_b(self, ctx, owner)
            }
            HostBehavior::L20MarkBatchA => {
                super::step::l20_batch_a::activate_l20_batch_a(self, ctx, owner)
            }
            HostBehavior::L20MarkBatchB => {
                super::step::l20_batch_b::activate_l20_batch_b(self, ctx, owner)
            }
            HostBehavior::U08YxsBatchA => {
                super::step::u08_batch_a::activate_u08_batch_a(self, ctx, owner)
            }
            HostBehavior::U08YxsBatchB => {
                super::step::u08_batch_b::activate_u08_batch_b(self, ctx, owner)
            }
            HostBehavior::U08YxsBatchC => {
                super::step::u08_batch_c::activate_u08_batch_c(self, ctx, owner)
            }
            HostBehavior::U08YxsBatchD => {
                super::step::u08_batch_d::activate_u08_batch_d(self, ctx, owner)
            }
            HostBehavior::L25ZuanBatchA => {
                super::step::l25_batch_a::activate_l25_batch_a(self, ctx, owner)
            }
            HostBehavior::L25ZuanBatchB => {
                super::step::l25_batch_b::activate_l25_batch_b(self, ctx, owner)
            }
            HostBehavior::L25ZuanBatchC => {
                super::step::l25_batch_c::activate_l25_batch_c(self, ctx, owner)
            }
            HostBehavior::L25ZuanBatchD => {
                super::step::l25_batch_d::activate_l25_batch_d(self, ctx, owner)
            }
            HostBehavior::L25ZuanBatchE => {
                super::step::l25_batch_e::activate_l25_batch_e(self, ctx, owner)
            }
            HostBehavior::L25ZuanBatchF => {
                super::step::l25_batch_f::activate_l25_batch_f(self, ctx, owner)
            }
            HostBehavior::L25ZuanBatchG => {
                super::step::l25_batch_g::activate_l25_batch_g(self, ctx, owner)
            }
            HostBehavior::U05OwBatchA
            | HostBehavior::U05OwBatchB
            | HostBehavior::U05OwBatchC
            | HostBehavior::U05OwBatchD => {
                super::step::u05_ow::activate_u05_ow(self, ctx, owner)
            }
            HostBehavior::L10ScsCompanion => {
                super::step::l10_scs::activate_l10_scs(self, ctx, owner)
            }
            HostBehavior::L11TrickCard => {
                super::step::l11_cards::activate_l11_zhaoshu(self, ctx, owner)
            }
            HostBehavior::L11EquipCards => {
                super::step::l11_equips::activate_l11_equips(self, ctx, owner)
            }
            HostBehavior::L11Daiyuan
            | HostBehavior::L11ServantSkills
            | HostBehavior::L11SubStates => {
                super::step::l11_servants::activate_l11_servants(self, ctx, owner)
            }
            HostBehavior::L11GeneralSkillsA => {
                super::step::l11_skills_a::activate_l11_skills_a(self, ctx, owner)
            }
            HostBehavior::L11GeneralSkillsB => {
                super::step::l11_skills_b::activate_l11_skills_b(self, ctx, owner)
            }
            HostBehavior::L11GeneralSkillsC => {
                super::step::l11_skills_c::activate_l11_skills_c(self, ctx, owner)
            }
            HostBehavior::L11ServantSkillsB => {
                super::step::l11_servants_b::activate_l11_servants_b(self, ctx, owner)
            }
            HostBehavior::L11GeneralSkillsD => {
                super::step::l11_skills_d::activate_l11_skills_d(self, ctx, owner)
            }
            HostBehavior::L11GeneralSkillsE => {
                super::step::l11_skills_e::activate_l11_skills_e(self, ctx, owner)
            }
            HostBehavior::L11GeneralSkillsF => {
                super::step::l11_skills_f::activate_l11_skills_f(self, ctx, owner)
            }
            HostBehavior::L11BuffsA => {
                super::step::l11_buffs_a::activate_l11_buffs_a(self, ctx, owner)
            }
            HostBehavior::L11BuffsB => {
                super::step::l11_buffs_b::activate_l11_buffs_b(self, ctx, owner)
            }
            HostBehavior::L11BuffHelpersA => {
                super::step::l11_buff_helpers_a::activate_l11_buff_helpers_a(self, ctx, owner)
            }
            HostBehavior::L11BuffsC => {
                super::step::l11_buffs_c::activate_l11_buffs_c(self, ctx, owner)
            }
            HostBehavior::L11BuffsD => {
                super::step::l11_buffs_d::activate_l11_buffs_d(self, ctx, owner)
            }
            HostBehavior::L11BuffHelpersB => {
                super::step::l11_buff_helpers_b::activate_l11_buff_helpers_b(self, ctx, owner)
            }
            HostBehavior::L01NeiKey | HostBehavior::L01AiRuleHooks => {
                super::step::l01_rules::activate_l01_skill(&self.name, self.behavior, ctx, owner)
            }
            HostBehavior::L11SystemHooks => {
                super::step::l11_system_hooks::activate_l11_system_hooks(self, ctx, owner)
            }
            HostBehavior::L11Weapons => {
                super::step::l11_weapons::activate_l11_weapons(self, ctx, owner)
            }
            HostBehavior::L11GeneralBossC | HostBehavior::L11BossLingfeng => {
                super::step::l11_boss_c::activate_l11_boss_c(self, ctx, owner)
            }
            HostBehavior::U05MainStoneBatch | HostBehavior::U05ZhenfaRecastSkill => {
                super::step::u05_b1_zhenfa::activate_u05_zhenfa(self, ctx, owner)
                    .map(|m| Box::new(m) as Box<dyn crate::catalog::SkillMachine>)
            }
            HostBehavior::U05MtgCharBatch | HostBehavior::U05MtgLandExtraBatch => {
                super::step::u05_b1_mtg::activate_u05_mtg(self, ctx, owner)
                    .map(|m| Box::new(m) as Box<dyn crate::catalog::SkillMachine>)
            }
            HostBehavior::U05YunchouCharBatch => {
                super::step::u05_b1_yunchou::activate_u05_yunchou_char(self, ctx, owner)
                    .map(|m| Box::new(m) as Box<dyn crate::catalog::SkillMachine>)
            }
            HostBehavior::U05GujianCardBatch
            | HostBehavior::U05GujianCharBatchA
            | HostBehavior::U05GujianCharBatchB
            | HostBehavior::U05GujianCharBatchC
            | HostBehavior::U05GujianCharBatchD
            | HostBehavior::U05GujianCharBatchE => {
                super::step::u05_b2_gujian::activate_u05_gujian(self, ctx, owner)
            }
            HostBehavior::U05HearthCards
            | HostBehavior::U05HearthBatchA
            | HostBehavior::U05HearthBatchB
            | HostBehavior::U05HearthBatchC
            | HostBehavior::U05HearthBatchD
            | HostBehavior::U05HearthBatchE
            | HostBehavior::U05Hearth2BatchA
            | HostBehavior::U05Hearth2BatchB
            | HostBehavior::U05Hearth2BatchC
            | HostBehavior::U05Hearth2BatchD
            | HostBehavior::U05Hearth2BatchE
            | HostBehavior::U05Hearth2BatchF => {
                super::step::u05_hearth::activate_u05_hearth(self, ctx, owner)
            }
            HostBehavior::U05GwentCharBatchA
            | HostBehavior::U05GwentCharBatchB
            | HostBehavior::U05GwentCharBatchC
            | HostBehavior::U05GwentCharBatchD => {
                super::step::u05_b5b_gwent::activate_u05_gwent_char(self, ctx, owner)
                    .map(|m| Box::new(m) as Box<dyn crate::catalog::SkillMachine>)
            }
            HostBehavior::U05XianjianBatchD => super::step::xianjian_batch::activate(self, ctx, owner)
                .map(|m| Box::new(m) as Box<dyn crate::catalog::SkillMachine>),
            _ => None,
        }
    }
}

pub(in crate::content::host) fn has_phase_use_activation(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    match behavior {
        HostBehavior::Xuanyuan => false,
        HostBehavior::SwdZhenjiu
        | HostBehavior::SwdLmazui
        | HostBehavior::SwdJianji
        | HostBehavior::SwdAojian
        | HostBehavior::SwdHuajin
        | HostBehavior::SwdNlianji
        | HostBehavior::SwdShehun
        | HostBehavior::SwdJuxi
        | HostBehavior::SwdPozhou2
        | HostBehavior::SwdSxianjing
        | HostBehavior::SwdJinlin
        | HostBehavior::Kurou
        | HostBehavior::Jishe
        | HostBehavior::Reguhuo
        | HostBehavior::Gongxin
        | HostBehavior::Moyan
        | HostBehavior::SwdTianshu
        | HostBehavior::Jilve
        | HostBehavior::Gxianyin
        | HostBehavior::Bingfeng
        | HostBehavior::Hxunzhi
        | HostBehavior::Songci
        | HostBehavior::SwdLinyun
        | HostBehavior::Niepan
        | HostBehavior::SwdGuxing
        | HostBehavior::SwdXuanzhou
        | HostBehavior::SwdFuniao
        | HostBehavior::SwdHeihuo
        | HostBehavior::SwdToudan
        | HostBehavior::SwdXuanying
        | HostBehavior::SwdMingfu
        | HostBehavior::SwdYijue
        | HostBehavior::SwdHuodan
        | HostBehavior::SwdYudun
        | HostBehavior::SwdZhanxing
        | HostBehavior::SwdLuanji
        | HostBehavior::SwdYunchou
        | HostBehavior::SwdTianhuo
        | HostBehavior::SwdHuangyu
        | HostBehavior::SwdXianyin
        | HostBehavior::SwdXingzhui
        | HostBehavior::SwdBingjian
        | HostBehavior::SwdShending
        | HostBehavior::SwdGuiyan
        | HostBehavior::SwdFengming
        | HostBehavior::SwdDuanyi
        | HostBehavior::SwdSenluo
        | HostBehavior::SwdZhanlu
        | HostBehavior::SwdXianghui
        | HostBehavior::SwdDuishi
        | HostBehavior::SwdShuchuiUse
        | HostBehavior::SwdJiefen
        | HostBehavior::ChangeCardNature
        | HostBehavior::GujianFoodLuyugeng
        | HostBehavior::GujianYuhengEquipSkill
        | HostBehavior::SwdLangeguaiyiEquipSkill
        | HostBehavior::SwdYufuluEquipSkill
        | HostBehavior::SwdShentouEquipSkill
        | HostBehavior::SwdDuanyue
        | HostBehavior::SwdXingdian
        | HostBehavior::SwdXuehuang
        | HostBehavior::SwdLianyaohuShouna
        | HostBehavior::SwdLianyaohuLianhua
        | HostBehavior::SwdKunlunjingcCard
        | HostBehavior::SwdSifeizhenmianCard
        | HostBehavior::SwdNigongStrike
        | HostBehavior::SwdShennongdingCard
        | HostBehavior::SwdTiebi
        | HostBehavior::SwdDatong
        | HostBehavior::YunchouBeige
        | HostBehavior::WudianDiyjuntun
        | HostBehavior::YunchouJieyan
        | HostBehavior::YunchouYiesheng
        | HostBehavior::ZhangbaSha
        | HostBehavior::WudianPaotai
        | HostBehavior::WudianShouhu
        | HostBehavior::WudianBaoleiSwitch
        | HostBehavior::XianjianXtanxi
        | HostBehavior::XianjianHuahu
        | HostBehavior::XianjianLuanjian
        | HostBehavior::XianjianCtianfu
        | HostBehavior::XianjianTianjian
        | HostBehavior::XianjianSajin
        | HostBehavior::XianjianMinsha
        | HostBehavior::XianjianZhuyue
        | HostBehavior::XianjianGuanri
        | HostBehavior::XianjianDiewu
        | HostBehavior::XianjianGuiyuan
        | HostBehavior::XianjianGuijin
        | HostBehavior::XianjianXuanmo
        | HostBehavior::XianjianYuexing
        | HostBehavior::WudianKangyin
        | HostBehavior::WudianJuedao
        | HostBehavior::WudianMujing
        | HostBehavior::WudianGuangshu
        | HostBehavior::WudianZiyu
        | HostBehavior::WudianKuoyin
        | HostBehavior::WudianHuhuan
        | HostBehavior::WudianBaoxue
        | HostBehavior::WudianLiangou
        | HostBehavior::WudianYinbo
        | HostBehavior::WudianQianggu
        | HostBehavior::WudianFeitiao
        | HostBehavior::WudianChoudu
        | HostBehavior::MtgYixialanRecast
        | HostBehavior::MtgYoulinDiscard
        | HostBehavior::MtgLinzhongjiantaStrike
        | HostBehavior::DddXiaxing
        | HostBehavior::MbcmJiushi
        | HostBehavior::MbcmFangzhu
        | HostBehavior::MbJuejin => true,
        HostBehavior::U01DddBatchA => matches!(
            name,
            "dddjuxian" | "dddshilie" | "ddddiedang" | "dddbingjian" | "dddtongyu"
        ),
        HostBehavior::U01DddBatchB => matches!(
            name,
            "dddjijian" | "dddjijian_old" | "dddquche" | "dddzhijie" | "ddd_jian"
        ),
        HostBehavior::U01DddBatchC => {
            matches!(name, "dddfusi_global" | "dddfengzheng" | "dddfenye")
        }
        HostBehavior::U01DddBatchD => matches!(name, "dddchengjing" | "dddduanbing"),
        HostBehavior::L28ThunderBatchA => matches!(
            name,
            "th_dunshi"
                | "th_chongxu"
                | "miaojian"
                | "th_pingcai"
                | "th_zhengjing"
                | "th_shengong"
                | "th_buxu"
        ),
        HostBehavior::L28ThunderBatchB => matches!(
            name,
            "th_jingxie" | "th_qiaosi" | "th_tiansuan" | "th_gushe" | "th_yeyan"
        ),
        HostBehavior::L27BeautifyBatchA => {
            matches!(name, "regushe" | "taoluan" | "qimou" | "reqimou" | "jinghe")
        }
        HostBehavior::L27BeautifyBatchB => matches!(
            name,
            "bolan_g"
                | "xinfu_xionghuo"
                | "mbwuling"
                | "spyanjiao"
                | "twxingzhui"
                | "twharvestinori"
                | "twzhouhu"
                | "twzuhuo"
                | "twzhouzu"
                | "jishe2"
                | "new_zhixi_recover"
        ),
        HostBehavior::L19DoomsdayBatchA => matches!(name, "ganran2" | "_huzhu"),
        HostBehavior::L19DoomsdayBatchB => matches!(name, "_yuanqiskill"),
        HostBehavior::L20MarkBatchA => matches!(name, "wengua" | "wengua5"),
        HostBehavior::L20MarkBatchB => matches!(name, "rechuhai" | "sbduanliang" | "_chongzhu"),
        HostBehavior::U08YxsBatchA => {
            matches!(name, "liebo" | "zhuxin" | "huli" | "yixin" | "zhijie" | "chujia")
        }
        HostBehavior::U08YxsBatchB => {
            matches!(name, "baihe" | "yinyang" | "yanyi" | "bingsheng" | "shentan")
        }
        HostBehavior::U08YxsBatchC => matches!(
            name,
            "lguiyin" | "tongyu" | "ducai" | "fanpu" | "fenghuo" | "fengyi" | "seyou"
        ),
        HostBehavior::U08YxsBatchD => matches!(
            name,
            "qiandu" | "xieling" | "qiangxi" | "miaobi" | "miaobix" | "guifu" | "lshengong"
        ),
        HostBehavior::L25ZuanBatchA => matches!(name, "ssr_yongsi_yuxi" | "_yongjinskill"),
        HostBehavior::L25ZuanBatchB => {
            matches!(name, "ssr_newfanjian" | "ssr_newrende" | "ssr_qiangxi" | "ssr_qingji")
        }
        HostBehavior::L25ZuanBatchC => matches!(
            name,
            "ssr_zhujiu"
                | "ssr_jiwu"
                | "ssr_chuli"
                | "ssr_newzhenjun"
                | "ssr_shendao"
                | "ssr_miedi"
                | "ssr_newkurou"
        ),
        HostBehavior::L25ZuanBatchD => matches!(
            name,
            "ssr_newlihun"
                | "ssr_yongsi"
                | "ssr_yijue"
                | "ssr_newbaiyi"
                | "ssr_liecheng"
                | "ssr_newyucong"
                | "ssr_newguose"
                | "ssr_newtianxiang"
        ),
        HostBehavior::L25ZuanBatchE => matches!(
            name,
            "ssr_jieyin"
                | "ssr_newjijie"
                | "ssr_jiqiao"
                | "ssr_nianlei"
                | "ssr_zaiqi"
                | "ssr_lieren"
                | "ssr_weilong"
                | "ssr_newzhiba"
                | "ssr_newluanwu"
                | "ssr_newduanxian"
        ),
        HostBehavior::L25ZuanBatchF => matches!(
            name,
            "ssr_jushou"
                | "ssr_tuntian"
                | "ssr_newmizhou"
                | "ssr_newluanji"
                | "ssr_zhuhai"
                | "ssr_xuanhuo"
        ),
        HostBehavior::L25ZuanBatchG => matches!(
            name,
            "ssr_lianhuan"
                | "ssr_newsance"
                | "ssr_zhijian"
                | "ssr_newguidao"
                | "ssr_guifu"
                | "ssr_fangtong"
        ),
        HostBehavior::L01NeiKey => true,
        HostBehavior::L01AiRuleHooks => matches!(
            name,
            crate::vocabulary::host::L01_FIX_EQZ_KEY
                | crate::vocabulary::host::L01_FIX_ECF_KEY
        ),
        HostBehavior::U05ZhenfaRecastSkill => name == "u05_zhenfa_recast",
        HostBehavior::U05MtgCharBatch => name == "lingyong",
        HostBehavior::U05YunchouCharBatch => {
            matches!(name, "xinfu_jingxie" | "ycnxinfu_qiaosi" | "xinfu_qiaosi")
        }
        HostBehavior::U05GujianCardBatch
        | HostBehavior::U05GujianCharBatchA
        | HostBehavior::U05GujianCharBatchB
        | HostBehavior::U05GujianCharBatchC
        | HostBehavior::U05GujianCharBatchD
        | HostBehavior::U05GujianCharBatchE => {
            super::step::u05_b2_gujian::is_phase_use_active_u05_gujian(behavior, name)
        }
        HostBehavior::U05OwBatchA
        | HostBehavior::U05OwBatchB
        | HostBehavior::U05OwBatchC
        | HostBehavior::U05OwBatchD => {
            super::step::u05_ow::has_phase_use_activation_u05_ow(behavior, name)
        }
        b @ (HostBehavior::U05HearthBatchA
        | HostBehavior::U05HearthBatchB
        | HostBehavior::U05HearthBatchC
        | HostBehavior::U05HearthBatchD
        | HostBehavior::U05HearthBatchE
        | HostBehavior::U05Hearth2BatchA
        | HostBehavior::U05Hearth2BatchB
        | HostBehavior::U05Hearth2BatchC
        | HostBehavior::U05Hearth2BatchD
        | HostBehavior::U05Hearth2BatchE
        | HostBehavior::U05Hearth2BatchF) => {
            super::step::u05_hearth::has_phase_use_activation_hearth(b, name)
        }
        HostBehavior::U05GwentCharBatchA
        | HostBehavior::U05GwentCharBatchB
        | HostBehavior::U05GwentCharBatchC
        | HostBehavior::U05GwentCharBatchD => {
            super::step::u05_b5b_gwent::has_phase_use_u05_gwent_char(behavior, name)
        }
        HostBehavior::U05XianjianBatchD => {
            super::step::xianjian_batch::has_phase_use_activation(behavior, name)
        }
        _ => false,
    }
}

pub(in crate::content::host) fn is_host_skill_bolan_eligible(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    if !has_phase_use_activation(behavior, name) {
        return false;
    }
    if name.starts_with('_') || name.ends_with("_card") || name.ends_with("_global") {
        return false;
    }
    if matches!(
        behavior,
        HostBehavior::U05OwBatchA
            | HostBehavior::U05OwBatchB
            | HostBehavior::U05OwBatchC
            | HostBehavior::U05OwBatchD
    ) && !super::step::u05_ow::is_bolan_eligible_u05_ow(name)
    {
        return false;
    }
    !matches!(
        behavior,
        HostBehavior::Niepan
            | HostBehavior::Bingfeng
            | HostBehavior::Hxunzhi
            | HostBehavior::SwdJinlin
            | HostBehavior::SwdTianshu
            | HostBehavior::SwdXuehuang
            | HostBehavior::WudianShouhu
            | HostBehavior::WudianDiyjuntun
            | HostBehavior::SwdShentouEquipSkill
            | HostBehavior::SwdYufuluEquipSkill
            | HostBehavior::SwdLangeguaiyiEquipSkill
            | HostBehavior::SwdLianyaohuShouna
            | HostBehavior::SwdLianyaohuLianhua
            | HostBehavior::SwdKunlunjingcCard
            | HostBehavior::SwdSifeizhenmianCard
            | HostBehavior::SwdShennongdingCard
            | HostBehavior::SwdShuchuiUse
            | HostBehavior::ChangeCardNature
            | HostBehavior::GujianFoodLuyugeng
            | HostBehavior::GujianYuhengEquipSkill
            | HostBehavior::ZhangbaSha
            | HostBehavior::MtgYixialanRecast
            | HostBehavior::MtgYoulinDiscard
            | HostBehavior::MtgLinzhongjiantaStrike
    ) && !matches!(
        name,
        "rechuhai"
            | "th_yeyan"
            | "qimou"
            | "reqimou"
            | "taoluan"
            | "dddchengjing"
            | "dddduanbing"
            | "dddzhijie"
            | "dddfengzheng"
            | "dddfenye"
            | "dddshilie"
            | "dddjijian_old"
            | "th_chongxu"
            | "th_tiansuan"
            | "wengua5"
            | "ssr_newzhiba"
            | "ssr_newluanwu"
            | "ssr_newduanxian"
            | "ssr_jiwu"
            | "ssr_yongsi_yuxi"
            | "xinfu_lingren"
            | "shouye"
            | "yixin"
            | "fengyi"
            | "qiandu"
            | "xieling"
            | "miaobix"
            | "lguiyin"
            | "tongyu"
            | "bolan"
            | "bolan_g"
            | "kotomi_chuanxiang"
            | "jiehuo"
            | "gjyuheng_plus_skill"
            | "gjyuheng_pro_skill"
    )
}
