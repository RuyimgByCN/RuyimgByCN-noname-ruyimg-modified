use super::*;

impl HostSkillBehavior {
    /// 墨守：延时锦囊（乐/兵）不能指定持有者为目标。
    pub(super) fn blocks_delayed_trick(&self) -> bool {
        self.behavior == HostBehavior::MoshouImmunity
            || (self.behavior == HostBehavior::L28ThunderBatchA && self.name == "xinfu_pdgyingshi")
    }

    pub(super) fn machine(&self, owner: PlayerId, event: &EventData) -> HostMachine {
        match self.behavior {
            HostBehavior::U01DddBatchA => {
                return super::step::u01_batch_a::machine_u01_batch_a(self, owner, event);
            }
            HostBehavior::U01DddBatchB => {
                return super::step::u01_batch_b::machine_u01_batch_b(self, owner, event);
            }
            HostBehavior::U01DddBatchC => {
                return super::step::u01_batch_c::machine_u01_batch_c(self, owner, event);
            }
            HostBehavior::U01DddBatchD => {
                return super::step::u01_batch_d::machine_u01_batch_d(self, owner, event);
            }
            HostBehavior::L28ThunderBatchA => {
                return super::step::l28_batch_a::machine_l28_batch_a(self, owner, event);
            }
            HostBehavior::L28ThunderBatchB => {
                return super::step::l28_batch_b::machine_l28_batch_b(self, owner, event);
            }
            HostBehavior::L27BeautifyBatchA => {
                return super::step::l27_batch_a::machine_l27_batch_a(self, owner, event);
            }
            HostBehavior::L27BeautifyBatchB => {
                return super::step::l27_batch_b::machine_l27_batch_b(self, owner, event);
            }
            HostBehavior::L19DoomsdayBatchA => {
                return super::step::l19_batch_a::machine_l19_batch_a(self, owner, event);
            }
            HostBehavior::L19DoomsdayBatchB => {
                return super::step::l19_batch_b::machine_l19_batch_b(self, owner, event);
            }
            HostBehavior::L20MarkBatchA => {
                return super::step::l20_batch_a::machine_l20_batch_a(self, owner, event);
            }
            HostBehavior::L20MarkBatchB
            | HostBehavior::FzLiegong
            | HostBehavior::FzXinKuanggu => {
                return super::step::l20_batch_b::machine_l20_batch_b(self, owner, event);
            }
            _ => {}
        }
        let (target, card) = match self.behavior {
            HostBehavior::ShaDamageForceDiscard => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::ShengkeCheck => match event {
                EventData::CardUsed(u) => (None, u.card),
                _ => (None, None),
            },
            HostBehavior::MaxRangeWeaponDamage => match event {
                EventData::Equipped { card, .. } => (None, Some(*card)),
                _ => (None, None),
            },
            HostBehavior::WangmeiZhikeCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            // 每步：出牌阶段开始者（被止息者）
            HostBehavior::NewMeibu => match event {
                EventData::Phase {
                    player,
                    begin: true,
                    ..
                } => (Some(*player), None),
                _ => (None, None),
            },
            // 迅影：同一目标（杀结算后对同目标再杀）
            HostBehavior::XianjianXunying => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            // 破云：受伤者（被弃者自选弃牌）
            HostBehavior::XianjianPoyun => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::SijiDiscardDraw => match event {
                EventData::CardsDiscarded { cards: _, .. } => (None, None),
                _ => (None, None),
            },
            HostBehavior::SanjianWeaponDamage => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::MoshouImmunity => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::ShuigongCard => match event {
                EventData::CardUsed(u) => (None, u.card),
                _ => (None, None),
            },
            HostBehavior::FudichouxinCompare => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            HostBehavior::ChenhuodajieCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::TouliangSwap => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            HostBehavior::LinghunzhihuoCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            HostBehavior::ZhaomingdanCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            HostBehavior::JihuocardDraw => match event {
                EventData::CardUsed(u) => (Some(u.player), u.card),
                _ => (None, None),
            },
            HostBehavior::GeanguanhuoCompare => match event {
                EventData::CardUsed(u) => (None, u.card),
                _ => (None, None),
            },
            HostBehavior::CaochuanCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            HostBehavior::ShezhanqunruCompare => match event {
                EventData::CardUsed(u) => (None, u.card),
                _ => (None, None),
            },
            HostBehavior::JinlianzhuCancel => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianShuaiyan => match event {
                EventData::Recover { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::WudianZhuchengGuard => match event {
                EventData::CardUsed(u) => (Some(u.player), u.card),
                _ => (None, None),
            },
            HostBehavior::WudianMujing => match event {
                EventData::CardUsed(u) => (Some(u.player), u.card),
                _ => (None, None),
            },
            HostBehavior::WudianGuangshuHeart => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianGuangshuDiamond => match event {
                EventData::Damage(d) => (Some(d.source.unwrap_or(owner)), None),
                _ => (None, None),
            },
            HostBehavior::WudianFeidan => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianZhiyuan => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianQiangguGuard => match event {
                EventData::CardUsed(u) => (Some(u.player), u.card),
                _ => (None, None),
            },
            HostBehavior::WudianPingzhang => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 冰心（A2 H80）：整回合结束者为 +1 护甲目标（global phaseEnd）。
            HostBehavior::XianjianBinxin => match event {
                EventData::TurnEndWindow { player, after: false } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianFeixu => match event {
                EventData::CardUsed(u) => (Some(u.player), None),
                EventData::CardResponded { card: _, player } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianAnwugu => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 灵火（A2 H83）：整回合结束者为 1 火伤目标（global phaseEnd，
            // filter 限他人）。
            HostBehavior::XianjianLinghuo => match event {
                EventData::TurnEndWindow { player, after: false } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianShuiyun5 => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianWangyou2 => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::XianjianChengxin => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianShenmu => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianHushaCount => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::XianjianShiying => match event {
                EventData::Death { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianZhaoyao => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianCtianfu3 => match event {
                EventData::Death { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::SwdGouhunluoCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), u.card),
                _ => (None, None),
            },
            // 雄姿：自身（补摸量在效果内按装备数计算）
            HostBehavior::YunchouXiongzi => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            // 雅量：伤害目标（双方经 targets=[source,target] 提取摸牌）
            HostBehavior::YunchouYaliang => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 戏才：伤害目标+伤害牌（获得面）
            HostBehavior::YunchouXicai => match event {
                EventData::Damage(d) => (Some(d.target), d.card),
                _ => (None, None),
            },
            // 狂骨·变体：伤害目标（距离判定面）
            HostBehavior::YunchouDiykuanggu => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 纵火：伤害目标（帧直写在 step）
            HostBehavior::YunchouZonghuo => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 烧营：受伤者（蔓延目标在 step 内按距离自算）
            HostBehavior::YunchouShaoying => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 蛊惑·变体（A2 H17）：自身（上游 player phaseBegin＝自己回合
            // 开始；原 Phase 帧提取臂同步迁 TurnBegin）
            HostBehavior::YunchouDiyguhuo => match event {
                EventData::TurnBegin { player } => (Some(*player), None),
                _ => (None, None),
            },
            // 狡黠：自身（摸 1）
            HostBehavior::YunchouJiaoxia => (Some(owner), None),
            // 复仇：目录锚（无提取）
            HostBehavior::WudianFuchou => (None, None),
            // 归汉：目录锚（无提取）
            HostBehavior::WudianGuihan => (None, None),
            HostBehavior::SwdGTiandao => (None, None),
            // 热血（A2 H42）：回合开始者为本次杀的目标（global phaseBegin）。
            HostBehavior::SwdRexue => match event {
                EventData::TurnBegin { player } => (Some(*player), None),
                _ => (None, None),
            },
            // 幻音：牌的使用者为关联目标，牌为关联卡
            HostBehavior::SwdHuanyin => match event {
                EventData::CardUsed(u) => (Some(u.player), u.card),
                _ => (None, None),
            },
            HostBehavior::SwdLingxian => (None, None),
            HostBehavior::SwdShehun => (None, None),
            HostBehavior::SwdHyunshen => (None, None),
            HostBehavior::SwdHyunshen2 => (None, None),
            HostBehavior::SwdDaixing => (None, None),
            HostBehavior::SwdDaixing2 => (None, None),
            HostBehavior::SwdDaixing3 => (None, None),
            HostBehavior::SwdJuxi => (None, None),
            HostBehavior::SwdJuxiCount => (None, None),
            HostBehavior::SwdPozhou => (None, None),
            HostBehavior::SwdPozhou2 => (None, None),
            // 陷阱·触发：牌的使用者为关联目标、牌为关联卡
            HostBehavior::SwdSxianjingGain => match event {
                EventData::CardUsed(u) => (Some(u.player), u.card),
                _ => (None, None),
            },
            HostBehavior::SwdSxianjing => (None, None),
            HostBehavior::SwdSxianjingDamage => (None, None),
            HostBehavior::SwdGuxing => (None, None),
            HostBehavior::SwdGuxing3 => (None, None),
            HostBehavior::SwdLingshi => (None, None),
            HostBehavior::SwdJinlin => (None, None),
            HostBehavior::SwdJinlin2 => (None, None),
            HostBehavior::HlssXiangxing => (None, None),
            HostBehavior::HlssYueyin => (None, None),
            HostBehavior::HlssYueyin2 => (None, None),
            HostBehavior::HlssFengqi => (None, None),
            HostBehavior::HlssFengqi2 => (None, None),
            HostBehavior::HlssGaiming => (None, None),
            HostBehavior::MbQianlong => (None, None),
            HostBehavior::MbWeitong => (None, None),
            HostBehavior::Niepan => (None, None),
            HostBehavior::Yuhuo => (None, None),
            HostBehavior::Shenwei => (None, None),
            HostBehavior::MbcmQingzheng => (None, None),
            HostBehavior::MbcmJiushi => match event {
                EventData::Damage(_) => (None, Some(1)),
                EventData::TurnOvered { .. } => (None, Some(2)),
                _ => (None, None),
            },
            HostBehavior::MbcmFangzhu | HostBehavior::MbJuejin => (None, None),
            HostBehavior::SwdLingwu => (None, None),
            HostBehavior::SwdShenyan => (None, None),
            HostBehavior::SwdRumeng => (None, None),
            HostBehavior::SwdYishan => (None, None),
            HostBehavior::SwdShouyin => (None, None),
            HostBehavior::SwdMeihuo => (None, None),
            HostBehavior::SwdJikong => (None, None),
            HostBehavior::SwdLinyun => (None, None),
            HostBehavior::SwdZhuhai => (None, None),
            HostBehavior::Songci => (None, None),
            HostBehavior::Tianxiang => (None, None),
            HostBehavior::Tianxiang2 => (None, None),
            HostBehavior::Hutian => (None, None),
            HostBehavior::Hxunzhi => (None, None),
            HostBehavior::Hxunzhi2 => (None, None),
            HostBehavior::Cyqiaoxie => (None, None),
            HostBehavior::Bingfeng => (None, None),
            HostBehavior::Bingfeng2 => (None, None),
            HostBehavior::Gxianyin => (None, None),
            HostBehavior::Jilve => (None, None),
            HostBehavior::Mailun => (None, None),
            HostBehavior::Tianlun => (None, None),
            HostBehavior::Zhenlie => (None, None),
            HostBehavior::Yihua => (None, None),
            HostBehavior::Lxianglong => (None, None),
            HostBehavior::SwdTianshu => (None, None),
            HostBehavior::Gaizao => (None, None),
            HostBehavior::Liaoyuan => (None, None),
            HostBehavior::XuanyuanjianDamage => (None, None),
            HostBehavior::XuanyuanjianBacklash => (None, None),
            HostBehavior::Xuanyuan => (None, None),
            HostBehavior::Shengong => (None, None),
            HostBehavior::Kunlunjing => (None, None),
            HostBehavior::Jifeng => (None, None),
            HostBehavior::Moyan => (None, None),
            HostBehavior::Miedao => (None, None),
            HostBehavior::Xiuluo => (None, None),
            HostBehavior::Mohua2 => (None, None),
            HostBehavior::Gongxin => (None, None),
            HostBehavior::Reguhuo => (None, None),
            HostBehavior::Rangjie => (None, None),
            HostBehavior::Xiefang => (None, None),
            HostBehavior::NewZhixi => (None, None),
            HostBehavior::NewMeibuRange => (None, None),
            HostBehavior::SbZhaXiang => (None, None),
            HostBehavior::Jishe => (None, None),
            HostBehavior::Shishi => (None, None),
            HostBehavior::Xunmeng => (None, None),
            HostBehavior::Zaibian => (None, None),
            HostBehavior::YxsFanji => (None, None),
            HostBehavior::YxsFanji2 => (None, None),
            HostBehavior::YxsMenshen => (None, None),
            HostBehavior::YxsMenshen3 => (None, None),
            HostBehavior::YxsMenshen2 => (None, None),
            HostBehavior::XiguaMopaiFanbei => (None, None),
            HostBehavior::XiguaXueliangFanbei => (None, None),
            HostBehavior::Kurou => (None, None),
            HostBehavior::ZhuWanzhongJi => (None, None),
            HostBehavior::FzLiegong => (None, None),
            HostBehavior::FzXinKuanggu => (None, None),
            HostBehavior::SwdNlianji => (None, None),
            // 铁壁：杀的首个非持有者目标（活/距/护甲校验在 filter）
            HostBehavior::SwdTiebi => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), None),
                _ => (None, None),
            },
            // 大同：自己（摸 2）
            HostBehavior::SwdDatong => (Some(owner), None),
            // 承剑：伤害来源（摸 1 面在效果）
            HostBehavior::SwdChengjian => match event {
                EventData::Damage(d) => (d.source, None),
                _ => (None, None),
            },
            // 缚斧：伤害目标（效果直写帧）
            HostBehavior::SwdJufu => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 花镜：伤害目标
            HostBehavior::SwdHuajing => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 护盾：自己（+1 护甲）
            HostBehavior::SwdHudun => (Some(owner), None),
            // 破星：伤害目标（效果直写帧）
            HostBehavior::SwdPoxing => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 流虹：自己（摸 1）
            HostBehavior::SwdLiuhong => (Some(owner), None),
            // 墨剑：杀的首个目标（摸 1；回 1 面在效果）
            HostBehavior::SwdMojian => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), None),
                _ => (None, None),
            },
            // 道法：伤害来源（弃其手牌）
            HostBehavior::SwdDaofa => match event {
                EventData::Damage(d) => (d.source, None),
                _ => (None, None),
            },
            // 玉鳞：无目标（弃自己装备牌 + 免伤）
            HostBehavior::SwdYulin => (None, None),
            // 凝霜：黑牌使用者（阶梯对象）
            HostBehavior::SwdNingshuang => match event {
                EventData::CardUsed(u) => (Some(u.player), None),
                _ => (None, None),
            },
            // 工神：用牌者（取消其使用）
            HostBehavior::SwdGongshen => match event {
                EventData::CardUsed(u) => (Some(u.player), None),
                _ => (None, None),
            },
            // 屠阵：受伤者（弃其非基本手牌）
            HostBehavior::SwdTuzhen => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 女娲石：濒死者（回复其 1）
            HostBehavior::SwdNvwashiGuard => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            // 悲歌：targets=[来源, 受伤者]
            HostBehavior::YunchouBeige => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 蝉缘·变体：死亡来源（减其上限）
            HostBehavior::YunchouDiychanyuan => match event {
                EventData::Death {
                    killer: Some(k), ..
                } => (Some(*k), None),
                _ => (None, None),
            },
            // 理断：获得者+所获装备（第 9 触点：card 必须提取）
            HostBehavior::WudianLiduan => match event {
                EventData::CardsMoved { cards, to, .. } => (Some(*to), cards.first().copied()),
                _ => (None, None),
            },
            // 厉焰·链杀：用水攻的持有者（目标取 targets 首）
            HostBehavior::YunchouLuweiyan2 => (Some(owner), None),
            // 红莲：伤害来源（弃其红牌）
            HostBehavior::YunchouHonglian => match event {
                EventData::Damage(d) => (d.source, None),
                _ => (None, None),
            },
            // 侠行（3D精选·character/skill.js:59）：记录受益角色与触发分支（None=准备阶段得闪，Some(1)=濒死得桃）
            HostBehavior::DddXiaxing => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                EventData::Dying { player, .. } => (Some(*player), Some(1)),
                _ => (None, None),
            },
            // 诱战（玩点论杀·character/skill.js:422）：记录杀使用者与杀实体牌
            HostBehavior::WudianYouzhan => match event {
                EventData::CardUsed(u) => (Some(u.player), u.card),
                _ => (None, None),
            },
            // 殁亡（L10 史诗卡牌·extension.js:2677）：记录触发分支（0=濒死，1=准备阶段归来，2=结束阶段崩解）
            HostBehavior::KMowang => match event {
                EventData::Dying { .. } => (None, Some(0)),
                EventData::Phase {
                    phase: crate::event::Phase::Prepare,
                    begin: true,
                    ..
                } => (None, Some(1)),
                EventData::Phase {
                    phase: crate::event::Phase::End,
                    begin: false,
                    ..
                } => (None, Some(2)),
                _ => (None, None),
            },
            _ => (None, None),
        };
        let stage = match self.behavior {
            HostBehavior::TargetDiscardEquipOrDamage => HostStage::TargetChoice,
            HostBehavior::MaxRangeWeaponDamage => HostStage::TargetChoice,
            HostBehavior::SanjianWeaponDamage => HostStage::SelectCard,
            HostBehavior::FudichouxinCompare => HostStage::CompareTargetPick,
            HostBehavior::CaochuanCard => HostStage::SelectNature,
            HostBehavior::ChangeCardNature => HostStage::SelectCard,
            HostBehavior::TouliangSwap => HostStage::SelectCard,
            HostBehavior::GeanguanhuoCompare => HostStage::CompareTargetPick,
            HostBehavior::GujianShujinsanCard => HostStage::SelectCard,
            HostBehavior::YunchouYiesheng => HostStage::SelectCard,
            HostBehavior::WudianDiyjuntun => HostStage::SelectCard,
            HostBehavior::GujianFeibiaoCard => HostStage::SelectCard,
            HostBehavior::JiguanBingpoCard => HostStage::ComparePlayerPick,
            HostBehavior::JiguanLongxuCard => HostStage::SelectCard,
            HostBehavior::GwentBaoxueCard => HostStage::SelectCard,
            HostBehavior::WudianKuoyin => HostStage::SelectCard,
            HostBehavior::WudianHuhuan => HostStage::SelectCard,
            HostBehavior::WudianBaoxue => HostStage::SelectCard,
            HostBehavior::WudianLiangou => HostStage::SelectCard,
            HostBehavior::WudianYinbo => HostStage::SelectCard,
            HostBehavior::WudianQianggu => HostStage::SelectCard,
            HostBehavior::WudianFeitiao => HostStage::SelectCard,
            HostBehavior::XianjianGuiyuan => HostStage::SelectCard,
            HostBehavior::XianjianDiewu => HostStage::SelectCard,
            HostBehavior::XianjianSajin => HostStage::SelectCard,
            HostBehavior::XianjianYuexing => HostStage::Act,
            HostBehavior::XianjianMinsha => HostStage::SelectCard,
            HostBehavior::XianjianZhuyue => HostStage::SelectCard,
            HostBehavior::XianjianGuanri => HostStage::SelectCard,
            HostBehavior::XianjianShuiyun => HostStage::SelectCard,
            HostBehavior::XianjianWangyou => HostStage::SelectCard,
            HostBehavior::XianjianXuanmo => HostStage::SelectCard,
            HostBehavior::XianjianHuahu => HostStage::TargetChoice,
            HostBehavior::XianjianXtanxi => HostStage::SelectCard,
            HostBehavior::XianjianTianjian => HostStage::SelectCard,
            HostBehavior::SwdDonghuangzhongCard => HostStage::SelectCard,
            _ => HostStage::Act,
        };
        let cards = match self.behavior {
            HostBehavior::SijiDiscardDraw => match event {
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 聚宝：本次弃入弃堆的牌（随机回收候选池）
            HostBehavior::XianjianJtjubao => match event {
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 摄灵：本次事件入弃堆的牌（回收候选）
            HostBehavior::XianjianSheling => match event {
                EventData::CardUsed(u) => u.card.into_iter().collect(),
                EventData::CardResponded { card, .. } => vec![*card],
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 双刃：本次弃入弃堆的牌（武器识别候选）
            HostBehavior::XianjianXshuangren => match event {
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 射影：伤害牌（判定黑回收目标）
            HostBehavior::XianjianSheying => match event {
                EventData::Damage(d) => d.card.map(|c| vec![c]).unwrap_or_default(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        let targets = match self.behavior {
            HostBehavior::ShuigongCard => match event {
                EventData::CardUsed(u) => u.targets.clone(),
                _ => Vec::new(),
            },
            HostBehavior::JinlianzhuCancel => match event {
                EventData::Damage(d) => d.source.map(|s| vec![s]).unwrap_or_default(),
                _ => Vec::new(),
            },
            // 雅量：伤害双方（座位序在效果内排序）
            HostBehavior::YunchouYaliang => match event {
                EventData::Damage(d) => vec![d.source.unwrap_or(d.target), d.target],
                _ => Vec::new(),
            },
            // 厉焰·链杀：水攻目标集
            HostBehavior::YunchouLuweiyan2 => match event {
                EventData::CardUsed(u) => u.targets.clone(),
                _ => Vec::new(),
            },
            // 悲歌：targets=[来源, 受伤者]
            HostBehavior::YunchouBeige => match event {
                EventData::Damage(d) => vec![d.source.unwrap_or(d.target), d.target],
                _ => Vec::new(),
            },
            HostBehavior::WudianYouzhan => match event {
                EventData::CardUsed(u) => u.targets.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        HostMachine {
            def_key: self.name.clone(),
            behavior: self.behavior,
            owner,
            target,
            card,
            cards,
            targets,
            stage,
        }
    }
}
