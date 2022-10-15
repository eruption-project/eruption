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


use color_eyre::Help;
use colored::Colorize;
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use eyre::Context;

use crate::dbus_client::dbus_system_bus;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "canvas" command
#[derive(Debug, clap::Parser)]
pub enum CanvasSubcommands {
    /// Get or set the global 'hue' adjustment during canvas post-processing
    Hue { hue: Option<f64> },

    /// Get or set the global 'saturation' adjustment during canvas post-processing
    Saturation { saturation: Option<f64> },

    /// Get or set the global 'lightness' adjustment during canvas post-processing
    Lightness { lightness: Option<f64> },
}

pub async fn handle_command(command: CanvasSubcommands) -> Result<()> {
    match command {
        CanvasSubcommands::Hue { hue } => hue_command(hue).await?,

        CanvasSubcommands::Saturation { saturation } => saturation_command(saturation).await?,

        CanvasSubcommands::Lightness { lightness } => lightness_command(lightness).await?,
    }

    Ok(())
}

async fn hue_command(hue: Option<f64>) -> Result<()> {
    if let Some(hue) = hue {
        set_canvas_hue(hue)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    } else {
        let result = get_canvas_hue()
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
        println!("{}", format!("Hue: {}", format!("{}", result).bold()));
    }

    Ok(())
}

async fn saturation_command(saturation: Option<f64>) -> Result<()> {
    if let Some(saturation) = saturation {
        set_canvas_saturation(saturation)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    } else {
        let result = get_canvas_saturation()
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
        println!(
            "{}",
            format!("Saturation: {}", format!("{}", result).bold())
        );
    }

    Ok(())
}

async fn lightness_command(lightness: Option<f64>) -> Result<()> {
    if let Some(lightness) = lightness {
        set_canvas_lightness(lightness)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    } else {
        let result = get_canvas_lightness()
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
        println!("{}", format!("Lightness: {}", format!("{}", result).bold()));
    }

    Ok(())
}

/// Get the current canvas hue value
pub async fn get_canvas_hue() -> Result<f64> {
    let result = dbus_system_bus("/org/eruption/canvas")
        .await?
        .get("org.eruption.Canvas", "Hue")
        .await?;

    Ok(result)
}

/// Set the current canvas hue value
pub async fn set_canvas_hue(value: f64) -> Result<()> {
    let arg = Box::new(value);

    dbus_system_bus("/org/eruption/canvas")
        .await?
        .set("org.eruption.Canvas", "Hue", arg)
        .await?;

    Ok(())
}

/// Get the current canvas saturation value
pub async fn get_canvas_saturation() -> Result<f64> {
    let result = dbus_system_bus("/org/eruption/canvas")
        .await?
        .get("org.eruption.Canvas", "Saturation")
        .await?;

    Ok(result)
}

/// Set the current canvas saturation value
pub async fn set_canvas_saturation(value: f64) -> Result<()> {
    let arg = Box::new(value);

    dbus_system_bus("/org/eruption/canvas")
        .await?
        .set("org.eruption.Canvas", "Saturation", arg)
        .await?;

    Ok(())
}

/// Get the current canvas lightness value
pub async fn get_canvas_lightness() -> Result<f64> {
    let result = dbus_system_bus("/org/eruption/canvas")
        .await?
        .get("org.eruption.Canvas", "Lightness")
        .await?;

    Ok(result)
}

/// Set the current canvas lightness value
pub async fn set_canvas_lightness(value: f64) -> Result<()> {
    let arg = Box::new(value);

    dbus_system_bus("/org/eruption/canvas")
        .await?
        .set("org.eruption.Canvas", "Lightness", arg)
        .await?;

    Ok(())
}
