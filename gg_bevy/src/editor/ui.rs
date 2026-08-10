use bevy::prelude::*;

use super::{EditorBoard, EditorState, EditorTile};

/// Marker: the tool-button background node for `tile`.
#[derive(Component)]
pub struct ToolButtonBg(EditorTile);

/// Marker: the board-info text node.
#[derive(Component)]
pub struct InfoText;

/// Marker: a placement-count text, colored red/green based on validity.
#[derive(Component)]
pub struct AgentCountText(pub EditorTile);

/// Marker: the transient message text node.
#[derive(Component)]
pub struct MessageText;

/// Marker: the filename input text node.
#[derive(Component)]
pub struct FilenameInputBox;

/// Marker: the small dot shown while there are unsaved changes.
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
        EditorTile::Empty => Color::srgb_u8(59, 124, 57),
        EditorTile::Wall => Color::srgb_u8(55, 50, 50),
        EditorTile::Agent => Color::srgb_u8(194, 63, 60),
        EditorTile::Ghost => Color::srgb_u8(182, 182, 182),
        EditorTile::Goblet(reward) => {
            if reward > 0 {
                Color::srgb_u8(187, 155, 68)
            } else {
                Color::srgb_u8(166, 160, 131)
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
            // Title row: name and unsaved-changes dot.
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..Default::default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("World Editor"),
                        TextFont {
                            font_size: 18.0,
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Node {
                            width: Val::Px(8.0),
                            height: Val::Px(8.0),
                            display: Display::None,
                            ..Default::default()
                        },
                        BackgroundColor(Color::srgb(1.0, 0.85, 0.0)),
                        DirtyIndicator,
                    ));
                });

            // Tool buttons (visual only — keyboard selects tool)
            for (label, tile) in [
                ("E  Empty", EditorTile::Empty),
                ("W  Wall", EditorTile::Wall),
                ("1  Agent", EditorTile::Agent),
                ("2  Ghost", EditorTile::Ghost),
                ("3  Goblet", EditorTile::Goblet(0)),
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

            panel.spawn(Node {
                height: Val::Px(4.0),
                ..default()
            });

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

            // Agent and ghost placement counts
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..Default::default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Agent:"),
                        TextFont {
                            font_size: 13.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.75, 0.75, 0.75)),
                    ));
                    row.spawn((
                        Text::new("0/1"),
                        TextFont {
                            font_size: 13.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.863, 0.275, 0.275)),
                        AgentCountText(EditorTile::Agent),
                    ));
                    row.spawn((
                        Text::new("  Ghost:"),
                        TextFont {
                            font_size: 13.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.75, 0.75, 0.75)),
                    ));
                    row.spawn((
                        Text::new("0/1"),
                        TextFont {
                            font_size: 13.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.302, 0.784, 0.376)),
                        AgentCountText(EditorTile::Ghost),
                    ));
                });

            panel.spawn(Node {
                height: Val::Px(4.0),
                ..default()
            });

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
                Interaction::default(),
                FilenameInputBox,
                Text::new(""),
                TextFont {
                    font_size: 11.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            panel.spawn(Node {
                height: Val::Px(4.0),
                ..default()
            });

            // Keybindings hint
            panel.spawn((
                Text::new(
                    "Keys:",
                ),
                TextFont {
                    font_size: 11.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
            panel.spawn((
                Text::new(
                    "E/W/1/2/3     select tool\n[/]           change reward\nS             save world\nTab           edit path\nG             toggle grid\n+/-           zoom in/out\nShift+drag    pan camera\nEsc           quit editor",
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
                            font_size: 20.0,
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    dialog.spawn((
                        Text::new("Save before exiting?"),
                        TextFont {
                            font_size: 13.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.75, 0.75, 0.75)),
                    ));
                    dialog
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            ..Default::default()
                        })
                        .with_children(|row| {
                            for (label, kind, color) in [
                                ("Save", ExitDialogButton::Save, Color::srgb(0.302, 0.784, 0.376)),
                                ("Discard", ExitDialogButton::Discard, Color::srgb(0.863, 0.275, 0.275)),
                                ("Cancel", ExitDialogButton::Cancel, Color::srgb(0.4, 0.4, 0.45)),
                            ] {
                                row.spawn((
                                    Button,
                                    Interaction::default(),
                                    Node {
                                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                        ..Default::default()
                                    },
                                    BackgroundColor(color.with_alpha(0.5)),
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

/// Syncs the unsaved-changes indicator and window title with `state.dirty`.
pub fn sync_dirty_indicator(
    state: Res<EditorState>,
    mut q: Query<&mut Node, With<DirtyIndicator>>,
    mut windows: Query<&mut Window>,
) {
    if let Ok(mut node) = q.single_mut() {
        node.display = if state.dirty { Display::Flex } else { Display::None };
    }

    if let Ok(mut window) = windows.single_mut() {
        let title = if state.dirty {
            "* Goblets and Ghouls World Editor"
        } else {
            "Goblets and Ghouls World Editor"
        };
        if window.title != title {
            window.title = title.to_string();
        }
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

#[allow(clippy::type_complexity)]
pub fn sync_ui(
    state: Res<EditorState>,
    board: Res<EditorBoard>,
    mut btn_q: Query<(&ToolButtonBg, &mut BackgroundColor), Without<FilenameInputBox>>,
    mut info_q: Query<
        &mut Text,
        (With<InfoText>, Without<MessageText>, Without<FilenameInputBox>),
    >,
    mut count_q: Query<
        (&AgentCountText, &mut Text, &mut TextColor),
        (Without<InfoText>, Without<MessageText>, Without<FilenameInputBox>),
    >,
    mut msg_q: Query<
        &mut Text,
        (With<MessageText>, Without<InfoText>, Without<FilenameInputBox>),
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
        let color = match tile {
            EditorTile::Goblet(_) => tool_color(EditorTile::Goblet(state.current_goblet_reward)),
            _ => tool_color(*tile),
        };
        *bg = BackgroundColor(color.with_alpha(alpha));
    }

    // Board info and goblet reward readout
    if let Ok(mut t) = info_q.single_mut() {
        *t = Text::new(format!(
            "Size: {}x{}\nGrid: {}\nGoblet reward: {}",
            board.width,
            board.height,
            if state.show_grid { "on" } else { "off" },
            state.current_goblet_reward,
        ));
    }

    // Placement counts are green when their corresponding board constraint holds.
    for (AgentCountText(tile), mut t, mut color) in count_q.iter_mut() {
        let count = match tile {
            EditorTile::Agent => board.agent_count(),
            EditorTile::Ghost => board.ghost_count(),
            _ => 0,
        };
        *t = Text::new(format!("{count}/1"));
        let valid = match tile {
            EditorTile::Agent => count == 1,
            EditorTile::Ghost => count <= 1,
            _ => false,
        };
        *color = TextColor(if valid {
            Color::srgb(0.302, 0.784, 0.376)
        } else {
            Color::srgb(0.863, 0.275, 0.275)
        });
    }

    // Filename input box
    if let Ok((mut t, mut bg)) = filename_q.single_mut() {
        let display = if state.filename_editing {
            format!("{}|", state.filename)
        } else if state.filename.is_empty() {
            "(Tab or click to set path)".to_string()
        } else {
            state.filename.clone()
        };
        *t = Text::new(display);
        *bg = BackgroundColor(if state.filename_editing {
            Color::srgb(0.25, 0.25, 0.30)
        } else {
            Color::srgb(0.15, 0.15, 0.18)
        });
    }

    // Message
    if let Ok(mut t) = msg_q.single_mut() {
        *t = Text::new(state.message.clone().unwrap_or_default());
    }
}
