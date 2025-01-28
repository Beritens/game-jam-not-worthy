use crate::asset_load::{GameData, GameInfos, ShopItem, UISounds};
use crate::game_manager::Scorer;
use crate::game_state::GameState;
use crate::game_state::GameState::Shop;
use crate::level_loading::SceneObject;
use crate::state_handling::{get_sotred_value, store_value};
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::audio::{AudioPlayer, PlaybackMode, PlaybackSettings};
use bevy::color::palettes::css::CRIMSON;
use bevy::color::Color;
use bevy::ecs::system::lifetimeless::SCommands;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{
    default, in_state, AlignItems, BackgroundColor, BuildChildren, Button, Changed, ChildBuild,
    ChildBuilder, Commands, Component, Entity, Interaction, IntoSystemConfigs, JustifyContent,
    NextState, Node, OnEnter, Query, Res, ResMut, Text, TextColor, TextFont, UiRect, Val, With,
};
use bevy::ui::FlexDirection;
use bevy_pkv::PkvStore;

#[derive(Component)]
struct SelectedOption;
const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);
const HOVERED_PRESSED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct UIStuffPlugin;

impl Plugin for UIStuffPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), (setup_shop));
        app.add_systems(OnEnter(GameState::InGame), (setup_game_ui));
        app.add_systems(OnEnter(GameState::Menu), (setup_main_menu));
        app.add_systems(OnEnter(GameState::Loading), (setup_loading_ui));
        app.add_systems(OnEnter(GameState::CompilingShaders), (setup_compiling_ui));
        app.add_systems(
            Update,
            (button_system, shop_action, setup_shop).run_if(in_state(GameState::Shop)),
        );
        app.add_systems(
            Update,
            (update_socre_display_system).run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            Update,
            (button_system, menu_action).run_if(in_state(GameState::Menu)),
        );
    }
}

//shop

#[derive(Component)]
enum ShopButtonAction {
    PLAY,
    KNOCKBACK,
    DAMAGE,
    SPEED,
    ARISE_COOLDOWN,
    ARISE_COUNT,
    HIT_COOLDOWN,
}
fn buy(
    shop_item: i32,
    key: &str,
    game_data_res: &Res<GameData>,
    game_datas: &ResMut<Assets<GameInfos>>,
    pkv: &mut ResMut<PkvStore>,
) {
    let curr_level = get_sotred_value(pkv, key);
    let Some(game_data) = game_datas.get(game_data_res.data.id()) else {
        return;
    };
    let cost = game_data.shop_items[shop_item as usize].shop_displays[curr_level as usize].cost;

    let curr_score = get_sotred_value(pkv, "score");
    store_value(pkv, "score", curr_score - cost);

    //remove money
    store_value(pkv, key, curr_level + 1);
}

