# hanfix

[![ci](https://github.com/NaNIsaac/hanfix/actions/workflows/ci.yml/badge.svg)](https://github.com/NaNIsaac/hanfix/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**한/영 전환을 깜빡하고 친 명령을 고쳐서 다시 실행합니다.**

[English](README.md)

## 빠른 설치

**1. 최신 릴리스 설치**

**Windows (PowerShell)**

<!-- test windows -->
```powershell
irm https://github.com/NaNIsaac/hanfix/releases/latest/download/install.ps1 | iex
```

**Linux / macOS**

<!-- test unix -->
```sh
curl -fsSL https://github.com/NaNIsaac/hanfix/releases/latest/download/install.sh | sh
```

**2. 셸에 `hf` 등록**

`hf`는 셸 함수입니다. 직전 명령은 셸만 알기 때문에 셸 시작 파일에 한 줄이 필요합니다. 한 번만 실행하세요.

**PowerShell**

<!-- test windows -->
```powershell
New-Item -ItemType Directory -Force (Split-Path $PROFILE) | Out-Null; Add-Content $PROFILE 'Invoke-Expression (hanfix --init powershell | Out-String)'
```

**bash**

<!-- test unix -->
```sh
echo 'eval "$(hanfix --init bash)"' >> ~/.bashrc
```

zsh(macOS 기본 셸)와 fish는 [셸 설정](#셸-설정)을 참고하세요.

**3. 새 터미널을 열어 `hf`가 등록됐는지 확인**

<!-- test windows -->
```powershell
Get-Command hf
```

<!-- test unix -->
```sh
bash -ic 'type hf'
```

변환만 해 볼 수도 있습니다. 결과를 출력만 하고 실행하지 않습니다.

<!-- test console -->
```console
$ hanfix ㅣㄴ -ㅁ
ls -a
$ hanfix dkssudgktpdy
안녕하세요
```

스크립트는 릴리스 실행 파일을 다운로드하고 SHA-256 체크섬을 검증합니다. 원격 스크립트를 실행하기 전에 [설치 스크립트](https://github.com/NaNIsaac/hanfix/tree/main)를 확인할 수 있습니다. Cargo 설치나 직접 다운로드는 [설치](#설치)를 참고하세요.

![hanfix demo](assets/demo.gif)

```console
$ ㅣㄴ -ㅁ
bash: ㅣㄴ: command not found
$ ㅗㄹ
ls -a  [Enter / Ctrl+C]
.  ..  .git  README.md	src
$ 햣 촏차ㅐㅕㅅ -ㅠ 럍-쇼ㅔㅐ
bash: 햣: command not found
$ ㅗㄹ
git checkout -b fix-typo  [Enter / Ctrl+C]
Switched to a new branch 'fix-typo'
```

- 직전 명령을 두벌식 → QWERTY로 되돌려 보여 주고, Enter를 누르면 실행합니다.
- 한글 자판이 켜진 채로 `hf`를 치면 `ㅗㄹ`이 됩니다. 그래서 `ㅗㄹ`도 같은 명령으로 등록합니다. 자판을 바꿀 필요가 없습니다.
- 글자를 직접 넘겨 바꿀 수도 있습니다: `hanfix dkssudgktpdy` → `안녕하세요`.
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

Termux(aarch64)는 CI에서 테스트합니다.

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
New-Item -ItemType Directory -Force (Split-Path $PROFILE) | Out-Null; Add-Content $PROFILE 'Invoke-Expression (hanfix --init powershell | Out-String)'
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
4. 고친 명령을 셸 기록에 추가한 뒤 실행합니다. 위 화살표로 다시 부를 수 있습니다. fish는 기록에 추가하지 않습니다.

[codigest](https://github.com/NaNIsaac/codigest)의 터미널 기록이 켜져 있어도(bash, zsh, PowerShell) 고친
명령은 터미널을 그대로 씁니다. 편집기나 `codex`도 평소처럼 열리고, codigest는 그 명령을 직접 입력한 것과
같은 규칙으로 기록합니다.

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

### 이스케이프된 한글 되돌리기

파일 이름이나 URL에 한글이 `트리` 대신 `_ED_8A_B8_EB_A6_AC`나 `%ED%8A%B8%EB%A6%AC`처럼 UTF-8 바이트로
이스케이프되어 들어 있을 때가 있습니다. `hanfix inspect`는 무엇이 들어 있는지 설명하고, `hanfix decode`는
지정한 형식으로만 되돌립니다.

```console
$ hanfix inspect _ED_8A_B8_EB_A6_AC
Detected: underscore-hex
Status: valid
Encoding: UTF-8
Reason: 6 escaped bytes are valid UTF-8; other characters are kept

Original:  _ED_8A_B8_EB_A6_AC
Candidate: 트리
$ hanfix decode --type underscore-hex _ED_8A_B8_EB_A6_AC
트리
$ hanfix decode --type percent %ED%8A%B8%EB%A6%AC
트리
$ hanfix decode --type underscore-hex report__ED_8A_B8.pdf
report_트.pdf
$ printf '%s' '_ED_8A_B8' | hanfix decode --type underscore-hex
트
```

- **underscore-hex** (`_XX`): `_` 뒤에 대문자 16진수 두 자리, 바이트 `80`-`FF`만 해당합니다. ASCII는 이스케이프할
  필요가 없으므로 `backup_2026_10_29`, `hello_bed`, `report__ED_8A_B8`의 첫 `_`는 그대로 두고, `__`를 공백으로
  바꾸지도 않습니다. `_ed_8a_b8`처럼 소문자로 된 것은 되돌리지 않으며, `inspect`가 이를 보면 안내 한 줄을 덧붙입니다.
- underscore-hex의 이어진 이스케이프는 UTF-8 글자를 시작할 수 없는 바이트(`80`-`BF`, `C0`, `C1`, `F5`-`FF`)에서도
  끝나고, 그 바이트는 글자로 둡니다. 그래서 `IMG_8123.JPG`와 `test_99.txt`는 그대로 나오고,
  `_ED_8A_B8_99.txt`는 `트_99.txt`가 됩니다. 글자를 시작하는 바이트는 나머지 바이트가 있어야 하므로,
  `트리_DB_backup.sql`을 `_ED_8A_B8_EB_A6_AC_DB_backup.sql`로 쓴 이름은 실패합니다. `_DB`는 2바이트 글자의
  시작인데 뒤가 없습니다.
- **percent** (`%XX`): URL 인코딩입니다. 모든 바이트, 대소문자 모두 받습니다(`%20`은 공백). 뒤에 16진수 두
  자리가 없는 `%`는 글자로 둡니다(`100% done`). `+`는 그대로 `+`입니다. 공백을 뜻하는 것은 HTML 폼 데이터뿐입니다. 모든 `%XX`를
  이스케이프로 보므로 `%ED%8A%B8%99`는 실패합니다.
- 두 형식에 나뉘어 들어간 바이트열(`%ED_8A_B8`)은 어느 쪽으로도 되돌리지 않습니다.
- 이어진 이스케이프는 하나의 바이트열로 보고, 완전하고 유효한 UTF-8이어야 합니다. 하나라도 아니면(`_ED_8A`는
  3바이트 글자의 앞 2바이트) `decode`는 아무것도 출력하지 않고, 이유를 표준 오류에 쓰고, 종료 코드 1로 끝납니다.
  `�`로 바꿔 넣지 않습니다. 표준 입력이 UTF-8이 아니면 `inspect`와 `decode` 모두 종료 코드 1로 끝납니다.
- `decode`는 결과만 출력합니다. 이스케이프가 없는 글자는 그대로 나오고 종료 코드는 0입니다. `inspect`는 UTF-8
  글자에서 실패하지 않고 아무것도 바꾸지 않습니다.
- `inspect`와 `decode`는 첫 번째 인자일 때만 하위 명령입니다. `hanfix -- inspect`와 `hf`는 지금처럼 글자를 변환합니다.
- 유효한 UTF-8이라는 것은 바이트가 글자를 이룬다는 뜻일 뿐, 원래 뜻한 글이라는 보장은 아닙니다. 그래서 hanfix는
  스스로 되돌리지 않습니다. 형식은 사용자가 정하고, `inspect`로 후보를 먼저 볼 수 있습니다.
- 모지바케는 아래의 별도 형식으로 다룹니다.

**모지바케**는 UTF-8 글자를 다른 인코딩으로 읽어서 깨진 글자입니다. UTF-8로 저장한 `안녕`을 windows-1252로 열면
`ì•ˆë…•`이 됩니다. `--type mojibake`는 이 과정을 거꾸로 합니다. 각 글자를 windows-1252 바이트로 되돌리고, 그
바이트를 UTF-8로 읽습니다.

```console
$ hanfix inspect 'ì•ˆë…•'
Detected: mojibake
Status: valid
Encoding: UTF-8 read as windows-1252
Reason: every character is in windows-1252, and its 6 bytes are valid UTF-8; valid UTF-8 does not prove this is the original text

Original:  ì•ˆë…•
Candidate: 안녕
$ hanfix decode --type mojibake --source-encoding windows-1252 'ì•ˆë…•'
안녕
$ hanfix decode --type mojibake --source-encoding windows-1252 'ì•ˆ�'
hanfix: error: lossy: U+FFFD at character 3 means a byte was already replaced; the original cannot be recovered
```

- `--source-encoding`(글자를 잘못 읽은 인코딩)는 꼭 지정해야 합니다. 지원하는 것은 `windows-1252`(`cp1252`도 같음)
  하나입니다. 256바이트를 모두 대응시키며, Microsoft가 정의하지 않은 다섯 바이트(`81`, `8D`, `8F`, `90`, `9D`)는
  브라우저처럼 같은 번호의 보이지 않는 제어 문자로 봅니다.
- 글자 전체를 변환합니다. windows-1252에 없는 글자(`안녕 ì•ˆë…•`의 `안`)가 있거나 바이트가 완전한 UTF-8이 아니면
  종료 코드 1로 실패하고 표준 출력에는 아무것도 나오지 않습니다.
- `�`(U+FFFD)는 hanfix가 글자를 받기 전에 이미 어떤 바이트가 바뀌었다는 뜻입니다. 어느 바이트였는지는 어떤 도구도
  알 수 없으므로 `decode`는 실패하고 `inspect`는 lossy(손실)로 알립니다. 복사해 붙여 넣는 과정에서 보이지 않는 제어
  문자가 빠져도 마찬가지로 되돌릴 수 없습니다.
- `inspect`는 모지바케처럼 보일 때만 후보를 보여 줍니다. UTF-8 여러 바이트 글자를 시작하는 바이트의 글자 뒤에 그
  글자를 잇는 바이트의 글자가 오는 경우입니다. 한글, ASCII, 일반 악센트 글자(`café`, `naïve`)는 후보가 없습니다.
  되돌린 결과가 다시 모지바케처럼 보이면 이유에 그렇게 적습니다(두 번 잘못 읽은 경우이니 결과를 한 번 더 되돌리세요).
- CP949 / EUC-KR(UTF-8을 한국어 Windows 글자로 연 경우, `?덈뀞` 같은 출력)은 지원하지 않습니다. 약 17,000자의 표가
  필요하고, 이런 잘못 읽기는 대개 도중에 바이트를 잃습니다.

**이스케이프된 이름의 파일 여러 개를 한꺼번에 바꾸기 (PowerShell).** 함수를 정의한 뒤 폴더에서
`Rename-Decoded`를 실행하면 `old -> new` 목록만 보여 줍니다. 실제로 바꾸려면 `-Apply`를 붙입니다. 되돌릴 수
없는 이름, 바뀌지 않는 이름, 파일 이름이 될 수 없는 결과(제어 문자, 끝의 점이나 공백, `NUL` 같은 Windows 장치 이름), 이미 있는 파일이나 같은
실행에서 먼저 바꾸기로 한 이름과 겹치는 경우(대소문자 무시)는 건너뜁니다.

```powershell
function Rename-Decoded {
    param([string]$Path = '.', [ValidateSet('underscore-hex', 'percent')][string]$Type = 'underscore-hex', [switch]$Apply)
    # hanfix 0.2.7 and older read "decode ..." as text to convert and exit 0, which would rename every file.
    $probe = (hanfix decode --type percent -- '%41' 2>$null) -join "`n"
    if ($LASTEXITCODE -ne 0 -or $probe -ne 'A') { throw 'hanfix too old: update with hanfix --update' }
    # Every name in the folder and every new name planned so far, ignoring case, so the preview lists
    # exactly what -Apply does.
    $taken = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    Get-ChildItem -LiteralPath $Path -Force -Name | ForEach-Object { [void]$taken.Add($_) }
    $enc = [Console]::OutputEncoding
    [Console]::OutputEncoding = [Text.Encoding]::UTF8
    try {
        foreach ($f in Get-ChildItem -LiteralPath $Path -File) {
            $out = @(hanfix decode --type $Type -- $f.Name 2>$null)
            if ($LASTEXITCODE -ne 0) { Write-Warning "skip $($f.Name): does not decode"; continue }
            $new = $out[0]
            if ($new -ceq $f.Name) { continue }
            # One line, no control characters, no trailing dot or space, which Windows drops ("a." -> "a"),
            # and no Windows device name.
            if ($out.Count -ne 1 -or $new -match '[\x00-\x1F\x7F]|[. ]$|^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(\.|$)' -or
                $new.IndexOfAny([IO.Path]::GetInvalidFileNameChars()) -ge 0) {
                Write-Warning "skip $($f.Name): '$new' is not a file name"; continue
            }
            if (-not $taken.Add($new)) { Write-Warning "skip $($f.Name): $new exists or is planned"; continue }
            "$($f.Name) -> $new"
            if ($Apply) { Rename-Item -LiteralPath $f.FullName -NewName $new }
        }
    } finally { [Console]::OutputEncoding = $enc }
}
```

### 옵션

```
usage: hanfix [-h] [-V] [--to {ko,en}] [--] [TEXT ...]
       hanfix --init {bash,zsh,fish,powershell} [--alias NAME]
       hanfix --update
       hanfix inspect [--] [TEXT ...]
       hanfix decode --type {underscore-hex,percent} [--] [TEXT ...]
       hanfix decode --type mojibake --source-encoding windows-1252 [--] [TEXT ...]
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
| `--type TYPE` | `decode` 전용: `underscore-hex`, `percent`, `mojibake` 중 하나. 꼭 지정해야 합니다. |
| `--source-encoding ENC` | `decode --type mojibake` 전용: `windows-1252`(또는 `cp1252`). 그때는 꼭 지정해야 합니다. |

- 옵션은 맨 앞에서만 읽습니다. 나머지는 전부 변환할 글자로 봅니다. 그래서 `hanfix ㅣㄴ -ㅁ`의 `-ㅁ`은 옵션이 아니라 글자(`-a`)입니다.
- `-`로 시작하는 영문 글자를 바꾸려면 앞에 `--`를 붙이세요: `hanfix -- -rk` → `-가`.
- 종료 코드는 성공 0, `decode` 실패와 `inspect`·`decode`의 입력이 UTF-8이 아닐 때 1, 잘못된 사용 2입니다.

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
cargo test           # 조합 사례, 한글 음절 11,172자 전체의 왕복 변환, 셸 코드, decode 사례, CLI
sh tests/shells.sh   # bash, zsh, fish에서 hf가 실제로 고친 명령을 실행하는지 (설치된 hanfix 사용)
pwsh tests/shells.ps1  # PowerShell에서 같은 확인과 위의 Rename-Decoded 예시
```

빠른 설치의 세팅 단계도 테스트합니다. 코드 블록 앞의 `<!-- test -->` 주석이 대상을 표시하고, CI가 공통
[readme workflow](https://github.com/NaNIsaac/.github/blob/main/.github/workflows/readme.yml)로 표시된 블록을
깨끗한 Linux, macOS, Windows에서 순서대로 실행합니다.

## 삭제

- Linux/macOS: `rm ~/.local/bin/hanfix`
- Windows: `Remove-Item -Recurse "$env:LOCALAPPDATA\Programs\hanfix"`

셸 시작 파일에 넣은 한 줄도 지우세요.

함께 보기: [hangeul](https://github.com/NaNIsaac/hangeul) - 한글 텍스트 처리용 파이썬 라이브러리.

## 라이선스

MIT
