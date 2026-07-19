from argparse import ArgumentParser
from pathlib import Path

def main():
    parser = ArgumentParser("konc")
    parser.add_argument("file", type=Path)
    args = parser.parse_args()

    print("you want to compile", args.file)

if __name__ == "__main__":
    main()
