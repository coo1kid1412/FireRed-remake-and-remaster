//! Present the resolved FireRed throw without consuming any additional RNG.
use super::{battle_screen, battle_stage, Preferences, SceneCamera};
use bevy::{math::Rot2, prelude::*};
use firered_native_prototype::battle::BallThrow;

const FLIGHT: f32 = 0.66;
const ABSORB: f32 = 0.30;
const LAND: f32 = 0.43;
const SETTLE: f32 = 0.15;
const SHAKE: f32 = 0.43;
const REVEAL: f32 = 0.55;
const SIZE: f32 = 38.;

pub(super) struct ThrowAnimation {
    outcome: BallThrow,
    result: String,
    time: f32,
}

impl ThrowAnimation {
    pub fn new(outcome: BallThrow, result: String) -> Self {
        Self { outcome, result, time: 0. }
    }

    fn reveal_at(&self) -> f32 {
        FLIGHT + ABSORB + LAND + SETTLE + SHAKE * f32::from(self.outcome.shakes)
    }

    pub fn opponent_scale(&self) -> f32 {
        let t = self.time;
        if t < FLIGHT + ABSORB {
            1. - ((t - FLIGHT) / ABSORB).clamp(0., 1.)
        } else if self.outcome.caught {
            0.
        } else {
            ((t - self.reveal_at()) / (REVEAL * 0.6)).clamp(0., 1.)
        }
    }
}

pub(super) fn tick(
    mut screen: ResMut<battle_screen::Screen>,
    time: Res<Time>,
    prefs: Res<Preferences>,
) {
    if prefs.menu {
        return;
    }
    if let Err(error) = advance(&mut screen, time.delta_secs()) {
        screen.error = error;
    }
}

fn advance(screen: &mut battle_screen::Screen, delta: f32) -> Result<(), String> {
    let Some(shot) = screen.throw_animation.as_mut() else { return Ok(()) };
    shot.time += delta;
    if shot.time < shot.reveal_at() + REVEAL {
        return Ok(());
    }
    let shot = screen.throw_animation.take().unwrap();
    screen.battle.log.clear();
    screen.battle.log.push(shot.result);
    if !shot.outcome.caught {
        screen.battle.enemy_turn(&screen.data)?;
    }
    Ok(())
}

#[derive(Component)]
pub(super) struct CaptureBall;
#[derive(Component)]
pub(super) struct BallCap;
#[derive(Component)]
pub(super) struct Flash;

pub(super) fn setup(mut commands: Commands) {
    commands.spawn((
        Flash,
        GlobalZIndex(223),
        Node { position_type: PositionType::Absolute, display: Display::None, ..default() },
        BorderRadius::MAX,
        BackgroundColor(Color::NONE),
        bevy::ui::FocusPolicy::Pass,
    ));
    commands.spawn((
        CaptureBall,
        GlobalZIndex(224),
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            width: px(SIZE),
            height: px(SIZE),
            border: UiRect::all(px(2)),
            overflow: Overflow::clip(),
            ..default()
        },
        BorderRadius::MAX,
        BorderColor::all(Color::srgb_u8(35, 43, 54)),
        BackgroundColor(Color::srgb_u8(246, 245, 235)),
        UiTransform::default(),
        bevy::ui::FocusPolicy::Pass,
    )).with_children(|ball| {
        ball.spawn((
            BallCap,
            Node { width: percent(100), height: percent(52), ..default() },
            BackgroundColor(Color::srgb_u8(224, 57, 67)),
        ));
        ball.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: percent(44),
                width: percent(100),
                height: px(4),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(35, 43, 54)),
        ));
        ball.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(11),
                left: px(11),
                width: px(12),
                height: px(12),
                border: UiRect::all(px(2)),
                ..default()
            },
            BorderRadius::MAX,
            BorderColor::all(Color::srgb_u8(35, 43, 54)),
            BackgroundColor(Color::WHITE),
        ));
    });
}

