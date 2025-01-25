use crate::game_state::GameState;
use crate::level_loading::SceneObject;
use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetServer;
use bevy::color::palettes::css::CRIMSON;
use bevy::color::Color;
use bevy::ecs::system::lifetimeless::SCommands;
use bevy::prelude::{
    default, in_state, AlignItems, BackgroundColor, BuildChildren, Button, Changed, ChildBuild,
    Commands, Component, Interaction, IntoSystemConfigs, JustifyContent, NextState, Node, OnEnter,
    Query, Res, ResMut, Text, TextColor, TextFont, UiRect, Val, With,
};
use bevy::ui::FlexDirection;

#[derive(Component)]
struct SelectedOption;
const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);
const HOVERED_PRESSED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct UIStuffPlugin;

impl Plugin for UIStuffPlugin {
    fn build(&self, app: &mut App) {
        //debug
        app.add_systems(OnEnter(GameState::Menu), (setup_main_menu));
        app.add_systems(
            Update,
            (button_system, menu_action).run_if(in_state(GameState::Menu)),
        );
    }
}
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
