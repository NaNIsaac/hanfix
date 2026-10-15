# hanfix

한/영 전환을 깜빡하고 친 명령을 고쳐서 다시 실행합니다. [thefuck](https://github.com/nvbn/thefuck)처럼 쓰는 도구입니다.

```
$ ㅣㄴ -ㅁ
ㅣㄴ: command not found
$ hf
ls -a  [Enter / Ctrl+C]
```

- 직전 명령을 두벌식(KS X 5002) → QWERTY로 되돌려 보여 주고, Enter를 누르면 실행합니다. Ctrl+C를 누르면 실행하지 않습니다.
- 고친 명령은 셸 기록에도 남습니다. 위 화살표로 다시 부를 수 있습니다.
- 한글 자판이 켜진 채로 `hf`를 치면 `ㅗㄹ`이 됩니다. 그래서 `ㅗㄹ`도 같은 명령으로 등록합니다. 자판을 바꿀 필요가 없습니다.
- 글자를 직접 넘겨 바꿀 수도 있습니다: `hanfix dkssudgktpdy` → `안녕하세요`.
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

## 셸에 연결하기

`hf`는 셸 함수입니다. 직전 명령은 셸만 알기 때문입니다. 프로필에 한 줄을 넣으세요.

**PowerShell** — `notepad $PROFILE`로 열고 추가합니다.

```powershell
Invoke-Expression (hanfix --init powershell | Out-String)
```

**bash / Git Bash** — `~/.bashrc`에 추가합니다.

```bash
eval "$(hanfix --init bash)"
```

새 셸을 열면 `hf`와 `ㅗㄹ`을 쓸 수 있습니다. 실제로 등록되는 코드는 `hanfix --init powershell`로 볼 수 있습니다.

zsh, fish는 아직 지원하지 않습니다.

**동작 순서**
1. 셸 기록에서 직전 명령을 꺼냅니다. PowerShell은 `Get-History`, bash는 `fc`를 씁니다.
2. 한글 자모를 영문 키로 되돌립니다. 한글이 없으면 `nothing to fix`를 출력하고 끝냅니다.
3. 고친 명령을 초록색으로 보여 주고 Enter를 기다립니다.
4. 기록에 추가한 뒤 실행합니다.

## 글자를 직접 바꾸기

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

**옵션은 맨 앞의 `--to`, `--init`, `-h`/`--help`만 받습니다.** 나머지는 전부 변환할 글자로 봅니다.
그래서 `hanfix ㅣㄴ -ㅁ`의 `-ㅁ`도 옵션이 아니라 `-a`로 바뀝니다.

## 입력 중인 줄 바로 고치기 (선택)

실행하기 전에, 지금 치고 있는 줄을 그 자리에서 고치는 단축키입니다. 이 단축키 설정은 직접 동작을 확인하지 못했습니다.

**PowerShell** (`Ctrl+Alt+h`):

```powershell
Set-PSReadLineKeyHandler -Chord 'Ctrl+Alt+h' -BriefDescription hanfix -ScriptBlock {
    $line = $null; $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
    $fixed = (hanfix $line | Out-String).TrimEnd()
    [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
    [Microsoft.PowerShell.PSConsoleReadLine]::Insert($fixed)
}
```

**bash** (`Alt+h`):

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

- **`hf`는 한글을 영문으로만 바꿉니다.** 명령은 영문이므로 이 방향만 의미가 있습니다.
- **한 줄에 한 방향만 바꿉니다.** `git ㅔㅕ노`처럼 영문과 한글이 섞여 있으면, 한글이 있으므로 전부 영문 쪽으로 바꿉니다. 이 경우는 우연히 `git push`가 되지만, 의도한 한글 단어가 섞여 있으면 그것까지 영문이 됩니다.
- **조합 결과는 하나뿐입니다.** 같은 키 입력이 실제 입력기에서 다르게 끊길 수 있는 경우(입력 도중 방향키를 누른 경우 등)는 알 수 없습니다.
- **두벌식만 지원합니다.** 세벌식, 그리고 QWERTY가 아닌 영문 자판(Dvorak 등)은 지원하지 않습니다.

## 테스트

```
python tests/test_hanfix.py
```

pytest로도 실행됩니다. 조합 사례, 왕복 변환(`한글 → 키 → 한글`), 명령어 사례, `--init` 출력을 확인합니다.

## 라이선스

MIT
