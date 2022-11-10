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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use std::{ops::RangeInclusive, sync::Arc};

use egui::Widget;
use tracing::error;

use crate::{
    ui::{self, TabPages},
    State,
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Pyroclasm {
    pub active_page: ui::TabPages,

    #[serde(skip)]
    pub toasts: egui_notify::Toasts,

    #[serde(skip)]
    pub modal_quit: Option<egui_modal::Modal>,
}

impl Default for Pyroclasm {
    fn default() -> Self {
        Self {
            active_page: Default::default(),

            toasts: Default::default(),
            modal_quit: None,
        }
    }
}

impl Pyroclasm {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let mut fonts = egui::FontDefinitions::default();

        // fonts.font_data.insert(
        //     "main_font".to_owned(),
        //     egui::FontData::from_static(include_bytes!("../resources/fonts/CuteFont-Regular.ttf")),
        // );

        // fonts.font_data.insert(
        //     "digital".to_owned(),
        //     egui::FontData::from_static(include_bytes!("../resources/fonts/digital-7.ttf")),
        // );

        // fonts
        //     .families
        //     .get_mut(&egui::FontFamily::Proportional)
        //     .unwrap()
        //     .insert(0, "main_font".to_owned());

        // fonts
        //     .families
        //     .get_mut(&egui::FontFamily::Monospace)
        //     .unwrap()
        //     .push("digital".to_owned());

        // cc.egui_ctx.set_fonts(fonts);

        let mut style = (*cc.egui_ctx.style()).clone();

        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(22.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Name("Title".into()),
                egui::FontId::new(28.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Name("MenuButton".into()),
                egui::FontId::new(28.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Name("Context".into()),
                egui::FontId::new(22.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(18.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(10.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();

        style.spacing.item_spacing = egui::vec2(14.0_f32, 14.0_f32);

        // style.explanation_tooltips = true;

        // debugging
        // style.debug.debug_on_hover = true;

        cc.egui_ctx.set_style(style);

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn setup_modals(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut modal = egui_modal::Modal::new(ctx, "quit_dialog");

        modal.show(|ui| {
            modal.title(ui, "Quit");

            modal.frame(ui, |ui| {
                modal.body(ui, "Do you want to quit the Pyroclasm UI?");
            });

            modal.buttons(ui, |ui| {
                modal.icon(ui, egui_modal::Icon::Info);

                // After clicking, the modal is automatically closed
                if modal.button(ui, "Quit").clicked() {
                    frame.set_visible(false);
                    frame.close();
                };

                // After clicking, the modal is automatically closed
                if modal.button(ui, "Cancel").clicked() {};
            });
        });

        self.modal_quit = Some(modal);
    }

    /// Render the window title bar
    fn title_bar(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let global_state = crate::STATE.read();

        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
                ui.with_layout(
                    egui::Layout::left_to_right(egui::Align::Min)
                        .with_cross_align(egui::Align::Center),
                    |ui| {
                        ui.add_space(8.0_f32);
                        ui.label(
                            egui::RichText::new("Pyroclasm UI")
                                .text_style(egui::TextStyle::Name("Title".into())),
                        );
                        ui.add_space(24.0_f32);
                    },
                );

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let mut brightness = global_state.current_brightness.unwrap_or(0) as f32;

                    ui.scope(|ui| {
                        ui.spacing_mut().slider_width = 230.0;

                        egui::Slider::new(&mut brightness, RangeInclusive::new(0.0_f32, 100.0_f32))
                            .integer()
                            .logarithmic(false)
                            .show_value(true)
                            .clamp_to_range(true)
                            .ui(ui)
                            .on_hover_text("Global brightness");
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0_f32, 0.0_f32);
                        ui.spacing_mut().button_padding = egui::vec2(6.0_f32, 6.0_f32);

                        egui::Frame::none()
                            .rounding(6.0_f32)
                            .inner_margin(egui::style::Margin::symmetric(8.0_f32, 6.0_f32))
                            .outer_margin(egui::style::Margin::symmetric(0.0_f32, 8.0_f32))
                            .show(ui, |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("❌")
                                            .color(egui::Color32::RED)
                                            .background_color(egui::Color32::TRANSPARENT)
                                            .text_style(egui::TextStyle::Name("MenuButton".into())),
                                    )
                                    .on_hover_text("Close")
                                    .clicked()
                                {
                                    self.modal_quit.as_ref().map(|f| f.open());
                                };
                            });

                        ui.separator();

                        egui::Frame::none()
                            .rounding(6.0_f32)
                            .inner_margin(egui::style::Margin::symmetric(8.0_f32, 6.0_f32))
                            .outer_margin(egui::style::Margin::symmetric(0.0_f32, 8.0_f32))
                            .show(ui, |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("⚙")
                                            .background_color(egui::Color32::TRANSPARENT)
                                            .text_style(egui::TextStyle::Name("MenuButton".into())),
                                    )
                                    .on_hover_text("Settings")
                                    .clicked()
                                {
                                    self.active_page = TabPages::Settings;
                                };

                                ui.add_space(6.0_f32);

                                ui.menu_button(
                                    egui::RichText::new("☰")
                                        .background_color(egui::Color32::TRANSPARENT)
                                        .text_style(egui::TextStyle::Name("MenuButton".into())),
                                    |ui| {
                                        ui.scope(|ui| {
                                            ui.spacing_mut().item_spacing =
                                                egui::vec2(0.0_f32, 0.0_f32);
                                            ui.spacing_mut().button_padding =
                                                egui::vec2(8.0_f32, 8.0_f32);

                                            if ui.button("Settings...").clicked() {
                                                ui.close_menu();

                                                self.active_page = TabPages::Settings;
                                            }

                                            if ui.button("About...").clicked() {
                                                ui.close_menu();

                                                self.toasts.info("About Pyroclasm UI");

                                                self.active_page = TabPages::About;
                                            }

                                            ui.separator();

                                            if ui.button("Quit").clicked() {
                                                frame.set_visible(false);
                                                frame.close();
                                            }
                                        })
                                    },
                                );
                            });
                    });
                });
            });

            // support dragging the window via the title bar
            let title_bar_rect = ui.max_rect();

            let title_bar_response = ui.interact(
                title_bar_rect,
                egui::Id::new("title_bar"),
                egui::Sense::click(),
            );

            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }
        });
    }

    /// Render the main menu
    fn menu_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // #[cfg(not(target_arch = "wasm32"))]
        // egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
        //     egui::menu::bar(ui, |ui| {
        //         ui.menu_button("App", |ui| {
        //             if ui.button("About").clicked() {
        //                 *active_page = ui::TabPages::About;
        //             }

        //             if ui.button("Quit").clicked() {
        //             }
        //         });
        //     });
        // });
    }

    /// Render the tab pages panel
    fn tab_pages(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        macro_rules! tab_button {
            ($ui: expr, $active_page: expr, $color: expr, $tab_page: expr, $text: expr, $hover: expr) => {
                $ui.spacing_mut().button_padding = egui::vec2(16.0_f32, 16.0_f32);

                if $ui
                    .add(
                        egui::Button::new($text).fill(if *$active_page == $tab_page {
                            $color.additive().linear_multiply(16.0_f32)
                        } else {
                            $color
                        }),
                    )
                    .clicked()
                {
                    *$active_page = $tab_page;
                }
            };
        }

        let active_page = &mut self.active_page;

        egui::TopBottomPanel::top("top_panel")
            .min_height(50.0_f32)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupAbout),
                        ui::TabPages::Start,
                        "Pyroclasm UI",
                        "Welcome Page"
                    );

                    ui.spacing();

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupCanvas),
                        ui::TabPages::Canvas,
                        "Unified Canvas",
                        "Canvas Setup and zone allocation"
                    );

                    ui.spacing();

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupDevices),
                        ui::TabPages::Keyboards,
                        "Keyboards",
                        "Configure keyboard devices"
                    );

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupDevices),
                        ui::TabPages::Mice,
                        "Mice",
                        "Configure Mouse devices"
                    );

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupDevices),
                        ui::TabPages::Misc,
                        "Miscellaneous",
                        "Configure Miscellaneous devices"
                    );

                    ui.spacing();

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupScripting),
                        ui::TabPages::Profiles,
                        "Profiles",
                        "Edit profiles and scripts"
                    );

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupScripting),
                        ui::TabPages::Macros,
                        "Macros",
                        "Assign and edit macros"
                    );

                    ui.spacing();

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupManagement),
                        ui::TabPages::Rules,
                        "Automation Rules",
                        "Configure automation rules"
                    );

                    ui.spacing();

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupManagement),
                        ui::TabPages::ColorSchemes,
                        "Color Schemes",
                        "Manage color schemes"
                    );

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupManagement),
                        ui::TabPages::Settings,
                        "Settings",
                        "Eruption global settings"
                    );

                    ui.spacing();

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupAbout),
                        ui::TabPages::Logs,
                        "Logs",
                        "Show logs"
                    );

                    ui.spacing();

                    tab_button!(
                        ui,
                        active_page,
                        color(Theme::GroupAbout),
                        ui::TabPages::About,
                        "About",
                        "About Eruption and Pyroclasm UI"
                    );

                    ui.spacing();

                    #[cfg(debug_assertions)]
                    {
                        tab_button!(
                            ui,
                            active_page,
                            color(Theme::GroupDebug),
                            ui::TabPages::Debug,
                            "Debug UI",
                            "Debug functionality of the Pyroclasm UI"
                        );

                        ui.spacing();
                    }
                })
            });
    }

    /// Render the footer panel
    fn footer(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let global_state = crate::STATE.read();

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if let Some(active_profile) = &global_state.active_profile {
                    ui.label(active_profile);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }

    /// Render the slot panel
    fn slot_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let global_state = crate::STATE.read();

        egui::TopBottomPanel::bottom("slot_panel").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let empty = vec!["".to_owned(), "".to_owned(), "".to_owned(), "".to_owned()];
                let slot_names = global_state.slot_names.as_ref();
                let slot_names = slot_names.unwrap_or_else(|| &empty);

                let active_slot = global_state.active_slot.unwrap_or(0);

                macro_rules! profile_button {
                    ($slot_index: expr) => {
                        if egui::Frame::none()
                            .fill(color(Theme::GroupProfile))
                            .inner_margin(8.0_f32)
                            .outer_margin(4.0_f32)
                            .shadow(egui::epaint::Shadow::small_dark())
                            .rounding(6.0_f32)
                            .show(ui, |ui| {
                                if ui
                                    .radio(
                                        $slot_index == active_slot,
                                        format!("{}", slot_names[$slot_index]),
                                    )
                                    .changed()
                                {
                                    crate::switch_to_slot($slot_index).unwrap_or_else(|e| {
                                        error!("Could not switch slots: {e}");
                                    });
                                }

                                #[derive(Debug, PartialEq)]
                                enum Enum {
                                    First,
                                    Second,
                                    Third,
                                }
                                let mut selected = Enum::First;

                                egui::ComboBox::from_id_source(format!(
                                    "Profile for slot {}",
                                    $slot_index
                                ))
                                .selected_text(format!("{:?}", &selected))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut selected, Enum::First, "First");
                                    ui.selectable_value(&mut selected, Enum::Second, "Second");
                                    ui.selectable_value(&mut selected, Enum::Third, "Third");
                                });
                            })
                            .response
                            .clicked()
                        {
                            crate::switch_to_slot($slot_index).unwrap_or_else(|e| {
                                error!("Could not switch slots: {e}");
                            });
                        }
                    };
                }

                profile_button!(0);
                profile_button!(1);
                profile_button!(2);
                profile_button!(3);
            });
        });
    }

    /// Render the "special" functions panel
    fn special_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("special_panel").show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::from_main_dir_and_cross_align(
                    egui::Direction::LeftToRight,
                    egui::Align::Center,
                ),
                |ui| {
                    ui.collapsing("Effects", |ui| {
                        let mut ambient_effect = false;
                        let mut audio_effects = false;

                        ui.checkbox(&mut ambient_effect, "Ambient Effect");
                        ui.checkbox(&mut audio_effects, "Audio Effects");
                    });
                },
            );
        });
    }
}

