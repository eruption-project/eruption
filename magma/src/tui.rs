/*  SPDX-License-Identifier: GPL-3.0-or-later  */

/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use crate::themes::THEME;
use crate::{custom_widgets, switch_to_slot, util, ConnectionState, STATE};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use itertools::Itertools;
use ratatui::layout::{Alignment, Layout, Margin};
use ratatui::layout::{Constraint, Direction, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Gauge, Padding, Paragraph, Tabs, Widget, Wrap,
};
use ratatui::Frame;
use std::io;
use std::sync::atomic::Ordering;

pub mod assistants;
pub mod hwdevices;
pub mod pages;

// use crate::translations::tr;

// type Result<T> = std::result::Result<T, eyre::Error>;

pub(crate) fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(15))? {
        let event = event::read()?;

        // call the event handler of the active page
        let page = &mut crate::PAGES.write().unwrap()[crate::ACTIVE_PAGE.load(Ordering::SeqCst)];
        page.handle_event(&event);

        // global event processing
        let connection_state = *crate::CONNECTION_STATE.read().unwrap();

        match connection_state {
            ConnectionState::Connected => {
                if let Event::Key(key) = event {
                    if key.kind != event::KeyEventKind::Press {
                        return Ok(true);
                    }

                    // quit with 'q' or ESCAPE
                    if key.code == KeyCode::Char('q')
                        || key.code == KeyCode::Char('Q')
                        || key.code == KeyCode::Esc
                    {
                        return Ok(true);
                    }

                    // show help page using '?' or 'h' keys
                    if key.code == KeyCode::Char('?') || key.code == KeyCode::Char('h') {
                        crate::ACTIVE_PAGE.store(11, Ordering::SeqCst);
                    }

                    // switch tabs
                    if key.code == KeyCode::Tab {
                        if key.modifiers == KeyModifiers::SHIFT {
                            let val = crate::ACTIVE_PAGE.fetch_sub(1, Ordering::SeqCst);

                            if val == 0 {
                                crate::ACTIVE_PAGE.store(12, Ordering::SeqCst);
                            }
                        } else {
                            let val = crate::ACTIVE_PAGE.fetch_add(1, Ordering::SeqCst);

                            if val >= 12 {
                                crate::ACTIVE_PAGE.store(0, Ordering::SeqCst);
                            }
                        }
                    }

                    // switch tabs backwards
                    if key.code == KeyCode::Backspace {
                        if key.modifiers == KeyModifiers::SHIFT {
                            let val = crate::ACTIVE_PAGE.fetch_add(1, Ordering::SeqCst);

                            if val >= 12 {
                                crate::ACTIVE_PAGE.store(0, Ordering::SeqCst);
                            }
                        } else {
                            let val = crate::ACTIVE_PAGE.fetch_sub(1, Ordering::SeqCst);

                            if val == 0 {
                                crate::ACTIVE_PAGE.store(12, Ordering::SeqCst);
                            }
                        }
                    }

                    // switch slots
                    if key.code == KeyCode::F(1) {
                        switch_to_slot(0).unwrap();
                    }
                    if key.code == KeyCode::F(2) {
                        switch_to_slot(1).unwrap();
                    }
                    if key.code == KeyCode::F(3) {
                        switch_to_slot(2).unwrap();
                    }
                    if key.code == KeyCode::F(4) {
                        switch_to_slot(3).unwrap();
                    }

                    // adjust brightness
                    if key.code == KeyCode::F(8) {
                        let mut brightness = crate::STATE
                            .read()
                            .unwrap()
                            .current_brightness
                            .unwrap_or_else(|| 0);

                        if key.modifiers == KeyModifiers::SHIFT {
                            brightness -= 1;
                        } else {
                            brightness -= 5;
                        }

                        brightness = brightness.clamp(0, 100);

                        util::set_brightness(brightness).unwrap_or_else(|e| {
                            tracing::error!("Could not adjust brightness: {e}")
                        });
                    }
                    if key.code == KeyCode::F(9) {
                        let mut brightness = crate::STATE
                            .read()
                            .unwrap()
                            .current_brightness
                            .unwrap_or_else(|| 0);

                        if key.modifiers == KeyModifiers::SHIFT {
                            brightness += 1;
                        } else {
                            brightness += 5;
                        }

                        brightness = brightness.clamp(0, 100);

                        util::set_brightness(brightness).unwrap_or_else(|e| {
                            tracing::error!("Could not adjust brightness: {e}")
                        });
                    }

                    // select tab N
                    if key.modifiers == KeyModifiers::ALT {
                        match key.code {
                            KeyCode::Char('1') => crate::ACTIVE_PAGE.store(0, Ordering::SeqCst),
                            KeyCode::Char('2') => crate::ACTIVE_PAGE.store(1, Ordering::SeqCst),
                            KeyCode::Char('3') => crate::ACTIVE_PAGE.store(2, Ordering::SeqCst),
                            KeyCode::Char('4') => crate::ACTIVE_PAGE.store(3, Ordering::SeqCst),
                            KeyCode::Char('5') => crate::ACTIVE_PAGE.store(4, Ordering::SeqCst),
                            KeyCode::Char('6') => crate::ACTIVE_PAGE.store(5, Ordering::SeqCst),
                            KeyCode::Char('7') => crate::ACTIVE_PAGE.store(6, Ordering::SeqCst),
                            KeyCode::Char('8') => crate::ACTIVE_PAGE.store(7, Ordering::SeqCst),
                            KeyCode::Char('9') => crate::ACTIVE_PAGE.store(8, Ordering::SeqCst),
                            KeyCode::Char('0') => crate::ACTIVE_PAGE.store(9, Ordering::SeqCst),

                            _ => { /* do nothing */ }
                        }
                    }
                }
            }

            ConnectionState::Initializing | ConnectionState::Disconnected => {
                if let Event::Key(key) = event {
                    if key.kind != event::KeyEventKind::Press {
                        return Ok(true);
                    }

                    // quit with 'q' or ESCAPE
                    if key.code == KeyCode::Char('q')
                        || key.code == KeyCode::Char('Q')
                        || key.code == KeyCode::Esc
                    {
                        return Ok(true);
                    }

                    // show help page using '?' or 'h' keys
                    // if key.code == KeyCode::Char('?') || key.code == KeyCode::Char('h') {
                    //     crate::ACTIVE_PAGE.store(1, Ordering::SeqCst);
                    // }

                    // switch tabs
                    /* if key.code == KeyCode::Tab {
                        if key.modifiers == KeyModifiers::SHIFT {
                            let val = crate::ACTIVE_PAGE.fetch_sub(1, Ordering::SeqCst);

                            if val == 0 {
                                crate::ACTIVE_PAGE.store(1, Ordering::SeqCst);
                            }
                        } else {
                            let val = crate::ACTIVE_PAGE.fetch_add(1, Ordering::SeqCst);

                            if val >= 1 {
                                crate::ACTIVE_PAGE.store(0, Ordering::SeqCst);
                            }
                        }
                    } */

                    // switch tabs backwards
                    /* if key.code == KeyCode::Backspace {
                        if key.modifiers == KeyModifiers::SHIFT {
                            let val = crate::ACTIVE_PAGE.fetch_add(1, Ordering::SeqCst);

                            if val >= 1 {
                                crate::ACTIVE_PAGE.store(0, Ordering::SeqCst);
                            }
                        } else {
                            let val = crate::ACTIVE_PAGE.fetch_sub(1, Ordering::SeqCst);

                            if val == 0 {
                                crate::ACTIVE_PAGE.store(1, Ordering::SeqCst);
                            }
                        }
                    } */
                }
            }
        }
    }

    Ok(false)
}

