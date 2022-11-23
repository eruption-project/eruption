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

use egui::{CentralPanel, CollapsingHeader};

#[derive(Default)]
pub struct DebugPage {}

impl DebugPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Debug Pyroclasm UI");

                    CollapsingHeader::new("Settings").show(ui, |ui| {
                        ctx.settings_ui(ui);
                    });

                    CollapsingHeader::new("Textures").show(ui, |ui| {
                        ctx.texture_ui(ui);
                    });

                    CollapsingHeader::new("Memory").show(ui, |ui| {
                        ctx.memory_ui(ui);
                    });

                    CollapsingHeader::new("Inspection").show(ui, |ui| {
                        ctx.inspection_ui(ui);
                    });
                });
            });
        });
    }
}
