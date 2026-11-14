use super::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Default, Deserialize, Resource)]
pub(super) struct Catalog {
    pub texts: HashMap<u32, String>,
    fixtures: HashMap<String, String>,
    species: HashMap<String, String>,
    moves: HashMap<String, String>,
    items: HashMap<String, String>,
    maps: HashMap<String, String>,
}

impl Catalog {
    pub fn load(assets: &Path) -> Self {
        let path = assets.join("locale/zh-CN.json");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn map<'a>(&'a self, map: &'a firered_native_prototype::Map) -> &'a str {
        self.maps.get(&map.id).map_or(&map.name, String::as_str)
    }

    pub fn species<'a>(&'a self, name: &'a str) -> &'a str {
        self.species.get(&name.to_uppercase()).map_or(name, String::as_str)
    }

    pub fn move_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.moves.get(id).map_or(id.trim_start_matches("MOVE_"), String::as_str)
    }

    pub fn argument<'a>(&'a self, value: &'a str) -> &'a str {
        let item = value.to_uppercase();
        self.items.get(&item).map_or_else(|| self.species(value), String::as_str)
    }

    pub fn dialog(&self, line: &str) -> String {
        let (body, prompt) = line.strip_suffix("\n\n1: Yes    2: No")
            .map_or((line, ""), |body| (body, "\n\n1 是　　2 否"));
        let key = body.replace("Pokémon", "Pokemon").replace("POKéMON", "POKEMON").replace("POKéDEX", "POKEDEX");
        let key = key.split_whitespace().collect::<Vec<_>>().join(" ");
        let localized = self.fixtures.get(&key).map(String::as_str).unwrap_or(body);
        let localized = match localized {
            "A party Pokémon fainted from poison." => "队伍中有宝可梦因中毒倒下了。",
            "1: Yes    2: No" => "1 是　　2 否",
            "RED obtained the POTION!" => "小赤得到了伤药！",
            "RED put the POTION away in\nthe BAG's ITEMS POCKET." => "小赤把伤药放进了包包的道具口袋。",
            "Please come see us if you need\nPOKé BALLS for catching POKéMON." => "如果需要精灵球来捕捉宝可梦，请来我们的商店看看。",
            "This interaction is not available in this slice yet." => "当前试玩版暂未开放此互动。",
            "This event is not ported yet. Turn off original field scripts in Settings to use the exploration fixture." => "此事件尚未移植。可在设置中关闭原版剧情脚本，使用探索模式。",
            "This event is not ported yet. Its state changes were discarded.\nTurn off original field scripts in Settings to use the exploration fixture." => "此事件尚未移植。可在设置中关闭原版剧情脚本，使用探索模式。",
            "RED scurried back home, protecting the exhausted and fainted Pokémon from further harm…" => "小赤带着疲惫不堪的宝可梦赶回了家……",
            "RED scurried to a Pokémon Center, protecting the exhausted and fainted Pokémon from further harm…" => "小赤带着疲惫不堪的宝可梦赶往了宝可梦中心……",
            other => other,
        };
        if let Some(n) = localized.strip_prefix("The caught Pokémon was transferred to Box ").and_then(|s| s.strip_suffix('.')) {
            return format!("捕捉到的宝可梦已传送至第 {n} 个盒子。{prompt}");
        }
        if let Some(n) = localized.strip_prefix("RED paid ¥").and_then(|s| s.strip_suffix(" as prize money… RED whited out!")) {
            return format!("小赤付出了 {n} 元作为奖金……眼前一片漆黑！{prompt}");
        }
        format!("{localized}{prompt}")
    }

    pub fn speaker<'a>(&'a self, name: &'a str) -> &'a str {
        match name {
            "" => "",
            "Mom" => "妈妈",
            "Daisy" => "娜娜美",
            "Clerk" => "店员",
            "Scientist" => "研究员",
            "Woman 1" | "Woman 2" | "Woman 3" => "女士",
            "Fat Man" | "Man" => "男子",
            "Boy" | "Little Boy" | "Gba Kid" => "男孩",
            "Little Girl" | "Lass" | "Picnicker" => "女孩",
            "Old Man 1" | "Old Man 2" | "Balding Man" => "老人",
            "Old Woman" => "老婆婆",
            "Youngster" => "短裤小子",
            "Camper" => "露营少年",
            "Brock" => "小刚",
            "Misty" => "小霞",
            "Lt Surge" => "马志士",
            "Erika" => "莉佳",
            "Koga" => "阿桔",
            "Nurse" => "护士",
            "Policeman" => "警察",
            "Fisher" => "垂钓者",
            "Sailor" => "水手",
            "Captain" => "船长",
            "Chef" => "厨师",
            "Worker F" | "Worker M" => "工作人员",
            "Rocket M" => "火箭队队员",
            "Pokémon Storage" => "宝可梦寄存系统",
            "Whiteout" => "眼前一片漆黑",
            "Poison" => "中毒",
            "Repel" => "除虫喷雾",
            "Original map scene" => "剧情",
            "Original field script" => "剧情事件",
            "Poke Maniac" => "怪兽狂",
            "Biker" => "飙车族",
            "Hiker" => "登山男",
            "Swimmer M" => "泳裤小伙",
            "Beauty" => "大姐姐",
            "Black Belt" => "空手道王",
            "Bug Catcher" => "捕虫少年",
            "Channeler" => "祈祷师",
            "Cooltrainer F" | "Cooltrainer M" => "精英训练家",
            "Crush Girl" => "格斗少女",
            "Gentleman" => "绅士",
            "Gym Guy" => "道馆向导",
            "Rocker" => "电工",
            "Clipboard" => "告示牌",
            "Pushable Boulder" => "巨石",
            "Water Var 0" => "水面",
            other => self.species(other),
        }
    }
}

