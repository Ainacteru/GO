#!/usr/bin/env bash
#
# Flash-only cargo runner: identical to tools/run.sh (build UF2 -> bootloader ->
# mount -> upload) but it does not start a defmt log session.
#
# Used by 'cargo flash', which is an alias in .cargo/config.toml that points the
# target runner here instead of at run.sh.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
exec "$SCRIPT_DIR/run.sh" --no-logs "$@"