enum Theme {
    GroupCanvas,
    GroupDevices,
    GroupScripting,
    GroupManagement,
    GroupAbout,
    GroupDebug,

    //
    GroupProfile,
}

fn color(item: Theme) -> egui::Color32 {
    match item {
        Theme::GroupCanvas => egui::Color32::DARK_RED,
        Theme::GroupDevices => egui::Color32::DARK_GREEN,
        Theme::GroupScripting => egui::Color32::DARK_BLUE,
        Theme::GroupManagement => egui::Color32::YELLOW,
        Theme::GroupAbout => egui::Color32::LIGHT_YELLOW,
        Theme::GroupDebug => egui::Color32::BLACK,

        //
        Theme::GroupProfile => egui::Color32::DARK_GRAY,
    }
    .to_opaque()
    .linear_multiply(0.075_f32)
}

impl eframe::App for Pyroclasm {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Rgba::TRANSPARENT
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // let Self {
        //     active_page,
        //     toasts,
        // } = self;

        // frame.set_decorations(false);

        // extra functionality
        self.toasts.show(ctx);

        self.setup_modals(ctx, frame);

        // render title area
        self.title_bar(ctx, frame);
        self.menu_panel(ctx, frame);

        // render client area
        self.tab_pages(ctx, frame);

        self.footer(ctx, frame);
        self.slot_panel(ctx, frame);
        self.special_panel(ctx, frame);

        match self.active_page {
            ui::TabPages::Start => {
                let mut page = ui::start::StartPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Canvas => {
                let mut page = ui::canvas::CanvasPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Keyboards => {
                let mut page = ui::keyboards::KeyboardsPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Mice => {
                let mut page = ui::mice::MicePage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Misc => {
                let mut page = ui::misc::MiscPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Profiles => {
                let mut page = ui::profiles::ProfilesPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Macros => {
                let mut page = ui::macros::MacrosPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Rules => {
                let mut page = ui::rules::RulesPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::ColorSchemes => {
                let mut page = ui::color_schemes::ColorSchemesPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Settings => {
                let mut page = ui::settings::SettingsPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::About => {
                let mut page = ui::about::AboutPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Logs => {
                let mut page = ui::logs::LogsPage::new();

                page.update(ctx, frame)
            }

            ui::TabPages::Debug => {
                let mut page = ui::debug::DebugPage::new();

                page.update(ctx, frame)
            }
        }
    }
}
