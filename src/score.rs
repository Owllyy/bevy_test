use bevy::{diagnostic::DiagnosticsStore, prelude::*};

#[derive(Default, Component, PartialEq)]
pub struct Score {
    pub score: u64,
    pub text: String,
}

#[derive(Default, Component, PartialEq)]
pub struct BestScore {
    pub score: u64,
    pub text: String,
}


#[derive(Bundle)]
pub struct ScoreBundle {
    pub score: Score,
    pub bundle: TextBundle,
}

#[derive(Bundle)]
pub struct BestScoreBundle {
    pub score: BestScore,
    pub bundle: TextBundle,
}

impl Default for ScoreBundle {
    fn default() -> Self {
        Self {
            score: Score{score:0, text: "Score : ".to_string()},
            bundle: TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Score : 0",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font_size: 30.0,
                    ..default()
                },
            ).with_style(Style{
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..Default::default()}),
        }
    }
}

impl Default for BestScoreBundle {
    fn default() -> Self {
        Self {
            score: BestScore{score:0, text: "Best : ".to_string()},
            bundle: TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Best : 0",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font_size: 30.0,
                    ..default()
                },
            ).with_style(Style{
                top: Val::Px(40.0),
                left: Val::Px(10.0),
                ..Default::default()}),
        }
    }
}

//todo optimisation
pub fn score_update( mut query: Query<(&mut Text, &Score)>) {
    if let Ok((mut text, score)) = query.get_single_mut() {
        text.sections[0].value = format!("{}{}", score.text, score.score);
    }
}

//todo optimisation
pub fn best_score_update( mut query: Query<(&mut BestScore, &mut Text)>) {
    if let Ok((score, mut text)) = query.get_single_mut() {
        text.sections[0].value = format!("{}{}", score.text, score.score);
    }
}