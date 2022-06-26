// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(target_os = "linux")]

//! Tauri utility helpers
#![warn(missing_docs, rust_2018_idioms)]

use std::path::Path;

#[cfg(target_os = "linux")]
use glib::{Error as GError, KeyFile, KeyFileFlags};

/// Defines the `/.flatpak-info` path used for detecting Flatpak sandbox information.
/// Note: This path should not be configable at runtime. The `/` directory is not normally user-writable.
/// Giving more (but not complete) confidence we're really in a Flatpak sandbox.
pub static WELL_KNOWN_PATH: &'static str = "/.flatpak-info";

/// Holds information about the Flatpak sandbox we're currently running in.
/// Corresponds to information taken from the `/.flatpak-info` file.
#[derive(Debug, Clone)]
pub struct FlatpakInfo {
  /// The reverse DNS application name the Flatpak is installed as.
  /// Example `app.tauri.api-example`
  /// Maps to `[Application] name`.
  pub application_name: String,

  /// The current runtime the Flatpak is using.
  /// Example `runtime/org.gnome.Platform/x86_64/42`.
  /// Maps to `[Application] runtime`.
  pub application_runtime: String,

  /// Architecture for this Flatpak instance.
  /// Example `x86_64`.
  /// Maps to `[Instance] arch`.
  pub instance_arch: String,

  /// Branch this Flatpak instance.
  /// Example `stable`.
  /// Maps to `[Instance] branch`.
  pub instance_branch: String,
}

impl FlatpakInfo {
  /// The identifier triple for this Flatpak.
  /// Example `app.tauri.api-example/x86_64/stable`.
  pub fn get_identifier_triple(&self) -> String {
    format!(
      "{}/{}/{}",
      self.application_name, self.instance_arch, self.instance_branch
    )
  }

  /// Attempts to find FlatpakInfo through a `/.flatpak-info` file.
  /// Returning `Ok(None)` if the file does not exist.
  /// `Ok(Some(FlatpakInfo))` when it does exist.
  /// Or `Err(glib::Error)` on IO/parsing errors.
  pub fn try_load() -> Result<Option<Self>, GError> {
    let path = Path::new(WELL_KNOWN_PATH);
    if !path.exists() {
      return Ok(None);
    }

    Ok(Some(Self::try_load_from_file(&path)?))
  }

  fn try_load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, GError> {
    let keyfile = KeyFile::new();
    keyfile.load_from_file(&path, KeyFileFlags::empty())?;
    Ok(FlatpakInfo {
      application_name: keyfile.string("Application", "name")?.to_string(),
      application_runtime: keyfile.string("Application", "runtime")?.to_string(),
      instance_arch: keyfile.string("Instance", "arch")?.to_string(),
      instance_branch: keyfile.string("Instance", "branch")?.to_string(),
    })
  }
}

#[test]
fn flatpak_info_try_load_from_file() {
  let path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .to_path_buf()
    .join("test")
    .join("fixtures")
    .join(".flatpak-info");

  let info = FlatpakInfo::try_load_from_file(&path).unwrap();

  assert_eq!("app.tauri.tauri-util-tests", info.application_name);
  assert_eq!(
    "runtime/org.gnome.Platform/x86_64/42",
    info.application_runtime
  );
  assert_eq!("x86_64", info.instance_arch);
  assert_eq!("dev", info.instance_branch);
  assert_eq!(
    "app.tauri.tauri-util-tests/x86_64/dev".to_string(),
    info.get_identifier_triple()
  );
}