fn render_tab_bar(frame: &mut Frame, area: Rect) {
    // render tab bar
    let tabs = Tabs::new(vec![
        "  Canvas  ",
        " Keyboards ",
        "   Mice   ",
        "   Misc   ",
        " Color Schemes ",
        " Automation Rules ",
        " Profiles & Scripts ",
        "  Macros  ",
        "  Keymaps  ",
        " Settings ",
        "   Logs   ",
        "   Help   ",
        "   About   ",
    ])
    .style(THEME.tabs)
    .highlight_style(THEME.tabs_selected)
    .divider("|")
    .select(crate::ACTIVE_PAGE.load(Ordering::SeqCst));
    frame.render_widget(tabs, area);
}

fn render_client_area(frame: &mut Frame, area: Rect) {
    // render client area
    let page = &mut crate::PAGES.write().unwrap()[crate::ACTIVE_PAGE.load(Ordering::SeqCst)];
    page.render(frame, area);
}

fn render_effects_area(frame: &mut Frame, area: Rect) {
    // render effects area

    let state = STATE.read().unwrap();

    let lines = vec![
        Line::styled(
            format!(
                "Ambient Effect: {}",
                if state.ambient_effect { "on" } else { "off " }
            ),
            THEME.description,
        ),
        Line::styled("", THEME.description),
        Line::styled(
            format!(
                "Audio Effects:  {}",
                if state.sound_fx { "on" } else { "off " }
            ),
            THEME.description,
        ),
    ];

    let effects = Paragraph::new(lines).alignment(Alignment::Left).block(
        Block::new()
            .title(format!(" Effects "))
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_style(THEME.borders)
            .padding(Padding::new(1, 1, 1, 1)),
    );
    frame.render_widget(
        effects,
        area.inner(&Margin {
            horizontal: 1,
            vertical: 0,
        }),
    );
}

