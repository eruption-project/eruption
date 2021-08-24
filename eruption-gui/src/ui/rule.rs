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
use glib::clone;
use gtk::prelude::*;

// type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum RuleError {
//     #[error("Unknown error: {description}")]
//     UnknownError { description: String },
// }

#[derive(Debug, Clone)]
pub struct Rule {
    pub index: Option<usize>,
    pub enabled: bool,
    pub sensor: String,
    pub selector: String,
    pub action: String,
    pub metadata: String,
}

impl Rule {
    pub fn new(
        index: Option<usize>,
        enabled: bool,
        sensor: String,
        selector: String,
        action: String,
        metadata: String,
    ) -> Self {
        Self {
            index,
            enabled,
            sensor,
            selector,
            action,
            metadata,
        }
    }
}

/// Shows the "new rule" dialog
pub fn show_new_rule_dialog<W: IsA<gtk::Window>>(parent: &W) -> (gtk::ResponseType, Option<Rule>) {
    let builder = gtk::Builder::from_resource("/org/eruption/eruption-gui/ui/rule.glade");

    let rule_dialog: gtk::Dialog = builder.object("rule_dialog").unwrap();

    let ok_button: gtk::Button = builder.object("ok_button").unwrap();
    let cancel_button: gtk::Button = builder.object("cancel_button").unwrap();

    let rule_enabled: gtk::CheckButton = builder.object("rule_enabled").unwrap();
    let sensor: gtk::ComboBox = builder.object("sensor").unwrap();
    let selector: gtk::Entry = builder.object("selector").unwrap();
    let action: gtk::Entry = builder.object("action").unwrap();

    ok_button.connect_clicked(clone!(@weak rule_dialog => move |_b| {
        rule_dialog.response(gtk::ResponseType::Ok);
    }));

    cancel_button.connect_clicked(clone!(@weak rule_dialog => move |_b| {
        rule_dialog.response(gtk::ResponseType::Cancel);
    }));

    rule_dialog.set_default_response(gtk::ResponseType::Cancel);
    rule_dialog.set_transient_for(Some(parent));
    rule_dialog.set_modal(true);

    rule_dialog.connect_response(|dialog, _response| {
        dialog.close();
    });

    rule_dialog.show_all();

    rule_enabled.set_active(true);
    sensor.set_active(Some(0));
    selector.set_text("");
    action.set_text("");

    let response = rule_dialog.run();

    let result = Some(Rule::new(
        None,
        rule_enabled.is_active(),
        sensor.active_id().unwrap().to_string(),
        selector.text().to_string(),
        action.text().to_string(),
        "".to_string(),
    ));

    (response, result)
}

/// Shows the "edit rule" dialog
pub fn show_edit_rule_dialog<W: IsA<gtk::Window>>(
    parent: &W,
    rule: &Rule,
) -> (gtk::ResponseType, Option<Rule>) {
    let builder = gtk::Builder::from_resource("/org/eruption/eruption-gui/ui/rule.glade");

    let rule_dialog: gtk::Dialog = builder.object("rule_dialog").unwrap();

    let ok_button: gtk::Button = builder.object("ok_button").unwrap();
    let cancel_button: gtk::Button = builder.object("cancel_button").unwrap();

    let rule_enabled: gtk::CheckButton = builder.object("rule_enabled").unwrap();
    let sensor: gtk::ComboBox = builder.object("sensor").unwrap();
    let selector: gtk::Entry = builder.object("selector").unwrap();
    let action: gtk::Entry = builder.object("action").unwrap();

    ok_button.connect_clicked(clone!(@weak rule_dialog => move |_b| {
        rule_dialog.response(gtk::ResponseType::Ok);
    }));

    cancel_button.connect_clicked(clone!(@weak rule_dialog => move |_b| {
        rule_dialog.response(gtk::ResponseType::Cancel);
    }));

    rule_dialog.set_default_response(gtk::ResponseType::Cancel);
    rule_dialog.set_transient_for(Some(parent));
    rule_dialog.set_modal(true);

    rule_dialog.connect_response(|dialog, _response| {
        dialog.close();
    });

    rule_dialog.show_all();

    rule_enabled.set_active(rule.enabled);
    sensor.set_active_id(Some(&rule.sensor));
    selector.set_text(&rule.selector);
    action.set_text(&rule.action);

    let response = rule_dialog.run();

    let result = Some(Rule::new(
        rule.index,
        rule_enabled.is_active(),
        sensor.active_id().unwrap().to_string(),
        selector.text().to_string(),
        action.text().to_string(),
        "".to_string(),
    ));

    (response, result)
}
