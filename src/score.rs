use bevy::{diagnostic::DiagnosticsStore, prelude::*};

#[derive(Default, Component, PartialEq)]
pub struct Score(pub u64);

#[derive(Bundle)]
pub struct ScoreBundle {
    score: Score,
    text: TextBundle,
}

impl Default for ScoreBundle {
    fn default() -> Self {
        Self { 
            score: Default::default(),
            text: TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Score : 0",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font_size: 50.0,
                    ..default()
                },
            ).with_style(Style{
                left: Val::Px(300.0),
                ..Default::default()}),
        }
    }
}

//todo optimisation
pub fn score_update( mut query: Query<(&mut Text, &Score)>) {
    if let Ok((mut text, score)) = query.get_single_mut() {
        text.sections[0].value = format!("Score : {}", score.0);
    }
}