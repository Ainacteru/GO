#!/usr/bin/env bash
#
# cargo runner for the GO flight computer (see .cargo/config.toml).
#
#   1. converts the ELF cargo just built into a UF2
#   2. asks the board to jump to the UF2 bootloader (if it is in app mode)
#   3. mounts the bootloader volume and copies the UF2 over
#   4. streams defmt logs in a tmux session attached to this terminal
#
# Env overrides: UF2CONV, OBJCOPY, GO_WAIT_TIMEOUT (seconds to wait for the board)
#
set -euo pipefail

ELF="${1:?cargo runner called without a binary path}"
ELF="$(readlink -f "$ELF")"
SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# --- config -----------------------------------------------------------------
APP_ID="16c0:27dd"      # USB id while running the firmware (CDC ACM)
BOOT_ID="6577:474f"     # USB id while in the UF2 bootloader
VOL_LABEL="GROSSBOOT"   # label of the bootloader's mass-storage volume
UF2_FAMILY="0x68ed2b88" # SAMD21
UF2_OFFSET="0x2000"     # application start, just past the bootloader
SESSION="go-defmt"      # tmux session holding the log stream

OBJCOPY="${OBJCOPY:-arm-none-eabi-objcopy}"
UF2CONV="${UF2CONV:-$HOME/Projects/Embedded/bootloader/microsoft-uf2/utils/uf2conv.py}"
BOARD_TIMEOUT="${GO_WAIT_TIMEOUT:-60}"

OUT_DIR="$PROJECT_DIR/output"
BIN="$OUT_DIR/out.bin"
UF2="$OUT_DIR/out.uf2"
LOGFILE="$OUT_DIR/defmt.log"
LOGGER_SH="$OUT_DIR/defmt-logger.sh"

# --- helpers ----------------------------------------------------------------
die() { echo "run: $*" >&2; exit 1; }

have_usb() { lsusb | rg -q "$1"; }

first_tty() { compgen -G "/dev/ttyACM*" | head -n1 || true; }

# wait_usb <vid:pid regex> <timeout seconds> <description>
wait_usb() {
    local id="$1" timeout="$2" what="$3" deadline
    deadline=$(( SECONDS + timeout ))
    until have_usb "$id"; do
        (( SECONDS < deadline )) || die "timed out after ${timeout}s waiting for $what"
        sleep 0.2
    done
}

# defmt-print opens the port with TIOCEXCL, so it must let go before we can
# write 'bootloader' to the same tty.
stop_logger() {
    tmux has-session -t "$SESSION" 2>/dev/null || return 0
    tmux send-keys -t "$SESSION:defmt" C-c 2>/dev/null || true
}

# wait_port_free <tty> <timeout seconds>
wait_port_free() {
    local deadline=$(( SECONDS + $2 ))
    until { : >"$1"; } 2>/dev/null; do
        (( SECONDS < deadline )) || return 1
        sleep 0.2
    done
}

for tool in "$OBJCOPY" lsusb lsblk udisksctl findmnt rg defmt-print tmux; do
    command -v "$tool" >/dev/null 2>&1 || die "missing required tool: $tool"
done
[[ -r "$UF2CONV" ]] || die "uf2conv.py not found at $UF2CONV (set UF2CONV=...)"