fn render_message_area(frame: &mut Frame, area: Rect) {
    // render message area, right inside of effects area
    let logs = Paragraph::new(Span::styled(
        "Successfully connected to the Eruption daemon",
        THEME.content,
    ))
    .alignment(Alignment::Center);
    frame.render_widget(logs, area.inner(&Margin::new(3, 3)));
}

fn render_slot_bar(frame: &mut Frame, area: &Vec<Rect>) {
    let state = STATE.read().unwrap();

    let active_slot = state.active_slot;
    // let active_profile = state.active_profile.as_ref();
    let slot_names = state.slot_names.clone().unwrap_or_else(|| {
        vec![
            "Profile Slot 1".to_string(),
            "Profile Slot 2".to_string(),
            "Profile Slot 3".to_string(),
            "Profile Slot 4".to_string(),
        ]
    });
    let slot_profiles = state.slot_profiles.clone().unwrap_or_else(|| {
        vec![
            "<unknown>".to_string(),
            "<unknown>".to_string(),
            "<unknown>".to_string(),
            "<unknown>".to_string(),
        ]
    });

    // render slot bar
    for i in 0..4 {
        if active_slot.is_some() && active_slot.unwrap() == i {
            let mut slot = custom_widgets::ComboBox::new(&slot_profiles)
                .alignment(Alignment::Center)
                .style(THEME.active_slot)
                .block(
                    Block::new()
                        .title(format!(" {} ", slot_names[i]))
                        .title_alignment(Alignment::Left)
                        .borders(Borders::ALL)
                        .border_style(THEME.borders_highlight)
                        .border_type(BorderType::Thick)
                        .padding(Padding::new(1, 1, 1, 1)),
                );

            slot.toggle_open();

            frame.render_widget(
                slot,
                area[i].inner(&Margin {
                    horizontal: 1,
                    vertical: 0,
                }),
            );
        } else {
            let slot = Paragraph::new(format!(" {} ", slot_profiles[i]))
                .alignment(Alignment::Center)
                .style(THEME.slot)
                .block(
                    Block::new()
                        .title(format!(" {} ", slot_names[i]))
                        .title_alignment(Alignment::Left)
                        .borders(Borders::ALL)
                        .border_style(THEME.borders)
                        .border_type(BorderType::Plain)
                        .padding(Padding::new(1, 1, 1, 1)),
                );

            frame.render_widget(
                slot,
                area[i].inner(&Margin {
                    horizontal: 1,
                    vertical: 0,
                }),
            );
        }
    }
}

fn render_footer(frame: &mut Frame, area: Rect) {
    // render footer bar
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Min(20),
            Constraint::Length(5),
            Constraint::Length(107),
            Constraint::Percentage(30),
        ])
        .split(area);

    let state = STATE.read().unwrap();

    // let active_slot = state.active_slot;
    let active_profile = &state.active_profile;
    let brightness = state.current_brightness.unwrap_or_else(|| 0);

    // render logo string and title
    let title = Paragraph::new("Magma TUI")
        .style(THEME.app_title)
        .alignment(Alignment::Left);
    frame.render_widget(title, layout[0]);

    let separator = Clear::default();
    frame.render_widget(separator, layout[1]);

    let brightness_indicator = Gauge::default()
        .label(format!("{brightness}%"))
        .style(THEME.content)
        .gauge_style(THEME.description)
        .percent(brightness as u16);
    frame.render_widget(brightness_indicator, layout[2]);

    let separator = Clear::default();
    frame.render_widget(separator, layout[3]);

    // help text
    let keys = [
        ("Esc", "Quit"),
        ("Tab", "Next Tab"),
        ("Backspace", "Previous Tab"),
        ("F1-F4", "Switch slots"),
        ("F8-F9", "Adjust brightness"),
        ("?", "Help"),
    ];

    let spans = keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {} ", key), THEME.key_binding.key);
            let desc = Span::styled(format!(" {} ", desc), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();

    let keybindings = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .fg(Color::Indexed(236))
        .bg(Color::Indexed(232));
    frame.render_widget(keybindings, layout[4]);

    let indicator = Paragraph::new(Span::styled(
        active_profile
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string()),
        THEME.app_title,
    ))
    .alignment(Alignment::Right);
    frame.render_widget(indicator, layout[5]);
}

