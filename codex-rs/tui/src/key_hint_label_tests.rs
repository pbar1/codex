//! Compact shortcut labels stay uniformly emphasized above secondary descriptions.

use super::*;
use pretty_assertions::assert_eq;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;
use ratatui::widgets::Paragraph;

#[test]
fn cached_picker_hint_matches_a_fresh_hint_after_palette_refresh() {
    use crate::terminal_palette::with_test_default_colors;
    use crate::terminal_probe::DefaultColors;
    let dark = DefaultColors {
        fg: (212, 190, 152),
        bg: (29, 32, 33),
    };
    let light = DefaultColors {
        fg: (101, 71, 53),
        bg: (242, 229, 188),
    };
    let (cached, captured) = with_test_default_colors(dark, || {
        (
            crate::bottom_pane::popup_consts::standard_popup_hint_line(),
            key_hint_style(),
        )
    });
    for colors in [light, dark] {
        with_test_default_colors(colors, || {
            let mut refreshed = cached.clone();
            refresh_cached_hint_styles(&mut refreshed, captured);
            assert_eq!(
                refreshed,
                crate::bottom_pane::popup_consts::standard_popup_hint_line()
            );
        });
    }
}

#[test]
fn modifier_combinations_stay_compact_inside_chords() {
    let cases = [
        (KeyModifiers::CONTROL, "⌃t"),
        (KeyModifiers::ALT, "⌥t"),
        (KeyModifiers::CONTROL | KeyModifiers::SHIFT, "⌃⇧t"),
        (KeyModifiers::CONTROL | KeyModifiers::ALT, "⌃⌥t"),
    ];
    for (modifiers, expected) in cases {
        let prefix = KeyBinding::new(KeyCode::Char('t'), modifiers);
        let chord = ShortcutHint::Chord {
            prefix,
            completion: plain(KeyCode::Enter),
        };
        assert_eq!(
            (prefix.display_label(), chord.display_label()),
            (expected.to_owned(), format!("{expected} enter")),
        );
    }
}

#[test]
fn hint_cells_remain_readable_across_terminal_palettes() {
    for (name, fg, bg, key_rgb) in [
        ("light", (32, 32, 32), (255, 255, 255), (65, 65, 65)),
        ("dark", (240, 240, 240), (18, 20, 30), (240, 240, 240)),
        (
            "low contrast",
            (255, 255, 255),
            (255, 255, 255),
            (118, 118, 118),
        ),
    ] {
        crate::terminal_palette::with_test_default_colors(
            crate::terminal_probe::DefaultColors { fg, bg },
            || {
                let keys = "ctrl+o t/f4";
                let hint = crate::footer_hint::shortcut(keys, "inspect activity");
                let width = hint.width();
                let mut terminal =
                    Terminal::new(TestBackend::new(/*width*/ 32, /*height*/ 1)).unwrap();
                terminal
                    .draw(|frame| {
                        frame.render_widget(
                            Paragraph::new(hint).style(
                                Style::default()
                                    .fg(Color::Green)
                                    .bg(crate::terminal_palette::rgb_color(bg))
                                    .bold()
                                    .dim(),
                            ),
                            frame.area(),
                        );
                    })
                    .unwrap();
                let cells = &terminal.backend().buffer().content[..width];
                assert_eq!(
                    cells[..keys.len()]
                        .iter()
                        .map(|cell| (cell.fg, cell.modifier))
                        .collect::<Vec<_>>(),
                    vec![(crate::terminal_palette::rgb_color(key_rgb), Modifier::BOLD); keys.len()],
                    "{name}"
                );
                assert!(
                    cells[keys.len()..]
                        .iter()
                        .all(|cell| cell.modifier == Modifier::empty()),
                    "{name}"
                );
            },
        );
    }
}
