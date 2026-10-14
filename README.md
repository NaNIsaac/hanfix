# hanfix

한/영 전환을 깜빡하고 친 글자를 되돌립니다. Fixes text typed in the wrong keyboard layout
(Korean 2-set ↔ QWERTY).

```
hanfix ㅣㄴ -ㅁ               ->  ls -a
hanfix 햣 ㄴㅅㅁ션             ->  git status
hanfix dkssudgktpdy          ->  안녕하세요
echo ㅊㅇ .. | hanfix         ->  cd ..
hanfix --to ko git           ->  햣        (force the direction)
```

방향은 자동으로 정합니다. 한글이 들어 있으면 영문 키로, 없으면 한글로 바꿉니다.
The direction is automatic: input with Hangul becomes QWERTY keys, anything else becomes Hangul.

## 설치 / Install

```
uv tool install -e D:/03_Develop/07_Util/hanfix
```

The only requirement is Python 3.10+ (standard library only).

## 단축키로 쓰기 / Shell key binding

지금 입력 중인 줄을 바로 고칩니다. Fix the line you are typing, in place.

**PowerShell** (`$PROFILE`에 추가 / add to your profile):

```powershell
Set-PSReadLineKeyHandler -Chord 'Ctrl+Alt+h' -BriefDescription hanfix -ScriptBlock {
    $line = $null; $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
    $fixed = (hanfix $line | Out-String).TrimEnd()
    [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
    [Microsoft.PowerShell.PSConsoleReadLine]::Insert($fixed)
}
```

**bash** (`~/.bashrc`):

```bash
bind -x '"\eh": READLINE_LINE=$(hanfix "$READLINE_LINE"); READLINE_POINT=${#READLINE_LINE}'
```

## 동작 / How it works

The tool uses the standard 2-set (두벌식) automaton:
- double vowels (ㅘ, ㅢ) and double finals (ㄳ, ㄺ, ㅄ) are combined;
- a final consonant moves to the next syllable when a vowel follows (`rkrk` → 가가, `rkqtdl` → 값이);
- syllables are composed with Unicode arithmetic, `0xAC00 + (cho·21 + jung)·28 + jong`.

Characters that are not on the letter keys pass through unchanged.

```
python tests/test_hanfix.py
```