pub(super) fn draw(
    screen: Res<battle_screen::Screen>,
    prefs: Res<Preferences>,
    stages: Res<battle_stage::Stages>,
    camera: Single<(&Camera, &GlobalTransform), With<SceneCamera>>,
    portraits: Query<(&battle_screen::MonImage, &ComputedNode, &UiGlobalTransform)>,
    mut ball: Single<(&mut Node, &mut UiTransform), (With<CaptureBall>, Without<Flash>)>,
    mut flash: Single<(&mut Node, &mut BackgroundColor), (With<Flash>, Without<CaptureBall>, Without<BallCap>)>,
    mut cap: Single<&mut BackgroundColor, (With<BallCap>, Without<Flash>)>,
) {
    let (camera, camera_transform) = *camera;
    let Some(size) = camera.logical_viewport_size() else { return };
    let Some(shot) = screen.throw_animation.as_ref().filter(|_| screen.active && !prefs.menu) else {
        ball.0.display = Display::None;
        flash.0.display = Display::None;
        return;
    };
    let (start, target, ground) = if battle_stage::enabled(&prefs, &stages) {
        let project = |point| camera.world_to_viewport(camera_transform, point).ok();
        (
            project(battle_stage::spot(2) + Vec3::new(0.38, 0.75, -0.50))
                .unwrap_or(size * Vec2::new(0.27, 0.62)),
            project(battle_stage::spot(1) + Vec3::Y * 0.34).unwrap_or(size * Vec2::new(0.55, 0.48)),
            project(battle_stage::spot(1) + Vec3::Y * 0.08).unwrap_or(size * Vec2::new(0.55, 0.54)),
        )
    } else {
        let portrait = |i| portraits.iter().find(|(label, node, _)| label.0 == i && node.size().x > 0.)
            .map(|(_, layout, transform)| transform.translation * layout.inverse_scale_factor());
        let start = portrait(0).unwrap_or(size * Vec2::new(0.2, 0.4));
        let target = portrait(1).unwrap_or(size * Vec2::new(0.63, 0.4));
        (start, target, target + Vec2::Y * 52.)
    };

    let time = shot.time;
    let reveal = shot.reveal_at();
    let mut pulse = 0.;
    let mut rotation = 0.;
    let mut scale = 1.;
    let mut visible = true;
    let position = if time < FLIGHT {
        let t = time / FLIGHT;
        rotation = std::f32::consts::TAU * 1.5 * t;
        scale = 1.2 - t * 0.34;
        start.lerp(target, t) - Vec2::Y * (size.y * 0.17 * 4. * t * (1. - t))
    } else if time < FLIGHT + ABSORB {
        let t = (time - FLIGHT) / ABSORB;
        pulse = (1. - t) * 0.76;
        target
    } else if time < FLIGHT + ABSORB + LAND {
        let t = (time - FLIGHT - ABSORB) / LAND;
        let bounce = if t < 0.65 { 0. } else { ((t - 0.65) / 0.35 * std::f32::consts::PI).sin() };
        target.lerp(ground, (t / 0.65).min(1.).powi(2)) - Vec2::Y * bounce * 18.
    } else if time < reveal {
        let t = (time - FLIGHT - ABSORB - LAND - SETTLE).max(0.) % SHAKE / SHAKE;
        rotation = (std::f32::consts::TAU * t).sin() * (1. - t) * 0.78;
        ground + Vec2::X * (rotation * 9.)
    } else {
        let t = (time - reveal) / REVEAL;
        visible = shot.outcome.caught || t < 0.2;
        pulse = (1. - t).max(0.) * 0.85;
        scale = if shot.outcome.caught { 1. } else { 1. + t };
        ground
    };

    let offset = position - Vec2::splat(SIZE / 2.);
    ball.0.display = if visible { Display::Flex } else { Display::None };
    ball.0.left = px(offset.x);
    ball.0.top = px(offset.y);
    ball.1.rotation = Rot2::radians(rotation);
    ball.1.scale = Vec2::splat(scale);
    cap.0 = match shot.outcome.ball {
        1 => Color::srgb_u8(151, 69, 173),
        2 => Color::srgb_u8(58, 64, 77),
        3 => Color::srgb_u8(62, 114, 232),
        _ => Color::srgb_u8(224, 57, 67),
    };
    let burst = 48. + 76. * (1. - pulse);
    flash.0.display = if pulse > 0.01 { Display::Flex } else { Display::None };
    flash.0.left = px(position.x - burst / 2.);
    flash.0.top = px(position.y - burst / 2.);
    flash.0.width = px(burst);
    flash.0.height = px(burst);
    flash.1.0 = if time >= reveal && shot.outcome.caught {
        Color::srgba(1., 0.82, 0.33, pulse)
    } else {
        Color::srgba(1., 0.94, 0.81, pulse)
    };
}

/// Test the pacing around the existing battle transaction without rendering.
pub(super) fn check(world: &firered_native_prototype::World) -> Result<(), String> {
    let mut game = firered_native_prototype::Game::new(world, "route1");
    let mut session = super::persistence::Session::open(Default::default(), false, &mut game, world);
    let mut screen = battle_screen::Screen::new(true);
    screen.battle = firered_native_prototype::battle::Battle::wild(
        &screen.data, screen.battle.mons[0].clone(), screen.battle.mons[1].clone(), 0, 0,
    );
    let item = session.data.ids["ITEM_POKE_BALL"];
    session.state.bag.add(&session.data, &mut session.state.events, item, 2);
    screen.throw_ball(&mut session, item, true)?;
    assert_eq!(screen.battle.turn, 0);
    assert_eq!(screen.opponent_scale(), 1.);
    assert!(screen.throw_ball(&mut session, item, true).is_err());
    assert!(session.state.bag.has(&session.data, item, 1));
    advance(&mut screen, FLIGHT + ABSORB + 0.04)?;
    assert_eq!(screen.battle.turn, 0);
    assert_eq!(screen.opponent_scale(), 0.);
    advance(&mut screen, 4.)?;
    assert!(screen.throw_animation.is_none());
    assert_eq!(screen.opponent_scale(), 1.);
    assert_eq!(screen.battle.turn, 1);
    assert!(!session.state.bag.has(&session.data, item, 2));

    let master = session.data.ids["ITEM_MASTER_BALL"];
    assert!(session.state.bag.add(&session.data, &mut session.state.events, master, 1));
    let turn = screen.battle.turn;
    screen.throw_ball(&mut session, master, true)?;
    advance(&mut screen, 1.0)?;
    assert!(!screen.battle.log.iter().any(|line| line.contains("was caught")));
    advance(&mut screen, 4.0)?;
    assert_eq!(screen.battle.turn, turn);
    assert!(screen.battle.caught);
    assert_eq!(screen.opponent_scale(), 0.);
    assert!(screen.battle.log[0].contains("was caught"));
    Ok(())
}
