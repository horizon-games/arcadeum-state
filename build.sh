#!/usr/bin/env bash

set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

export CARGO_INCREMENTAL=0

bold=$(tput bold)
normal=$(tput sgr0)

# Save current directory.
pushd . >/dev/null

for SRC in runtime/wasm
do
  echo "${bold}Building webassembly binary in $SRC...${normal}"
  cd "$PROJECT_ROOT/$SRC"

  ./build.sh

  cd - >> /dev/null
done

# Restore initial directory.
popd >/dev/null

echo "${bold}Building substrate node...${normal}"
cargo build --release

echo "${bold}Building bindings...${normal}"
(cd bindings && yarn && yarn build)

echo "${bold}Building ttt...${normal}"
(cd examples/ttt/src-ts && yarn && yarn build)
(cd examples/ttt/test && yarn && yarn build)

echo "${bold}Building coin...${normal}"
(cd examples/coin/src-ts && yarn && yarn build)
