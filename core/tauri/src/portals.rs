// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(feature = "shell-open-api")]

use crate::api::shell::XdgDesktopPortalOptions;
use crate::scope::ShellScopeError;

// See https://flatpak.github.io/xdg-desktop-portal/

const NAMESPACE: &'static str = "org.freedesktop.portal.Desktop";
const OBJECT_PATH: &'static str = "/org/freedesktop/portal/desktop";

// TODO: check for file:// paths which is not allowed in OpenURI. Should be using OpenFile.
pub(crate) fn portal_open_uri(
  path: &str,
  options: Option<XdgDesktopPortalOptions>,
) -> Result<(), ShellScopeError> {
  let con = dbus::blocking::Connection::new_session().unwrap();

  let mut msg = dbus::Message::new_method_call(
    NAMESPACE,
    OBJECT_PATH,
    "org.freedesktop.portal.OpenURI", // interface
    "OpenURI",                        // member
  )
  .unwrap();

  msg.append_items(&[
    "".into(), // parent_window handle
    path.into(), // uri
    dbus::arg::messageitem::MessageItem::Array(
      dbus::arg::messageitem::MessageItemArray::new(vec![], "a{sv}".into()).unwrap(),
    ),
  ]);

  dbus::blocking::BlockingSender::send_with_reply_and_block(
    &con,
    msg,
    std::time::Duration::new(5, 0),
  )
  .unwrap();

  Ok(())
}
