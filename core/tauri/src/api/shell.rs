// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to shell.

use crate::ShellScope;
use std::str::FromStr;

/// Program or Protocol to use on the [`open()`] call.
pub enum OpenWith {
  /// Program to use on the [`open()`] call.
  Program(Program),
  /// Protocol to use on the [`open()`] call.
  Protocol(Protocol),
}

/// Protocol to use on the [`open()`] call.
pub enum Protocol {
  /// Use the Portal API on the D-Bus session bus.
  XdgDesktopPortal(Option<XdgDesktopPortalOptions>),
}

/// Options for Portal API based [`open()`] calls.
pub enum XdgDesktopPortalOptions {
  /// Specify the "ask" option. Asks the user to choose an app.
  /// If this is not passed, the portal may use a default or pick the last choice.
  Ask,
}

/// Program to use on the [`open()`] call.
pub enum Program {
  /// Use the `open` program.
  Open,
  /// Use the `start` program.
  Start,
  /// Use the `xdg-open` program.
  XdgOpen,
  /// Use the `gio` program.
  Gio,
  /// Use the `gnome-open` program.
  GnomeOpen,
  /// Use the `kde-open` program.
  KdeOpen,
  /// Use the `wslview` program.
  WslView,
  /// Use the `Firefox` program.
  Firefox,
  /// Use the `Google Chrome` program.
  Chrome,
  /// Use the `Chromium` program.
  Chromium,
  /// Use the `Safari` program.
  Safari,
}

impl From<Program> for OpenWith {
  fn from(program: Program) -> Self {
    OpenWith::Program(program)
  }
}

impl From<Protocol> for OpenWith {
  fn from(protocol: Protocol) -> Self {
    OpenWith::Protocol(protocol)
  }
}

impl FromStr for OpenWith {
  type Err = super::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let p = match s.to_lowercase().as_str() {
      "open" => Self::Program(Program::Open),
      "start" => Self::Program(Program::Start),
      "xdg-open" => Self::Program(Program::XdgOpen),
      "xdg-desktop-portal" => Self::Protocol(Protocol::XdgDesktopPortal(None)),
      "gio" => Self::Program(Program::Gio),
      "gnome-open" => Self::Program(Program::GnomeOpen),
      "kde-open" => Self::Program(Program::KdeOpen),
      "wslview" => Self::Program(Program::WslView),
      "firefox" => Self::Program(Program::Firefox),
      "chrome" | "google chrome" => Self::Program(Program::Chrome),
      "chromium" => Self::Program(Program::Chromium),
      "safari" => Self::Program(Program::Safari),
      _ => return Err(super::Error::UnknownProgramName(s.to_string())),
    };
    Ok(p)
  }
}

impl Program {
  pub(crate) fn name(self) -> &'static str {
    match self {
      Self::Open => "open",
      Self::Start => "start",
      Self::XdgOpen => "xdg-open",

      Self::Gio => "gio",
      Self::GnomeOpen => "gnome-open",
      Self::KdeOpen => "kde-open",
      Self::WslView => "wslview",

      #[cfg(target_os = "macos")]
      Self::Firefox => "Firefox",
      #[cfg(not(target_os = "macos"))]
      Self::Firefox => "firefox",

      #[cfg(target_os = "macos")]
      Self::Chrome => "Google Chrome",
      #[cfg(not(target_os = "macos"))]
      Self::Chrome => "google-chrome",

      #[cfg(target_os = "macos")]
      Self::Chromium => "Chromium",
      #[cfg(not(target_os = "macos"))]
      Self::Chromium => "chromium",

      #[cfg(target_os = "macos")]
      Self::Safari => "Safari",
      #[cfg(not(target_os = "macos"))]
      Self::Safari => "safari",
    }
  }
}

/// Opens path or URL with the program specified in `with`, or system default if `None`.
///
/// The path will be matched against the shell open validation regex, defaulting to `^https?://`.
/// A custom validation regex may be supplied in the config in `tauri > allowlist > scope > open`.
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::{api::shell::open, Manager};
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default browser
///     open(&app.shell_scope(), "https://github.com/tauri-apps/tauri", None)?;
///     Ok(())
///   });
/// ```
pub fn open<P: AsRef<str>, W: Into<OpenWith>>(
  scope: &ShellScope,
  path: P,
  with: Option<W>,
) -> crate::api::Result<()> {
  scope
    .open(path.as_ref(), with.map(Into::into))
    .map_err(|err| crate::api::Error::Shell(format!("failed to open: {}", err)))
}
