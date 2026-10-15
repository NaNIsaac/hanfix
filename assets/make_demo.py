"""Render assets/demo.gif, the terminal animation in the README. Needs Pillow; run on Windows (fonts).

    python assets/make_demo.py
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

FONTS = Path("C:/Windows/Fonts")
SIZE = 22
LATIN = ImageFont.truetype(str(FONTS / "consola.ttf"), SIZE)
HANGUL = ImageFont.truetype(str(FONTS / "malgun.ttf"), SIZE + 1)
CW, LH = round(LATIN.getlength("m")), 32          # cell width, line height
COLS, ROWS, PAD = 52, 11, 22
BG, FG, DIM, GREEN, BLUE, RED = "#1e2430", "#d8dee9", "#7b8496", "#a3d977", "#81a1c1", "#e5737a"


def wide(c):
    return "\uac00" <= c <= "\ud7a3" or "\u3131" <= c <= "\u3163"


def render(lines):
    img = Image.new("RGB", (2 * PAD + COLS * CW, 2 * PAD + ROWS * LH + 26), BG)
    d = ImageDraw.Draw(img)
    for i, x in enumerate(("#e5737a", "#ebcb8b", "#a3d977")):     # window buttons
        d.ellipse((PAD + i * 22, 12, PAD + i * 22 + 12, 24), fill=x)
    y = PAD + 26
    for segs in lines[-ROWS:]:
        col = 0
        for text, color in segs:
            for c in text:
                x = PAD + col * CW
                if wide(c):
                    d.text((x + (2 * CW - HANGUL.getlength(c)) / 2, y), c, font=HANGUL, fill=color)
                    col += 2
                else:
                    d.text((x, y + 4), c, font=LATIN, fill=color)
                    col += 1
        y += LH
    return img


frames, durations, screen = [], [], []


def snap(ms, cursor=False):
    s = [list(line) for line in screen]
    if cursor:
        s[-1] = s[-1] + [("\u2588", DIM)]
    frames.append(render(s))
    durations.append(ms)


def prompt():
    screen.append([("~/project ", BLUE), ("$ ", FG)])


def type_(text, color=FG):
    for c in text:
        screen[-1].append((c, color))
        snap(90, cursor=True)


def out(*segs, ms=60):
    screen.append(list(segs))
    snap(ms)


prompt(); snap(700, cursor=True)
type_("ㅣㄴ -ㅁ"); snap(500, cursor=True)
out(("ㅣㄴ: command not found", RED), ms=900)
prompt(); snap(500, cursor=True)
type_("ㅗㄹ"); snap(400, cursor=True)
out(("ls -a", GREEN), ("  [Enter / Ctrl+C]", DIM), ms=1300)
out((".  ..  .git  Cargo.toml  README.md  install.sh  src", FG), ms=1000)
prompt(); snap(400, cursor=True)
type_("햣 ㄴㅅㅁ션"); snap(500, cursor=True)
out(("햣: command not found", RED), ms=800)
prompt(); type_("hf"); snap(300, cursor=True)
out(("git status", GREEN), ("  [Enter / Ctrl+C]", DIM), ms=1300)
out(("On branch main", FG))
out(("nothing to commit, working tree clean", FG), ms=900)
prompt(); type_("hanfix dkssudgktpdy"); snap(400, cursor=True)
out(("안녕하세요", FG), ms=600)
prompt(); snap(2500, cursor=True)

frames[0].save(Path(__file__).with_name("demo.gif"), save_all=True, append_images=frames[1:],
               duration=durations, loop=0, optimize=True)
