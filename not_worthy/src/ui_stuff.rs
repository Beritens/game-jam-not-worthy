use crate::asset_load::{GameData, GameInfos, ShopItem};
use crate::game_state::GameState;
use crate::game_state::GameState::Shop;
use crate::level_loading::SceneObject;
use crate::state_handling::{get_progress, store_progress};
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets};
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
        app.add_systems(OnEnter(GameState::Menu), (setup_main_menu));
        app.add_systems(
            Update,
            (button_system, shop_action, setup_shop).run_if(in_state(GameState::Shop)),
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
    let curr_level = get_progress(pkv, key);
    let Some(game_data) = game_datas.get(game_data_res.data.id()) else {
        return;
    };
    let cost = game_data.shop_items[shop_item as usize].shop_displays[curr_level as usize].cost;

    //remove money
    store_progress(pkv, key, curr_level + 1);
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
) {
    for (interaction, shop_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(shop_screen) = shop_screen_query.get_single() {
                commands.entity(shop_screen).insert(Outdated);
            }
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
) {
    let button_node: Node = Node {
        width: Val::Px(300.0),
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

    let text_color = if disabled {
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
                margin: UiRect::all(Val::Px(10.0)),
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
            if (disabled) {
                parent
                    .spawn((
                        SceneObject,
                        Button,
                        button_node,
                        BackgroundColor(NORMAL_BUTTON),
                    ))
                    .with_children(|parent| {
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
                            });
                    });
            } else {
                parent
                    .spawn((
                        SceneObject,
                        Button,
                        button_node,
                        BackgroundColor(NORMAL_BUTTON),
                        action,
                    ))
                    .with_children(|parent| {
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
                                parent.spawn((
                                    SceneObject,
                                    Text::new(cost.to_string()),
                                    button_text_font.clone(),
                                    TextColor(text_color),
                                ));
                            });
                    });
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
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
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

    let knockback_level = get_progress(&mut pkv, "knockback");
    let damage_level = get_progress(&mut pkv, "damage");
    let speed_level = get_progress(&mut pkv, "speed");
    let arise_cooldown_level = get_progress(&mut pkv, "arise_cooldown");
    let arise_count_level = get_progress(&mut pkv, "arise_count");
    let attack_cooldown_level = get_progress(&mut pkv, "attack_cooldown");
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
                        margin: UiRect::all(Val::Px(50.0)),
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
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));

                    get_shop_item(
                        parent,
                        game_data.shop_items[0].name.clone(),
                        game_data.shop_items[0].shop_displays[knockback_level as usize]
                            .text
                            .clone(),
                        game_data.shop_items[0].shop_displays[knockback_level as usize]
                            .cost
                            .clone(),
                        ShopButtonAction::KNOCKBACK,
                        game_data.shop_items[0].shop_displays[knockback_level as usize].cost < 0,
                    );

                    get_shop_item(
                        parent,
                        game_data.shop_items[1].name.clone(),
                        game_data.shop_items[1].shop_displays[damage_level as usize]
                            .text
                            .clone(),
                        game_data.shop_items[1].shop_displays[damage_level as usize]
                            .cost
                            .clone(),
                        ShopButtonAction::DAMAGE,
                        game_data.shop_items[1].shop_displays[damage_level as usize].cost < 0,
                    );

                    get_shop_item(
                        parent,
                        game_data.shop_items[2].name.clone(),
                        game_data.shop_items[2].shop_displays[speed_level as usize]
                            .text
                            .clone(),
                        game_data.shop_items[2].shop_displays[speed_level as usize]
                            .cost
                            .clone(),
                        ShopButtonAction::SPEED,
                        game_data.shop_items[2].shop_displays[speed_level as usize].cost < 0,
                    );

                    get_shop_item(
                        parent,
                        game_data.shop_items[3].name.clone(),
                        game_data.shop_items[3].shop_displays[arise_cooldown_level as usize]
                            .text
                            .clone(),
                        game_data.shop_items[3].shop_displays[arise_cooldown_level as usize]
                            .cost
                            .clone(),
                        ShopButtonAction::ARISE_COOLDOWN,
                        game_data.shop_items[3].shop_displays[arise_cooldown_level as usize].cost
                            < 0,
                    );

                    get_shop_item(
                        parent,
                        game_data.shop_items[4].name.clone(),
                        game_data.shop_items[4].shop_displays[arise_count_level as usize]
                            .text
                            .clone(),
                        game_data.shop_items[4].shop_displays[arise_count_level as usize]
                            .cost
                            .clone(),
                        ShopButtonAction::ARISE_COUNT,
                        game_data.shop_items[4].shop_displays[arise_count_level as usize].cost < 0,
                    );

                    get_shop_item(
                        parent,
                        game_data.shop_items[5].name.clone(),
                        game_data.shop_items[5].shop_displays[attack_cooldown_level as usize]
                            .text
                            .clone(),
                        game_data.shop_items[5].shop_displays[attack_cooldown_level as usize]
                            .cost
                            .clone(),
                        ShopButtonAction::HIT_COOLDOWN,
                        game_data.shop_items[5].shop_displays[attack_cooldown_level as usize].cost
                            < 0,
                    );
                });

            parent
                .spawn((
                    SceneObject,
                    Node {
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::End,
                        // margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
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
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
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
                        margin: UiRect::all(Val::Px(50.0)),
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
                            margin: UiRect::all(Val::Px(50.0)),
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
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Play => {
                    game_state.set(GameState::InGame);
                }
            }
        }
    }
}
