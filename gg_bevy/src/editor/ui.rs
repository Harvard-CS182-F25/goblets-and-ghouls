use bevy::prelude::*;

use super::{EditorBoard, EditorState, EditorTile};

/// Marker: the tool-button background node for `tile`.
#[derive(Component)]
pub struct ToolButtonBg(EditorTile);

/// Marker: the board-info text node.
#[derive(Component)]
pub struct InfoText;

/// Marker: the transient message text node.
#[derive(Component)]
pub struct MessageText;

/// Marker: the filename input text node.
#[derive(Component)]
pub struct FilenameInputBox;

/// Marker: the "unsaved changes" indicator, shown whenever `state.dirty`.
#[derive(Component)]
pub struct DirtyIndicator;

/// Marker: the unsaved-changes exit-confirmation dialog's root overlay.
#[derive(Component)]
pub struct ExitDialogRoot;

/// Which exit-confirmation dialog button an entity is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ExitDialogButton {
    Save,
    Discard,
    Cancel,
}

const PANEL_WIDTH: f32 = 240.0;

fn tool_color(tile: EditorTile) -> Color {
    match tile {
        EditorTile::Empty => Color::srgb(0.25, 0.25, 0.29),
        EditorTile::Wall => Color::srgb(0.0, 0.0, 0.0),
        EditorTile::Agent => Color::srgb(1.0, 0.0, 0.0),
        EditorTile::Ghost => Color::srgb(1.0, 1.0, 1.0),
        EditorTile::Goblet(reward) => {
            if reward > 0 {
                Color::srgb_u8(255, 215, 0)
            } else {
                Color::srgb_u8(255, 69, 0)
            }
        }
    }
}

