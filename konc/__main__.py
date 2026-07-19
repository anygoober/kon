from argparse import ArgumentParser
from pathlib import Path

from .checker import check


def main():
    parser = ArgumentParser("konc")
    parser.add_argument("file", type=Path)
    args = parser.parse_args()

    with open(args.file, "r", encoding="utf-8") as f:
        source_text = f.read()

    check(source_text)
    print("checks passed")


if __name__ == "__main__":
    main()
