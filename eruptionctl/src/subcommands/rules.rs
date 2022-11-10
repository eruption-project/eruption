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

use colored::*;
use std::fmt;

use indexmap::IndexMap;

use crate::{dbus_client::dbus_session_bus, translations::tr};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("Parse error: {description}")]
    Parse { description: String },
}

/// Sub-commands of the "rules" command
#[derive(Debug, clap::Parser)]
pub enum RulesSubcommands {
    /// List all available rules
    #[clap(display_order = 0, about(tr!("rules-list")))]
    List,

    /// Add a new rule
    #[clap(display_order = 1, about(tr!("rules-add")))]
    Add { rule: Vec<String> },

    /// Remove a rule by its index
    #[clap(display_order = 2, about(tr!("rules-remove")))]
    Remove { rule_index: usize },

    /// Mark a rule as enabled
    #[clap(display_order = 3, about(tr!("rules-enable")))]
    Enable { rule_index: usize },

    /// Mark a rule as disabled
    #[clap(display_order = 4, about(tr!("rules-disable")))]
    Disable { rule_index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WindowFocusedSelectorMode {
    WindowName,
    WindowInstance,
    WindowClass,
}

impl fmt::Display for WindowFocusedSelectorMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WindowFocusedSelectorMode::WindowName => {
                write!(f, "Name")?;
            }

            WindowFocusedSelectorMode::WindowInstance => {
                write!(f, "Instance")?;
            }

            WindowFocusedSelectorMode::WindowClass => {
                write!(f, "Class")?;
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selector {
    ProcessExec {
        comm: String,
    },
    WindowFocused {
        mode: WindowFocusedSelectorMode,
        regex: String,
    },
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Selector::ProcessExec { comm } => {
                write!(
                    f,
                    "On process execution: comm: '{}'",
                    comm.to_string().bold()
                )?;
            }

            Selector::WindowFocused { mode, regex } => {
                write!(
                    f,
                    "On window focused: {}: '{}'",
                    mode,
                    regex.to_string().bold()
                )?;
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    SwitchToProfile { profile_name: String },
    SwitchToSlot { slot_index: u64 },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::SwitchToProfile { profile_name } => {
                write!(f, "Switch to profile: {}", profile_name.to_string().bold())?;
            }

            Action::SwitchToSlot { slot_index } => {
                write!(
                    f,
                    "Switch to slot: {}",
                    format!("{}", slot_index + 1).bold()
                )?;
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// Specifies whether the rule is enabled
    pub enabled: bool,

    /// Set to true if the rule is auto-generated
    pub internal: bool,
}

impl std::default::Default for RuleMetadata {
    fn default() -> Self {
        RuleMetadata {
            enabled: true,
            internal: false,
        }
    }
}

impl fmt::Display for RuleMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "enabled: {}", self.enabled)?;
        write!(f, ", internal: {}", self.internal)?;

        Ok(())
    }
}

fn print_usage_examples() {
    println!(
        r#"
Please see below for some examples:

Process:
rules add exec <comm> [<profile-name.profile>|<slot number>]

rules add exec gnome-calc.* /var/lib/eruption/profiles/profile1.profile
rules add exec gnome-calc.* 2


Window:
rules add window-[class|instance|name] <regex> [<profile-name.profile>|<slot number>]

rules add window-name '.*YouTube.*Mozilla Firefox' /var/lib/eruption/profiles/profile1.profile
rules add window-instance gnome-calculator 2

You may want to use the command line tool `xprop` to find the relevant information
"#
    );
}

pub async fn handle_command(command: RulesSubcommands) -> Result<()> {
    match command {
        RulesSubcommands::List => list_command().await,
        RulesSubcommands::Add { rule } => add_command(&rule).await,
        RulesSubcommands::Remove { rule_index } => remove_command(rule_index).await,
        RulesSubcommands::Enable { rule_index } => enable_command(rule_index).await,
        RulesSubcommands::Disable { rule_index } => disable_command(rule_index).await,
    }
}

async fn list_command() -> Result<()> {
    let rules = enum_rules().await?;

    for (index, (selector, (metadata, action))) in rules.iter().enumerate() {
        if metadata.internal {
            // render internal rules as italic text
            let text = format!("{:3}: {} => {} ({})", index, selector, action, metadata).italic();
            println!("{}", text);
        } else if !metadata.enabled {
            // render disabled rules as dimmed text
            let text = format!("{:3}: {} => {} ({})", index, selector, action, metadata).dimmed();
            println!("{}", text);
        } else {
            println!("{:3}: {} => {} ({})", index, selector, action, metadata);
        }
    }

    Ok(())
}

async fn add_command(rule: &[String]) -> Result<()> {
    if rule.len() != 3 {
        eprintln!("Malformed rule definition");
        print_usage_examples();
    } else {
        let sensor = rule[0].to_owned();
        let selector = rule[1].to_owned();
        let action = rule[2].to_owned();
        let metadata = RuleMetadata {
            enabled: true,
            internal: false,
        }
        .to_string();

        let (new_selector, _new_metadata, mut new_action) =
            parse_rule(&(sensor, selector, action, metadata))?;

        // slot indices are 0-based
        if let Action::SwitchToSlot { slot_index } = new_action {
            new_action = Action::SwitchToSlot {
                slot_index: slot_index.saturating_sub(1),
            };
        }

        let rules = enum_rules().await?;
        let new_rule_index = rules.len().saturating_sub(1);

        let mut result: IndexMap<Selector, (RuleMetadata, Action)> = IndexMap::new();

        for (index, (selector, (metadata, action))) in rules.iter().enumerate() {
            if index == new_rule_index {
                if !metadata.internal {
                    result.insert(selector.clone(), (metadata.clone(), action.clone()));
                }

                let new_metadata = RuleMetadata {
                    enabled: true,
                    internal: false,
                };

                result.insert(
                    new_selector.clone(),
                    (new_metadata.clone(), new_action.clone()),
                );
            } else if !metadata.internal {
                result.insert(selector.clone(), (metadata.clone(), action.clone()));
            }
        }

        set_rules(&result).await?;
    }

    Ok(())
}

async fn remove_command(rule_index: usize) -> Result<()> {
    let rules = enum_rules().await?;
    let mut result = IndexMap::new();

    for (index, (selector, (metadata, action))) in rules.iter().enumerate() {
        if index == rule_index || metadata.internal {
            continue;
        } else if !metadata.internal {
            result.insert(selector.clone(), (metadata.clone(), action.clone()));
        }
    }

    set_rules(&result).await?;

    Ok(())
}

async fn enable_command(rule_index: usize) -> Result<()> {
    let rules = enum_rules().await?;
    let mut result = IndexMap::new();

    for (index, (selector, (metadata, action))) in rules.iter().enumerate() {
        if index == rule_index && !metadata.internal {
            let metadata = RuleMetadata {
                enabled: true,
                ..metadata.clone()
            };

            result.insert(selector.clone(), (metadata.clone(), action.clone()));
        } else if !metadata.internal {
            result.insert(selector.clone(), (metadata.clone(), action.clone()));
        }
    }

    set_rules(&result).await?;

    Ok(())
}

async fn disable_command(rule_index: usize) -> Result<()> {
    let rules = enum_rules().await?;
    let mut result = IndexMap::new();

    for (index, (selector, (metadata, action))) in rules.iter().enumerate() {
        if index == rule_index && !metadata.internal {
            let metadata = RuleMetadata {
                enabled: false,
                ..metadata.clone()
            };

            result.insert(selector.clone(), (metadata.clone(), action.clone()));
        } else if !metadata.internal {
            result.insert(selector.clone(), (metadata.clone(), action.clone()));
        }
    }

    set_rules(&result).await?;

    Ok(())
}

async fn enum_rules() -> Result<IndexMap<Selector, (RuleMetadata, Action)>> {
    let (result,): (Vec<(String, String, String, String)>,) = dbus_session_bus(
        "org.eruption.process_monitor",
        "/org/eruption/process_monitor/rules",
    )
    .await?
    .method_call("org.eruption.process_monitor.Rules", "EnumRules", ())
    .await?;

    let rules = parse_rules(&result)?;

    Ok(rules)
}

async fn set_rules(rules: &IndexMap<Selector, (RuleMetadata, Action)>) -> Result<()> {
    let mut generated_rules: Vec<(String, String, String, String)> = Vec::new();

    for (_index, (selector, (metadata, action))) in rules.iter().enumerate() {
        let (sensor, selector) = match selector {
            Selector::ProcessExec { comm } => ("exec".to_string(), comm.to_owned()),

            Selector::WindowFocused { mode, regex } => match mode {
                WindowFocusedSelectorMode::WindowClass => {
                    ("window-class".to_string(), regex.to_owned())
                }

                WindowFocusedSelectorMode::WindowInstance => {
                    ("window-instance".to_string(), regex.to_owned())
                }

                WindowFocusedSelectorMode::WindowName => {
                    ("window-name".to_string(), regex.to_owned())
                }
            },
        };

        let action = match action {
            Action::SwitchToProfile { profile_name } => profile_name.to_owned(),
            Action::SwitchToSlot { slot_index } => format!("{slot_index}"),
        };

        let metadata = format!(
            "({}, user-defined)",
            if metadata.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );

        generated_rules.push((sensor, selector, action, metadata));
    }

    dbus_session_bus(
        "org.eruption.process_monitor",
        "/org/eruption/process_monitor/rules",
    )
    .await?
    .method_call(
        "org.eruption.process_monitor.Rules",
        "SetRules",
        (generated_rules,),
    )
    .await?;

    Ok(())
}

fn parse_rule(rule: &(String, String, String, String)) -> Result<(Selector, RuleMetadata, Action)> {
    let sensor = &rule.0;
    let selector = &rule.1;
    let action = &rule.2;
    let metadata = &rule.3;

    let mut parsed_selector = None;
    let parsed_action;

    // parse metadata
    let enabled = metadata.contains("enabled");

    let internal = metadata.contains("internal");

    let parsed_metadata = RuleMetadata { enabled, internal };

    // parse sensor and selector
    if sensor.contains("exec") {
        parsed_selector = Some(Selector::ProcessExec {
            comm: selector.to_owned(),
        });
    } else if sensor.contains("window-class") {
        parsed_selector = Some(Selector::WindowFocused {
            mode: WindowFocusedSelectorMode::WindowClass,
            regex: selector.to_owned(),
        });
    } else if sensor.contains("window-instance") {
        parsed_selector = Some(Selector::WindowFocused {
            mode: WindowFocusedSelectorMode::WindowInstance,
            regex: selector.to_owned(),
        });
    } else if sensor.contains("window-name") {
        parsed_selector = Some(Selector::WindowFocused {
            mode: WindowFocusedSelectorMode::WindowName,
            regex: selector.to_owned(),
        });
    }

    // parse action
    if parsed_selector.is_none() {
        Err(RuleError::Parse {
            description: "Syntax error in selector".to_owned(),
        }
        .into())
    } else if action.contains(".profile") {
        parsed_action = Action::SwitchToProfile {
            profile_name: action.to_owned(),
        };

        Ok((parsed_selector.unwrap(), parsed_metadata, parsed_action))
    } else {
        parsed_action = Action::SwitchToSlot {
            slot_index: action.parse::<u64>()?,
        };

        Ok((parsed_selector.unwrap(), parsed_metadata, parsed_action))
    }
}

fn parse_rules(
    rules: &[(String, String, String, String)],
) -> Result<IndexMap<Selector, (RuleMetadata, Action)>> {
    let mut result = IndexMap::new();

    for rule in rules {
        let (selector, rule_metadata, action) = parse_rule(rule)?;
        result.insert(selector, (rule_metadata, action));
    }

    Ok(result)
}