if [[ "$ELF" == */debug/* ]]; then
    echo "run: warning: flashing a debug build; use 'cargo run --release'" >&2
    echo "     if it overflows flash or runs too slowly." >&2
fi

# --- 1. build a uf2 from the elf cargo just produced ------------------------
# Always rebuild from "$ELF" so we can never flash a stale image and then
# decode its logs against a newer symbol table.
mkdir -p "$OUT_DIR"
"$OBJCOPY" -O binary "$ELF" "$BIN"
"$UF2CONV" "$BIN" -b "$UF2_OFFSET" -f "$UF2_FAMILY" -o "$UF2" >/dev/null
echo "run: $(basename "$ELF") -> $(basename "$UF2") ($(stat -c%s "$BIN") bytes)"

# --- 2. get the board into the bootloader -----------------------------------
have_usb "$APP_ID|$BOOT_ID" || echo "run: waiting for board..."
wait_usb "$APP_ID|$BOOT_ID" "$BOARD_TIMEOUT" "board to be plugged in"

if have_usb "$APP_ID"; then
    tty="$(first_tty)"
    [[ -n "$tty" ]] || die "board is in app mode but no /dev/ttyACM* exists"

    stop_logger   # release the port before writing to it
    wait_port_free "$tty" 5 || die "$tty is held by another process (defmt-print? screen?)"

    echo "run: app mode on $tty, requesting bootloader"
    deadline=$(( SECONDS + 10 ))
    while have_usb "$APP_ID"; do
        (( SECONDS < deadline )) || die "board ignored 'bootloader'; double-tap reset to enter it manually"
        # 2>/dev/null first: redirections apply left to right, so this also
        # swallows the shell's own "Device or resource busy" on open
        printf 'bootloader\n' 2>/dev/null >"$tty" || true
        sleep 0.5
    done
fi

wait_usb "$BOOT_ID" 15 "bootloader to enumerate"
echo "run: in bootloader"

# --- 3. mount the uf2 volume and upload -------------------------------------
dev=""
deadline=$(( SECONDS + 15 ))
while :; do
    # '|| true': pipefail + set -e would otherwise kill us silently on any
    # non-zero exit inside these pipelines (findmnt returns 1 when unmounted).
    dev="$(lsblk -o PATH,LABEL | awk -v l="$VOL_LABEL" '$2 == l { print $1; exit }' || true)"
    [[ -n "$dev" ]] && break
    (( SECONDS < deadline )) || die "no volume labelled $VOL_LABEL appeared"
    sleep 0.3
done

mnt="$(findmnt -n -o TARGET --first-only --source "$dev" || true)"
if [[ -z "$mnt" ]]; then
    udisksctl mount -b "$dev" >/dev/null 2>&1 || true   # may already be automounted
    mnt="$(findmnt -n -o TARGET --first-only --source "$dev" || true)"
fi
[[ -n "$mnt" ]] || die "could not mount $dev"

echo "run: uploading to $mnt"
# The bootloader resets the instant the last block lands, so cp/sync often
# report an I/O error on close. Success is judged by the board coming back.
cp "$UF2" "$mnt/" || true
sync || true

# --- 4. stream defmt logs in tmux -------------------------------------------
wait_usb "$APP_ID" 30 "board to re-enumerate after flashing"
sleep 0.5   # give udev a moment to create the CDC ACM node
tty="$(first_tty)"
[[ -n "$tty" ]] || die "no /dev/ttyACM* after flashing"
echo "run: flashed, logging from $tty"

# Generated so tmux never has to re-quote anything; also handy to run by hand.
# No 'exec bash' here: the pane runs a persistent interactive shell instead, so
# C-c kills only defmt-print and can never take the pane (and session) down.
elf_q="$(printf '%q' "$ELF")"
tty_q="$(printf '%q' "$tty")"
log_q="$(printf '%q' "$LOGFILE")"
cat > "$LOGGER_SH" <<EOF
#!/usr/bin/env bash
# generated by $SCRIPT_DIR/run.sh - do not edit
#
# stdout is a pipe (tee), which makes defmt-print drop its colours, so force
# them back on and strip the escapes again on the way into the log file.
export CLICOLOR_FORCE=1
# C-c stops the logger and closes the whole session, dropping you back to the
# terminal you ran cargo from. The pane scrollback goes with it; $LOGFILE keeps
# the full history. Use C-b d instead to leave the logger running.
trap 'tmux kill-session -t $SESSION 2>/dev/null; exit 130' INT
defmt-print -e $elf_q serial --path $tty_q 2>&1 | tee >(sed -u 's/\x1b\[[0-9;]*m//g' >> $log_q)
echo "[logger stopped - cargo run to reflash, C-b d to detach]"
EOF
chmod +x "$LOGGER_SH"

if ! tmux has-session -t "$SESSION" 2>/dev/null; then
    # history-limit is fixed when a pane is created and the server does not
    # exist yet, hence the throwaway first window.
    tmux new-session -d -s "$SESSION" -n boot -x 200 -y 50 'sleep 86400'
    tmux set-option -g history-limit 100000 >/dev/null
    tmux new-window -t "$SESSION" -n defmt      # plain interactive shell
    tmux kill-window -t "$SESSION:boot"
    tmux set-option -t "$SESSION" mouse on >/dev/null
    sleep 0.5                                   # let the shell come up
elif ! tmux list-windows -t "$SESSION" -F '#{window_name}' | rg -qx defmt; then
    tmux new-window -t "$SESSION" -n defmt
    sleep 0.5
else
    # Reuse the window so earlier runs stay in the scrollback. Stop the old
    # logger first: two defmt-print instances cannot share the port.
    tmux send-keys -t "$SESSION:defmt" C-c
    sleep 0.3
fi

tmux send-keys -t "$SESSION:defmt" "$(printf '%q' "$LOGGER_SH")" C-m

if [[ -n "${TMUX:-}" ]]; then
    exec tmux switch-client -t "$SESSION"    # already inside tmux, do not nest
elif [[ -t 0 && -t 1 ]]; then
    exec tmux attach-session -t "$SESSION"   # take over this terminal
else
    echo "run: no tty; attach with: tmux attach -t $SESSION" >&2
fi
