# hanfix

한/영 전환을 깜빡하고 친 명령을 고쳐서 다시 실행합니다.

A small CLI that fixes and re-runs shell commands typed with the Korean (2-set) keyboard layout still on, and converts text between Hangul and QWERTY keys.

![hanfix demo](assets/demo.gif)

```
$ ㅣㄴ -ㅁ
ㅣㄴ: command not found
$ ㅗㄹ
ls -a  [Enter / Ctrl+C]
```

- 직전 명령을 두벌식 → QWERTY로 되돌려 보여 주고, Enter를 누르면 실행합니다.
- 한글 자판이 켜진 채로 `hf`를 치면 `ㅗㄹ`이 됩니다. 그래서 `ㅗㄹ`도 같은 명령으로 등록합니다. 자판을 바꿀 필요가 없습니다.
- 글자를 직접 넘겨 바꿀 수도 있습니다: `hanfix dkssudgktpdy` → `안녕하세요`.
- 실행 파일 하나로 동작합니다. Python 같은 런타임이 필요 없습니다.
- Linux, macOS, Windows를 지원하고, 셸은 bash, zsh, fish, PowerShell을 지원합니다.

## 설치

**Linux / macOS**

```sh
curl -fsSL https://github.com/NaNIsaac/hanfix/releases/latest/download/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://github.com/NaNIsaac/hanfix/releases/latest/download/install.ps1 | iex
```

