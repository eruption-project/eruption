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
*/

// use eyre;
// use gtk::prelude::*;

// type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum AboutError {
//     #[error("Unknown error: {description}")]
//     UnknownError { description: String },
// }

// Shows the rule dialog
// pub fn show_rule_dialog<W: IsA<gtk::Window>>(parent: &W) {
//     let builder = gtk::Builder::from_resource("/org/eruption/eruption-gui/ui/rule.glade");
//     let rule_dialog: gtk::Dialog = builder.get_object("rule_dialog").unwrap();

//     rule_dialog.set_transient_for(Some(parent));
//     rule_dialog.set_modal(true);

//     rule_dialog.run();
//     rule_dialog.hide();
// }
