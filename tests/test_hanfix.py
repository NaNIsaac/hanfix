"""python tests/test_hanfix.py (or pytest)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from hanfix.cli import compose, decompose, fix, main  # noqa: E402

CASES = {
    "dkssud": "안녕", "dkssudgktpdy": "안녕하세요", "rkr": "각", "rkrk": "가가",
    "rhk": "과", "dkfr": "앍", "dkfrl": "알기", "dkfrdl": "앍이", "qkqhrk": "바보가", "dmlwk": "의자",
    "rkqt": "값", "rkqtdl": "값이", "Tkd": "쌍", "ekfr": "닭", "gksrmf": "한글",
    "dkssud, gktpdy!": "안녕, 하세요!", "rr": "ㄱㄱ", "k": "ㅏ",
}


def test_compose():
    for keys, ko in CASES.items():
        assert compose(keys) == ko, (keys, compose(keys), ko)


def test_round_trip():
    for keys, ko in CASES.items():
        assert compose(decompose(ko)) == ko, ko


def test_commands():
    assert fix("ㅣㄴ -ㅁ") == "ls -a"
    assert fix("햣 ㄴㅅㅁ션") == "git status"
    assert fix("ㅊㅇ ..") == "cd .."
    assert fix("dkssud") == "안녕"
    assert fix("ls", to="ko") == "ㅣㄴ"


def test_init():
    assert main(["--init", "bash"]) == 0 and main(["--init", "fish"]) == 2


if __name__ == "__main__":
    test_compose()
    test_round_trip()
    test_commands()
    test_init()
    print("hanfix: all checks passed")
