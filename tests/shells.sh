#!/bin/sh
# Shell integration: after a mistyped command, `hf -y` must run the fixed one, and a second call must
# find nothing to fix. Runs each shell interactively in a pseudo-terminal (util-linux `script`).
set -u
export LANG=C.UTF-8 LC_ALL=C.UTF-8
fail=0
esc=$(printf '\033')
for sh in bash zsh fish; do
    command -v "$sh" >/dev/null 2>&1 || { echo "skip $sh (not installed)"; continue; }
    case $sh in
        bash) init='eval "$(hanfix --init bash)"'; run='bash --norc -i' ;;
        zsh) init='eval "$(hanfix --init zsh)"'; run='zsh -f -i' ;;
        fish) init='hanfix --init fish | source'; run='fish -N -i' ;;
    esac
    home=$(mktemp -d)
    # bash and zsh also run with a stand-in for codigest's __cdg_run, defined after hanfix
    cdg=
    [ "$sh" = fish ] || cdg='__cdg_run() { echo "cdg_run:$1"; eval "$1"; }; echo "runners:$CODIGEST_RUNNERS"'
    # One line per second: fish discards typed-ahead input after a failed command.
    out=$(for line in "$init" "$cdg" 'ㄷ초ㅐ hanfix-ok' 'hf -y' 'ㅗㄹ -y' exit; do printf '%s\n' "$line"; sleep 1; done |
        HOME=$home timeout 30 script -qec "$run" /dev/null 2>&1 | tr -d '\r\007' | sed "s/$esc[]P][^$esc]*$esc[\\]//g; s/$esc\[[0-9;?>]*[A-Za-z]//g; s/$esc[()][A-Z0-9]//g; s/$esc[=>]//g")
    # fish keeps no history entry for the fixed command, so its second call runs the same fix again
    if [ "$sh" = fish ]; then second='[ "$(printf "%s\n" "$out" | grep -cx hanfix-ok)" -eq 2 ]'
    else second='printf "%s\n" "$out" | grep -q "nothing to fix" && printf "%s\n" "$out" | grep -qx "runners: hf ㅗㄹ" &&
        printf "%s\n" "$out" | grep -qx "cdg_run:echo hanfix-ok"'; fi
    if printf '%s\n' "$out" | grep -qx 'hanfix-ok' && eval "$second"; then
        echo "ok   $sh"
    else
        echo "FAIL $sh"; printf '%s\n' "$out" | sed 's/^/    /'; fail=1
    fi
done
if command -v fish >/dev/null 2>&1; then  # the alias on its own, without a terminal
    if fish -N -c 'hanfix --init fish | source; functions -q ㅗㄹ'; then echo "ok   fish alias"
    else echo "FAIL fish alias"; fail=1; fi
fi
exit $fail
