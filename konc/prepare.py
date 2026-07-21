with open("out/kon_parser.py", "r+") as f:
    contents = f.read()
    f.seek(0)
    f.write("# type: ignore\n\n" + contents)
    f.truncate()

from out.kon_parser import parse

parse("")
print("OK: importable")
