# hanfix

한/영 전환을 깜빡하고 친 글자를 원래 의도한 글자로 되돌리는 명령줄 도구입니다.

```
$ hanfix 햣 ㄴㅅㅁ션
git status

$ hanfix dkssudgktpdy
안녕하세요
```

- 터미널에서 `ls -a`를 치려다 `ㅣㄴ -ㅁ`이 나온 경우, 반대로 메신저에 `dkssud`이 찍힌 경우를 고칩니다.
- 표준 두벌식(KS X 5002) 자판과 QWERTY 자판 사이의 변환입니다.
- Python 표준 라이브러리만 씁니다. 의존성이 없습니다.

## 설치

[uv](https://docs.astral.sh/uv/)를 쓰는 경우:

```
uv tool install git+https://github.com/NaNIsaac/hanfix
```

pip를 쓰는 경우:

```
pip install git+https://github.com/NaNIsaac/hanfix
```

Python 3.10 이상이 필요합니다. 설치하면 `hanfix` 명령이 생깁니다.

## 사용법

```
hanfix ㅣㄴ -ㅁ               ->  ls -a
hanfix 햣 ㄴㅅㅁ션             ->  git status
hanfix dkssudgktpdy          ->  안녕하세요
echo ㅊㅇ .. | hanfix         ->  cd ..          (표준 입력도 받음)
hanfix --to ko git           ->  햣             (방향 지정)
hanfix --to en 한글           ->  gksrmf
```

**방향은 자동으로 정합니다.**
- 입력에 한글이 하나라도 있으면 영문 키 입력으로 되돌립니다(`ko → en`).
- 한글이 없으면 한글로 조합합니다(`en → ko`).
- 자동 판단이 틀리면 맨 앞에 `--to ko` 또는 `--to en`을 붙여 방향을 정하세요.

**옵션은 맨 앞의 `--to`와 `-h`/`--help`만 받습니다.** 나머지는 전부 변환할 글자로 봅니다.
그래서 `hanfix ㅣㄴ -ㅁ`의 `-ㅁ`도 옵션이 아니라 `-a`로 바뀝니다.

## 단축키로 쓰기

지금 입력하고 있는 줄을 그 자리에서 고치도록 단축키를 걸 수 있습니다.
아래 설정은 예시이고, 직접 동작을 확인하지는 못했습니다. 다른 단축키와 겹치면 키를 바꾸세요.

**PowerShell** — `notepad $PROFILE`로 프로필을 열고 추가합니다. `Ctrl+Alt+h`를 누르면 줄 전체가 바뀝니다.

```powershell
Set-PSReadLineKeyHandler -Chord 'Ctrl+Alt+h' -BriefDescription hanfix -ScriptBlock {
    $line = $null; $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
    $fixed = (hanfix $line | Out-String).TrimEnd()
    [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
    [Microsoft.PowerShell.PSConsoleReadLine]::Insert($fixed)
}
```

**bash** — `~/.bashrc`에 추가합니다. `Alt+h`를 누르면 줄 전체가 바뀝니다.

```bash
bind -x '"\eh": READLINE_LINE=$(hanfix "$READLINE_LINE"); READLINE_POINT=${#READLINE_LINE}'
```

## 동작 원리

**영문 → 한글**은 실제 두벌식 입력기와 같은 오토마타로 조합합니다.
- 키마다 자모를 찾습니다(`r` → ㄱ, `k` → ㅏ). Shift 키는 쌍자음과 ㅒ, ㅖ만 따로 처리하고, 나머지 대문자는 소문자와 같게 봅니다.
- 겹모음(ㅘ, ㅢ 등)과 겹받침(ㄳ, ㄺ, ㅄ 등)을 합칩니다.
- 받침 뒤에 모음이 오면 받침을 다음 글자로 넘깁니다.
  - `rkrk` → 가가
  - `rkqtdl` → 값이: 겹받침은 뒤쪽 자음만 넘어갑니다.
- ㄸ, ㅃ, ㅉ은 받침이 될 수 없으므로 항상 새 글자를 시작합니다.
- 음절은 유니코드 공식으로 만듭니다: `0xAC00 + (초성·21 + 중성)·28 + 종성`.

**한글 → 영문**은 위 공식을 거꾸로 풀어 초성·중성·종성을 꺼낸 뒤, 겹자모를 나눠 키로 바꿉니다.
낱자모(ㅣ, ㄴ 등)도 바뀝니다.

자판 글자가 아닌 문자(숫자, 공백, 기호)는 그대로 둡니다.

## 한계

- **한 줄에 한 방향만 바꿉니다.** `git ㅔㅕ노`처럼 영문과 한글이 섞여 있으면, 한글이 있으므로 전부 영문 쪽으로 바꿉니다. 이 경우는 우연히 `git push`가 되지만, 의도한 한글 단어가 섞여 있으면 그것까지 영문이 됩니다.
- **조합 결과는 하나뿐입니다.** 같은 키 입력이 실제 입력기에서 다르게 끊길 수 있는 경우(입력 도중 방향키를 누른 경우 등)는 알 수 없습니다.
- **두벌식만 지원합니다.** 세벌식, 그리고 QWERTY가 아닌 영문 자판(Dvorak 등)은 지원하지 않습니다.

## 테스트

```
python tests/test_hanfix.py
```

pytest로도 실행됩니다. 조합 사례, 왕복 변환(`한글 → 키 → 한글`), 명령어 사례를 확인합니다.

## 라이선스

MIT
