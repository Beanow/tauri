// Copyright 2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  super::common::CommandExt,
  debian::{generate_desktop_file, generate_icon_files},
};
use crate::Settings;

use anyhow::Context;
use handlebars::Handlebars;
use log::info;
use serde::Serialize;

use std::{fs, path::PathBuf, process::Command};

#[derive(Serialize)]
struct ManifestMap {
  app_id: String,
  app_name: String,
  main_binary: String,
  deb_package_name: String,
  project_out_directory: String,
  cargo_cache_dir: String,
  yarn_cache_dir: String,
  target_cache_dir: String,
  workdir: String,
  local_dir: String,
  rel_app_dir: String,
  rel_tauri_dir: String,
  rel_project_out_directory: String,
  binary: Vec<String>,
  skip_list: Vec<String>,
  use_node_cli: bool,
  tauri_cli_version: String,
}

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // TODO: Sidecar/external binaries? (These might be working already)
  // TODO: Don't hardcode source path (in the manifest)
  // TODO: Don't hardcode `src-tauri` (in the manifest)
  // TODO: metainfo file? (we should have most of the necessary data for a basic one)
  // TODO: Don't write desktop file and icons to `flatpak/usr`
  // TODO: Allow specifying extra permissions in config

  //
  // Step 1a: Generate the Flapak manifest file
  //

  let workdir = &settings.flatpak().workdir;
  let skip_list = &settings.flatpak().skip_list;
  let rel_project_out_directory = settings
    .project_out_directory()
    .strip_prefix(&workdir)?
    .to_path_buf();

  info!(action = "Flatpak workdir"; "Your workdir is {}", &workdir.to_string_lossy().to_string());

  // Location for build artifacts (Flatpak manifest and bundle)
  let output_dir = settings.project_out_directory().join("bundle/flatpak");
  // Workspace for local (.flatpak) builds.
  let local_dir = output_dir.join("local");
  let local_build_dir = output_dir.join("local_build");

  // Location for build files (Flatpak repo, build directory, etc.)
  // let build_dir = settings
  //   .project_out_directory()
  //   .join("bundle/flatpak_build");

  // Location of Flatpak manifest file
  let manifest_path = local_dir.join(format!("{}.json", settings.bundle_identifier()));

  // Location of flatpak-cargo-generator.py script
  // let flatpak_cargo_generator_path = build_dir.join("flatpak-cargo-generator.py");

  // Location of Flatpak repository
  let flatpak_repository_dir = output_dir.join("repo");

  // Name of Flatpak single-file bundle
  let bundle_name = format!("{}.flatpak", settings.bundle_identifier());

  // Location of Flatpak single-file bundle
  let flatpak_bundle_path = output_dir.join(&bundle_name);

  // Place we can store caches for our build process
  let cache_dir = output_dir.join(".cache");
  let cargo_cache_dir = cache_dir.join("cargo");
  let yarn_cache_dir = cache_dir.join("yarn");
  let target_cache_dir = cache_dir.join("target");

  // Start with clean build and output directories
  if local_dir.exists() {
    fs::remove_dir_all(&local_dir)?;
  }
  if local_build_dir.exists() {
    fs::remove_dir_all(&local_build_dir)?;
  }
  // fs::create_dir_all(&output_dir)?;
  fs::create_dir_all(&local_dir)?;
  fs::create_dir_all(&cargo_cache_dir)?;
  fs::create_dir_all(&yarn_cache_dir)?;
  fs::create_dir_all(&target_cache_dir)?;
  // fs::create_dir_all(&build_dir)?;

  // Generate the desktop and icon files on the host
  // It's easier to do it here than in the sandbox
  generate_desktop_file(&settings, &local_dir)?;
  generate_icon_files(&settings, &local_dir)?;

  // Get the name of each binary that should be installed
  let mut binary_installs = Vec::new();
  for binary in settings.binaries() {
    binary_installs.push(binary.name().to_string());
  }

  for external_binary in settings.external_binaries() {
    dbg!(&external_binary);
  }

  // Create the map for the manifest template file
  let arch = match settings.binary_arch() {
    "x86" => "i386",
    "x86_64" => "amd64",
    other => other,
  };

  let data = ManifestMap {
    project_out_directory: settings.project_out_directory().to_string_lossy().into(),
    app_id: settings.bundle_identifier().to_string(),
    app_name: settings.product_name().to_string(),
    main_binary: settings.main_binary_name().to_string(),
    cargo_cache_dir: cargo_cache_dir.to_string_lossy().into(),
    yarn_cache_dir: yarn_cache_dir.to_string_lossy().into(),
    target_cache_dir: target_cache_dir.to_string_lossy().into(),
    local_dir: local_dir.to_string_lossy().into(),
    rel_app_dir: settings.flatpak().rel_app_dir.to_string_lossy().into(),
    rel_tauri_dir: settings.flatpak().rel_tauri_dir.to_string_lossy().into(),
    rel_project_out_directory: rel_project_out_directory.to_string_lossy().into(),
    workdir: workdir.to_string_lossy().into(),
    deb_package_name: format!(
      "{}_{}_{}",
      settings.main_binary_name(),
      settings.version_string(),
      arch
    ),
    skip_list: skip_list
      .iter()
      .map(|p| p.to_string_lossy().to_string())
      .collect(),
    binary: binary_installs,
    use_node_cli: settings.flatpak().use_node_cli,
    tauri_cli_version: settings.flatpak().tauri_cli_version.to_string(),
  };

  // Render the manifest template
  let mut handlebars = Handlebars::new();
  handlebars
    .register_template_string("flatpak", include_str!("flatpak/manifest-template"))
    .expect("Failed to register template for handlebars");
  handlebars.set_strict_mode(true);
  let manifest = handlebars.render("flatpak", &data)?;

  info!(action = "Bundling"; "{} ({})", bundle_name, manifest_path.display());

  // Write the manifest
  fs::write(&manifest_path, manifest)?;

  //
  // Step 1b: Generate the Cargo sources file for the manifest
  //

  // fs::write(
  //   &flatpak_cargo_generator_path,
  //   include_str!("flatpak/flatpak-cargo-generator.py"),
  // )?;

  // Include submodule with common packages.
  Command::new("git")
    .arg("clone")
    .arg("https://github.com/flathub/shared-modules.git")
    .current_dir(&local_dir)
    .output_ok()
    .context("failed to generate Cargo sources file for Flatpak manifest")?;

  //
  // Step 2: Build the Flatpak to a temporary repository
  //

  Command::new("flatpak-builder")
    .arg("--user")
    .arg(format!("--install-deps-from={}", "flathub")) //TODO: configureable remote
    .arg(format!(
      "--state-dir={}/.flatpak-builder",
      &output_dir.display()
    ))
    .arg(format!("--repo={}", &flatpak_repository_dir.display()))
    .arg(&local_build_dir)
    .arg(&manifest_path)
    .output_ok()
    .context("failed to build Flatpak")?;

  //
  // Step 3: Export the Flatpak bundle from the temporary repository
  //

  Command::new("flatpak")
    .arg("build-bundle")
    .arg(&flatpak_repository_dir)
    .arg(&flatpak_bundle_path)
    .arg(&settings.bundle_identifier())
    .output_ok()
    .context("failed to export Flatpak bundle")?;

  Ok(vec![flatpak_bundle_path])
}
