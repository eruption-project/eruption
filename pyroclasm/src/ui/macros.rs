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

use crate::highlighting;

#[derive(Default)]
pub struct MacrosPage {}

impl MacrosPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Macros");

            let code =
                std::fs::read_to_string("/usr/share/eruption/scripts/lib/macros/user-macros.lua")
                    .unwrap();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    show_code(ui, "lua", &code);
                });
        });
    }
}

fn show_code(ui: &mut egui::Ui, lang: &str, code: &str) {
    let code = remove_leading_indentation(code.trim_start_matches('\n'));
    highlighting::code_editor_ui(ui, &lang, &code);
}

fn remove_leading_indentation(code: &str) -> String {
    fn is_indent(c: &u8) -> bool {
        matches!(*c, b' ' | b'\t')
    }

    let first_line_indent = code.bytes().take_while(is_indent).count();

    let mut out = String::new();

    let mut code = code;
    while !code.is_empty() {
        let indent = code.bytes().take_while(is_indent).count();
        let start = first_line_indent.min(indent);
        let end = code
            .find('\n')
            .map_or_else(|| code.len(), |endline| endline + 1);
        out += &code[start..end];
        code = &code[end..];
    }
    out
}
