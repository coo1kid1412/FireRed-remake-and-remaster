#!/bin/sh
set -eu
cd "$(dirname "$0")"

revision=037335f4c725d7c9aecdac87066f2002b4bd7e14
reference=reference/pokefirered
if [ ! -d "$reference/.git" ]; then
    git init -q "$reference"
fi
if ! git -C "$reference" rev-parse --verify "$revision^{commit}" >/dev/null 2>&1; then
    git -C "$reference" fetch --depth=1 https://github.com/pret/pokefirered.git "$revision"
fi
git -C "$reference" checkout --quiet --detach "$revision"

rom=../FireRedLeafGreenRecomp/variants/firered/roms/firered_usa.gba
if [ -f "$rom" ]; then
    python3 tools/import_world.py
else
    python3 tools/import_world.py --source-only
fi
python3 tools/import_core.py
python3 tools/import_field.py
python3 tools/import_actors.py
python3 tools/import_encounters.py
python3 tools/import_battle.py
python3 tools/check_battle_source.py

target=x86_64-apple-darwin
cargo +stable run --locked --target "$target" --no-default-features --bin firered-core-check
cargo +stable test --locked --target "$target" --no-default-features --lib
cargo +stable build --locked --target "$target" --bin firered-native-prototype
