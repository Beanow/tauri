#!/usr/bin/env bash
# Copyright 2019-2021 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

echo "Building API definitions..."
cd tooling/api
yarn && yarn build
cd ../..

echo "Building the Tauri Rust CLI..."
cd tooling/cli
cargo install --path .
cd ../..
echo "Tauri Rust CLI installed. Run it with '$ cargo tauri [COMMAND]'."

function node_cli() {
    cd tooling/cli/node
    yarn && yarn build && yarn link
    cd ../../..
    echo "Tauri Node.js CLI installed. use `yarn link @tauri-apps/cli` and run it with '$ yarn tauri [COMMAND]'."
}

if [ -z ${INSTALL_NODE_CLI} ]; then
    echo "Do you want to install the Node.js CLI?"
    select yn in "Yes" "No"; do
        case $yn in
            Yes )
                node_cli
                break;;
            No ) break;;
        esac
    done
elif [ ${INSTALL_NODE_CLI} = 1 ]; then
    node_cli;
fi