pub(super) fn apply_font(
    p: Res<Preferences>,
    font: Res<ChineseFont>,
    mut query: Query<&mut TextFont, With<Text>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if p.is_changed() || font.is_changed() {
        let handle = if p.zh { font.0.clone().unwrap_or_default() } else { Handle::default() };
        for mut text in &mut query {
            if text.font != handle {
                text.font = handle.clone();
            }
        }
        let title = if p.zh { "宝可梦 火红 · 3D 试玩版" } else { "FireRed Native 3D Prototype" };
        if window.title != title {
            window.title = title.to_owned();
        }
    }
}

#[derive(Resource)]
pub(super) struct ChineseFont(pub Option<Handle<Font>>);


pub(super) fn field_error(error: &str) -> &str {
    match error.split(':').next().unwrap_or(error) {
        "Save reset failed" => "无法重置存档。详情见运行日志。",
        "Native save unavailable" => "无法读取本地存档，原存档已保留。",
        "Native save failed" => "无法写入存档。详情见运行日志。",
        "Original map scene unavailable" => "当前地图剧情暂不可用。",
        "Pokédex update unavailable" => "无法更新宝可梦图鉴。",
        "Capture unavailable" => "无法完成捕捉。请检查宝可梦寄存空间。",
        "Whiteout unavailable" | "Poison whiteout unavailable" => "无法返回恢复地点。",
        "Original field poison unavailable" => "无法结算地图上的中毒效果。",
        "Original wild encounter unavailable" => "当前野生宝可梦遭遇暂不可用。",
        "Actor update failed" => "当前人物动作暂不可用。",
        "Quick start failed" => "无法快速开始。",
        _ => "操作暂不可用。详情见运行日志。",
    }
}