설치 스크립트는 다음을 합니다.
1. 운영체제와 CPU에 맞는 실행 파일을 [Releases](https://github.com/NaNIsaac/hanfix/releases)에서 내려받습니다.
2. SHA-256 체크섬을 확인합니다.
3. 실행 파일을 설치 폴더에 넣습니다.
   - Linux/macOS: `~/.local/bin`
   - Windows: `%LOCALAPPDATA%\Programs\hanfix`. 이 폴더를 사용자 PATH에 추가합니다.
4. 셸 설정에 넣을 한 줄을 알려 줍니다.

설치 폴더는 `HANFIX_INSTALL_DIR`로, 버전은 `HANFIX_VERSION=v0.2.3`처럼 바꿀 수 있습니다.

Rust가 있으면 소스에서 바로 설치할 수도 있습니다.

```sh
cargo install --git https://github.com/NaNIsaac/hanfix
```

## 업데이트

```sh
hanfix --update
```

- 설치 스크립트를 다시 실행해 최신 릴리스로 바꿉니다. 설치 위치는 지금 실행 중인 hanfix가 있는 폴더입니다.
- 체크섬 확인도 처음 설치할 때와 같이 합니다.
- 설치 명령(`curl … | sh`, `irm … | iex`)을 다시 실행해도 같은 결과입니다.
- `cargo install`로 설치했다면 `cargo install --git https://github.com/NaNIsaac/hanfix --force`를 쓰세요.
- 지금 버전은 `hanfix -V`로 확인합니다.

## 셸 설정

`hf`는 셸 함수입니다. 직전 명령은 셸만 알기 때문입니다. 셸 시작 파일에 한 줄을 넣고 새 터미널을 여세요.

| 셸 | 파일 | 넣을 줄 |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(hanfix --init bash)"` |
| zsh | `~/.zshrc` | `eval "$(hanfix --init zsh)"` |
| fish | `~/.config/fish/config.fish` | `hanfix --init fish \| source` |
| PowerShell | `$PROFILE` | `Invoke-Expression (hanfix --init powershell \| Out-String)` |

PowerShell에서는 이렇게 한 번 실행하면 됩니다.

```powershell
Add-Content $PROFILE 'Invoke-Expression (hanfix --init powershell | Out-String)'
```

실제로 등록되는 코드는 `hanfix --init bash`처럼 실행해서 볼 수 있습니다.

## 사용법

### 직전 명령 고치기

| 명령 | 동작 |
|---|---|
| `hf` 또는 `ㅗㄹ` | 고친 명령을 보여 주고, Enter를 누르면 실행합니다. Ctrl+C를 누르면 실행하지 않습니다. |
| `hf -y` | 묻지 않고 바로 실행합니다. |

**동작 순서**
1. 셸 기록에서 직전 명령을 꺼냅니다. `hf` 자신은 건너뜁니다.
2. 한글 자모를 영문 키로 되돌립니다. 바꿀 것이 없으면 `nothing to fix`를 출력하고 끝냅니다.
3. 고친 명령을 초록색으로 보여 주고 Enter를 기다립니다.
4. 고친 명령을 셸 기록에 추가한 뒤 실행합니다. 위 화살표로 다시 부를 수 있습니다. fish는 기록에 추가하지 않습니다(fish 3에는 기록 추가 명령이 없음).

함수 이름은 `--alias`로 바꿀 수 있습니다. 한글 자판 쪽 이름도 함께 등록됩니다.
예를 들어 `--alias fk`이면 `fk`와 `라`가 등록됩니다.

```sh
eval "$(hanfix --init bash --alias fk)"
```

### 글자를 직접 바꾸기

```
hanfix ㅣㄴ -ㅁ               ->  ls -a
hanfix 햣 ㄴㅅㅁ션             ->  git status
hanfix dkssudgktpdy          ->  안녕하세요
echo ㅊㅇ .. | hanfix         ->  cd ..          (표준 입력도 받음)
hanfix --to ko git           ->  햣             (방향 지정)
hanfix --to en 한글           ->  gksrmf
```

**방향은 자동으로 정합니다.** 한글이 하나라도 있으면 영문 키로, 없으면 한글로 바꿉니다.
자동 판단이 틀리면 `--to ko` 또는 `--to en`으로 방향을 정하세요.

### 옵션

```
usage: hanfix [-h] [-V] [--to {ko,en}] [--] [TEXT ...]
       hanfix --init {bash,zsh,fish,powershell} [--alias NAME]
       hanfix --update
```

| 옵션 | 설명 |
|---|---|
| `-h`, `--help` | 도움말을 출력합니다. |
| `-V`, `-v`, `--version` | 버전을 출력합니다. |
| `--update` | 최신 릴리스로 업데이트합니다. |
| `--to {ko,en}` | 변환 방향을 정합니다. `ko`는 영문 키 → 한글, `en`은 한글 → 영문 키입니다. |
| `--init SHELL` | `hf` 함수를 정의하는 셸 코드를 출력합니다. |
| `--alias NAME` | `--init`이 만드는 함수 이름입니다. 기본값은 `hf`입니다. |
| `--` | 옵션의 끝을 표시합니다. 이후는 모두 글자로 봅니다. |

- 옵션은 맨 앞에서만 읽습니다. 나머지는 전부 변환할 글자로 봅니다. 그래서 `hanfix ㅣㄴ -ㅁ`의 `-ㅁ`은 옵션이 아니라 글자(`-a`)입니다.
- `-`로 시작하는 영문 글자를 바꾸려면 앞에 `--`를 붙이세요: `hanfix -- -rk` → `-가`.
- 종료 코드는 성공 0, 잘못된 사용 2입니다.

### 입력 중인 줄 바로 고치기 (선택)

실행하기 전에, 지금 치고 있는 줄을 그 자리에서 고치는 단축키입니다. 이 설정은 직접 동작을 확인하지 못했습니다.

**PowerShell** (`Ctrl+Alt+h`)

```powershell
Set-PSReadLineKeyHandler -Chord 'Ctrl+Alt+h' -BriefDescription hanfix -ScriptBlock {
    $line = $null; $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
    $enc = [Console]::OutputEncoding; [Console]::OutputEncoding = [Text.Encoding]::UTF8
    try { $fixed = (hanfix -- $line | Out-String).TrimEnd() } finally { [Console]::OutputEncoding = $enc }
    [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
    [Microsoft.PowerShell.PSConsoleReadLine]::Insert($fixed)
}
```

**bash** (`Alt+h`)

```bash
bind -x '"\eh": READLINE_LINE=$(hanfix -- "$READLINE_LINE"); READLINE_POINT=${#READLINE_LINE}'
```

## 동작 원리

**영문 → 한글**은 실제 두벌식(KS X 5002) 입력기와 같은 오토마타로 조합합니다.
- 키마다 자모를 찾습니다(`r` → ㄱ, `k` → ㅏ). Shift 키는 쌍자음과 ㅒ, ㅖ만 따로 처리하고, 나머지 대문자는 소문자와 같게 봅니다.
- 겹모음(ㅘ, ㅢ 등)과 겹받침(ㄳ, ㄺ, ㅄ 등)을 합칩니다.
- 받침 뒤에 모음이 오면 받침을 다음 글자로 넘깁니다.
  - `rkrk` → 가가
  - `rkqtdl` → 값이: 겹받침은 뒤쪽 자음만 넘어갑니다.
- ㄸ, ㅃ, ㅉ은 받침이 될 수 없으므로 항상 새 글자를 시작합니다.
- 음절은 유니코드 공식으로 만듭니다: `0xAC00 + (초성×21 + 중성)×28 + 종성`.

**한글 → 영문**은 위 공식을 거꾸로 풀어 초성·중성·종성을 꺼냅니다. 겹자모는 나눈 뒤 키로 바꿉니다.

자판 글자가 아닌 문자(숫자, 공백, 기호)는 그대로 둡니다.

## 한계

- **`hf`는 한글을 영문으로만 바꿉니다.** 명령은 영문이므로 이 방향만 의미가 있습니다.
- **한 줄에 한 방향만 바꿉니다.** `git ㅔㅕ노`처럼 영문과 한글이 섞여 있으면, 한글이 있으므로 전부 영문 쪽으로 바꿉니다. 의도한 한글 단어가 섞여 있으면 그것까지 영문이 됩니다.
- **두벌식만 지원합니다.** 세벌식, 그리고 QWERTY가 아닌 영문 자판(Dvorak 등)은 지원하지 않습니다.

## 개발

```sh
cargo test           # 조합 사례, 한글 음절 11,172자 전체의 왕복 변환, 셸 코드
sh tests/shells.sh   # bash, zsh, fish에서 hf가 실제로 고친 명령을 실행하는지 (설치된 hanfix 사용)
pwsh tests/shells.ps1
```

- CI는 Linux, macOS, Windows에서 위 검사를 모두 돌립니다.
- `v*` 태그를 push하면 6개 플랫폼용 실행 파일이 Releases에 올라갑니다: Linux·macOS·Windows × x86_64·arm64.
- 데모 GIF는 `python assets/make_demo.py`로 다시 만듭니다. Pillow가 필요합니다.

## 삭제

- Linux/macOS: `rm ~/.local/bin/hanfix`
- Windows: `Remove-Item -Recurse "$env:LOCALAPPDATA\Programs\hanfix"`

셸 시작 파일에 넣은 한 줄도 지우세요.

## 라이선스

MIT
