use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::COUNT;
pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (update_fps_text, update_units_text));
    }
}
// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct UnitsText;
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                display: Display::Grid,
                position_type: PositionType::Absolute,
                left: Val::Percent(1.),
                top: Val::Percent(1.),
                bottom: Val::Auto,
                right : Val::Auto,
                padding: UiRect::all(Val::Px(4.0)),
                // justify_content: JustifyContent::FlexStart,
                // flex_direction : FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
            ZIndex(i32::MAX),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    // Create a Text with multiple child spans.
                    Text::new("FPS: "),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                ))
                .with_child((
                    TextSpan::default(),
                    TextFont {
                        font_size: 14.0,
                        // If no font is specified, the default font (a minimal subset of FiraMono) will be used.
                        ..default()
                    },
                    TextColor(GOLD.into()),
                    FpsText,
                ));
            parent
                .spawn((
                    // Create a Text with multiple child spans.
                    Text::new("Starting Unit Count: "),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                ))
                .with_child((
                    TextSpan::default(),
                    TextFont {
                        font_size: 14.0,
                        // If no font is specified, the default font (a minimal subset of FiraMono) will be used.
                        ..default()
                    },
                    TextColor(GOLD.into()),
                    UnitsText,
                ));
        });
}

fn update_units_text(mut query: Query<&mut TextSpan, With<UnitsText>>) {
    for mut span in &mut query {
        **span = COUNT.to_string();
    }
}
fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                **span = format!("{value:.2}");
            }
        }
    }
}
