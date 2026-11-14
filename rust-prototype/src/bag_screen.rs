//! Inventory and party targeting backed by the native save, in the field and in battles.
use super::{battle_screen, locale, persistence, Preferences, Simulation};
use bevy::prelude::*;
use firered_native_prototype::core::{Medicine, MedicineChange, MedicineError, Slot};

const ROWS: usize = 7;
const POCKETS: [usize; 5] = [0, 2, 1, 3, 4];
const POCKET_NAMES: [(&str, &str); 5] = [
    ("道具", "Items"), ("精灵球", "Poké Balls"), ("重要物品", "Key Items"),
    ("招式学习器", "TMs"), ("树果", "Berries"),
];

#[derive(Resource, Default)]
pub(super) struct BagScreen {
    open: bool,
    handled: bool,
    pocket: usize,
    cursor: usize,
    target: Option<u16>,
    notice: String,
}

impl BagScreen {
    pub fn captured_input(&self) -> bool {
        self.open || self.handled
    }

    fn slots<'a>(&self, session: &'a persistence::Session) -> Vec<&'a Slot> {
        session.state.bag.pockets[POCKETS[self.pocket]]
            .iter().filter(|slot| slot.item != 0 && slot.quantity > 0).collect()
    }

    fn row_offset(&self) -> usize {
        if self.target.is_some() { 0 } else { self.cursor.saturating_sub(ROWS - 1) }
    }

    fn press(
        &mut self,
        action: Pick,
        session: &mut persistence::Session,
        screen: &mut battle_screen::Screen,
        locale: &locale::Catalog,
        prefs: &Preferences,
    ) {
        let zh = prefs.zh;
        let say = |cn: &str, en: &str| if zh { cn.to_owned() } else { en.to_owned() };
        match action {
            Pick::Close => self.open = false,
            Pick::Pocket(i) => {
                self.pocket = i;
                self.target = None;
                self.cursor = 0;
                self.notice.clear();
            }
            Pick::Confirm | Pick::Row(_) => {
                if let Pick::Row(row) = action {
                    self.cursor = self.row_offset() + row;
                }
                if let Some(item) = self.target {
                    if self.cursor >= session.state.party.len() {
                        return;
                    }
                    let used = if screen.active {
                        screen.use_medicine(session, item, self.cursor)
                    } else {
                        session.state.use_medicine(&session.data, item, self.cursor)
                    };
                    match used {
                        Ok(change) => {
                            let name = locale.party_species(&session.data, session.state.party[self.cursor].species, zh);
                            let result = medicine_notice(&name, change, zh);
                            self.target = None;
                            self.cursor = 0;
                            self.notice = result.clone();
                            if screen.active {
                                screen.battle.log.insert(0, result);
                                self.open = false;
                            }
                        }
                        Err(error) => {
                            self.notice = match error {
                                MedicineError::OutOfStock => say("这个道具已用完。", "This item is out of stock."),
                                MedicineError::NoEffect => say("用在这只宝可梦身上没有效果。", "It would have no effect on this Pokémon."),
                                _ => say("现在无法使用这个道具。", "This item cannot be used here."),
                            };
                        }
                    }
                } else if let Some(item) = self.slots(session).get(self.cursor).map(|slot| slot.item) {
                    if Medicine::from_item(&session.data, item).is_some() {
                        if session.state.party.is_empty() {
                            self.notice = say("队伍里还没有宝可梦。", "Your party is empty.");
                        } else {
                            self.target = Some(item);
                            self.cursor = screen.party_slot.filter(|_| screen.active).unwrap_or(0);
                            self.notice.clear();
                        }
                    } else if self.pocket == 1 && screen.active {
                        match screen.throw_ball(session, item, prefs.effects) {
                            Ok(()) => self.open = false,
                            Err(error) => self.notice = if zh { locale.battle_line(&error) } else { error },
                        }
                    } else {
                        self.notice = if self.pocket == 1 {
                            say("在野生宝可梦对战中投球。", "Use Poké Balls during a wild battle.")
                        } else {
                            say("当前试玩版暂不能使用这个道具。", "This item is not usable in this build.")
                        };
                    }
                }
            }
        }
    }
}