#[derive(Component)]
struct Outdated;
fn shop_action(
    interaction_query: Query<
        (&Interaction, &ShopButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut pkv: ResMut<PkvStore>,
    game_data_res: Res<GameData>,
    game_datas: ResMut<Assets<GameInfos>>,
    shop_screen_query: Query<Entity, With<ShopScreen>>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    ui_sounds: Res<UISounds>,
) {
    for (interaction, shop_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(shop_screen) = shop_screen_query.get_single() {
                commands.entity(shop_screen).insert(Outdated);
            }
            commands.spawn((
                AudioPlayer::new(ui_sounds.button_2.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..Default::default()
                },
            ));
            match shop_button_action {
                ShopButtonAction::PLAY => {
                    game_state.set(GameState::InGame);
                }
                ShopButtonAction::KNOCKBACK => {
                    buy(0, "knockback", &game_data_res, &game_datas, &mut pkv)
                }

                ShopButtonAction::DAMAGE => buy(1, "damage", &game_data_res, &game_datas, &mut pkv),
                ShopButtonAction::SPEED => buy(2, "speed", &game_data_res, &game_datas, &mut pkv),
                ShopButtonAction::ARISE_COOLDOWN => {
                    buy(3, "arise_cooldown", &game_data_res, &game_datas, &mut pkv)
                }
                ShopButtonAction::ARISE_COUNT => {
                    buy(4, "arise_count", &game_data_res, &game_datas, &mut pkv)
                }
                ShopButtonAction::HIT_COOLDOWN => {
                    buy(5, "attack_cooldown", &game_data_res, &game_datas, &mut pkv)
                }
                _ => {}
            }
        }
    }
}
fn get_shop_item(
    parent: &mut ChildBuilder,
    label: String,
    text: String,
    cost: i32,
    action: ShopButtonAction,
    disabled: bool,
    too_expensive: bool,
) {
    let button_node: Node = Node {
        width: Val::Vh(27.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font: TextFont = TextFont {
        font_size: 18.0,
        ..default()
    };

    let label_text_font: TextFont = TextFont {
        font_size: 24.0,

        ..default()
    };

    let text_color = if disabled || too_expensive {
        DISABLED_TEXT_COLOR
    } else {
        TEXT_COLOR
    };

    parent
        .spawn((
            SceneObject,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                margin: UiRect::all(Val::Vh(1.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneObject,
                Text::new(label + ":"),
                label_text_font.clone(),
                TextColor(text_color),
            ));
            let mut button =
                parent.spawn((SceneObject, button_node, BackgroundColor(NORMAL_BUTTON)));
            button.with_children(|parent| {
                // let icon = asset_server.load("textures/Game Icons/right.png");
                // parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                parent
                    .spawn((
                        SceneObject,
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            SceneObject,
                            Text::new(text),
                            button_text_font.clone(),
                            TextColor(text_color),
                        ));
                        if (!disabled) {
                            parent.spawn((
                                SceneObject,
                                Text::new(cost.to_string()),
                                button_text_font.clone(),
                                TextColor(text_color),
                            ));
                        }
                    });
            });
            if (!disabled && !too_expensive) {
                button.insert((Button, action));
            }
        });
}
#[derive(Component)]
struct ShopScreen;
fn setup_shop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_data_res: Res<GameData>,
    mut game_datas: ResMut<Assets<GameInfos>>,
    mut pkv: ResMut<PkvStore>,
    shop_query: Query<Entity, With<ShopScreen>>,
    outdated_query: Query<Entity, With<Outdated>>,
) {
    let button_node = Node {
        width: Val::Vh(27.0),
        height: Val::Vh(6.0),
        margin: UiRect::all(Val::Vh(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    if let Ok(shop_screen) = shop_query.get_single() {
        if let Ok(outdated) = outdated_query.get(shop_screen) {
            commands.entity(shop_screen).despawn_recursive();
        } else {
            return;
        }
    }

    let knockback_level = get_sotred_value(&mut pkv, "knockback");
    let damage_level = get_sotred_value(&mut pkv, "damage");
    let speed_level = get_sotred_value(&mut pkv, "speed");
    let arise_cooldown_level = get_sotred_value(&mut pkv, "arise_cooldown");
    let arise_count_level = get_sotred_value(&mut pkv, "arise_count");
    let attack_cooldown_level = get_sotred_value(&mut pkv, "attack_cooldown");

    let curr_score = get_sotred_value(&mut pkv, "score");
    let Some(game_data) = game_datas.get(game_data_res.data.id()) else {
        return;
    };
    commands
        .spawn((
            SceneObject,
            Node {
                width: Val::Percent(100.0),
                max_height: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ShopScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SceneObject,
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Vh(4.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn((
                        SceneObject,
                        Text::new("Blacksmith"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Vh(4.0)),
                            ..default()
                        },
                    ));

                    let knockback_cost =
                        game_data.shop_items[0].shop_displays[knockback_level as usize].cost;
                    get_shop_item(
                        parent,
                        game_data.shop_items[0].name.clone(),
                        game_data.shop_items[0].shop_displays[knockback_level as usize]
                            .text
                            .clone(),
                        knockback_cost,
                        ShopButtonAction::KNOCKBACK,
                        knockback_cost < 0,
                        knockback_cost > curr_score,
                    );

                    let damage_cost =
                        game_data.shop_items[1].shop_displays[damage_level as usize].cost;
                    get_shop_item(
                        parent,
                        game_data.shop_items[1].name.clone(),
                        game_data.shop_items[1].shop_displays[damage_level as usize]
                            .text
                            .clone(),
                        damage_cost,
                        ShopButtonAction::DAMAGE,
                        damage_cost < 0,
                        damage_cost > curr_score,
                    );

                    let speed_cost =
                        game_data.shop_items[2].shop_displays[speed_level as usize].cost;
                    get_shop_item(
                        parent,
                        game_data.shop_items[2].name.clone(),
                        game_data.shop_items[2].shop_displays[speed_level as usize]
                            .text
                            .clone(),
                        speed_cost,
                        ShopButtonAction::SPEED,
                        speed_cost < 0,
                        speed_cost > curr_score,
                    );

                    let arise_cooldown_cost =
                        game_data.shop_items[3].shop_displays[arise_cooldown_level as usize].cost;
                    get_shop_item(
                        parent,
                        game_data.shop_items[3].name.clone(),
                        game_data.shop_items[3].shop_displays[arise_cooldown_level as usize]
                            .text
                            .clone(),
                        arise_cooldown_cost,
                        ShopButtonAction::ARISE_COOLDOWN,
                        arise_cooldown_cost < 0,
                        arise_cooldown_cost > curr_score,
                    );

                    let arise_count_cost =
                        game_data.shop_items[4].shop_displays[arise_count_level as usize].cost;
                    get_shop_item(
                        parent,
                        game_data.shop_items[4].name.clone(),
                        game_data.shop_items[4].shop_displays[arise_count_level as usize]
                            .text
                            .clone(),
                        arise_count_cost,
                        ShopButtonAction::ARISE_COUNT,
                        arise_count_cost < 0,
                        arise_count_cost > curr_score,
                    );

                    let attack_cooldown_cost =
                        game_data.shop_items[5].shop_displays[attack_cooldown_level as usize].cost;
                    get_shop_item(
                        parent,
                        game_data.shop_items[5].name.clone(),
                        game_data.shop_items[5].shop_displays[attack_cooldown_level as usize]
                            .text
                            .clone(),
                        attack_cooldown_cost,
                        ShopButtonAction::HIT_COOLDOWN,
                        attack_cooldown_cost < 0,
                        attack_cooldown_cost > curr_score,
                    );
                });

            parent
                .spawn((
                    SceneObject,
                    Node {
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SceneObject,
                        Text::new(format!("Current Points: {}", curr_score.to_string())),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Vh(4.0)),
                            ..default()
                        },
                    ));
                    parent
                        .spawn((
                            SceneObject,
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            ShopButtonAction::PLAY,
                        ))
                        .with_children(|parent| {
                            // let icon = asset_server.load("textures/Game Icons/right.png");
                            // parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                            parent.spawn((
                                SceneObject,
                                Text::new("Play"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
}

//main menu
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

#[derive(Component)]
struct OnMainMenuScreen;
#[derive(Component)]
enum MenuButtonAction {
    Play,
}
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const DISABLED_TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_node = Node {
        width: Val::Vh(27.0),
        height: Val::Vh(6.0),
        margin: UiRect::all(Val::Vh(1.8)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands
        .spawn((
            SceneObject,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SceneObject,
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Vh(4.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn((
                        SceneObject,
                        Text::new("Grumpy Sword"),
                        TextFont {
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Vh(4.0)),
                            ..default()
                        },
                    ));

                    // Display three buttons for each action available from the main menu:
                    // - new game
                    // - settings
                    // - quit
                    parent
                        .spawn((
                            SceneObject,
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Play,
                        ))
                        .with_children(|parent| {
                            // let icon = asset_server.load("textures/Game Icons/right.png");
                            // parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                            parent.spawn((
                                SceneObject,
                                Text::new("Play"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
}
fn menu_action(
    mut commands: Commands,
    ui_sounds: Res<UISounds>,
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.spawn((
                AudioPlayer::new(ui_sounds.button_1.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..Default::default()
                },
            ));
            match menu_button_action {
                MenuButtonAction::Play => {
                    game_state.set(GameState::InGame);
                }
            }
        }
    }
}

//in game

#[derive(Component)]
struct ScoreDispay;
fn setup_game_ui(mut commands: Commands) {
    let text_font = TextFont {
        font_size: 33.0,
        ..default()
    };
    commands
        .spawn((
            SceneObject,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::End,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneObject,
                ScoreDispay,
                Text::new("0"),
                text_font.clone(),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::all(Val::Vh(4.0)),
                    ..default()
                },
            ));
        });
}

fn update_socre_display_system(
    scorer_query: Query<&Scorer>,
    mut display_query: Query<&mut Text, With<ScoreDispay>>,
) {
    let Ok(scorer) = scorer_query.get_single() else {
        return;
    };

    for mut text in display_query.iter_mut() {
        text.0 = scorer.current.to_string();
    }
}

//loading
fn setup_loading_ui(mut commands: Commands) {
    let text_font = TextFont {
        font_size: 33.0,
        ..default()
    };
    commands
        .spawn((
            SceneObject,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneObject,
                Text::new("Loading Assets"),
                text_font.clone(),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::all(Val::Vh(4.0)),
                    ..default()
                },
            ));
        });
}

//compiling
#[derive(Component)]
pub struct CompText;
fn setup_compiling_ui(mut commands: Commands) {
    let text_font = TextFont {
        font_size: 33.0,
        ..default()
    };
    commands
        .spawn((
            SceneObject,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                CompText,
                SceneObject,
                Text::new("Compiling Shaders"),
                text_font.clone(),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::all(Val::Vh(4.0)),
                    ..default()
                },
            ));
        });
}