fn render_alternate_footer(frame: &mut Frame, area: Rect) {
    // render footer bar

    // render logo string and title
    let title =
        Paragraph::new(Span::styled("Magma TUI", THEME.app_title)).alignment(Alignment::Left);
    frame.render_widget(title, area);

    // help text
    let keys = [
        ("Esc", "Quit"),
        // ("?", "Help")
    ];

    let spans = keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {} ", key), THEME.key_binding.key);
            let desc = Span::styled(format!(" {} ", desc), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();

    let keybindings = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .fg(Color::Indexed(236))
        .bg(Color::Indexed(232));
    frame.render_widget(keybindings, area);
}

pub(crate) fn clear_screen(frame: &mut Frame) {
    // clear screen
    Clear.render(frame.size(), frame.buffer_mut());
}

pub(crate) fn render_ui(frame: &mut Frame) {
    let connection_state = *crate::CONNECTION_STATE.read().unwrap();

    match connection_state {
        ConnectionState::Connected => {
            // clear screen
            Clear.render(frame.size(), frame.buffer_mut());

            Block::new()
                .style(THEME.root)
                .borders(Borders::ALL)
                .border_style(THEME.borders)
                .render(frame.size(), frame.buffer_mut());

            // generate layout
            let (title_area, layout, effects_area, slots_area, footer_area) =
                calculate_layout(frame.size().inner(&Margin::new(1, 1)));

            render_tab_bar(frame, title_area);

            render_client_area(frame, layout[0][0].inner(&Margin::new(2, 1)));

            render_effects_area(frame, effects_area[0]);
            render_message_area(frame, effects_area[0]);

            render_slot_bar(frame, &slots_area);
            render_footer(frame, footer_area.inner(&Margin::new(1, 0)));
        }

        ConnectionState::Initializing | ConnectionState::Disconnected => {
            // clear screen
            Clear.render(frame.size(), frame.buffer_mut());

            Block::new()
                .style(THEME.root)
                .borders(Borders::ALL)
                .border_style(THEME.borders)
                .render(frame.size(), frame.buffer_mut());

            let (title_area, layout, footer_area) =
                calculate_alternate_layout(frame.size().inner(&Margin::new(1, 1)));

            let title = Paragraph::new("").dark_gray().alignment(Alignment::Center);
            frame.render_widget(title, title_area);

            /* let text = render(cfonts::Options {
                text: String::from("Connecting to Eruption..."),
                font: Fonts::FontSimple,
                ..cfonts::Options::default()
            })
            .vec
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>(); */

            let text = "Connecting to Eruption...";
            let p = Paragraph::new(text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false });
            frame.render_widget(p, layout[1].inner(&Margin::new(2, 1)));

            render_alternate_footer(frame, footer_area.inner(&Margin::new(1, 0)));
        }
    }
}

fn calculate_layout(area: Rect) -> (Rect, Vec<Vec<Rect>>, Vec<Rect>, Vec<Rect>, Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let title_area = layout[0];

    let main_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(layout[1])
        .iter()
        .map(|&area| {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .split(area)
                .to_vec()
        })
        .collect_vec();

    let effects_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(layout[2])
        .iter()
        .cloned()
        .collect_vec();

    let slots_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(layout[3])
        .iter()
        .cloned()
        .collect_vec();

    let footer_area = layout[5];

    (
        title_area,
        main_areas,
        effects_area,
        slots_area,
        footer_area,
    )
}

fn calculate_alternate_layout(area: Rect) -> (Rect, Vec<Rect>, Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let title_area = layout[0];

    let main_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Percentage(50)])
        .split(layout[1])
        .iter()
        .cloned()
        .collect_vec();

    let footer_area = layout[2];

    (title_area, main_areas, footer_area)
}