fn medicine_notice(name: &str, change: MedicineChange, zh: bool) -> String {
    let hp = change.restored_hp;
    match (zh, hp > 0, change.cured_status) {
        (true, true, true) => format!("{name}恢复了 {hp} 点体力，异常状态解除了！"),
        (true, true, false) => format!("{name}恢复了 {hp} 点体力！"),
        (true, false, _) => format!("{name}的异常状态解除了！"),
        (false, true, true) => format!("{name} recovered {hp} HP and was cured!"),
        (false, true, false) => format!("{name} recovered {hp} HP!"),
        (false, false, _) => format!("{name} was cured!"),
    }
}

#[derive(Clone, Copy, Component)]
pub(super) enum Pick {
    Close,
    Confirm,
    Pocket(usize),
    Row(usize),
}
#[derive(Component)]
pub(super) struct BagPanel;
#[derive(Component)]
pub(super) enum Label {
    Title,
    Pocket(usize),
    Row(usize),
    Prompt,
    Close,
}

fn button(commands: &mut ChildSpawnerCommands, action: Pick, label: Label, font_size: f32, node: Node) {
    commands.spawn((Button, action, node, BorderRadius::all(px(8)), BackgroundColor(Color::srgb_u8(37, 61, 74))))
        .with_children(|button| {
            button.spawn((label, Text::new(""), TextFont { font_size, ..default() }, TextColor(Color::WHITE)));
        });
}

pub(super) fn setup(mut commands: Commands) {
    commands.spawn((
        BagPanel,
        GlobalZIndex(240),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.),
            height: percent(100.),
            display: Display::None,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.08, 0.11, 0.67)),
        bevy::ui::FocusPolicy::Block,
    )).with_children(|overlay| {
        overlay.spawn((
            Node {
                width: percent(86.),
                max_width: px(640),
                padding: UiRect::all(px(22)),
                row_gap: px(9),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderRadius::all(px(16)),
            BackgroundColor(Color::srgb_u8(21, 38, 49)),
        )).with_children(|panel| {
            panel.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            }).with_children(|title| {
                title.spawn((Label::Title, Text::new(""), TextFont { font_size: 28., ..default() }, TextColor(Color::WHITE)));
                button(title, Pick::Close, Label::Close, 16., Node { padding: UiRect::axes(px(12), px(8)), ..default() });
            });
            panel.spawn(Node { column_gap: px(6), flex_wrap: FlexWrap::Wrap, row_gap: px(5), ..default() }).with_children(|tabs| {
                for i in 0..POCKETS.len() {
                    button(tabs, Pick::Pocket(i), Label::Pocket(i), 16., Node { padding: UiRect::axes(px(10), px(6)), ..default() });
                }
            });
            for i in 0..ROWS {
                button(panel, Pick::Row(i), Label::Row(i), 19., Node {
                    padding: UiRect::axes(px(13), px(6)),
                    min_height: px(38),
                    width: percent(100.),
                    align_items: AlignItems::Center,
                    ..default()
                });
            }
            panel.spawn((
                Label::Prompt, Text::new(""), TextFont { font_size: 17., ..default() },
                TextColor(Color::srgb_u8(191, 222, 220)),
            ));
        });
    });
}

