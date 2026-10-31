use super::*;

fn require(ok: bool) -> Result<(), String> {
    ok.then_some(())
        .ok_or_else(|| "invalid public presentation metadata".into())
}
fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && s != "."
        && s != ".."
        && !s.chars().any(|c| c.is_control() || "/\\%?#:".contains(c))
}
fn path(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.split('/').all(token)
}
fn hex(s: &str, len: usize) -> bool {
    s.len() == len
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn repository(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && !s
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
}
fn owner(v: &FrozenOwnerKey) -> bool {
    match v {
        FrozenOwnerKey::Package {
            namespace,
            package_id,
            version,
            logic_hash,
            content_hash,
        } => {
            token(namespace)
                && token(package_id)
                && token(version)
                && hex(logic_hash, 64)
                && hex(content_hash, 64)
        }
        FrozenOwnerKey::Original {
            repository,
            adopted_revision,
            catalog_sha256,
            asset_manifest_sha256,
        } => {
            self::repository(repository)
                && hex(adopted_revision, 40)
                && hex(catalog_sha256, 64)
                && hex(asset_manifest_sha256, 64)
        }
    }
}
fn definition(v: &QualifiedDefinitionKey, kind: PresentationDefinitionKind) -> bool {
    owner(&v.owner)
        && v.kind == kind
        && match (&v.owner, &v.local_id) {
            (FrozenOwnerKey::Package { .. }, PresentationLocalId::Package(id)) => {
                kind != PresentationDefinitionKind::Card || *id > 0
            }
            (FrozenOwnerKey::Original { .. }, PresentationLocalId::Original(id)) => {
                token(&id.pack) && token(&id.raw_symbol)
            }
            _ => false,
        }
}
fn asset(v: &BoundAudioAsset) -> bool {
    owner(&v.source)
        && hex(&v.digest, 64)
        && v.size > 0
        && v.size <= MAX_PRESENTATION_REVISION
        && match (&v.source, &v.reference) {
            (
                FrozenOwnerKey::Package {
                    namespace: ns,
                    package_id: pid,
                    ..
                },
                PresentationAssetRef::Package {
                    namespace,
                    package_id,
                    asset_id,
                },
            ) => {
                ns == namespace
                    && pid == package_id
                    && token(asset_id)
                    && v.digest_algorithm == PresentationDigestAlgorithm::Blake3
            }
            (
                FrozenOwnerKey::Original { .. },
                PresentationAssetRef::Original {
                    path: p,
                    source_path,
                    ..
                },
            ) => {
                path(p)
                    && path(source_path)
                    && (p.starts_with("audio-declared/")
                        || p.starts_with("audio/")
                        || p.starts_with("audio-compat/"))
                    && source_path.starts_with("audio/")
                    && v.digest_algorithm == PresentationDigestAlgorithm::Sha256
            }
            _ => false,
        }
}
fn voice(v: &Option<PublicVoice>) -> bool {
    v.as_ref().is_none_or(|v| {
        definition(&v.role, PresentationDefinitionKind::General)
            && v.form.as_deref().is_none_or(token)
            && !v.assets.is_empty()
            && v.assets.len() <= 1024
            && v.assets.iter().all(asset)
            && v.assets
                .iter()
                .enumerate()
                .all(|(i, a)| !v.assets[..i].contains(a))
    })
}
fn player(p: PlayerId, players: u32) -> bool {
    p.raw() > 0 && p.raw() <= players
}
fn targets(v: &Option<Vec<PlayerId>>, players: u32) -> bool {
    v.as_ref().is_none_or(|v| {
        v.len() <= 16
            && v.iter().all(|p| player(*p, players))
            && v.iter().enumerate().all(|(i, p)| !v[..i].contains(p))
    })
}
fn effective(v: &EffectivePresentationCard) -> bool {
    // These are the existing CardKind::key values; no kernel dependency belongs in this leaf crate.
    CARD_KINDS.contains(&v.kind.as_str()) && token(&v.key)
}

impl PublicPresentationBatch {
    /// Validate leaf constraints. This does not authorize public disclosure or resolve resources.
    pub fn validate(&self) -> Result<(), String> {
        require(
            self.revision <= MAX_PRESENTATION_REVISION
                && self.occurrences.len() <= MAX_PRESENTATION_OCCURRENCES
                && self.unavailable.len() <= 28
                && self
                    .unavailable
                    .iter()
                    .enumerate()
                    .all(|(i, v)| !self.unavailable[..i].contains(v))
                && (self.status != PresentationStatus::Complete || self.unavailable.is_empty())
                && (self.status != PresentationStatus::Overflow || self.occurrences.is_empty()),
        )?;
        for (index, v) in self.occurrences.iter().enumerate() {
            require(v.ordinal as usize == index)?;
        }
        self.validate_players(u32::MAX)
    }

    /// Validate API one-based player references against the accompanying observer seat count.
    pub fn validate_players(&self, players: u32) -> Result<(), String> {
        for v in &self.occurrences {
            let ok = match &v.event {
                PublicPresentationEvent::CardUsed {
                    actor,
                    targets: ts,
                    effective: e,
                    voice: v,
                } => player(*actor, players) && targets(ts, players) && effective(e) && voice(v),
                PublicPresentationEvent::CardResponded {
                    actor,
                    effective: e,
                    voice: v,
                } => player(*actor, players) && effective(e) && voice(v),
                PublicPresentationEvent::SkillActivated {
                    actor,
                    skill,
                    targets: ts,
                    voice: v,
                } => {
                    player(*actor, players)
                        && definition(&skill.definition, PresentationDefinitionKind::Skill)
                        && targets(ts, players)
                        && voice(v)
                }
                PublicPresentationEvent::DamageDealt {
                    source,
                    target,
                    amount,
                    ..
                } => {
                    source.is_none_or(|p| player(p, players))
                        && player(*target, players)
                        && *amount > 0
                }
                PublicPresentationEvent::DamagePrevented { source, target, .. } => {
                    source.is_none_or(|p| player(p, players)) && player(*target, players)
                }
                PublicPresentationEvent::HpRecovered { actor, amount } => {
                    player(*actor, players) && *amount > 0
                }
                PublicPresentationEvent::PlayerDied {
                    actor,
                    killer,
                    voice: v,
                } => {
                    player(*actor, players) && killer.is_none_or(|p| player(p, players)) && voice(v)
                }
            };
            require(ok)?;
        }
        Ok(())
    }
}

const CARD_KINDS: &[&str] = &[
    "sha",
    "shan",
    "tao",
    "horse_plus",
    "horse_minus",
    "weapon",
    "armor",
    "huosha",
    "tiesuo",
    "nanman",
    "wanjian",
    "juedou",
    "wuxie",
    "lebu",
    "bingliang",
    "shandian",
    "guohe",
    "shunshou",
    "jiu",
    "leisha",
    "shoulijian",
    "wuxingpan",
    "wangmeizhike",
    "sanjian",
    "shuigong",
    "fudichouxin",
    "chenhuodajie",
    "suolianjia",
    "hongshui",
    "toulianghuanzhu",
    "linghunzhihuo",
    "zhaomingdan",
    "jihuocard",
    "youdishenru",
    "geanguanhuo",
    "caochuanjiejian",
    "shezhanqunru",
    "shujinsan",
    "mianlijinzhen",
    "feibiao",
    "jinlianzhu",
    "tanhuadong",
    "yougeng",
    "qingtuan",
    "yuanbaorou",
    "gudonggeng",
    "mapodoufu",
    "chunbing",
    "liyutang",
    "molicha",
    "xiajiao",
    "mizhilianou",
    "luyugeng",
    "bingpotong",
    "qiankunbiao",
    "longxugou",
    "shenhuofeiya",
    "ziyangdan",
    "yunvyuanshen",
    "shihuifen",
    "liufengsan",
    "shatang",
    "heilonglinpian",
    "liutouge",
    "mutoumianju",
    "dinvxuanshuang",
    "gjyuheng",
    "dunpaigedang",
    "zhiliaobo",
    "yuansuhuimie",
    "xingjiegoutong",
    "chuansongmen",
    "tanshezhiren",
    "shijieshu",
    "yanziyaoshui",
    "aozuzhilei",
    "wenyi",
    "zhihuanjun",
    "baoxueyaoshui",
    "qinpendayu",
    "birinongwu",
    "ciguhanshuang",
    "kunenfayin",
    "shanbengshu",
    "poxiao",
    "zumoshoukao",
    "monkey",
    "xuelunyang",
    "mianju",
    "jiuwei",
    "wuzhong",
    "taoyuan",
    "wugu",
    "jiedao",
    "renwang",
    "qinggang",
    "xuanyuanjian",
    "qilin",
    "hanbing",
    "zhuge",
    "guanshi",
    "qinglong",
    "zhangba",
    "cixiong",
    "fangtian",
    "hufu",
    "liuxinghuoyu",
    "caoyao",
    "chilongya",
    "tianxianjiu",
    "xixueguizhihuan",
    "guiyoujie",
    "shentoumianju",
    "xingjunyan",
    "qinglonglingzhu",
    "zhufangshenshi",
    "fengxueren",
    "yufulu",
    "yangpijuan",
    "daihuofenglun",
    "xiayuncailing",
    "langeguaiyi",
    "longlushanfeng",
    "haidao",
    "shuimomuxue",
    "feixu",
    "yixialan",
    "shamolvzhou",
    "linzhongjianta",
    "bingheyaosai",
    "youlin",
    "lindixiliu",
    "dizaizhen",
    "jiguan",
    "xuejibingbao",
    "shenmiguo",
    "fengyinzhidan",
    "lianyaohu",
    "qiankundai",
    "shennongding",
    "nvwashi",
    "kunlunjingc",
    "donghuangzhong",
    "kongdongyin",
    "jingleishan",
    "chiyuxi",
    "guisheqi",
    "xianluhui",
    "guilingzhitao",
    "sifeizhenmian",
    "shuchui",
    "sadengjinhuan",
    "haotianta",
    "yuruyi",
    "yuchanqian",
    "yuchankun",
    "yuchanzhen",
    "yuchanxun",
    "yuchankan",
    "yuchanli",
    "yuchangen",
    "yuchandui",
    "shencaojie",
    "qinglongzhigui",
    "zhuquezhizhang",
    "baishouzhihu",
    "xuanwuzhihuang",
    "huanglinzhicong",
    "hslingjian",
    "tianfuzhen",
    "qixingzhen",
    "niaoxiangzhen",
    "changshezhen",
    "pozhenjue",
];
