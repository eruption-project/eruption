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
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;

// type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum AboutError {
//     #[error("Unknown error: {description}")]
//     UnknownError { description: String },
// }

/// Shows the about dialog
pub fn show_about_dialog<W: IsA<gtk::Window>>(parent: &W) {
    let builder = gtk::Builder::from_resource("/org/eruption/eruption-gui/ui/about.glade");
    let about_dialog: gtk::AboutDialog = builder.get_object("about_dialog").unwrap();

    about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));

    // place logo
    let logo_pixbuf = Pixbuf::from_resource_at_scale(
        "/org/eruption/eruption-gui/img/eruption-gui.png",
        100,
        100,
        true,
    )
    .unwrap();
    about_dialog.set_logo(Some(&logo_pixbuf));

    about_dialog.set_transient_for(Some(parent));
    about_dialog.set_modal(true);

    about_dialog.run();
    about_dialog.hide();
}
