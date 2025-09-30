mod components;
pub mod events;

use bevy::{
    color::palettes::css::{DARK_GRAY, DARK_GREEN, GRAY},
    log,
    prelude::*,
};

use components::{MenuButtonAction, MenuUIRoot};
use events::LoadSettingsEvent;

pub struct MainMenuPlugin<T> {
    pub running_state: T,
}

impl<T: States> Plugin for MainMenuPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.running_state.clone()), Self::create_menu)
            .add_systems(
                Update,
                (Self::change_background_color, Self::menu_action)
                    .chain()
                    .run_if(in_state(self.running_state.clone())),
            )
            .add_systems(OnExit(self.running_state.clone()), Self::cleanup_menu);
        app.add_message::<LoadSettingsEvent>();
    }
}

impl<T> MainMenuPlugin<T> {
    fn create_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
        let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            MenuUIRoot,
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(50.0),
                    ..Default::default()
                },
                children![
                    Self::button("Start game", font.clone(), MenuButtonAction::StartGame),
                    Self::button("Quit", font.clone(), MenuButtonAction::Quit),
                ]
            )],
        ));

        log::info!("Main menu initialized");
    }

    fn button(label: &str, font: Handle<Font>, action: MenuButtonAction) -> impl Bundle {
        (
            Node {
                width: Val::Px(250.0),
                padding: UiRect::all(Val::Px(16.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            BackgroundColor(Color::from(GRAY)),
            Button,
            action,
            children![(
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )],
        )
    }

    fn menu_action(
        mut interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut load_settings: MessageWriter<LoadSettingsEvent>,
        mut exit: MessageWriter<AppExit>,
    ) {
        for (interaction, action) in &mut interaction_query {
            if *interaction == Interaction::Pressed {
                match action {
                    MenuButtonAction::StartGame => {
                        load_settings.write(LoadSettingsEvent);
                    }
                    MenuButtonAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
            }
        }
    }

    fn change_background_color(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut background_color) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    *background_color = Color::from(DARK_GREEN).into();
                }
                Interaction::Hovered => {
                    *background_color = Color::from(DARK_GRAY).into();
                }
                Interaction::None => {
                    *background_color = Color::from(GRAY).into();
                }
            }
        }
    }

    fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUIRoot>>) {
        let entity = query.single().unwrap();
        commands.entity(entity).despawn();
        log::info!("Main menu closed");
    }
}
