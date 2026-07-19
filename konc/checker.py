from dataclasses import dataclass
from enum import Enum
from typing import Any, Callable

from .include.kon_parser import Expr, Item, Stmt, StringPart, parse


class DeclNamespace(Enum):
    Types = 0
    Values = 1


@dataclass(kw_only=True)
class Expectant:
    target: str
    namespace: DeclNamespace
    origin: str
    check: Callable[[Any], None] | None


@dataclass
class State:
    context: str
    decl_types: dict[str, Any]
    decl_values: dict[str, Any]

    # mainly for hoisting
    expectants: dict[str, str]

    def add_type_decl(self, name: str, decl: Any):
        self.decl_types[name] = decl

    def add_value_decl(self, name: str, decl: Any):
        self.decl_values[name] = decl

    def concat(self, other: "State") -> "State":
        self.decl_types |= other.decl_types
        self.decl_values |= other.decl_values

        return self


@dataclass
class GlobalState(State): ...


@dataclass
class LocalState(State):
    shadows: dict[str, str]


def check(source: str): ...