impl Catalog {
    pub fn battle_line(&self, line: &str) -> String {
        let message = match line {
            "But it failed!" => "但是失败了！",
            "Fire's power was weakened!" => "火属性招式的威力减弱了！",
            "It's not very effective..." => "效果不好……",
            "It's super effective!" => "效果绝佳！",
            "OAK: A battle is won by reducing your opponent's HP to zero." => "大木：把对手的体力降到零，就能赢得对战。",
            "OAK: Inflicting damage is key to any battle." => "大木：给对手造成伤害，是赢得对战的关键。",
            "OAK: Lowering the foe's stats can put you at an advantage." => "大木：降低对手的能力，可以让你占据优势。",
            "Cannot run from a trainer battle!" | "OAK: You cannot run from a TRAINER battle!" => "无法从训练家对战中逃跑！",
            "Got away safely!" => "顺利逃跑了！",
            "Can't escape!" => "无法逃跑！",
            "The trainer blocked the BALL!" => "对方训练家挡住了精灵球！",
            "Oh no! The Pokémon broke free!" => "哎呀！宝可梦挣脱了！",
            "Aww! It appeared to be caught!" => "哎呀！还以为抓住了呢！",
            "Aargh! Almost had it!" => "啊！差一点就抓住了！",
            "Shoot! It was so close too!" => "真可惜！就差一点点！",
            "The stat cannot fall any lower!" => "能力已经无法再降低了！",
            "Your team became cloaked in a mystical veil!" => "我方被神秘的面纱包围了！",
            "The foe's team became cloaked in a mystical veil!" => "对方被神秘的面纱包围了！",
            "Your team's Safeguard faded!" => "我方的神秘守护消失了！",
            "The foe's Safeguard faded!" => "对方的神秘守护消失了！",
            "The attack missed!" => "攻击没有命中！",
            "Safeguard protected the target!" => "神秘守护保护了目标！",
            "That move has no PP" => "这个招式没有剩余 PP。",
            "A world battle cannot be restarted." => "这场剧情对战无法重来。",
            "Finish this world battle before returning." => "请先完成这场对战。",
            "The battle has already ended!" | "Battle has ended; reset to play again" => "对战已经结束。",
            "The Pokémon Boxes are full!" => "宝可梦盒子已满。",
            "No POKE BALLS are available." => "没有可用的精灵球。",
            "Unsupported move effect" => "此招式的效果尚未移植。",
            "Safari Ball needs Safari catch-factor setup" => "当前场景无法使用狩猎球。",
            _ => "",
        };
        if !message.is_empty() {
            return message.into();
        }
        if let Some((who, action)) = line.split_once(" used ") {
            let id = format!("MOVE_{}", action.trim_end_matches('!').replace(' ', "_"));
            return format!("{}使出了{}！", self.species(who), self.move_name(&id));
        }
        if let Some((trainer, sent)) = line.split_once(" sent out ") {
            if let Some((who, partner)) = sent.trim_end_matches('!').split_once("! Go, ") {
                let trainer = match trainer { "RIVAL" => "劲敌", "SAMMY" => "捕虫少年萨米", name => name };
                return format!("{trainer}派出了{}！上吧，{}！", self.species(who), self.species(partner));
            }
        }
        if let Some((who, partner)) = line.strip_prefix("Wild ")
            .and_then(|s| s.trim_end_matches('!').split_once(" appeared! Go, ")) {
            return format!("野生的{}出现了！上吧，{}！", self.species(who), self.species(partner));
        }
        if let Some(who) = line.strip_prefix("Gotcha! ").and_then(|s| s.strip_suffix(" was caught!")) {
            return format!("抓到了！{}被捕捉了！", self.species(who));
        }
        if let Some(who) = line.strip_prefix("It doesn't affect ").and_then(|s| s.strip_suffix("...")) {
            return format!("这对{}没有效果……", self.species(who));
        }
        if let Some((who, result)) = line.split_once(" fainted. ") {
            let result = match result { "You won!" => "你赢了！", "RIVAL won!" => "劲敌获胜！", "The Trainer won!" => "对方训练家获胜！", _ => "野生宝可梦获胜！" };
            return format!("{}倒下了。{result}", self.species(who));
        }
        if let Some((who, xp)) = line.strip_suffix(" experience points!").and_then(|s| s.split_once(" gained ")) {
            return format!("{}获得了 {xp} 点经验值！", self.species(who));
        }
        if let Some((who, level)) = line.trim_end_matches('!').split_once(" grew to level ") {
            return format!("{}升到了 {level} 级！", self.species(who));
        }
        let suffixes = [
            ("'s Defense won't go higher!", "的防御已经无法再提高了！"),
            ("'s Defense rose!", "的防御提高了！"),
            ("'s Speed fell!", "的速度降低了！"),
            ("'s Attack fell!", "的攻击降低了！"),
            ("'s Defense fell!", "的防御降低了！"),
            ("'s Accuracy fell!", "的命中率降低了！"),
            (" fled from battle!", "逃离了对战！"),
            (" was poisoned!", "中毒了！"), (" was burned!", "烧伤了！"),
            (" was frozen solid!", "被冻住了！"), (" is frozen solid!", "被冻住了！"),
            (" is paralyzed! It may be unable to move!", "麻痹了！可能无法行动！"),
            (" is paralyzed! It can't move!", "因麻痹无法行动！"),
            (" became confused!", "混乱了！"), (" is confused!", "正处于混乱中！"),
            (" snapped out of confusion!", "解除了混乱！"),
            (" woke up in the uproar!", "被吵醒了！"), (" is fast asleep.", "睡得很沉。"),
            (" woke up!", "醒过来了！"), (" thawed out!", "解冻了！"),
            ("'s Soundproof blocks Sing!", "的隔音特性挡住了唱歌！"),
            (" is already asleep!", "已经睡着了！"), (" stayed awake!", "保持了清醒！"),
            (" fell asleep!", "睡着了！"),
        ];
        for (en, zh) in suffixes {
            if let Some(who) = line.strip_suffix(en) {
                return format!("{}{zh}", self.species(who));
            }
        }
        if let Some((who, hp)) = line.strip_suffix(" HP restored").and_then(|s| s.split_once(" regained health! ")) {
            return format!("{}恢复了 {hp} 点体力！", self.species(who));
        }
        if let Some((who, hp)) = line.strip_suffix(" HP restored").and_then(|s| s.split_once(" had its energy drained! ")) {
            return format!("{}的能量被吸走了！使用者恢复了 {hp} 点体力。", self.species(who));
        }
        if let Some(report) = line.strip_suffix(" damage") {
            if let Some((who, rest)) = report.split_once(" is hurt by ") {
                if let Some((cause, hp)) = rest.split_once("! ") {
                    let cause = if cause == "poison" { "中毒" } else { "烧伤" };
                    return format!("{}因{cause}损失了 {hp} 点体力！", self.species(who));
                }
            }
            if let Some(hp) = report.strip_prefix("It hurt itself in its confusion! ") {
                return format!("混乱中伤到了自己！损失 {hp} 点体力。");
            }
            if let Some((who, hp)) = report.split_once(" sucked up the liquid ooze! ") {
                return format!("{}吸到了污泥！损失 {hp} 点体力。", self.species(who));
            }
            return format!("造成了 {report} 点伤害。");
        }
        if let Some(hp) = line.strip_suffix(" damage - critical hit!") {
            return format!("击中要害！造成了 {hp} 点伤害。");
        }
        if line.starts_with("Ball item ") {
            return "当前版本暂不支持此精灵球。".into();
        }
        line.to_owned()
    }
}