pub(super) fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut bag: ResMut<BagScreen>,
    mut session: ResMut<persistence::Session>,
    mut screen: ResMut<battle_screen::Screen>,
    prefs: Res<Preferences>,
    simulation: Res<Simulation>,
    locale: Res<locale::Catalog>,
    bag_button: Query<(&Interaction, &battle_screen::Action), Changed<Interaction>>,
    actions: Query<(&Pick, &Interaction), Changed<Interaction>>,
) {
    if bag.handled { bag.handled = false; }
    if !prefs.bag || prefs.menu || screen.throw_animation.is_some() {
        bag.open = false;
        return;
    }
    let may_open = if screen.active {
        screen.integrated && screen.battle.winner.is_none()
    } else {
        !simulation.game.busy() && simulation.game.dialogue.is_none()
    };
    if !may_open {
        bag.open = false;
        return;
    }
    let toggle = keys.just_pressed(KeyCode::KeyI) || bag_button.iter()
        .any(|(i, a)| *i == Interaction::Pressed && *a == battle_screen::Action::Bag);
    if toggle {
        if bag.open {
            bag.open = false;
            bag.handled = true;
        } else {
            *bag = BagScreen { open: true, ..default() };
        }
        return;
    }
    if !bag.open { return; }
    if keys.just_pressed(KeyCode::Escape) {
        if bag.target.take().is_some() {
            bag.cursor = 0;
            bag.notice.clear();
        } else {
            bag.open = false;
        }
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::ArrowRight) {
        let step = if keys.just_pressed(KeyCode::ArrowLeft) { POCKETS.len() - 1 } else { 1 };
        let pocket = (bag.pocket + step) % POCKETS.len();
        bag.press(Pick::Pocket(pocket), &mut session, &mut screen, &locale, &prefs);
    } else if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowDown) {
        let len = if bag.target.is_some() { session.state.party.len() } else { bag.slots(&session).len() };
        bag.cursor = if keys.just_pressed(KeyCode::ArrowUp) { bag.cursor.saturating_sub(1) }
            else { (bag.cursor + 1).min(len.saturating_sub(1)) };
        bag.notice.clear();
    } else if let Some(action) = if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        Some(Pick::Confirm)
    } else {
        actions.iter().find_map(|(a, i)| (*i == Interaction::Pressed).then_some(*a))
    } {
        bag.press(action, &mut session, &mut screen, &locale, &prefs);
    }
    if !bag.open {
        bag.handled = true;
    }
}

pub(super) fn draw(
    bag: Res<BagScreen>, session: Res<persistence::Session>, screen: Res<battle_screen::Screen>,
    prefs: Res<Preferences>, locale: Res<locale::Catalog>,
    mut panel: Single<&mut Node, With<BagPanel>>,
    mut labels: Query<(&Label, &mut Text)>,
    mut buttons: Query<(&Pick, &mut Node, &mut BackgroundColor, &Interaction), Without<BagPanel>>,
) {
    let display = if bag.open { Display::Flex } else { Display::None };
    if panel.display != display {
        panel.display = display;
    }
    if !bag.open {
        return;
    }
    let zh = prefs.zh;
    let slots = bag.slots(&session);
    let len = if bag.target.is_some() { session.state.party.len() } else { slots.len() };
    let offset = bag.row_offset();
    for (label, mut text) in &mut labels {
        let line = match *label {
            Label::Title => bag.target.map_or_else(
                || if zh { "背包".into() } else { "Bag".into() },
                |item| if zh { format!("使用 {}", locale.bag_item(&session.data, item, zh)) }
                    else { format!("Use {}", locale.bag_item(&session.data, item, zh)) },
            ),
            Label::Pocket(i) => if zh { POCKET_NAMES[i].0.into() } else { POCKET_NAMES[i].1.into() },
            Label::Close => if zh { "关闭".into() } else { "Close".into() },
            Label::Prompt if !bag.notice.is_empty() => bag.notice.clone(),
            Label::Prompt if bag.target.is_some() => if zh { "↑↓ 选择宝可梦　回车 使用　Esc 返回".into() }
                else { "↑↓ Select a Pokémon    Enter Use    Esc Back".into() },
            Label::Prompt if len == 0 => if zh { "这个口袋是空的。".into() } else { "This pocket is empty.".into() },
            Label::Prompt => {
                let paging = if len > ROWS { format!("  {}/{}", bag.cursor + 1, len) } else { String::new() };
                if zh { format!("←→ 切换口袋　↑↓ 选择　回车 确认　Esc 返回{paging}") }
                else { format!("←→ Pockets    ↑↓ Select    Enter Confirm    Esc Back{paging}") }
            }
            Label::Row(i) if bag.target.is_some() => {
                let Some(mon) = session.state.party.get(i) else { continue };
                let name = locale.party_species(&session.data, mon.species, zh);
                let (hp, max_hp, status) = if screen.active && screen.party_slot == Some(i) {
                    let active = &screen.battle.mons[0];
                    (active.hp, active.stats[0], active.status)
                } else {
                    (mon.hp, mon.stats[0], mon.status)
                };
                let condition = match (hp, status) {
                    (0, _) => if zh { "濒死" } else { "Fainted" },
                    (_, s) if s & 0x88 != 0 => if zh { "中毒" } else { "Poisoned" },
                    (_, s) if s & 7 != 0 => if zh { "睡眠" } else { "Asleep" },
                    (_, s) if s & 0x10 != 0 => if zh { "灼伤" } else { "Burned" },
                    (_, s) if s & 0x20 != 0 => if zh { "冰冻" } else { "Frozen" },
                    (_, s) if s & 0x40 != 0 => if zh { "麻痹" } else { "Paralyzed" },
                    _ => "",
                };
                format!("{name}    Lv. {}     HP {hp}/{max_hp}    {condition}", mon.level)
            }
            Label::Row(i) => {
                let Some(slot) = slots.get(i + offset) else { continue };
                format!("{}   ×{}", locale.bag_item(&session.data, slot.item, zh), slot.quantity)
            }
        };
        if text.0 != line { text.0 = line; }
    }
    for (pick, mut node, mut color, interaction) in &mut buttons {
        let (visible, selected) = match *pick {
            Pick::Pocket(i) => (bag.target.is_none(), bag.pocket == i),
            Pick::Row(i) => (i + offset < len, i + offset == bag.cursor),
            _ => (true, false),
        };
        node.display = if visible { Display::Flex } else { Display::None };
        color.0 = if selected { Color::srgb_u8(47, 104, 105) }
            else if *interaction == Interaction::Hovered { Color::srgb_u8(50, 77, 90) }
            else { Color::srgb_u8(37, 61, 74) };
    }
}

