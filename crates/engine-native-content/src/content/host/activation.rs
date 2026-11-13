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
            HostBehavior::U01DddBatchA => {
                super::step::u01_batch_a::activate_u01_batch_a(self, ctx, owner)
            }
            HostBehavior::U01DddBatchB => {
                super::step::u01_batch_b::activate_u01_batch_b(self, ctx, owner)
            }
            HostBehavior::U01DddBatchC => {
                super::step::u01_batch_c::activate_u01_batch_c(self, ctx, owner)
            }
            HostBehavior::U01DddBatchD => {
                super::step::u01_batch_d::activate_u01_batch_d(self, ctx, owner)
            }
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
            _ => None,
        }
    }
}
