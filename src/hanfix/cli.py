"""hanfix: fix text typed in the wrong keyboard layout (Korean 2-set ↔ QWERTY).

    hanfix dkssudgktpdy      -> 안녕하세요
    hanfix ㅣㄴ -ㅁ           -> ls -a
    echo 햣 | hanfix          -> git

Direction is detected per run: input containing Hangul is turned back into QWERTY keys,
anything else is composed into Hangul. Force it with --to ko / --to en.

    hanfix --init powershell | bash    -> shell function `hf` (alias `ㅗㄹ`) that fixes and re-runs
                                          the previous command, like thefuck
"""

from __future__ import annotations

import sys

# 두벌식 (KS X 5002) key -> jamo; shifted keys that are not listed fall back to the lower-case key
KEY2JAMO = dict(zip("rRseEfaqQtTdwWczxvgkoiOjpuPhynbml",
                    "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎㅏㅐㅑㅒㅓㅔㅕㅖㅗㅛㅜㅠㅡㅣ"))
JAMO2KEY = {j: k for k, j in KEY2JAMO.items()}

CHO = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"
JUNG = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ"
JONG = ["", *"ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ"]
VOWEL_PAIRS = {("ㅗ", "ㅏ"): "ㅘ", ("ㅗ", "ㅐ"): "ㅙ", ("ㅗ", "ㅣ"): "ㅚ", ("ㅜ", "ㅓ"): "ㅝ",
               ("ㅜ", "ㅔ"): "ㅞ", ("ㅜ", "ㅣ"): "ㅟ", ("ㅡ", "ㅣ"): "ㅢ"}
FINAL_PAIRS = {("ㄱ", "ㅅ"): "ㄳ", ("ㄴ", "ㅈ"): "ㄵ", ("ㄴ", "ㅎ"): "ㄶ", ("ㄹ", "ㄱ"): "ㄺ",
               ("ㄹ", "ㅁ"): "ㄻ", ("ㄹ", "ㅂ"): "ㄼ", ("ㄹ", "ㅅ"): "ㄽ", ("ㄹ", "ㅌ"): "ㄾ",
               ("ㄹ", "ㅍ"): "ㄿ", ("ㄹ", "ㅎ"): "ㅀ", ("ㅂ", "ㅅ"): "ㅄ"}
SPLIT = {v: k for k, v in {**VOWEL_PAIRS, **FINAL_PAIRS}.items()}
BASE, NO_FINAL = 0xAC00, {"ㄸ", "ㅃ", "ㅉ"}     # these never close a syllable


def is_hangul(ch: str) -> bool:
    return "가" <= ch <= "힣" or "ㄱ" <= ch <= "ㅣ"


def compose(keys: str) -> str:
    """QWERTY keystrokes -> Hangul, with the usual 2-set automaton (final consonant moves on)."""
    out: list[str] = []
    cho = jung = jong = ""

    def flush():
        nonlocal cho, jung, jong
        if cho and jung:
            out.append(chr(BASE + (CHO.index(cho) * 21 + JUNG.index(jung)) * 28 + JONG.index(jong)))
        else:
            out.append(cho + jung)
        cho = jung = jong = ""

    for k in keys:
        j = KEY2JAMO.get(k) or KEY2JAMO.get(k.lower())
        if j is None:                               # not a letter key: pass through
            flush()
            out.append(k)
            continue
        if j in CHO:                                # consonant
            if jung and cho and not jong and j not in NO_FINAL:
                jong = j
            elif jong and (jong, j) in FINAL_PAIRS:
                jong = FINAL_PAIRS[(jong, j)]
            else:
                flush()
                cho = j
        else:                                       # vowel
            if jong:                                # the last final consonant starts the next syllable
                first, moved = SPLIT.get(jong, ("", jong))
                jong = first
                flush()
                cho, jung = moved, j
            elif jung and (jung, j) in VOWEL_PAIRS:
                jung = VOWEL_PAIRS[(jung, j)]
            elif cho and not jung:
                jung = j
            else:
                flush()
                jung = j
    flush()
    return "".join(out)


def decompose(text: str) -> str:
    """Hangul -> the QWERTY keystrokes that type it."""
    def keys(jamo: str) -> str:
        return "".join(JAMO2KEY[x] for x in SPLIT.get(jamo, (jamo,)))

    out = []
    for ch in text:
        if "가" <= ch <= "힣":
            n = ord(ch) - BASE
            out.append(keys(CHO[n // 588]) + keys(JUNG[n // 28 % 21]) + (keys(JONG[n % 28]) if n % 28 else ""))
        elif "ㄱ" <= ch <= "ㅣ":
            out.append(keys(ch))
        else:
            out.append(ch)
    return "".join(out)


def fix(text: str, to: str | None = None) -> str:
    to = to or ("en" if any(is_hangul(c) for c in text) else "ko")
    return decompose(text) if to == "en" else compose(text)


# Shell functions that fix and re-run the previous command, the way `thefuck` does. Both names are
# defined: `hf`, and `ㅗㄹ`, which is what typing "hf" gives when the Korean layout is still on.
INIT = {
    "powershell": r"""
function global:hf {
    $last = (Get-History -Count 1).CommandLine
    if (-not $last) { Write-Host 'hanfix: no previous command'; return }
    $fixed = (hanfix --to en $last | Out-String).TrimEnd()
    if ($fixed -eq $last) { Write-Host 'hanfix: nothing to fix'; return }
    Write-Host "$fixed  [Enter / Ctrl+C]" -ForegroundColor Green -NoNewline
    $null = [Console]::ReadLine()
    try { [Microsoft.PowerShell.PSConsoleReadLine]::AddToHistory($fixed) } catch {}
    Invoke-Expression $fixed
}
Set-Alias -Name 'ㅗㄹ' -Value hf -Scope Global
""",
    "bash": r"""
hf() {
    local last fixed
    last=$(fc -ln -2 -2 2>/dev/null | sed 's/^[[:space:]]*//')
    [ -z "$last" ] && { echo 'hanfix: no previous command'; return 1; }
    fixed=$(hanfix --to en "$last")
    [ "$fixed" = "$last" ] && { echo 'hanfix: nothing to fix'; return 1; }
    printf '\033[32m%s\033[0m  [Enter / Ctrl+C]' "$fixed"; read -r _
    history -s "$fixed"
    eval "$fixed"
}
alias ㅗㄹ=hf
""",
}


def main(argv: list[str] | None = None) -> int:
    # Only a leading --to / --init / -h are options: everything else is text, including "-ㅁ" (= "-a").
    args = list(sys.argv[1:] if argv is None else argv)
    to = None
    if args[:1] in (["-h"], ["--help"]):
        print(__doc__)
        return 0
    if args[:1] == ["--init"]:
        shell = args[1] if len(args) > 1 else ""
        if shell not in INIT:
            sys.stderr.write(f"hanfix --init: choose one of {', '.join(INIT)}\n")
            return 2
        sys.stdout.write(INIT[shell].lstrip())
        return 0
    if args[:1] == ["--to"] and len(args) > 1 and args[1] in ("ko", "en"):
        to, args = args[1], args[2:]
    text = " ".join(args) if args else sys.stdin.read().rstrip("\n")
    sys.stdout.write(fix(text, to) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