pub(super) fn check(world: &firered_native_prototype::World) -> Result<(), String> {
    use firered_native_prototype::{battle::{Battle, Mon}, core::{OtId, Pokemon}};
    let game = &mut firered_native_prototype::Game::new(world, "route1");
    let mut session = persistence::Session::open(Default::default(), false, game, world);
    let species = session.data.ids["SPECIES_CHARMANDER"];
    let trainer_id = session.state.trainer_id;
    let starter = Pokemon::create(&session.data, &mut session.state.rng, species, 12, 31,
        Some(0), OtId::Player, trainer_id)?;
    session.state.party = vec![starter.clone(), starter];
    let potion = session.data.ids["ITEM_POTION"];
    session.state.bag.add(&session.data, &mut session.state.events, potion, 2);
    let mut screen = battle_screen::Screen::new(true);
    let player = Mon::from_pokemon(&screen.data, &session.data, &session.state.party[1])?;
    let opponent = screen.battle.mons[1].clone();
    screen.battle = Battle::wild(&screen.data, player, opponent, 255, 42);
    screen.party_slot = Some(1);
    screen.integrated = true;
    assert_eq!(screen.use_medicine(&mut session, potion, 1), Err(MedicineError::NoEffect));
    assert_eq!(screen.battle.turn, 0);
    screen.battle.mons[0].hp = 1;
    let prefs = Preferences { zh: false, ..Preferences::load(Default::default()) };
    let catalog = locale::Catalog::default();
    let mut bag = BagScreen { open: true, ..default() };
    bag.press(Pick::Confirm, &mut session, &mut screen, &catalog, &prefs);
    assert_eq!(bag.target, Some(potion));
    assert_eq!(bag.cursor, 1);
    bag.press(Pick::Confirm, &mut session, &mut screen, &catalog, &prefs);
    assert!(!bag.open);
    assert!(screen.battle.log[0].contains("recovered 20 HP"));
    assert_eq!(screen.battle.turn, 1);
    assert!(session.state.bag.has(&session.data, potion, 1));
    assert!(!session.state.bag.has(&session.data, potion, 2));
    assert_eq!(session.state.party[0].hp, session.state.party[0].stats[0]);
    Ok(())
}
