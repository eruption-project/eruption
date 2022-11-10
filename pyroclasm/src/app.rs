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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct Pyroclasm {}

impl Pyroclasm {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "main_font".to_owned(),
            egui::FontData::from_static(include_bytes!("../resources/fonts/CuteFont-Regular.ttf")),
        );

        // fonts.font_data.insert(
        //     "digital".to_owned(),
        //     egui::FontData::from_static(include_bytes!("../resources/fonts/digital-7.ttf")),
        // );

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "main_font".to_owned());

        // fonts
        //     .families
        //     .get_mut(&egui::FontFamily::Monospace)
        //     .unwrap()
        //     .push("digital".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        let mut style = (*cc.egui_ctx.style()).clone();

        style.spacing.item_spacing = egui::vec2(22.0, 30.0);
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(54.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Name("Context".into()),
                egui::FontId::new(33.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(28.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(24.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(24.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(20.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();

        cc.egui_ctx.set_style(style);

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Pyroclasm {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // let Self {} = self;

        #[cfg(not(target_arch = "wasm32"))]
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Pyroclasm UI");
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.heading("Slots");
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Side Panel");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pyroclasm UI for Eruption");
        });
    }
}
