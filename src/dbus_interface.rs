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

#[cfg(feature = "dbus")]
use dbus::{
    ffidisp::BusType, ffidisp::Connection, ffidisp::NameFlag, tree::EmitsChangedSignal,
    tree::Factory,
};
use failure::Fail;
use log::*;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::constants;

/// D-Bus messages and signals that are processed by the main thread
#[derive(Debug, Clone)]
pub enum Message {
    LoadScript(PathBuf),
}

pub type Result<T> = std::result::Result<T, DbusApiError>;

#[derive(Debug, Fail)]
pub enum DbusApiError {
    #[fail(display = "D-Bus not connected")]
    BusNotConnected {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

/// D-Bus API support
#[cfg(feature = "dbus")]
pub struct DbusApi {
    connection: Option<Connection>,
}

#[cfg(feature = "dbus")]
impl DbusApi {
    /// Initialize the D-Bus API
    pub fn new(dbus_tx: Sender<Message>) -> Self {
        let c = Connection::get_private(BusType::System).unwrap();
        c.register_name("org.eruption.control", NameFlag::ReplaceExisting as u32)
            .unwrap();

        let f = Factory::new_fn::<()>();
        let tree = f
            .tree(())
            .add(
                f.object_path("/status", ()).introspectable().add(
                    f.interface("org.eruption.control.Status", ()).add_p(
                        f.property::<bool, _>("Active", ())
                            .emits_changed(EmitsChangedSignal::False)
                            .on_get(|i, _m| {
                                i.append(true);
                                Ok(())
                            })
                            .on_set(|i, _m| {
                                let _b: bool = i.read()?;
                                Ok(())
                            }),
                    ),
                ),
            )
            .add(
                f.object_path("/script", ()).introspectable().add(
                    f.interface("org.eruption.control.Script", ()).add_p(
                         f.property::<&str, _>("Name", ())
                            .emits_changed(EmitsChangedSignal::False)
                            .on_get(|i, _m| {
                                let result = "test";
                                i.append(result);
                                Ok(())
                            })
                            // .on_set(
                            //  |i, m| {
                            //     let b: bool = try!(i.read());
                            //     i.append(result);
                            //     Ok(())
                            // }   
                            // ),
                    ),
                ),
            )
            .add(
                f.object_path("/script", ()).introspectable().add(
                    f.interface("org.eruption.control.Script", ()).add_m(
                        f.method("ChangeScript", (), move |m| {
                            let n: &str = m.msg.read1()?;
                            // let result = DbusApi::load_script(n);

                            dbus_tx
                                .send(Message::LoadScript(PathBuf::from(n)))
                                .unwrap_or_else(|e| {
                                    error!("Could not send a pending D-Bus event: {}", e)
                                });

                            let s = true;
                            Ok(vec![m.msg.method_return().append1(s)])
                        })
                        .inarg::<&str, _>("filename")
                        .outarg::<bool, _>("status"),
                    ),
                ),
            );

        tree.set_registered(&c, true).unwrap();
        c.add_handler(tree);

        DbusApi {
            connection: Some(c),
        }
    }

    /// Get the next event from D-Bus
    pub fn get_next_event(&self) -> Result<()> {
        match self.connection {
            Some(ref connection) => {
                if let Some(item) = connection.incoming(constants::DBUS_TIMEOUT_MILLIS).next() {
                    trace!("Message: {:?}", item);

                    Ok(())
                } else {
                    trace!("Received a timeout message");

                    Ok(())
                }
            }

            None => Err(DbusApiError::BusNotConnected {}),
        }
    }
}

/// Initialize the Eruption D-Bus API support
#[cfg(feature = "dbus")]
pub fn initialize(dbus_tx: Sender<Message>) -> Result<DbusApi> {
    Ok(DbusApi::new(dbus_tx))
}

/// An empty dummy struct
#[cfg(not(feature = "dbus"))]
pub struct DbusApi {}

/// Get an empty dummy implementation of DbusApi
#[cfg(not(feature = "dbus"))]
pub fn initialize_dummy() -> DbusApi {
    DbusApi {}
}