pub fn setup_ui(mut commands: Commands) {
    // ── right-side panel ──────────────────────────────────────────────────────
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(6.0),
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.07, 0.07, 0.09, 0.95)),
        ))
        .with_children(|panel| {
            // Title
            panel.spawn((
                Text::new("World Editor"),
                TextFont {
                    font_size: 18.0,
                    ..Default::default()
                },
                TextColor(Color::WHITE),
            ));

            // Unsaved-changes indicator
            panel.spawn((
                Text::new("Unsaved changes"),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(1.0, 0.6, 0.2)),
                Node {
                    display: Display::None,
                    ..Default::default()
                },
                DirtyIndicator,
            ));

            // Tool buttons (visual only — keyboard selects tool)
            for (label, tile) in [
                ("E  Empty", EditorTile::Empty),
                ("W  Wall", EditorTile::Wall),
                ("A  Agent", EditorTile::Agent),
                ("H  Ghost", EditorTile::Ghost),
                ("O  Goblet", EditorTile::Goblet(0)),
            ] {
                let color = tool_color(tile);
                panel
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            ..Default::default()
                        },
                        BackgroundColor(color.with_alpha(0.4)),
                        ToolButtonBg(tile),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 15.0,
                                ..Default::default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }

            // Goblet reward readout
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                RewardText,
            ));

            // Board info
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 13.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                InfoText,
            ));

            // Save-path label
            panel.spawn((
                Text::new("Save path:"),
                TextFont {
                    font_size: 11.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));

            // Filename input box
            panel.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                    width: Val::Percent(100.0),
                    ..Default::default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                FilenameInputBox,
                Text::new(""),
                TextFont {
                    font_size: 11.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Keybindings hint
            panel.spawn((
                Text::new(
                    "Keys:\nE/W/A/H/O or 1/2/3: tool\n[ / ]: goblet reward\nS: save\nTab: edit path\nG: toggle grid\n=/-: zoom\nShift+Click Drag: pan\nEsc: quit (prompts if unsaved)",
                ),
                TextFont {
                    font_size: 11.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            // Message (save feedback, errors)
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(1.0, 0.7, 0.5)),
                MessageText,
            ));
        });

    // ── unsaved-changes exit-confirmation dialog (modal overlay) ────────────────
    commands
        .spawn((
            ExitDialogRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(100),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(14.0),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                ))
                .with_children(|dialog| {
                    dialog.spawn((
                        Text::new("You have unsaved changes."),
                        TextFont {
                            font_size: 16.0,
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    dialog
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            ..Default::default()
                        })
                        .with_children(|row| {
                            for (label, kind) in [
                                ("Save & Exit", ExitDialogButton::Save),
                                ("Discard & Exit", ExitDialogButton::Discard),
                                ("Cancel", ExitDialogButton::Cancel),
                            ] {
                                row.spawn((
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                        ..Default::default()
                                    },
                                    BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                                    kind,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(label),
                                        TextFont {
                                            font_size: 13.0,
                                            ..Default::default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                        });
                });
        });
}

/// Toggles the "unsaved changes" indicator on/off with `state.dirty`.
pub fn sync_dirty_indicator(state: Res<EditorState>, mut q: Query<&mut Node, With<DirtyIndicator>>) {
    if let Ok(mut node) = q.single_mut() {
        node.display = if state.dirty { Display::Flex } else { Display::None };
    }
}

/// Shows/hides the exit-confirmation modal with `state.exit_dialog_open`.
pub fn sync_exit_dialog(state: Res<EditorState>, mut q: Query<&mut Node, With<ExitDialogRoot>>) {
    if let Ok(mut node) = q.single_mut() {
        node.display = if state.exit_dialog_open {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Marker: the goblet-reward readout text node.
#[derive(Component)]
pub struct RewardText;

#[allow(clippy::type_complexity)]
pub fn sync_ui(
    state: Res<EditorState>,
    board: Res<EditorBoard>,
    mut btn_q: Query<(&ToolButtonBg, &mut BackgroundColor), Without<FilenameInputBox>>,
    mut reward_q: Query<
        &mut Text,
        (With<RewardText>, Without<InfoText>, Without<MessageText>, Without<FilenameInputBox>),
    >,
    mut info_q: Query<
        &mut Text,
        (With<InfoText>, Without<MessageText>, Without<FilenameInputBox>, Without<RewardText>),
    >,
    mut msg_q: Query<
        &mut Text,
        (With<MessageText>, Without<InfoText>, Without<FilenameInputBox>, Without<RewardText>),
    >,
    mut filename_q: Query<
        (&mut Text, &mut BackgroundColor),
        (With<FilenameInputBox>, Without<ToolButtonBg>),
    >,
) {
    // Highlight the active tool button (goblet button always matches by variant, ignoring reward)
    for (ToolButtonBg(tile), mut bg) in btn_q.iter_mut() {
        let selected = std::mem::discriminant(tile) == std::mem::discriminant(&state.current_tool);
        let alpha = if selected { 0.95 } else { 0.35 };
        *bg = BackgroundColor(tool_color(*tile).with_alpha(alpha));
    }

    // Goblet reward readout
    if let Ok(mut t) = reward_q.single_mut() {
        *t = Text::new(format!("Goblet reward: {}", state.current_goblet_reward));
    }

    // Board info
    if let Ok(mut t) = info_q.single_mut() {
        *t = Text::new(format!(
            "{}x{}  Agents:{}/1  Ghosts:{}/1\nGrid: {}",
            board.width,
            board.height,
            board.agent_count(),
            board.ghost_count(),
            if state.show_grid { "on" } else { "off" },
        ));
    }

    // Filename input box
    if let Ok((mut t, mut bg)) = filename_q.single_mut() {
        let display = if state.filename_editing {
            format!("{}|", state.filename)
        } else if state.filename.is_empty() {
            "(Tab to set path)".to_string()
        } else {
            state.filename.clone()
        };
        *t = Text::new(display);
        *bg = BackgroundColor(if state.filename_editing {
            Color::srgb(0.20, 0.20, 0.26)
        } else {
            Color::srgb(0.12, 0.12, 0.15)
        });
    }

    // Message
    if let Ok(mut t) = msg_q.single_mut() {
        *t = Text::new(state.message.clone().unwrap_or_default());
    }
}
