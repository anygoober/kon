from .include.kon_parser import parse as native_parse

def parse(source: str):
    items = native_parse(source)
    print(items)
