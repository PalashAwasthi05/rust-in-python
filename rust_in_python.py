"""A standard-library-only interpreter for a scalar subset of Rust.

Run: python3 rust_in_python.py program.rs
Demo: python3 rust_in_python.py --demo

Supported types include integers, bool, string slices, arrays, structs, and enums.
Includes type and const generic parameters, trait and inherent implementations,
declarative macros, and checked integer arithmetic.
This subset checks moves, initialization, and local shared and mutable loans.
Reference returns support elision and named lifetimes on function signatures.
References in aggregates and the standard library are outside its scope.
"""

from __future__ import annotations

import argparse
from collections import deque
from dataclasses import dataclass, field, replace
import json
import operator
import os
from pathlib import Path
import re
import subprocess
import sys
from tempfile import TemporaryDirectory


class RustError(Exception):
    pass


class RustPanic(Exception):
    pass


@dataclass(frozen=True)
class Token:
    text: str
    line: int
    column: int
    context: tuple = ()
    definition: int | None = None
    module: tuple = ()
    child_module: tuple | None = None

    @property
    def key(self):
        return (self.text, self.context) if self.context else self.text

    def binding(self, key):
        if isinstance(key, tuple):
            name, context = key
            return replace(self, text=name, context=context)
        return replace(self, text=key, context=())

    def error(self, message):
        return RustError(f"{self.line}:{self.column}: {message}")


INTEGER_WIDTHS = {f"{sign}{width}": width for sign in ("i", "u") for width in (8, 16, 32, 64, 128)}
INTEGER_WIDTHS.update(isize=64, usize=64)
INTEGER_TYPES = set(INTEGER_WIDTHS)
INTEGER_SUFFIX = re.compile(f"(?:{'|'.join(INTEGER_WIDTHS)})$")
INTEGER_BOUNDS = {kind: (-(1 << (width - 1)), 1 << (width - 1)) if kind.startswith("i") else (0, 1 << width)
                  for kind, width in INTEGER_WIDTHS.items()}
CONST_TYPES = INTEGER_TYPES | {"bool", "char"}
CHAR_LIMIT = 0x110000
SURROGATES = range(0xD800, 0xE000)
SCALAR_BOUNDS = {**INTEGER_BOUNDS, "char": (0, CHAR_LIMIT)}


class Character(str):
    """A Unicode scalar value, with its own Rust debug representation."""


TOKEN = re.compile(
    r'\s+|//[^\n]*|/\*|b?r(?P<hashes>#{0,255})"[\s\S]*?"(?P=hashes)|'
    r'b?"(?:(?:\\[\s\S])|[^"\\])*"|'
    r"b?'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[^\r\n])|[^'\\\r\n\t])'|"
    r"'[A-Za-z_][A-Za-z_0-9]*|"
    rf'(?:0x[0-9a-fA-F_]+|0o[0-7_]+|0b[01_]+|[0-9][0-9_]*)(?:{"|".join(INTEGER_WIDTHS)})?|'
    r'[A-Za-z_][A-Za-z_0-9]*|'
    r'\.\.=|\.\.|::|->|=>|==|!=|<=|>=|&&|\|\||[+*/%&|^\-]=|[{}()[\]#$?.:,;=!+*/%&<>|^\-]'
)
COMMENT_EDGE = re.compile(r'/\*|\*/')
IDENT = re.compile(r'[A-Za-z_][A-Za-z_0-9]*\Z')
LIFETIME = re.compile(r"'[A-Za-z_][A-Za-z_0-9]*\Z")
RESERVED = set("""
    as async await break const continue crate dyn else enum extern false fn for
    gen if impl in let loop match mod move mut pub ref return self Self static
    struct super trait true type unsafe use where while abstract become box do
    final macro override priv try typeof unsized virtual yield _
""".split())
PRECEDENCE = {
    "=": 1, "+=": 1, "-=": 1, "*=": 1, "/=": 1, "%=": 1, "&=": 1, "|=": 1, "^=": 1, "<<=": 1, ">>=": 1,
    "||": 2, "&&": 3, "==": 4, "!=": 4, "<": 4, "<=": 4, ">": 4, ">=": 4,
    "|": 5, "^": 6, "&": 7, "<<": 8, ">>": 8, "+": 9, "-": 9, "*": 10, "/": 10, "%": 10,
}
BITWISE = {"&": operator.and_, "|": operator.or_, "^": operator.xor}
SHIFTS = {"<<", ">>"}
BLOCK_EXPRESSIONS = {"block", "if", "while", "while_let", "loop", "for", "match"}
PRIMITIVES = CONST_TYPES | {"&str", "()"}
NEVER = "!"
SCALAR_CONSTANTS = {f"{kind}::{name}": (result, value)
                    for kind, (lo, hi) in INTEGER_BOUNDS.items()
                    for name, result, value in (("MIN", kind, lo), ("MAX", kind, hi - 1),
                                                ("BITS", "u32", INTEGER_WIDTHS[kind]))}
SCALAR_CONSTANTS.update({"char::MIN": ("char", Character("\0")),
                         "char::MAX": ("char", Character(chr(CHAR_LIMIT - 1)))})


def lex(source):
    offset, line, column = 0, 1, 1
    while offset < len(source):
        match = TOKEN.match(source, offset)
        if not match:
            raise Token(source[offset], line, column).error(
                f"unexpected character {source[offset]!r}"
            )
        word = match.group()
        end = match.end()
        if word == "/*":
            depth = 1
            for edge in COMMENT_EDGE.finditer(source, end):
                depth += 1 if edge.group() == "/*" else -1
                if depth == 0:
                    end = edge.end()
                    break
            if depth:
                raise Token(word, line, column).error("unclosed block comment")
        elif not (word.isspace() or word.startswith("//")):
            yield Token(word, line, column)
        consumed = source[offset:end]
        lines = consumed.count("\n")
        line += lines
        column = len(consumed.rsplit("\n", 1)[1]) + 1 if lines else column + len(consumed)
        offset = end
    yield Token("", line, column)


DELIMITERS = {"(": ")", "[": "]", "{": "}"}


@dataclass(frozen=True)
class TokenGroup:
    opening: Token
    items: tuple
    closing: Token


@dataclass(frozen=True)
class OpaqueFragment:
    token: Token
    fragment: str
    items: tuple


def token_trees(tokens):
    stack = [(None, [])]
    for token in tokens:
        if not token.text:
            if len(stack) != 1:
                raise stack[-1][0].error("unclosed delimiter")
            return tuple(stack[0][1]), token
        if token.text in DELIMITERS:
            stack.append((token, []))
        elif token.text in DELIMITERS.values():
            opening, items = stack[-1]
            if opening is None or DELIMITERS[opening.text] != token.text:
                raise token.error("mismatched delimiter")
            stack.pop()
            stack[-1][1].append(TokenGroup(opening, tuple(items), token))
        else:
            stack[-1][1].append(token)
    raise AssertionError("missing end of input")


def flatten_trees(trees):
    for tree in trees:
        if isinstance(tree, TokenGroup):
            yield tree.opening
            yield from flatten_trees(tree.items)
            yield tree.closing
        elif isinstance(tree, OpaqueFragment):
            grouped = tree.fragment in {"expr", "pat"}
            if grouped:
                yield replace(tree.token, text="(")
            yield from flatten_trees(tree.items)
            if grouped:
                yield replace(tree.token, text=")")
        else:
            yield tree


def tree_text(tree):
    if isinstance(tree, TokenGroup):
        return tree.opening.text
    return "@opaque" if isinstance(tree, OpaqueFragment) else tree.text


@dataclass(frozen=True)
class MacroCapture:
    name: Token
    fragment: str = ""


@dataclass(frozen=True)
class MacroRepeat:
    body: tuple
    separator: Token | None
    count: str


def macro_sequence(items, *, matcher):
    result = []
    index = 0
    while index < len(items):
        tree = items[index]
        index += 1
        if isinstance(tree, TokenGroup):
            result.append(replace(tree, items=macro_sequence(tree.items, matcher=matcher)))
            continue
        if tree.text != "$":
            result.append(tree)
            continue
        if index >= len(items):
            raise tree.error("expected a macro metavariable or repetition")
        next_item = items[index]
        index += 1
        if isinstance(next_item, TokenGroup):
            if next_item.opening.text != "(" or index >= len(items):
                raise tree.error("expected a macro repetition")
            separator = None
            if tree_text(items[index]) not in {"*", "+", "?"}:
                separator = items[index]
                index += 1
            if index >= len(items) or tree_text(items[index]) not in {"*", "+", "?"}:
                raise tree.error("expected '*', '+', or '?' after macro repetition")
            count = tree_text(items[index])
            index += 1
            if count == "?" and separator is not None:
                raise tree.error("optional repetition cannot have a separator")
            result.append(MacroRepeat(macro_sequence(next_item.items, matcher=matcher), separator, count))
            continue
        if not IDENT.fullmatch(next_item.text):
            raise next_item.error("invalid macro metavariable")
        fragment = ""
        if matcher:
            if index + 1 >= len(items) or tree_text(items[index]) != ":":
                raise next_item.error("macro matcher requires a fragment specifier")
            fragment = tree_text(items[index + 1])
            index += 2
            if fragment not in {"tt", "ident", "literal", "expr", "ty", "pat", "path", "block"}:
                raise next_item.error(f"unsupported macro fragment {fragment!r}")
        result.append(MacroCapture(next_item, fragment))
    return tuple(result)


class MacroExpander:
    def __init__(self):
        self.steps = 0
        self.match_steps = 0
        self.definitions = 0
        self.qualified = {}
        self.module_macros = {}

    def expand(self, items, inherited=None, depth=0):
        if depth > 64:
            raise next(flatten_trees(items), Token("macro", 1, 1)).error("macro expansion recursion limit reached")
        macros = dict(inherited or {})
        result = []
        index = 0
        while index < len(items):
            item = items[index]
            if isinstance(item, Token) and item.child_module is not None:
                self.module_macros[item.child_module] = dict(macros)
                index += 1
                continue
            if tree_text(item) == "macro_rules" and index + 3 < len(items):
                marker, name, body = items[index + 1:index + 4]
                if tree_text(marker) != "!" or not isinstance(name, Token) or not isinstance(body, TokenGroup):
                    raise item.error("invalid macro_rules definition")
                self.definitions += 1
                macros[name.text] = (self.definitions, self.rules(body))
                result.append(replace(name, text="@macro_definition", definition=self.definitions))
                index += 4
                if index < len(items) and tree_text(items[index]) == ";":
                    index += 1
                continue
            macro = macros.get(item.text) if isinstance(item, Token) else None
            argument = index + 2
            if isinstance(item, Token) and index + 4 < len(items) and tree_text(items[index + 1]) == "::":
                path = [item.text]
                position = index + 1
                while position + 1 < len(items) and tree_text(items[position]) == "::":
                    path.append(tree_text(items[position + 1]))
                    position += 2
                macro = self.qualified.get(tuple(path))
                argument = position + 1
            if (macro and argument < len(items) and tree_text(items[argument - 1]) == "!"
                    and isinstance(items[argument], TokenGroup)):
                self.steps += 1
                if self.steps > 10_000:
                    raise item.error("macro expansion limit reached")
                definition, rules = macro
                expanded = self.invoke(item, rules, items[argument].items, (definition, self.steps))
                index = argument + 1
                result.extend(self.expand(expanded, macros, depth + 1))
                if (expanded and tree_text(expanded[0]) in {"#", "pub", "fn", "struct", "enum", "trait", "impl", "type", "const"}
                        and index < len(items) and tree_text(items[index]) == ";"):
                    index += 1
                continue
            if isinstance(item, (TokenGroup, OpaqueFragment)):
                item = replace(item, items=self.expand(item.items, macros, depth))
            result.append(item)
            index += 1
        return tuple(result)

    @staticmethod
    def rules(group):
        items = group.items
        rules = []
        index = 0
        while index < len(items):
            if index + 2 >= len(items):
                raise group.opening.error("incomplete macro rule")
            matcher, arrow, body = items[index:index + 3]
            if not isinstance(matcher, TokenGroup) or tree_text(arrow) != "=>" or not isinstance(body, TokenGroup):
                raise group.opening.error("expected a macro matcher and transcriber")
            rules.append((macro_sequence(matcher.items, matcher=True),
                          macro_sequence(body.items, matcher=False)))
            index += 3
            if index < len(items):
                if tree_text(items[index]) not in {";", ","}:
                    raise group.opening.error("expected ';' between macro rules")
                index += 1
        return rules

    def invoke(self, token, rules, items, mark):
        self.match_steps = 0
        for matcher, body in rules:
            for offset, captures in self.match(matcher, items, 0, {}, ()):
                if offset == len(items):
                    return tuple(self.render(body, captures, (), mark))
        raise token.error(f"no matching rule for {token.text}!")

    def match(self, nodes, items, pos, captures, path):
        self.match_steps += 1
        if self.match_steps > 100_000:
            token = next(flatten_trees(items), Token("macro", 1, 1))
            raise token.error("macro matcher search limit reached")
        if not nodes:
            yield pos, captures
            return
        node, *rest = nodes
        if isinstance(node, MacroRepeat):
            matches = [(pos, captures)]
            stack = [(pos, captures, 0)]
            while stack:
                cursor, stored, count = stack.pop()
                if node.count == "?" and count:
                    continue
                start = cursor
                if count and node.separator is not None:
                    if start == len(items) or tree_text(items[start]) != tree_text(node.separator):
                        continue
                    start += 1
                for end, found in self.match(node.body, items, start, stored, (*path, count)):
                    if end > start:
                        matches.append((end, found))
                        stack.append((end, found, count + 1))
            if node.count == "+":
                matches = matches[1:]
            for end, found in sorted(matches, key=lambda match: -match[0]):
                yield from self.match(rest, items, end, found, path)
            return
        if pos == len(items):
            return
        item = items[pos]
        if isinstance(node, MacroCapture):
            length = 1 if node.fragment in {"tt", "ident", "literal", "block"} else len(items) - pos
            if node.fragment == "literal" and tree_text(item) == "-":
                length = min(2, len(items) - pos)
            for end in range(pos + length, pos, -1):
                part = items[pos:end]
                if self.fragment(node.fragment, part):
                    key = (node.name.text, path)
                    if key in captures:
                        raise node.name.error(f"duplicate macro matcher binding {node.name.text!r}")
                    yield from self.match(rest, items, end, captures | {key: (node.fragment, part)}, path)
        elif isinstance(node, TokenGroup):
            if isinstance(item, TokenGroup) and node.opening.text == item.opening.text:
                for end, found in self.match(node.items, item.items, 0, captures, path):
                    if end == len(item.items):
                        yield from self.match(rest, items, pos + 1, found, path)
        elif isinstance(item, Token) and node.text == item.text:
            yield from self.match(rest, items, pos + 1, captures, path)

    @staticmethod
    def fragment(fragment, items):
        item = items[0]
        if fragment == "tt":
            return True
        if len(items) == 1 and isinstance(item, OpaqueFragment):
            return fragment == item.fragment
        if fragment == "ident":
            return isinstance(item, Token) and bool(IDENT.fullmatch(item.text))
        if fragment == "literal":
            if len(items) == 2:
                return (tree_text(item) == "-" and isinstance(items[1], Token)
                        and items[1].text[0].isdigit())
            return isinstance(item, Token) and (item.text[0].isdigit() or item.text.startswith('"')
                                               or item.text in {"true", "false"})
        if fragment == "block":
            return isinstance(item, TokenGroup) and item.opening.text == "{"
        tokens = list(flatten_trees(items))
        first = tokens[0]
        parser = Parser("", tokens=(*tokens, Token("", first.line, first.column)), fragment=True)
        try:
            if fragment == "expr":
                parser.expression()
            elif fragment == "ty":
                parser.type_name()
            elif fragment == "pat":
                parser.pattern()
            elif fragment == "path":
                parser.expression_path(parser.identifier())
            return not parser.current.text
        except RustError:
            return False

    @staticmethod
    def variables(nodes):
        for node in nodes:
            if isinstance(node, MacroCapture):
                yield node.name.text
            elif isinstance(node, MacroRepeat):
                yield from MacroExpander.variables(node.body)
            elif isinstance(node, TokenGroup):
                yield from MacroExpander.variables(node.items)

    def render(self, nodes, captures, path, mark):
        for index, node in enumerate(nodes):
            if isinstance(node, MacroCapture):
                if node.name.text == "crate":
                    following = nodes[index + 1] if index + 1 < len(nodes) else None
                    if not isinstance(following, Token) or following.text != "::":
                        raise node.name.error("expected a path after $crate")
                    yield replace(node.name, text="crate")
                    continue
                for level in range(len(path), -1, -1):
                    key = (node.name.text, path[:level])
                    if key in captures:
                        fragment, items = captures[key]
                        if fragment not in {"ident", "tt"}:
                            yield OpaqueFragment(node.name, fragment, items)
                        else:
                            yield from items
                        break
                else:
                    raise node.name.error(f"metavariable {node.name.text!r} has no value at this repetition depth")
            elif isinstance(node, TokenGroup):
                yield replace(node, items=tuple(self.render(node.items, captures, path, mark)))
            elif isinstance(node, MacroRepeat):
                names = set(self.variables(node.body))
                if not names:
                    token = next((node for node in node.body if isinstance(node, Token)), Token("macro", 1, 1))
                    raise token.error("macro repetition needs a metavariable")
                indices = {key[len(path)] for name, key in captures
                           if name in names and len(key) > len(path) and key[:len(path)] == path}
                for position, index_in_path in enumerate(sorted(indices)):
                    if position and node.separator is not None:
                        yield node.separator
                    yield from self.render(node.body, captures, (*path, index_in_path), mark)
            else:
                yield replace(node, context=(*node.context, mark))


MODULE_SEPARATOR = "§"
OPTION = MODULE_SEPARATOR.join(("core", "option", "Option"))
RESULT = MODULE_SEPARATOR.join(("core", "result", "Result"))
PRELUDE_TYPES = {"Option": OPTION, "Result": RESULT}
PRELUDE_VARIANTS = {"Some": OPTION, "None": OPTION, "Ok": RESULT, "Err": RESULT}
PRELUDE_MACROS = """
macro_rules! assert {
    ($condition:expr $(,)?) => {{ if !$condition { panic!("assertion failed"); } }};
    ($condition:expr, $format:expr $(, $argument:expr)* $(,)?) => {{
        if !$condition { panic!($format $(,$argument)*); }
    }};
}
macro_rules! assert_eq {
    ($left:expr, $right:expr $(,)?) => {{
        let left = &$left;
        let right = &$right;
        if *left != *right { panic!("assertion failed: left != right"); }
    }};
    ($left:expr, $right:expr, $format:expr $(, $argument:expr)* $(,)?) => {{
        let left = &$left;
        let right = &$right;
        if *left != *right { panic!($format $(,$argument)*); }
    }};
}
macro_rules! assert_ne {
    ($left:expr, $right:expr $(,)?) => {{
        let left = &$left;
        let right = &$right;
        if *left == *right { panic!("assertion failed: left == right"); }
    }};
}
macro_rules! matches {
    ($value:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {{
        match $value { $pattern $(if $guard)? => true, _ => false }
    }};
}
"""


@dataclass(frozen=True)
class ModuleSymbol:
    path: tuple
    kind: str
    visible_from: tuple
    token: Token

    @property
    def name(self):
        return MODULE_SEPARATOR.join(self.path)


@dataclass(frozen=True)
class Import:
    path: tuple
    alias: str
    token: Token
    visible_from: tuple


@dataclass
class ModuleUnit:
    path: tuple
    directory: Path | None
    items: tuple = ()
    symbols: dict = field(default_factory=dict)
    imports: list = field(default_factory=list)
    aliases: dict = field(default_factory=dict)
    declarations: list = field(default_factory=list)
    trait_imports: dict = field(default_factory=dict)
    glob_aliases: dict = field(default_factory=dict)
    hidden_traits: set = field(default_factory=set)


class Modules:
    def __init__(self, trees, source_path):
        directory = Path(source_path).resolve().parent if source_path is not None else None
        root = ModuleUnit((), directory)
        self.units = {(): root}
        self.exported = {}
        self.macro_paths = {}
        self.constants = {}
        self.type_aliases = {}
        self.alias_names = set()
        root.items = self.scan(root, trees)

    def register(self, unit, token, kind, boundary):
        if token.text in unit.symbols:
            raise token.error(f"duplicate item {token.text!r}")
        symbol = ModuleSymbol((*unit.path, token.text), kind, boundary, token)
        unit.symbols[token.text] = symbol
        return symbol

    @staticmethod
    def item_end(items, index, kind):
        if kind == "macro_rules":
            end = index + 4
            if end < len(items) and tree_text(items[end]) == ";":
                end += 1
            return end
        angles = 0
        for position in range(index + 1, len(items)):
            item = items[position]
            word = tree_text(item)
            if kind != "const":
                if word == "<":
                    angles += 1
                elif word in {">", ">="}:
                    angles = max(0, angles - 1)
            if angles == 0:
                if word == ";":
                    return position + 1
                if kind not in {"const", "type", "use"} and isinstance(item, TokenGroup) and word == "{":
                    return position + 1
        return len(items)

    def scan(self, unit, items, *, expanded=False):
        output = []
        index = 0
        while index < len(items):
            start = index
            kept_attributes = []
            export = False
            custom_path = None
            while index + 1 < len(items) and tree_text(items[index]) == "#":
                inner = tree_text(items[index + 1]) == "!"
                position = index + (2 if inner else 1)
                if position >= len(items) or not isinstance(items[position], TokenGroup):
                    break
                attribute = items[position]
                if not inner and len(attribute.items) == 1 and tree_text(attribute.items[0]) == "macro_export":
                    export = True
                elif (not inner and len(attribute.items) == 3
                      and tuple(map(tree_text, attribute.items[:2])) == ("path", "=")
                      and isinstance(attribute.items[2], Token)):
                    if not attribute.items[2].text.startswith(('"', 'r"', 'r#')):
                        raise attribute.items[2].error("module path must be a string literal")
                    custom_path = decode_string(attribute.items[2])
                else:
                    kept_attributes.extend(items[index:position + 1])
                index = position + 1
            if index >= len(items):
                output.extend(kept_attributes)
                break
            boundary = unit.path
            if tree_text(items[index]) == "pub":
                index += 1
                boundary = ()
                if index < len(items) and isinstance(items[index], TokenGroup) and tree_text(items[index]) == "(":
                    boundary = self.visibility(unit, items[index])
                    index += 1
            if index >= len(items):
                output.extend(items[start:])
                break
            word = tree_text(items[index])
            if word == "mod" and index + 2 < len(items):
                token, body = items[index + 1:index + 3]
                if not isinstance(token, Token):
                    raise items[index].error("expected a module name")
                symbol = self.register(unit, token, "module", boundary)
                directory = unit.directory / token.text if unit.directory is not None else None
                if custom_path is not None and unit.directory is not None:
                    directory = (unit.directory / custom_path).resolve().parent
                child = ModuleUnit(symbol.path, directory)
                self.units[symbol.path] = child
                if isinstance(body, TokenGroup) and tree_text(body) == "{":
                    contents = body.items
                elif tree_text(body) == ";":
                    contents = self.load_module(unit, token, custom_path)
                else:
                    raise token.error("expected an inline module or ';'")
                child.items = self.scan(child, contents)
                if not expanded:
                    output.append(replace(token, text="@module", child_module=symbol.path))
                index += 3
                continue
            if word == "use":
                end = self.item_end(items, index, "use")
                if end == len(items) and tree_text(items[-1]) != ";":
                    raise items[index].error("expected ';' after use")
                unit.imports.extend(self.import_tree(items[index + 1:end - 1], (), boundary))
                index = end
                continue
            if word in {"fn", "struct", "enum", "trait", "const", "type", "impl", "macro_rules"}:
                end = self.item_end(items, index, word)
                if word not in {"impl", "macro_rules"} and index + 1 < len(items):
                    name = items[index + 1]
                    if isinstance(name, Token):
                        kind = word
                        if word == "struct" and not any(isinstance(part, TokenGroup)
                                                        for part in items[index + 2:end]):
                            kind = "unit_struct"
                        original = unit.symbols.get(name.text)
                        if not expanded or not original or original.token is not name:
                            self.register(unit, name, kind, boundary)
                if word == "const" or (word == "type" and expanded):
                    unit.declarations.append(items[index:end])
                else:
                    if export and word == "macro_rules":
                        name, body = items[index + 2:index + 4]
                        if name.text in self.exported and not expanded:
                            raise name.error(f"duplicate exported macro {name.text!r}")
                        self.exported[name.text] = body
                    output.extend(kept_attributes)
                    output.extend(items[index:end])
                index = end
                continue
            output.extend(kept_attributes)
            output.append(items[index])
            index += 1
        return tuple(output)

    @staticmethod
    def load_module(unit, token, custom_path=None):
        if unit.directory is None:
            raise token.error("an external module requires a source file path")
        if custom_path is not None:
            filename = unit.directory / custom_path
            if not filename.is_file():
                raise token.error(f"module {token.text!r} needs {filename}")
            filenames = (filename,)
        else:
            filenames = (unit.directory / f"{token.text}.rs", unit.directory / token.text / "mod.rs")
        found = [path for path in filenames if path.is_file()]
        if len(found) != 1:
            if found:
                raise token.error(f"module {token.text!r} exists in both {filenames[0]} and {filenames[1]}")
            raise token.error(f"module {token.text!r} needs {filenames[0]} or {filenames[1]}")
        try:
            return token_trees(lex(found[0].read_text(encoding="utf-8")))[0]
        except RustError as error:
            raise RustError(f"{found[0]}: {error}") from error

    @staticmethod
    def visibility(unit, group):
        words = tuple(token.text for token in flatten_trees(group.items))
        if words == ("crate",):
            return ()
        if words == ("self",):
            return unit.path
        if words == ("super",) and unit.path:
            return unit.path[:-1]
        if words[:2] == ("in", "crate"):
            boundary = words[3::2]
            if words[2::2] == ("::",) * len(boundary) and unit.path[:len(boundary)] == boundary:
                return boundary
        raise group.opening.error("visibility must name an ancestor module")

    def import_tree(self, items, prefix, boundary):
        if not items:
            return []
        index = 0
        parts = list(prefix)
        while index < len(items):
            item = items[index]
            if isinstance(item, TokenGroup) and tree_text(item) == "{":
                entries = []
                start = 0
                for position, child in enumerate((*item.items, Token(",", item.closing.line, item.closing.column))):
                    if tree_text(child) == ",":
                        entries.extend(self.import_tree(item.items[start:position], tuple(parts), boundary))
                        start = position + 1
                return entries
            if not isinstance(item, Token):
                raise item.opening.error("invalid import path")
            name = item.text
            index += 1
            if name == "*":
                return [Import(tuple(parts), "*", item, boundary)]
            if name != "self" or not parts or index < len(items) and tree_text(items[index]) == "::":
                parts.append(name)
            if index < len(items) and tree_text(items[index]) == "::":
                index += 1
                continue
            alias = parts[-1]
            if index + 1 < len(items) and tree_text(items[index]) == "as":
                alias = tree_text(items[index + 1])
                index += 2
            if index != len(items):
                raise item.error("unexpected token in import")
            return [Import(tuple(parts), alias, item, boundary)]
        return []

    def resolve(self, path, unit, token, *, origin=None):
        origin = unit.path if origin is None else origin
        path = list(path)
        root = unit
        absolute = False
        if path[0] in {"crate", "self", "super"}:
            head = path.pop(0)
            absolute = True
            if head == "crate":
                root = self.units[()]
            if head == "super":
                current = unit.path
                while True:
                    if not current:
                        raise token.error("super goes past the crate root")
                    current = current[:-1]
                    if not path or path[0] != "super":
                        break
                    path.pop(0)
                root = self.units[current]
        if not path:
            return ModuleSymbol(root.path, "module", (), token)
        first = path.pop(0)
        symbol = self.lookup(root, first, origin, token)
        if not symbol:
            if absolute:
                raise token.error("unresolved path")
            return self.prelude((first, *path), token)
        while path and symbol.kind == "module":
            nested = self.units[symbol.path]
            symbol = self.lookup(nested, path.pop(0), origin, token)
            if not symbol:
                raise token.error("unresolved path")
        if not path:
            return symbol
        if len(path) == 1 and symbol.kind in {"struct", "unit_struct", "enum", "trait"}:
            return ModuleSymbol((*symbol.path[:-1], f"{symbol.path[-1]}::{path[0]}"), "associated", (), token)
        raise token.error("unresolved associated path")

    @staticmethod
    def prelude(path, token):
        head, *tail = path
        if head in {"std", "core"} and len(tail) >= 2:
            module, head, *tail = tail
            if (module, head) not in {("option", "Option"), ("result", "Result")}:
                return None
        if head in PRELUDE_TYPES and len(tail) <= 1:
            parts = tuple(PRELUDE_TYPES[head].split(MODULE_SEPARATOR))
            if tail:
                return ModuleSymbol((*parts[:-1], f"{parts[-1]}::{tail[0]}"), "associated", (), token)
            return ModuleSymbol(parts, "enum", (), token)
        if head in PRELUDE_VARIANTS and not tail:
            parts = tuple(PRELUDE_VARIANTS[head].split(MODULE_SEPARATOR))
            return ModuleSymbol((*parts[:-1], f"{parts[-1]}::{head}"), "associated", (), token)
        return None

    @staticmethod
    def lookup(unit, name, origin, token):
        symbol = unit.symbols.get(name) or unit.aliases.get(name)
        if symbol is None and name in unit.glob_aliases:
            symbol = unit.glob_aliases[name]
            if symbol is None:
                raise token.error(f"ambiguous glob import {name!r}")
        if symbol and origin[:len(symbol.visible_from)] != symbol.visible_from:
            raise token.error(f"{name!r} is private")
        return symbol

    def link_imports(self, *, strict=True):
        for unit in self.units.values():
            unit.aliases.clear()
            unit.trait_imports.clear()
            unit.glob_aliases.clear()
            unit.hidden_traits.clear()
        imports = [(unit, item) for unit in self.units.values() for item in unit.imports]
        while True:
            previous = [(dict(unit.aliases), dict(unit.glob_aliases)) for unit in self.units.values()]
            unresolved = []
            for unit, item in imports:
                if (len(item.path) == 3 and item.path[0] in {"std", "core"}
                        and TRAIT_MODULES.get(item.path[2]) == item.path[1]):
                    unit.trait_imports[item.alias] = item.path[2]
                    continue
                try:
                    target = self.resolve(item.path, unit, item.token)
                except RustError as error:
                    unresolved.append((unit, item, error))
                    continue
                if target is None:
                    if self.is_macro_import(unit, item):
                        continue
                    unresolved.append((unit, item, item.token.error(f"unresolved import {'::'.join(item.path)}")))
                    continue
                if item.alias == "*":
                    if target.kind != "module":
                        unresolved.append((unit, item, item.token.error("glob imports require a module")))
                        continue
                    module = self.units[target.path]
                    for name, member in (module.glob_aliases | module.aliases | module.symbols).items():
                        if (member and unit.path[:len(member.visible_from)] == member.visible_from
                                and item.visible_from[:len(member.visible_from)] == member.visible_from):
                            alias = replace(member, visible_from=item.visible_from)
                            if name not in unit.glob_aliases:
                                unit.glob_aliases[name] = alias
                            elif unit.glob_aliases[name] and unit.glob_aliases[name].path != alias.path:
                                unit.glob_aliases[name] = None
                elif item.alias == "_":
                    if target.kind == "trait":
                        unit.hidden_traits.add(target.name)
                else:
                    if item.visible_from[:len(target.visible_from)] != target.visible_from:
                        unresolved.append((unit, item, item.token.error(f"cannot re-export private item {item.alias!r}")))
                        continue
                    existing = unit.symbols.get(item.alias) or unit.aliases.get(item.alias)
                    if existing and existing.path != target.path:
                        raise item.token.error(f"duplicate import {item.alias!r}")
                    unit.aliases[item.alias] = replace(target, visible_from=item.visible_from)
            current = [(unit.aliases, unit.glob_aliases) for unit in self.units.values()]
            if current == previous:
                if strict:
                    for unit, item, error in unresolved:
                        if not self.is_macro_import(unit, item):
                            raise error
                return

    @staticmethod
    def macro_path(unit, path):
        head, *tail = path
        if head == "crate":
            return tuple(tail)
        if head == "self":
            return (*unit.path, *tail)
        if head == "super":
            base = unit.path[:-1]
            while tail and tail[0] == "super":
                base = base[:-1]
                tail.pop(0)
            return (*base, *tail)
        alias = unit.aliases.get(head)
        if alias and alias.kind == "module":
            return (*alias.path, *tail)
        return (*unit.path, *path)

    def is_macro_import(self, unit, item):
        entry = self.macro_paths.get(self.macro_path(unit, item.path))
        return bool(entry and unit.path[:len(entry[1])] == entry[1])

    def link_macros(self, exports):
        self.macro_paths = {(name,): (macro, ()) for name, macro in exports.items()}
        while True:
            size = len(self.macro_paths)
            for unit in self.units.values():
                for item in unit.imports:
                    if item.alias not in {"_", "*"} and self.is_macro_import(unit, item):
                        definition, visible = self.macro_paths[self.macro_path(unit, item.path)]
                        if item.visible_from[:len(visible)] != visible:
                            raise item.token.error(f"cannot re-export private macro {item.alias!r}")
                        self.macro_paths.setdefault((*unit.path, item.alias), (definition, item.visible_from))
            if len(self.macro_paths) == size:
                return

    def macro_qualifiers(self, unit):
        qualified = {}
        for path, (macro, visible) in self.macro_paths.items():
            if unit.path[:len(visible)] != visible:
                continue
            qualified[("crate", *path)] = macro
            for ancestor in range(len(unit.path), -1, -1):
                if path[:ancestor] == unit.path[:ancestor] and len(path) > ancestor:
                    prefix = ("super",) * (len(unit.path) - ancestor) or ("self",)
                    qualified[(*prefix, *path[ancestor:])] = macro
                    if ancestor == len(unit.path):
                        qualified[path[ancestor:]] = macro
            aliases = unit.glob_aliases | unit.aliases
            for local, symbol in aliases.items():
                if symbol and symbol.kind == "module" and path[:len(symbol.path)] == symbol.path:
                    qualified[(local, *path[len(symbol.path):])] = macro
        return qualified

    def name_token(self, unit, token, parts, *, bare=False, type_name=False):
        symbol = self.resolve(parts, unit, token)
        if symbol:
            if bare and len(parts) == 1 and symbol.kind not in {"unit_struct", "associated", "const"}:
                return token
            return replace(token, text=symbol.name)
        if len(parts) > 1:
            return replace(token, text="::".join(parts))
        if type_name and unit.path:
            return replace(token, text=MODULE_SEPARATOR.join((*unit.path, token.text)))
        return token

    def expand(self):
        expander = MacroExpander()
        builtin_items, _ = token_trees(lex(PRELUDE_MACROS))
        builtins = {}
        for index in range(0, len(builtin_items), 4):
            _, _, name, body = builtin_items[index:index + 4]
            expander.definitions += 1
            builtins[name.text] = (expander.definitions, expander.rules(body))
        exports = {}
        for name, body in self.exported.items():
            expander.definitions += 1
            exports[name] = (expander.definitions, expander.rules(body))
        self.link_macros(exports)
        self.link_imports(strict=False)
        self.link_macros(exports)
        pending = deque(self.units)
        visited = set()
        while pending:
            path = pending.popleft()
            if path in visited:
                continue
            visited.add(path)
            unit = self.units[path]
            macros = builtins | expander.module_macros.get(unit.path, {})
            macros.update({item.alias: self.macro_paths[self.macro_path(unit, item.path)][0]
                           for item in unit.imports if self.is_macro_import(unit, item)})
            expander.qualified = self.macro_qualifiers(unit)
            expanded = expander.expand(unit.items, macros)
            unit.items = self.scan(unit, expanded, expanded=True)
            pending.extend(path for path in self.units if path not in visited)
        self.link_imports()
        self.alias_names = {symbol.name for unit in self.units.values()
                            for symbol in unit.symbols.values() if symbol.kind == "type"}
        self.read_declarations()

    def read_declarations(self):
        pending = [(unit, segment) for unit in self.units.values() for segment in unit.declarations]
        while pending:
            remaining = []
            for unit, segment in pending:
                last = next(flatten_trees(segment))
                parser = Parser("", tokens=(*flatten_trees(segment), replace(last, text="")),
                                modules=self, unit=unit)
                try:
                    if parser.accept("const"):
                        parser.constant_item()
                    else:
                        parser.expect("type")
                        alias = parser.type_alias()
                        self.type_aliases[alias.name.text] = alias
                except RustError as error:
                    remaining.append((unit, segment, error))
            if len(remaining) == len(pending):
                raise remaining[0][2]
            pending = [(unit, segment) for unit, segment, _ in remaining]


@dataclass
class Expr:
    kind: str
    token: Token
    args: tuple = ()
    label: Token | None = None
    type_arguments: tuple | None = None
    auto_borrow: bool = False
    unsuffixed: bool = False
    qualified: tuple | None = None
    local_callee: Token | None = None


@dataclass
class Function:
    name: Token
    params: list
    result: object
    body: Expr
    return_inputs: tuple = ()
    type_parameters: tuple = ()
    associated_const: bool = False
    method_parameters: tuple = ()
    lifetimes: dict = field(default_factory=dict)
    self_sized: bool = False
    visible_from: tuple = ()
    captures: tuple = ()
    consumes_captures: bool = False


@dataclass(frozen=True)
class Constructor:
    path: str
    annotation: object
    style: str
    fields: tuple
    visibility: tuple = ()

    def index(self, token):
        for index, (name, _) in enumerate(self.fields):
            if name == token.text:
                return index
        raise token.error(f"{self.path or self.annotation} has no field {token.text!r}")


@dataclass(frozen=True)
class ReferenceType:
    inner: object
    mutable: bool
    lifetime: str | None = field(default=None, compare=False)

    def __str__(self):
        prefix = "&mut " if self.mutable else "&"
        return f"{prefix}{self.inner}"


@dataclass(frozen=True)
class FunctionType:
    parameters: tuple
    result: object
    closure: str | None = None

    def __str__(self):
        arguments = ", ".join(map(str, self.parameters))
        kind = "closure" if self.closure else "fn"
        return f"{kind}({arguments}) -> {self.result}"


@dataclass(frozen=True)
class FunctionValue:
    name: str
    arguments: dict = field(default_factory=dict)
    environment: dict = field(default_factory=dict)


@dataclass(frozen=True)
class Capture:
    name: Token
    annotation: object
    by_move: bool
    mutable: bool = False


@dataclass(frozen=True, eq=False)
class TypeParameter:
    name: str
    bounds: set = field(default_factory=set)
    const_type: str | None = None

    def __str__(self):
        return self.name


@dataclass(frozen=True, eq=False)
class TypeVariable:
    name: str
    token: Token
    const_type: str | None = None
    integer_literal: bool = False

    def __str__(self):
        return self.name


@dataclass(frozen=True)
class AppliedType:
    name: str
    arguments: tuple

    def __str__(self):
        return f"{self.name}<{', '.join(map(str, self.arguments))}>"


@dataclass(frozen=True)
class ConstValue:
    value: int | bool | Character
    kind: str | None = field(default=None, compare=False)
    boolean: bool = field(init=False)

    def __post_init__(self):
        object.__setattr__(self, "boolean", type(self.value) is bool)

    def __str__(self):
        return display(self.value, True)


@dataclass(frozen=True)
class ConstExpression:
    operator: str
    operands: tuple
    kind: str
    token: Token = field(compare=False)
    integer: str | None = None

    def __str__(self):
        if len(self.operands) == 1:
            return f"({self.operator}{self.operands[0]})"
        return f"({self.operands[0]} {self.operator} {self.operands[1]})"


def const_kind(value):
    return value.const_type if isinstance(value, (TypeParameter, TypeVariable)) else value.kind


def check_constant_kind(value, kind, token):
    if isinstance(value, (TypeParameter, TypeVariable)) and value.const_type == kind:
        return
    if isinstance(value, ConstExpression) and value.kind == kind:
        return
    if not isinstance(value, ConstValue) or (value.kind and value.kind != kind):
        raise token.error(f"expected a {kind} constant")
    if kind == "bool" and value.boolean:
        return
    if kind == "char" and isinstance(value.value, Character):
        return
    if kind in INTEGER_TYPES and type(value.value) is int:
        checked(value.value, token, kind)
        return
    raise token.error(f"expected a {kind} constant")


def fold_const(expression):
    if not all(isinstance(value, ConstValue) for value in expression.operands):
        return expression
    values = tuple(value.value for value in expression.operands)
    word, token, kind = expression.operator, expression.token, expression.kind
    if word == "&&":
        result = values[0] and values[1]
    elif word == "||":
        result = values[0] or values[1]
    elif word in COMPARE:
        result = COMPARE[word](*values)
    elif word == "as":
        source = expression.integer
        if source in INTEGER_TYPES:
            checked(values[0], token, source)
        result = convert_scalar(values[0], kind)
    elif word == "!":
        if kind in INTEGER_TYPES:
            checked(values[0], token, kind)
        result = complement(values[0], kind)
    elif len(values) == 1:
        result = checked(-values[0], token, kind)
    else:
        result = binary(word, *values, token, expression.integer or kind)
    return ConstValue(result, kind)


@dataclass(frozen=True)
class TupleType:
    elements: tuple

    def __str__(self):
        return tuple_text(map(str, self.elements))


def tuple_text(elements):
    elements = tuple(elements)
    comma = "," if len(elements) == 1 else ""
    return f"({', '.join(elements)}{comma})"


@dataclass(frozen=True)
class ArrayType:
    element: object
    length: object

    def __str__(self):
        return f"[{self.element}; {self.length}]"


@dataclass(frozen=True)
class SliceType:
    element: object

    def __str__(self):
        return f"[{self.element}]"


@dataclass(frozen=True)
class AssociatedType:
    owner: object
    item: str
    trait: str | None = None
    arguments: tuple = ()

    def __str__(self):
        suffix = f"<{', '.join(map(str, self.arguments))}>" if self.arguments else ""
        return f"{self.owner}::{self.item}{suffix}"


@dataclass(frozen=True)
class LifetimeArgument:
    name: str

    def __str__(self):
        return self.name


@dataclass(frozen=True)
class AssociatedDeclaration:
    parameters: tuple
    bounds: set


@dataclass(frozen=True)
class AssociatedDefinition:
    parameters: tuple
    body: object


@dataclass(frozen=True)
class TypeAlias:
    name: Token
    lifetimes: tuple
    parameters: tuple
    target: object

    def specialize(self, arguments, token):
        supplied_lifetimes = tuple(arg for arg in arguments if isinstance(arg, LifetimeArgument))
        if arguments[:len(supplied_lifetimes)] != supplied_lifetimes:
            raise token.error("lifetime arguments must precede type and const arguments")
        if supplied_lifetimes and len(supplied_lifetimes) != len(self.lifetimes):
            raise token.error(f"{self.name.text} expects {len(self.lifetimes)} lifetime arguments")
        types = arguments[len(supplied_lifetimes):]
        if len(types) != len(self.parameters):
            raise token.error(f"{self.name.text} expects {len(self.parameters)} type or const arguments")
        for parameter, arg in zip(self.parameters, types):
            if parameter.const_type:
                check_constant_kind(arg, parameter.const_type, token)
            elif isinstance(arg, (ConstValue, ConstExpression)) or getattr(arg, "const_type", None):
                raise token.error("expected a type argument, found a constant")
        if not supplied_lifetimes:
            supplied_lifetimes = (LifetimeArgument("'_"),) * len(self.lifetimes)
        return substitute(self.target, dict(zip((*self.lifetimes, *self.parameters), (*supplied_lifetimes, *types))))


@dataclass(frozen=True)
class DynType:
    trait: str
    bindings: tuple = ()

    def __str__(self):
        return f"dyn {self.trait}"


@dataclass(frozen=True)
class Trait:
    name: Token
    self_type: TypeParameter
    bounds: set
    methods: dict
    associated: dict


@dataclass(frozen=True)
class Implementation:
    token: Token
    target: object
    trait: str | None
    parameters: tuple
    methods: dict
    associated: dict


def match_types(template, actual, parameters, result):
    if template in parameters:
        if template in result:
            return result[template] == actual
        result[template] = actual
        return True
    if isinstance(template, AppliedType) and isinstance(actual, AppliedType):
        return (template.name == actual.name and len(template.arguments) == len(actual.arguments)
                and all(match_types(left, right, parameters, result)
                        for left, right in zip(template.arguments, actual.arguments)))
    if isinstance(template, ArrayType) and isinstance(actual, ArrayType):
        return (match_types(template.element, actual.element, parameters, result)
                and match_types(template.length, actual.length, parameters, result))
    if isinstance(template, TupleType) and isinstance(actual, TupleType):
        return len(template.elements) == len(actual.elements) and all(
            match_types(left, right, parameters, result)
            for left, right in zip(template.elements, actual.elements)
        )
    return template == actual


def overlapping_types(first, second):
    if isinstance(first, TypeParameter) or isinstance(second, TypeParameter):
        return True
    if isinstance(first, AppliedType) and isinstance(second, AppliedType):
        return (first.name == second.name and len(first.arguments) == len(second.arguments)
                and all(overlapping_types(a, b) for a, b in zip(first.arguments, second.arguments)))
    if isinstance(first, ArrayType) and isinstance(second, ArrayType):
        return (overlapping_types(first.element, second.element)
                and overlapping_types(first.length, second.length))
    if isinstance(first, TupleType) and isinstance(second, TupleType):
        return len(first.elements) == len(second.elements) and all(
            overlapping_types(a, b) for a, b in zip(first.elements, second.elements)
        )
    return first == second


TRAIT_MODULES = {
    "Clone": "clone", "Copy": "marker", "Sized": "marker", "Default": "default",
    "Debug": "fmt", "Display": "fmt", "PartialEq": "cmp", "Eq": "cmp",
    "PartialOrd": "cmp", "Ord": "cmp",
}
SUPERTRAITS = {
    "Copy": {"Clone"}, "Eq": {"PartialEq"}, "PartialOrd": {"PartialEq"},
    "Ord": {"Eq", "PartialOrd", "PartialEq"},
}
DERIVABLE = set(TRAIT_MODULES) - {"Display", "Sized"}


def substitute(annotation, arguments):
    if isinstance(annotation, ConstExpression):
        return fold_const(replace(annotation, operands=tuple(substitute(value, arguments)
                                                            for value in annotation.operands)))
    if isinstance(annotation, ReferenceType):
        lifetime = arguments.get(LifetimeArgument(annotation.lifetime), None) if annotation.lifetime else None
        return replace(annotation, inner=substitute(annotation.inner, arguments),
                       lifetime=(None if lifetime.name == "'_" else lifetime.name)
                       if lifetime else annotation.lifetime)
    if isinstance(annotation, FunctionType):
        return FunctionType(tuple(substitute(kind, arguments) for kind in annotation.parameters),
                            substitute(annotation.result, arguments), annotation.closure)
    if isinstance(annotation, AppliedType):
        return AppliedType(annotation.name, tuple(substitute(arg, arguments) for arg in annotation.arguments))
    if isinstance(annotation, TupleType):
        return TupleType(tuple(substitute(kind, arguments) for kind in annotation.elements))
    if isinstance(annotation, ArrayType):
        return ArrayType(substitute(annotation.element, arguments), substitute(annotation.length, arguments))
    if isinstance(annotation, SliceType):
        return SliceType(substitute(annotation.element, arguments))
    if isinstance(annotation, AssociatedType):
        return replace(annotation, owner=substitute(annotation.owner, arguments),
                       arguments=tuple(substitute(arg, arguments) for arg in annotation.arguments))
    if isinstance(annotation, DynType):
        return DynType(annotation.trait, tuple((name, substitute(value, arguments))
                                                for name, value in annotation.bindings))
    return arguments.get(annotation, annotation)


@dataclass
class Program:
    functions: dict = field(default_factory=dict)
    types: dict = field(default_factory=dict)
    type_aliases: dict = field(default_factory=dict)
    constructors: dict = field(default_factory=dict)
    patterns: dict = field(default_factory=dict)
    expr_types: dict = field(default_factory=dict)
    derives: dict = field(default_factory=dict)
    call_types: dict = field(default_factory=dict)
    type_parameters: dict = field(default_factory=dict)
    default_variants: dict = field(default_factory=dict)
    traits: dict = field(default_factory=dict)
    impls: list = field(default_factory=list)
    call_functions: dict = field(default_factory=dict)
    dispatches: dict = field(default_factory=dict)
    traits_in_scope: dict = field(default_factory=dict)
    values_in_scope: dict = field(default_factory=dict)

    def add_prelude(self):
        for name, shape in ((OPTION, (("None", ()), ("Some", ("T",)))),
                            (RESULT, (("Ok", ("T",)), ("Err", ("E",))))):
            letters = ("T",) if name == OPTION else ("T", "E")
            parameters = {letter: TypeParameter(letter) for letter in letters}
            constructors = tuple(
                Constructor(f"{name}::{variant}", name, "tuple" if payload else "unit",
                            tuple((str(index), parameters[letter]) for index, letter in enumerate(payload)))
                for variant, payload in shape
            )
            token = Token(name, 1, 1)
            self.types[name] = (token, constructors)
            self.type_parameters[name] = tuple(parameters.values())
            self.derives[name] = {trait: token for trait in ("Clone", "Copy", "Debug", "PartialEq",
                                                            "Eq", "PartialOrd", "Ord")}
            self.constructors.update({ctor.path: ctor for ctor in constructors})
        self.derives[OPTION]["Default"] = self.types[OPTION][0]
        self.default_variants[OPTION] = f"{OPTION}::None"

    def projection_trait(self, annotation):
        if annotation.trait:
            trait = self.traits.get(annotation.trait)
            return trait if trait and annotation.item in trait.associated else None
        if isinstance(annotation.owner, TypeParameter):
            bounds = self.trait_closure(annotation.owner.bounds)
            matches = [trait for name, trait in self.traits.items()
                       if name in bounds and annotation.item in trait.associated]
            if len(matches) == 1:
                return matches[0]
        return None

    def normalize(self, annotation, seen=()):
        if isinstance(annotation, AssociatedType):
            owner = self.normalize(annotation.owner, seen)
            annotation = replace(annotation, owner=owner,
                                 arguments=tuple(self.normalize(arg, seen) for arg in annotation.arguments))
            if isinstance(owner, DynType):
                return dict(owner.bindings).get(annotation.item, annotation)
            if annotation in seen:
                return annotation
            trait = self.projection_trait(annotation)
            if trait:
                annotation = replace(annotation, trait=trait.name.text)
                if not isinstance(owner, (TypeParameter, TypeVariable)):
                    implementation = self.implementation(owner, trait.name.text)
                    if implementation:
                        impl, arguments = implementation
                        if annotation.item in impl.associated:
                            definition = impl.associated[annotation.item]
                            if len(definition.parameters) == len(annotation.arguments):
                                arguments = arguments | dict(zip(definition.parameters, annotation.arguments))
                                return self.normalize(substitute(definition.body, arguments), (*seen, annotation))
            return annotation
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.normalize(annotation.inner, seen))
        if isinstance(annotation, FunctionType):
            return FunctionType(tuple(self.normalize(kind, seen) for kind in annotation.parameters),
                                self.normalize(annotation.result, seen), annotation.closure)
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.normalize(arg, seen) for arg in annotation.arguments))
        if isinstance(annotation, TupleType):
            return TupleType(tuple(self.normalize(kind, seen) for kind in annotation.elements))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.normalize(annotation.element, seen), annotation.length)
        if isinstance(annotation, SliceType):
            return SliceType(self.normalize(annotation.element, seen))
        return annotation

    def trait_closure(self, bounds):
        pending = list(bounds)
        result = set()
        while pending:
            trait = pending.pop()
            if trait in result:
                continue
            result.add(trait)
            pending.extend(SUPERTRAITS.get(trait, ()))
            if trait in self.traits:
                pending.extend(self.traits[trait].bounds)
        return result

    def implementation(self, annotation, trait, seen=()):
        key = (annotation, trait)
        if key in seen:
            return None
        for impl in self.impls:
            if impl.trait != trait:
                continue
            arguments = {}
            if match_types(impl.target, annotation, impl.parameters, arguments) and all(
                parameter in arguments for parameter in impl.parameters
            ) and all(
                self.implements(arguments[parameter], bound, seen=(*seen, key))
                for parameter in impl.parameters for bound in parameter.bounds
            ):
                return impl, arguments
        return None

    def layout(self, annotation):
        if isinstance(annotation, TupleType):
            fields = tuple((str(index), kind) for index, kind in enumerate(annotation.elements))
            return None, (Constructor("", annotation, "tuple", fields),)
        name = annotation.name if isinstance(annotation, AppliedType) else annotation
        declaration = self.types.get(name)
        if not declaration:
            return None
        if not isinstance(annotation, AppliedType):
            return declaration
        token, constructors = declaration
        arguments = dict(zip(self.type_parameters[name], annotation.arguments))
        return token, tuple(self.apply_constructor(ctor, annotation, arguments) for ctor in constructors)

    @staticmethod
    def apply_constructor(ctor, annotation, arguments):
        return replace(ctor, annotation=annotation,
                       fields=tuple((name, substitute(kind, arguments)) for name, kind in ctor.fields))

    def implements(self, annotation, trait, assumed=(), *, seen=()):
        if isinstance(annotation, DynType):
            return trait in self.trait_closure({annotation.trait})
        if trait == "Sized":
            return not isinstance(annotation, SliceType)
        if isinstance(annotation, FunctionType):
            if annotation.closure:
                captures = self.functions[annotation.closure].captures
                return trait in {"Copy", "Clone"} and all(
                    self.implements(capture.annotation, trait, assumed) if capture.by_move else not capture.mutable
                    for capture in captures
                )
            return trait in {"Copy", "Clone", "PartialEq", "Eq"}
        if isinstance(annotation, AssociatedType):
            normalized = self.normalize(annotation)
            if not isinstance(normalized, AssociatedType):
                return self.implements(normalized, trait, assumed, seen=seen)
            declaration = self.projection_trait(normalized)
            return bool(declaration and trait in self.trait_closure(
                declaration.associated[normalized.item].bounds
            ))
        if isinstance(annotation, TypeParameter):
            if annotation in assumed or trait in self.trait_closure(annotation.bounds):
                return True
            return trait in self.traits and self.implementation(annotation, trait, seen) is not None
        if trait in self.traits:
            return self.implementation(annotation, trait, seen) is not None
        if isinstance(annotation, ReferenceType):
            if trait in {"Copy", "Clone"}:
                return not annotation.mutable
            return trait != "Default" and self.implements(annotation.inner, trait, assumed)
        if isinstance(annotation, AppliedType):
            if trait == "Default" and annotation.name in self.default_variants:
                return True
            return (trait in self.derives.get(annotation.name, {})
                    and all(self.implements(arg, trait, assumed) for p, arg in
                            zip(self.type_parameters[annotation.name], annotation.arguments) if not p.const_type))
        if isinstance(annotation, TupleType):
            if trait == "Display" or (len(annotation.elements) > 12 and trait not in {"Copy", "Clone"}):
                return False
            return all(self.implements(kind, trait, assumed) for kind in annotation.elements)
        if isinstance(annotation, ArrayType):
            if trait == "Display":
                return False
            if trait == "Default" and (not isinstance(annotation.length, ConstValue)
                                       or annotation.length.value > 32):
                return False
            if trait == "Default" and annotation.length.value == 0:
                return True
            return self.implements(annotation.element, trait, assumed)
        if isinstance(annotation, SliceType):
            return trait in {"Debug", "PartialEq", "Eq", "PartialOrd", "Ord"} and self.implements(
                annotation.element, trait, assumed)
        if annotation in PRIMITIVES:
            return trait != "Display" or annotation != "()"
        return trait in self.derives.get(annotation, {})

    def is_copy(self, annotation):
        return self.implements(annotation, "Copy")


class Parser:
    def __init__(self, source, *, tokens=None, fragment=False, modules=None, unit=None):
        self.tokens = iter(lex(source) if tokens is None else tokens)
        self.fragment = fragment
        self.lookahead = deque()
        self.self_type = None
        self.current_trait = None
        self.sized_receiver = False
        self.modules = modules
        self.unit = unit
        self.constants = modules.constants if modules else {}
        self.current = next(self.tokens)
        self.type_parameters = {}
        self.trait_names = {name: name for name in TRAIT_MODULES if TRAIT_MODULES[name] != "fmt"}
        if unit:
            self.trait_names.update(unit.trait_imports)

    def advance(self):
        token = self.current
        if not token.text:
            raise token.error("unexpected end of input")
        self.current = self.lookahead.popleft() if self.lookahead else next(self.tokens)
        return replace(token, module=self.unit.path) if self.unit and self.unit.path else token

    def accept(self, text):
        if self.current.text == text:
            return self.advance()
        return None

    def expect(self, text):
        if text == ">" and self.current.text == ">=":
            token = self.current
            self.current = Token("=", token.line, token.column + 1)
            return Token(">", token.line, token.column)
        token = self.accept(text)
        if token is None:
            raise self.current.error(f"expected {text!r}, found {self.current.text!r}")
        return token

    def identifier(self):
        token = self.current
        if IDENT.fullmatch(token.text) and token.text not in RESERVED:
            return self.advance()
        raise token.error("expected an identifier")

    def qualified_token(self, token, parts, *, bare=False, type_name=False):
        if self.modules:
            return self.modules.name_token(self.unit, token, parts, bare=bare, type_name=type_name)
        return replace(token, text="::".join(parts))

    def declare_token(self, token):
        if self.unit:
            return replace(token, text=MODULE_SEPARATOR.join((*self.unit.path, token.text)))
        return token

    def member_visibility(self):
        public = self.accept("pub")
        if not public:
            return self.unit.path if self.unit else ()
        opening = self.accept("(")
        if not opening:
            return ()
        parts = []
        while self.current.text != ")":
            parts.append(self.advance())
        closing = self.expect(")")
        if not self.unit:
            return ()
        return Modules.visibility(self.unit, TokenGroup(opening, tuple(parts), closing))

    def type_name(self, *, static_string=False, signature=None, output=False):
        token = self.advance()
        if token.text == "_":
            return None
        if token.text == "fn":
            self.expect("(")
            arguments = []
            while self.current.text != ")":
                arguments.append(self.type_name(signature=signature))
                if not self.accept(","):
                    break
            self.expect(")")
            result = self.type_name(signature=signature) if self.accept("->") else "()"
            if static_string and isinstance(result, ReferenceType):
                raise token.error("reference-returning function pointer fields are unsupported")
            return FunctionType(tuple(arguments), result)
        if token.text == "dyn":
            trait = self.trait_path()
            bindings = []
            if self.accept("<"):
                while self.current.text != ">":
                    name = self.identifier()
                    self.expect("=")
                    bindings.append((name.text, self.type_name(signature=signature)))
                    if not self.accept(","):
                        break
                self.expect(">")
            return DynType(trait, tuple(sorted(bindings)))
        if token.text == "Self" and self.self_type is not None:
            if self.accept("::"):
                return self.associated_projection(self.self_type, self.current_trait)
            return self.self_type
        if token.text == "<":
            owner = self.type_name(signature=signature)
            self.expect("as")
            trait = self.trait_path()
            self.expect(">")
            self.expect("::")
            return self.associated_projection(owner, trait)
        if token.text in CONST_TYPES:
            return token.text
        if token.text == "[":
            element = self.type_name(static_string=static_string, signature=signature)
            if self.accept("]"):
                return SliceType(element)
            self.expect(";")
            length = self.const_expression(self.expression(), "usize")
            self.expect("]")
            return ArrayType(element, length)
        if token.text == "&":
            lifetime = self.advance() if LIFETIME.fullmatch(self.current.text) else None
            if lifetime and lifetime.text == "'_":
                lifetime = None
            mutable = bool(self.accept("mut"))
            if not mutable and self.accept("str"):
                if lifetime and lifetime.text != "'static":
                    raise lifetime.error("named lifetimes on string slices are unsupported")
                if (static_string or output) and not lifetime:
                    raise token.error("this position requires &'static str")
                return "&str"
            if static_string:
                raise token.error("references in fields are unsupported")
            if lifetime and lifetime.text == "'static" and mutable:
                raise lifetime.error("mutable 'static references are unsupported")
            if lifetime and lifetime.text != "'static" and (signature is None or lifetime.text not in signature):
                raise lifetime.error(f"undeclared or unsupported lifetime {lifetime.text}")
            return ReferenceType(self.type_name(), mutable, lifetime.text if lifetime else None)
        if token.text == "(":
            if self.accept(")"):
                return "()"
            first = self.type_name(static_string=static_string, signature=signature, output=output)
            if not self.accept(","):
                self.expect(")")
                return first
            kinds = [first]
            while self.current.text != ")":
                kinds.append(self.type_name(static_string=static_string, signature=signature, output=output))
                if not self.accept(","):
                    break
            self.expect(")")
            return TupleType(tuple(kinds))
        if token.text == "!":
            return NEVER
        if ((IDENT.fullmatch(token.text) and token.text not in RESERVED)
                or token.text in {"crate", "self", "super"}):
            if self.current.text == "::" and token.text in self.type_parameters:
                self.advance()
                return self.associated_projection(self.type_parameters[token.text])
            if token.text in self.type_parameters:
                return self.type_parameters[token.text]
            parts = [token.text]
            while self.accept("::"):
                parts.append(self.advance().text if self.current.text == "super" else self.identifier().text)
            name = self.qualified_token(token, parts, type_name=True).text
            is_alias = self.modules and name in self.modules.alias_names
            arguments = self.type_arguments(lifetimes=is_alias) if self.accept("<") else None
            if is_alias:
                alias = self.modules.type_aliases.get(name)
                if alias is None:
                    raise token.error(f"cyclic or unresolved type alias {name!r}")
                target = alias.specialize(arguments or (), token)
                if isinstance(target, ReferenceType):
                    if static_string:
                        raise token.error("references in fields are unsupported")
                    if (target.lifetime and target.lifetime != "'static"
                            and (signature is None or target.lifetime not in signature)):
                        raise token.error(f"undeclared or unsupported lifetime {target.lifetime}")
                return target
            return AppliedType(name, arguments) if arguments is not None else name
        raise token.error(f"unsupported type {token.text!r}")

    def associated_projection(self, owner, trait=None):
        name = self.identifier()
        arguments = self.type_arguments(lifetimes=True) if self.accept("<") else ()
        return AssociatedType(owner, name.text, trait, arguments)

    def parse(self, program=None):
        program = Program() if program is None else program
        while self.current.text:
            if self.accept("@macro_definition"):
                continue
            if self.current.text == "#" and self.peek() == "!":
                self.expect("#")
                self.expect("!")
                self.expect("[")
                attribute = self.identifier()
                if attribute.text == "feature":
                    self.expect("(")
                    self.expect("generic_const_exprs")
                    self.accept(",")
                    self.expect(")")
                elif attribute.text in {"allow", "warn", "deny", "forbid"}:
                    self.skip_group()
                else:
                    raise attribute.error("unsupported inner attribute")
                self.expect("]")
                continue
            if self.accept("use"):
                self.trait_import()
                continue
            if self.accept("const"):
                self.constant_item()
                continue
            derives = self.attributes()
            self.accept("pub")
            item = self.accept("struct") or self.accept("enum")
            if item:
                self.type_item(program, item, derives)
                continue
            if derives:
                raise self.current.error("derive requires a struct or enum")
            if self.accept("trait"):
                self.trait_item(program)
                continue
            if token := self.accept("impl"):
                self.impl_item(program, token)
                continue
            self.expect("fn")
            name = self.declare_token(self.identifier())
            if name.text in program.functions:
                raise name.error(f"duplicate function {name.text!r}")
            program.functions[name.text] = self.function(name)
        return program

    def function(self, name, *, method=False, declaration=False):
        outer_parameters = self.type_parameters
        lifetimes, parameters = self.generic_parameters()
        self.type_parameters = outer_parameters | parameters
        self.expect("(")
        params = []
        receiver = False
        while self.current.text != ")":
            if method and not params and self.is_self_parameter():
                param, mutable, annotation = self.self_parameter(lifetimes)
                receiver = True
            else:
                mutable = bool(self.accept("mut"))
                param = self.accept("_") or self.identifier()
                if mutable and param.text == "_":
                    raise param.error("wildcard parameters cannot be mutable")
                self.expect(":")
                annotation = self.type_name(signature=lifetimes)
            params.append((param, mutable, annotation))
            if not self.accept(","):
                break
        self.expect(")")
        result = self.type_name(signature=lifetimes, output=True) if self.accept("->") else "()"
        self.sized_receiver = False
        self.where_clause(lifetimes)
        sized_receiver = self.sized_receiver
        if receiver and isinstance(params[0][2], ReferenceType) and isinstance(result, ReferenceType) and not result.lifetime:
            return_inputs = (0,)
        else:
            return_inputs = self.return_inputs(name, params, result, lifetimes)
        body = None if declaration and self.accept(";") else self.block()
        function = Function(name, params, result, body, return_inputs, tuple(self.type_parameters.values()),
                            method_parameters=tuple(parameters.values()) if method else (),
                            lifetimes=lifetimes, self_sized=sized_receiver)
        self.type_parameters = outer_parameters
        return function

    def constant_item(self):
        name = self.declare_token(self.identifier())
        if name.text in self.constants:
            raise name.error(f"duplicate constant {name.text!r}")
        self.expect(":")
        annotation = self.type_name()
        if annotation not in CONST_TYPES:
            raise name.error("unsupported constant type")
        self.expect("=")
        value = self.const_expression(self.expression(), annotation)
        if not isinstance(value, ConstValue) or (value.kind and value.kind != annotation):
            raise name.error(f"expected a {annotation} constant")
        if ((annotation == "bool") != value.boolean
                or (annotation == "char") != isinstance(value.value, Character)):
            raise name.error(f"expected a {annotation} constant")
        if annotation in INTEGER_TYPES:
            checked(value.value, name, annotation)
        self.expect(";")
        self.constants[name.text] = ConstValue(value.value, annotation)

    def type_alias(self):
        name = self.declare_token(self.identifier())
        if name.text in PRIMITIVES:
            raise name.error(f"duplicate type {name.text!r}")
        lifetimes, parameters = self.generic_parameters()
        self.type_parameters = parameters
        self.where_clause(lifetimes)
        self.expect("=")
        target = self.type_name(signature=lifetimes, output=True)
        if isinstance(target, ReferenceType) and target.lifetime is None:
            raise name.error("a reference type alias requires an explicit lifetime")
        self.expect(";")
        return TypeAlias(name, tuple(map(LifetimeArgument, lifetimes)), tuple(parameters.values()), target)

    def peek(self, offset=0):
        while len(self.lookahead) <= offset:
            self.lookahead.append(next(self.tokens, Token("", self.current.line, self.current.column)))
        return self.lookahead[offset].text

    def is_self_parameter(self):
        if self.current.text == "self":
            return True
        if self.current.text == "mut":
            return self.peek() == "self"
        if self.current.text != "&":
            return False
        offset = int(LIFETIME.fullmatch(self.peek()) is not None)
        if self.peek(offset) == "mut":
            offset += 1
        return self.peek(offset) == "self"

    def self_parameter(self, lifetimes):
        borrowed = self.accept("&")
        lifetime = self.advance() if borrowed and LIFETIME.fullmatch(self.current.text) else None
        if lifetime and lifetime.text not in lifetimes:
            raise lifetime.error("undeclared receiver lifetime")
        mutable = bool(self.accept("mut"))
        name = self.expect("self")
        annotation = self.self_type
        if borrowed:
            annotation = ReferenceType(annotation, mutable, lifetime.text if lifetime else None)
        return name, mutable and not borrowed, annotation

    def trait_item(self, program):
        name = self.declare_token(self.identifier())
        if name.text in program.traits or name.text in TRAIT_MODULES:
            raise name.error(f"duplicate trait {name.text!r}")
        bounds = self.trait_bounds() if self.accept(":") else set()
        self_type = TypeParameter("Self", {name.text, *bounds})
        old_self = self.self_type
        self.self_type = self_type
        self.current_trait = name.text
        self.type_parameters = {"Self": self_type}
        self.expect("{")
        methods, associated = self.methods(program, declaration=True, prefix=f"@trait:{name.text}")
        self.self_type = old_self
        self.current_trait = None
        self.type_parameters = {}
        program.traits[name.text] = Trait(name, self_type, bounds, methods, associated)

    def impl_item(self, program, token):
        lifetimes, parameters = self.generic_parameters()
        if lifetimes:
            raise token.error("impl lifetime parameters are unsupported")
        self.type_parameters = parameters
        target = self.type_name()
        trait = None
        if self.accept("for"):
            if not isinstance(target, str):
                raise token.error("expected a trait name in impl")
            trait = target
            target = self.type_name()
        self.where_clause(lifetimes)
        self.expect("{")
        old_self = self.self_type
        self.self_type = target
        self.current_trait = trait
        methods, associated = self.methods(program, declaration=False, prefix=f"@impl:{len(program.impls)}")
        self.self_type = old_self
        self.current_trait = None
        program.impls.append(Implementation(token, target, trait, tuple(parameters.values()), methods, associated))
        self.type_parameters = {}

    def methods(self, program, *, declaration, prefix):
        methods = {}
        associated = {}
        while not self.accept("}"):
            visible = self.member_visibility()
            if self.accept("type"):
                name = self.identifier()
                if not self.current_trait:
                    raise name.error("associated types require a trait")
                if name.text in associated:
                    raise name.error(f"duplicate associated type {name.text!r}")
                lifetimes, parameters = self.generic_parameters()
                outer = self.type_parameters
                self.type_parameters = outer | parameters
                ordered = (*map(LifetimeArgument, lifetimes), *parameters.values())
                if declaration:
                    bounds = self.trait_bounds() if self.accept(":") else set()
                    self.associated_where(lifetimes)
                    associated[name.text] = AssociatedDeclaration(ordered, bounds)
                else:
                    self.associated_where(lifetimes)
                    self.expect("=")
                    associated[name.text] = AssociatedDefinition(ordered, self.type_name(signature=lifetimes))
                    self.associated_where(lifetimes)
                self.type_parameters = outer
                self.expect(";")
                continue
            constant = self.accept("const")
            if not constant:
                self.expect("fn")
            name = self.identifier()
            if name.text in methods:
                raise name.error(f"duplicate method {name.text!r}")
            internal = replace(name, text=f"{prefix}:{name.text}")
            if constant:
                self.expect(":")
                annotation = self.type_name()
                body = None
                if self.accept("="):
                    body = self.expression()
                    self.const_expression(body)
                elif not declaration:
                    raise name.error("associated constant needs a value")
                self.expect(";")
                function = Function(internal, [], annotation, body,
                                    type_parameters=tuple(self.type_parameters.values()), associated_const=True)
            else:
                function = self.function(internal, method=True, declaration=declaration)
            function.visible_from = visible
            methods[name.text] = function
            if function.body is not None:
                program.functions[internal.text] = function
        return methods, associated

    def associated_where(self, lifetimes):
        if not self.accept("where"):
            return
        while self.current.text not in {";", "="}:
            owner = self.advance()
            if owner.text != "Self" and owner.text not in self.type_parameters:
                raise owner.error("associated lifetime bounds require Self or a type parameter")
            self.expect(":")
            for bound in self.lifetime_bounds():
                if bound.text not in lifetimes and bound.text != "'static":
                    raise bound.error(f"undeclared lifetime {bound.text}")
            if not self.accept(","):
                break

    def generic_parameters(self):
        lifetimes = {}
        types = {}
        if not self.accept("<"):
            return lifetimes, types
        while self.current.text != ">":
            token = self.advance()
            if LIFETIME.fullmatch(token.text) and token.text not in {"'static", "'_"}:
                if types:
                    raise token.error("lifetime parameters must precede type parameters")
                if token.text in lifetimes:
                    raise token.error(f"duplicate lifetime {token.text}")
                lifetimes[token.text] = self.lifetime_bounds() if self.accept(":") else set()
            elif token.text == "const":
                name = self.identifier()
                if name.text in types:
                    raise name.error(f"duplicate generic parameter {name.text}")
                self.expect(":")
                token = self.current
                kind = self.type_name()
                if kind not in CONST_TYPES:
                    raise token.error("unsupported const parameter type")
                types[name.text] = TypeParameter(name.text, const_type=kind)
            elif IDENT.fullmatch(token.text) and token.text not in RESERVED:
                if token.text in types:
                    raise token.error(f"duplicate type parameter {token.text}")
                bounds = self.trait_bounds() if self.accept(":") else set()
                types[token.text] = TypeParameter(token.text, bounds)
            else:
                raise token.error("expected a lifetime or type parameter")
            if not self.accept(","):
                break
        self.expect(">")
        for bounds in lifetimes.values():
            self.require_lifetimes(bounds, lifetimes)
        return lifetimes, types

    def trait_path(self):
        token = self.advance() if self.current.text in {"crate", "self", "super"} else self.identifier()
        parts = [token.text]
        while self.accept("::"):
            parts.append(self.advance().text if self.current.text == "super" else self.identifier().text)
        if len(parts) == 1 and parts[0] in self.trait_names:
            return self.trait_names[parts[0]]
        if (len(parts) == 3 and parts[0] in {"std", "core"}
                and TRAIT_MODULES.get(parts[2]) == parts[1]):
            return parts[2]
        if self.modules:
            return self.qualified_token(token, parts, type_name=True).text
        if len(parts) == 1:
            return parts[0]
        raise token.error(f"unsupported trait {'::'.join(parts)!r}")

    def trait_import(self):
        trait = self.trait_path()
        alias = self.identifier().text if self.accept("as") else trait
        self.expect(";")
        self.trait_names[alias] = trait

    def trait_bounds(self):
        bounds = {self.trait_path()}
        while self.accept("+"):
            bounds.add(self.trait_path())
        return bounds

    def lifetime_bounds(self):
        bounds = set()
        while True:
            token = self.advance()
            if not LIFETIME.fullmatch(token.text):
                raise token.error("expected a lifetime bound")
            bounds.add(token)
            if not self.accept("+"):
                break
        return bounds

    @staticmethod
    def require_lifetimes(tokens, lifetimes):
        for token in tokens:
            if token.text not in lifetimes:
                raise token.error(f"undeclared or unsupported lifetime {token.text}")

    def where_clause(self, lifetimes):
        if not self.accept("where"):
            return
        while self.current.text not in {"{", ";"}:
            if self.current.text == "[":
                bound = self.type_name()
                if not isinstance(bound, ArrayType) or bound.element != "()":
                    raise self.current.error("const evaluation bounds use unit arrays")
                self.expect(":")
                if not self.accept(","):
                    break
                continue
            lhs = self.advance()
            self.expect(":")
            if LIFETIME.fullmatch(lhs.text):
                self.require_lifetimes((lhs,), lifetimes)
                bounds = self.lifetime_bounds()
                self.require_lifetimes(bounds, lifetimes)
                lifetimes[lhs.text].update(bounds)
            elif lhs.text == "Self" and self.self_type is not None:
                if self.trait_bounds() != {"Sized"}:
                    raise lhs.error("method Self bounds currently support Sized")
                self.sized_receiver = True
            elif lhs.text in self.type_parameters:
                self.type_parameters[lhs.text].bounds.update(self.trait_bounds())
            else:
                raise lhs.error(f"unknown type parameter {lhs.text}")
            if not self.accept(","):
                break

    @staticmethod
    def return_inputs(name, params, result, lifetimes):
        if not isinstance(result, ReferenceType) or result.lifetime == "'static":
            return ()
        regions = {}
        input_count = 0
        for index, (_, _, annotation) in enumerate(params):
            if isinstance(annotation, ReferenceType):
                input_count += 1
                lifetime = annotation.lifetime if annotation.lifetime else index
                regions.setdefault(lifetime, []).append(index)
            elif annotation == "&str":
                # String slices count for elision even though their lifetime
                # metadata is outside the sized-reference implementation.
                regions[index] = []
                input_count += 1
        if result.lifetime is None:
            if input_count != 1:
                raise name.error("missing lifetime for reference return")
            region = next(iter(regions))
        else:
            region = result.lifetime
        permitted = {region}
        while True:
            longer = {lifetime for lifetime, bounds in lifetimes.items()
                      if any(bound.text in permitted for bound in bounds)}
            if longer <= permitted:
                break
            permitted.update(longer)
        inputs = sorted(index for lifetime in permitted for index in regions.get(lifetime, ()))
        if not inputs:
            raise name.error("return lifetime must be tied to a reference input")
        return tuple(inputs)

    def attributes(self):
        derives = {}
        while self.accept("#"):
            self.expect("[")
            self.expect("derive")
            self.expect("(")
            while self.current.text != ")":
                trait = self.identifier()
                if trait.text not in DERIVABLE:
                    raise trait.error(f"unsupported derive {trait.text!r}")
                if trait.text in derives:
                    raise trait.error(f"duplicate derive {trait.text!r}")
                derives[trait.text] = trait
                if not self.accept(","):
                    break
            self.expect(")")
            self.expect("]")
        return derives

    def type_item(self, program, item, derives):
        name = self.declare_token(self.identifier())
        if name.text in program.types or name.text in PRIMITIVES:
            raise name.error(f"duplicate type {name.text!r}")
        lifetimes, self.type_parameters = self.generic_parameters()
        if lifetimes:
            raise name.error("lifetime parameters on types are unsupported")
        self.where_clause(lifetimes)
        constructors = []
        if item.text == "struct":
            constructor = self.constructor(name.text, name.text)
            if constructor.style != "record":
                self.where_clause(lifetimes)
                self.expect(";")
            constructors.append(constructor)
        else:
            self.expect("{")
            variants = set()
            while self.current.text != "}":
                marker = self.accept("#")
                if marker:
                    self.expect("[")
                    self.expect("default")
                    self.expect("]")
                variant = self.identifier()
                if variant.text in variants:
                    raise variant.error(f"duplicate variant {variant.text!r}")
                variants.add(variant.text)
                ctor = self.constructor(name.text, f"{name.text}::{variant.text}")
                if marker:
                    if "Default" not in derives or ctor.style != "unit":
                        raise marker.error("#[default] requires a unit variant on an enum deriving Default")
                    if name.text in program.default_variants:
                        raise marker.error("only one enum variant can be default")
                    program.default_variants[name.text] = ctor.path
                constructors.append(ctor)
                if not self.accept(","):
                    break
            self.expect("}")
        program.types[name.text] = (name, tuple(constructors))
        program.type_parameters[name.text] = tuple(self.type_parameters.values())
        self.type_parameters = {}
        program.derives[name.text] = derives
        for constructor in constructors:
            program.constructors[constructor.path] = constructor

    def constructor(self, annotation, path):
        fields = []
        visibility = []
        variant = "::" in path
        if self.accept("{"):
            names = set()
            while self.current.text != "}":
                visible = self.field_visibility(variant)
                name = self.identifier()
                if name.text in names:
                    raise name.error(f"duplicate field {name.text!r}")
                names.add(name.text)
                self.expect(":")
                fields.append((name.text, self.type_name(static_string=True)))
                visibility.append((name.text, visible))
                if not self.accept(","):
                    break
            self.expect("}")
            return Constructor(path, annotation, "record", tuple(fields), tuple(visibility))
        if self.accept("("):
            while self.current.text != ")":
                visible = self.field_visibility(variant)
                name = str(len(fields))
                fields.append((name, self.type_name(static_string=True)))
                visibility.append((name, visible))
                if not self.accept(","):
                    break
            self.expect(")")
            return Constructor(path, annotation, "tuple", tuple(fields), tuple(visibility))
        return Constructor(path, annotation, "unit", ())

    def field_visibility(self, variant):
        if variant:
            if self.current.text == "pub":
                raise self.current.error("enum variant fields inherit visibility")
            return ()
        return self.member_visibility()

    def block(self):
        token = self.expect("{")
        statements = []
        tail = None
        while self.current.text != "}":
            if marker := self.accept("@macro_definition"):
                statements.append(Expr("macro_definition", marker))
                continue
            if self.accept(";"):
                continue
            let = self.accept("let")
            if let:
                pattern = self.pattern()
                annotation = None
                if self.accept(":") and not self.accept("_"):
                    annotation = self.type_name()
                value = self.expression() if self.accept("=") else None
                otherwise = self.block() if value and self.accept("else") else None
                if value is None and (annotation is None or pattern.kind != "bind"):
                    raise pattern.token.error("uninitialized let requires a name and explicit type")
                if otherwise and value.kind in BLOCK_EXPRESSIONS:
                    raise value.token.error("parenthesize this expression before let-else")
                if otherwise and value.kind == "binary" and value.token.text in {"&&", "||"}:
                    raise value.token.error("parenthesize the boolean expression before let-else")
                self.expect(";")
                statements.append(Expr("let", pattern.token, (pattern, annotation, value, otherwise)))
                continue
            value = self.expression(statement=True)
            if self.accept(";"):
                statements.append(Expr("discard", value.token, (value, False)))
            elif self.current.text == "}":
                tail = value
                break
            elif value.kind in BLOCK_EXPRESSIONS:
                statements.append(Expr("discard", value.token, (value, True)))
            else:
                raise self.current.error("expected ';' or '}'")
        self.expect("}")
        return Expr("block", token, (statements, tail))

    def expression(self, minimum=1, *, allow_record=True, statement=False):
        left = self.prefix(allow_record)
        while self.current.text in {".", "[", "?", "("}:
            if statement and left.kind in BLOCK_EXPRESSIONS and self.current.text in {"[", "("}:
                return left
            if self.current.text == "(":
                left = Expr("indirect_call", left.token, (left, *self.arguments()))
                continue
            if question := self.accept("?"):
                left = Expr("try", question, (left,))
                continue
            if opening := self.accept("["):
                index = None if self.current.text in {"..", "..="} else self.expression()
                separator = self.accept("..") or self.accept("..=")
                if separator:
                    stop = None if self.current.text == "]" else self.expression()
                    if separator.text == "..=" and stop is None:
                        raise separator.error("inclusive range requires an end")
                    left = Expr("range_index", opening, (left, index, stop, separator.text == "..="))
                else:
                    left = Expr("index", opening, (left, index))
                self.expect("]")
                continue
            dot = self.expect(".")
            name = self.advance()
            if not (IDENT.fullmatch(name.text) or re.fullmatch(r'0|[1-9][0-9]*', name.text)):
                raise name.error("expected a field name or tuple field index")
            type_arguments = None
            if self.accept("::"):
                self.expect("<")
                type_arguments = self.type_arguments()
            if self.current.text == "(":
                left = Expr("method", name, (left, *self.arguments()), type_arguments=type_arguments)
            else:
                if type_arguments is not None:
                    raise name.error("explicit method arguments require a call")
                left = Expr("field", dot, (left, name))
        if statement and left.kind in BLOCK_EXPRESSIONS:
            return left
        while minimum <= PRECEDENCE["*"] and (cast := self.accept("as")):
            left = Expr("cast", cast, (left, self.type_name()))
        compared = False
        while True:
            word, shift = self.binary_operator()
            precedence = PRECEDENCE.get(word, 0)
            if precedence < minimum:
                break
            token = self.advance()
            if shift:
                self.advance()
                token = replace(token, text=word)
            if precedence == 4:
                if compared:
                    raise token.error("comparison operators cannot be chained")
                compared = True
            right = self.expression(
                precedence if precedence == 1 else precedence + 1, allow_record=allow_record
            )
            left = Expr("assign" if precedence == 1 else "binary", token, (left, right))
        return left

    def binary_operator(self):
        first = self.current
        if first.text in {"<", ">"} and self.peek() in {first.text, first.text + "="}:
            second = self.lookahead[0]
            if first.line == second.line and first.column + 1 == second.column:
                return first.text + second.text, True
        return first.text, False

    def skip_group(self):
        close = DELIMITERS[self.advance().text]
        while self.current.text != close:
            if self.current.text in DELIMITERS:
                self.skip_group()
            else:
                self.advance()
        self.expect(close)

    def arguments(self, opening="("):
        self.expect(opening)
        close = DELIMITERS[opening]
        args = []
        while self.current.text != close:
            args.append(self.expression())
            if not self.accept(","):
                break
        self.expect(close)
        return tuple(args)

    def prefix(self, allow_record=True):
        if self.current.text == "{":
            return self.block()
        token = self.advance()
        word = token.text
        if word in {"|", "||"} or word == "move" and self.current.text in {"|", "||"}:
            return self.closure(token, self.advance().text if word == "move" else word)
        if word == "<":
            owner = self.type_name()
            self.expect("as")
            trait = self.trait_path()
            self.expect(">")
            self.expect("::")
            member = self.identifier()
            if self.current.text == "(":
                return Expr("call", member, self.arguments(), qualified=(owner, trait))
            return Expr("name", member, qualified=(owner, trait))
        if word == "Self" and self.self_type is not None:
            if self.current.text != "::":
                owner = self.self_type
                arguments = owner.arguments if isinstance(owner, AppliedType) else None
                name = owner.name if isinstance(owner, AppliedType) else owner
                if not isinstance(name, str):
                    raise token.error("abstract Self is not a constructor")
                path = replace(token, text=name)
                if self.current.text == "(":
                    return Expr("call", path, self.arguments(), type_arguments=arguments)
                if allow_record and self.current.text == "{":
                    return Expr("record", path, (self.record_fields(self.expression, "name")[0],),
                                type_arguments=arguments)
                return Expr("name", path, type_arguments=arguments)
            self.expect("::")
            member = self.identifier()
            if self.current.text == "(":
                return Expr("call", member, self.arguments(), qualified=(self.self_type, self.current_trait))
            return Expr("name", member, qualified=(self.self_type, self.current_trait))
        if LIFETIME.fullmatch(word):
            if word in {"'static", "'_"} or word[1:] in RESERVED:
                raise token.error("invalid label")
            self.expect(":")
            if self.current.text not in {"{", "loop", "while", "for"}:
                raise token.error("a label must precede a block or loop")
            labeled = self.prefix(allow_record)
            labeled.label = token
            return labeled
        if word == "if":
            pattern, condition = self.condition()
            yes = self.block()
            no = None
            if self.accept("else"):
                no = self.prefix() if self.current.text == "if" else self.block()
            if pattern:
                return self.conditional_match(token, condition, pattern, yes, no)
            return Expr("if", token, (condition, yes, no))
        if word == "while":
            pattern, condition = self.condition()
            body = self.block()
            if pattern:
                return Expr("while_let", token, (condition, pattern, body))
            return Expr("while", token, (condition, body))
        if word == "loop":
            return Expr("loop", token, (self.block(),))
        if word == "for":
            mutable = bool(self.accept("mut"))
            name = self.identifier()
            self.expect("in")
            start = self.expression()
            inclusive = bool(self.accept("..="))
            if not inclusive:
                self.expect("..")
            end = self.expression(allow_record=False)
            return Expr("for", token, (name, mutable, start, end, inclusive, self.block()))
        if word == "match":
            return self.match_expression(token)
        if word in {"return", "break", "continue"}:
            label = None
            if word != "return" and LIFETIME.fullmatch(self.current.text):
                label = self.advance()
            value = None
            if word != "continue" and self.current.text not in {";", "}", ")", "]", ",", "=>", "else"}:
                value = self.expression(allow_record=allow_record)
            return Expr(word, token, (value,), label)
        if word in {"-", "!"}:
            return Expr("unary", token, (self.expression(PRECEDENCE["*"] + 1, allow_record=allow_record),))
        if word in {"&", "*"}:
            mutable = bool(self.accept("mut")) if word == "&" else False
            operand = self.expression(PRECEDENCE["*"] + 1, allow_record=allow_record)
            return Expr("borrow" if word == "&" else "deref", token, (operand, mutable))
        if word == "(":
            if self.accept(")"):
                return Expr("literal", token, ("()", None))
            value = self.expression()
            if self.accept(","):
                items = [value]
                while self.current.text != ")":
                    items.append(self.expression())
                    if not self.accept(","):
                        break
                self.expect(")")
                return Expr("tuple", token, tuple(items))
            self.expect(")")
            return Expr("group", token, (value,))
        if word == "[":
            if self.accept("]"):
                return Expr("array", token)
            first = self.expression()
            if self.accept(";"):
                count = self.const_expression(self.expression(), "usize")
                self.expect("]")
                return Expr("array_repeat", token, (first, count))
            elements = [first]
            while self.accept(",") and self.current.text != "]":
                elements.append(self.expression())
            self.expect("]")
            return Expr("array", token, tuple(elements))
        if (parameter := self.type_parameters.get(word)) and parameter.const_type:
            return Expr("const_value", token, (parameter,))
        if word in self.constants:
            constant = self.constants[word]
            return Expr("literal", token, (constant.kind, constant.value))
        literal = self.literal(token)
        if literal:
            return literal
        if word == "self" and self.current.text != "::" and (self.self_type is not None or self.fragment):
            return Expr("name", token)
        if (IDENT.fullmatch(word) and word not in RESERVED) or word in {"crate", "self", "super"}:
            if self.accept("!"):
                opening = self.current.text
                if opening not in DELIMITERS:
                    raise self.current.error("expected a macro delimiter")
                if word in {"panic", "unreachable", "unimplemented", "todo"}:
                    return Expr("panic", token, self.arguments(opening))
                if word != "println":
                    if not self.fragment:
                        raise token.error(f"unknown macro {word!r}")
                    self.skip_group()
                    return Expr("literal", token, ("i32", 0))
                return Expr("print", token, self.arguments(opening))
            local = token if self.current.text != "::" else None
            path, type_arguments = self.expression_path(token, allow_record=allow_record)
            if path.text in SCALAR_CONSTANTS:
                return Expr("literal", path, SCALAR_CONSTANTS[path.text])
            if path.text in self.constants:
                constant = self.constants[path.text]
                return Expr("literal", path, (constant.kind, constant.value))
            if allow_record and self.current.text == "{":
                return Expr("record", path, (self.record_fields(self.expression, "name")[0],),
                            type_arguments=type_arguments)
            if self.current.text == "(":
                return Expr("call", path, self.arguments(), type_arguments=type_arguments,
                            local_callee=local)
            owner, separator, member = path.text.partition("::")
            if separator and owner in self.type_parameters:
                return Expr("name", replace(path, text=member), qualified=(self.type_parameters[owner], None))
            return Expr("name", path, type_arguments=type_arguments)
        raise token.error(f"unexpected token {word!r}")

    def closure(self, token, opening):
        params = []
        if opening == "|":
            while self.current.text != "|":
                mutable = bool(self.accept("mut"))
                name = self.accept("_") or self.identifier()
                annotation = self.type_name() if self.accept(":") else None
                params.append((name, mutable, annotation))
                if not self.accept(","):
                    break
            self.expect("|")
        result = self.type_name() if self.accept("->") else None
        body = self.block() if result is not None else self.expression()
        return Expr("closure", token, (params, result, body, token.text == "move"))

    @staticmethod
    def literal(token, negative=False):
        word = token.text
        if word and word[0].isdigit():
            suffix = INTEGER_SUFFIX.search(word)
            kind = suffix.group() if suffix else "i32"
            digits = (word[:suffix.start()] if suffix else word).replace("_", "")
            base = {"0b": 2, "0o": 8, "0x": 16}.get(digits[:2], 10)
            if base != 10:
                digits = digits[2:]
            if not digits:
                raise token.error("expected digits after the integer base prefix")
            digits = digits.lstrip("0") or "0"
            if len(digits) > 128 or int(digits, base) >= 1 << 128:
                raise token.error("integer literal out of range")
            value = int(digits, base)
            return Expr("literal", token, (kind, -value if negative else value), unsuffixed=suffix is None)
        if negative:
            raise token.error("expected an integer after '-'")
        if word in {"true", "false"}:
            return Expr("literal", token, ("bool", word == "true"))
        if word.startswith(('"', 'r"', 'r#')):
            return Expr("literal", token, ("&str", decode_string(token)))
        if word.startswith(('b"', 'br"', 'br#')):
            value = decode_string(replace(token, text=word[1:]), byte=True)
            return Expr("byte_string", token, tuple(map(ord, value)))
        if word.startswith("b'"):
            value = decode_string(replace(token, text=word[1:]), byte=True)
            return Expr("literal", token, ("u8", ord(value)))
        if len(word) > 2 and word.startswith("'") and word.endswith("'"):
            return Expr("literal", token, ("char", Character(decode_string(token))))
        return None

    def expression_path(self, token, *, allow_record=True):
        parts = [token.text]
        arguments = None
        while self.accept("::"):
            if self.accept("<"):
                if arguments is not None:
                    raise token.error("type arguments may appear only once in the path")
                arguments = self.type_arguments()
            else:
                member = self.advance() if self.current.text == "super" else self.identifier()
                parts.append(member.text)
        if parts[0] in self.type_parameters and len(parts) > 1:
            return replace(token, text="::".join(parts)), arguments
        starts_constructor = self.current.text == "(" or (allow_record and self.current.text == "{")
        path = self.qualified_token(token, parts, bare=not starts_constructor and arguments is None)
        return path, arguments

    def type_arguments(self, *, lifetimes=False):
        arguments = []
        while self.current.text not in {">", ">="}:
            word = self.current.text
            parameter = self.type_parameters.get(word)
            if self.accept("_"):
                arguments.append(None)
            elif lifetimes and LIFETIME.fullmatch(word):
                arguments.append(LifetimeArgument(self.advance().text))
            elif (word in {"{", "-", "true", "false"} or self.unqualified_constant()
                  or word and word[0].isdigit()
                  or word.endswith("'") and word.startswith(("'", "b'"))
                  or (parameter and parameter.const_type)):
                arguments.append(self.const_argument())
            else:
                arguments.append(self.type_name())
            if not self.accept(","):
                break
        self.expect(">")
        return tuple(arguments)

    def unqualified_constant(self):
        token = self.current
        if self.constants and IDENT.fullmatch(token.text) and self.peek() != "::":
            path = self.qualified_token(token, (token.text,), bare=True)
            return path.text in self.constants
        return False

    def const_argument(self):
        if self.accept("{"):
            value = self.const_expression(self.expression())
            self.expect("}")
            return value
        negative = self.accept("-")
        token = self.advance()
        if not negative and self.constants and IDENT.fullmatch(token.text):
            path = self.qualified_token(token, (token.text,), bare=True)
            if path.text in self.constants:
                return self.constants[path.text]
        parameter = self.type_parameters.get(token.text)
        if parameter and parameter.const_type and not negative:
            return parameter
        literal = self.literal(token, negative=bool(negative))
        if literal is None or literal.args[0] not in CONST_TYPES:
            raise token.error("expected a constant value or parameter")
        kind, value = literal.args
        return ConstValue(value, None if literal.unsuffixed else kind)

    def const_expression(self, expr, hint=None):
        expr = ungroup(expr)
        if expr.kind == "const_value":
            return expr.args[0]
        if expr.kind == "literal" and expr.args[0] in CONST_TYPES:
            kind, value = expr.args
            return ConstValue(value, None if expr.unsuffixed else kind)
        if expr.kind == "cast":
            operand, target = expr.args
            if target not in SCALAR_BOUNDS:
                raise expr.token.error("constant cast requires an integer or char target")
            preferred = "u8" if target == "char" else target
            value = self.const_expression(operand, preferred)
            actual = const_kind(value) or preferred
            if not scalar_cast_supported(actual, target):
                raise expr.token.error(f"cannot cast {actual} to {target}")
            return fold_const(ConstExpression("as", (value,), target, expr.token, actual))
        if expr.kind == "unary" and expr.token.text == "!":
            value = self.const_expression(expr.args[0], hint)
            kind = const_kind(value) or hint or "i32"
            if kind != "bool" and kind not in INTEGER_TYPES:
                raise expr.token.error(f"cannot negate {kind} with '!'")
            return fold_const(ConstExpression("!", (value,), kind, expr.token, kind))
        if expr.kind == "unary" and expr.token.text == "-":
            value = self.const_expression(expr.args[0], hint)
            kind = const_kind(value) or hint
            if kind in INTEGER_TYPES and INTEGER_BOUNDS[kind][0] == 0:
                raise expr.token.error(f"cannot negate {kind} with '-'")
            if isinstance(value, ConstValue) and type(value.value) is int:
                result = -value.value
                if kind in INTEGER_TYPES:
                    checked(result, expr.token, kind)
                return ConstValue(result, value.kind)
            if kind in INTEGER_TYPES and kind.startswith("i"):
                return ConstExpression("-", (value,), kind, expr.token)
        if expr.kind == "binary":
            left, right = (self.const_expression(arg, hint) for arg in expr.args)
            word = expr.token.text
            left_kind, right_kind = const_kind(left), const_kind(right)
            if word in SHIFTS:
                kind = left_kind or (hint if hint in INTEGER_TYPES else "i32")
                if kind not in INTEGER_TYPES or (right_kind and right_kind not in INTEGER_TYPES):
                    raise expr.token.error("shift operands must be integers")
                return fold_const(ConstExpression(word, (left, right), kind, expr.token, kind))
            if not all(isinstance(arg, ConstValue) for arg in (left, right)):
                kind = left_kind or right_kind or hint or "i32"
                if left_kind and right_kind and left_kind != right_kind:
                    raise expr.token.error("constant operands have different types")
                if kind == "char" and (left_kind != kind or right_kind != kind):
                    raise expr.token.error("character comparison requires two characters")
                if word in COMPARE or (kind == "bool" and word in {"&&", "||", *BITWISE}):
                    return ConstExpression(word, (left, right), "bool", expr.token, kind)
                if kind in INTEGER_TYPES and word in {*ARITHMETIC, "/", "%"}:
                    return ConstExpression(word, (left, right), kind, expr.token, kind)
            if all(isinstance(arg, ConstValue) for arg in (left, right)):
                if left.boolean and right.boolean and word in BITWISE:
                    return ConstValue(BITWISE[word](left.value, right.value), "bool")
                if left.boolean and right.boolean and word in {"&&", "||"}:
                    return ConstValue((left.value and right.value) if word == "&&"
                                      else (left.value or right.value), "bool")
                if word in COMPARE and left.boolean == right.boolean and (
                    not left.kind or not right.kind or left.kind == right.kind
                ):
                    if isinstance(left.value, Character) != isinstance(right.value, Character):
                        raise expr.token.error("character comparison requires two characters")
                    if type(left.value) is int:
                        kind = left.kind or right.kind or (hint if hint in INTEGER_TYPES else "i32")
                        checked(left.value, expr.token, kind)
                        checked(right.value, expr.token, kind)
                    return ConstValue(COMPARE[word](left.value, right.value), "bool")
                kind = left.kind or right.kind
                if (type(left.value) is int and type(right.value) is int and word in {*ARITHMETIC, "/", "%"}
                        and (not left.kind or not right.kind or left.kind == right.kind)):
                    return ConstValue(binary(word, left.value, right.value, expr.token,
                                             kind or hint or "i32"), kind)
        raise expr.token.error("unsupported constant expression")

    def record_fields(self, parse_value, shorthand):
        self.expect("{")
        fields = []
        rest = False
        while self.current.text != "}":
            if shorthand == "bind" and self.accept(".."):
                rest = True
                break
            mutable = shorthand == "bind" and bool(self.accept("mut"))
            name = self.identifier()
            if self.accept(":"):
                if mutable:
                    raise name.error("put 'mut' on the binding after ':'")
                value = parse_value()
            else:
                args = (mutable,) if shorthand == "bind" else ()
                value = Expr(shorthand, name, args)
            fields.append((name, value))
            if not self.accept(","):
                break
        self.expect("}")
        return tuple(fields), rest

    def pattern(self, *, array_item=False):
        first = self.range_pattern(array_item=array_item)
        alternatives = [first]
        while self.accept("|"):
            alternatives.append(self.range_pattern(array_item=array_item))
        if len(alternatives) > 1:
            return Expr("or", first.token, tuple(alternatives))
        return first

    def range_pattern(self, *, array_item=False):
        first = None if self.current.text in {"..", "..="} else self.simple_pattern()
        separator = self.accept("..") or self.accept("..=")
        if separator is None:
            return first
        if first is not None and first.kind != "literal":
            raise first.token.error("expected an integer range bound")
        last = None
        if self.current.text not in {",", ")", "]", "}", "=>", "if", "|", "=", ":"}:
            last = self.simple_pattern()
        if last is not None and last.kind != "literal":
            raise last.token.error("expected an integer range bound")
        if last is None and first is None and separator.text == "..":
            return Expr("rest", separator)
        if last is None and (first is None or separator.text == "..="):
            raise separator.error("expected an integer range bound")
        if array_item and last is None:
            raise separator.error("open-ended ranges in array patterns require parentheses")
        return Expr("pattern_range", separator, (first, last, separator.text == "..="))

    def simple_pattern(self):
        token = self.advance()
        if token.text == "_":
            return Expr("wild", token)
        if token.text == "(":
            if self.accept(")"):
                return Expr("literal", token, ("()", None))
            inner = self.pattern()
            if self.accept(","):
                items = [inner]
                while self.current.text != ")":
                    items.append(self.pattern())
                    if not self.accept(","):
                        break
                self.expect(")")
                return Expr("pattern_anonymous_tuple", token, tuple(items))
            self.expect(")")
            if inner.kind == "rest":
                return Expr("pattern_anonymous_tuple", token, (inner,))
            return inner
        if token.text == "[":
            items = []
            while self.current.text != "]":
                items.append(self.pattern(array_item=True))
                if not self.accept(","):
                    break
            self.expect("]")
            return Expr("pattern_array", token, tuple(items))
        literal = self.literal(self.advance(), negative=True) if token.text == "-" else self.literal(token)
        if literal:
            return literal
        if token.text == "mut":
            return Expr("bind", self.identifier(), (True,))
        if token.text not in {"crate", "self", "super"} and (
            not IDENT.fullmatch(token.text) or token.text in RESERVED
        ):
            raise token.error("expected a pattern")
        path, type_arguments = self.expression_path(token)
        if path.text in SCALAR_CONSTANTS:
            return Expr("literal", path, SCALAR_CONSTANTS[path.text])
        if path.text in self.constants:
            constant = self.constants[path.text]
            return Expr("literal", path, (constant.kind, constant.value))
        if self.current.text == "{":
            return Expr("pattern_record", path, self.record_fields(self.pattern, "bind"),
                        type_arguments=type_arguments)
        if self.accept("("):
            patterns = []
            while self.current.text != ")":
                patterns.append(self.pattern())
                if not self.accept(","):
                    break
            self.expect(")")
            return Expr("pattern_tuple", path, tuple(patterns), type_arguments=type_arguments)
        if path != token or type_arguments is not None:
            return Expr("pattern_unit", path, type_arguments=type_arguments)
        return Expr("bind", token, (False,))

    def match_expression(self, token):
        scrutinee = self.expression(allow_record=False)
        self.expect("{")
        arms = []
        while self.current.text != "}":
            pattern = self.pattern()
            guard = self.expression(allow_record=False) if self.accept("if") else None
            self.expect("=>")
            body = self.expression()
            arms.append((pattern, guard, body))
            if not self.accept(",") and body.kind not in BLOCK_EXPRESSIONS:
                break
        self.expect("}")
        return Expr("match", token, (scrutinee, tuple(arms)))

    def condition(self):
        pattern = None
        if self.accept("let"):
            pattern = self.pattern()
            self.expect("=")
        return pattern, self.expression(allow_record=False)

    @staticmethod
    def conditional_match(token, scrutinee, pattern, yes, no):
        fallback = no if no is not None else Expr("literal", token, ("()", None))
        arms = ((pattern, None, yes), (Expr("wild", token), None, fallback))
        return Expr("match", token, (scrutinee, arms))


ESCAPES = {'n': '\n', 'r': '\r', 't': '\t', '0': '\0', '"': '"', "'": "'", '\\': '\\'}
STRING_ESCAPE = re.compile(r'\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]{1,12}\}|\n\s*|.)')


def decode_string(token, *, byte=False):
    if token.text.startswith(('r"', 'r#')):
        opening = token.text.index('"')
        source = token.text[opening + 1:-opening]
        if "\r" in source or (byte and not source.isascii()):
            raise token.error("invalid character in raw string literal")
        return source
    source = token.text[1:-1]
    if byte and not source.isascii():
        raise token.error("non-ASCII character in byte literal")
    parts = []
    start = 0
    while (slash := source.find("\\", start)) != -1:
        parts.append(source[start:slash])
        match = STRING_ESCAPE.match(source, slash)
        if not match:
            raise token.error("invalid string escape")
        escape = match.group()[1:]
        if escape in ESCAPES:
            parts.append(ESCAPES[escape])
        elif escape.startswith("\n"):
            pass
        elif len(escape) == 3 and escape[0] == "x" and (byte or int(escape[1:], 16) < 128):
            parts.append(chr(int(escape[1:], 16)))
        elif escape.startswith("u{") and not byte:
            digits = escape[2:-1].rstrip("_").replace("_", "")
            codepoint = int(digits, 16) if digits else -1
            if not (1 <= len(digits) <= 6 and 0 <= codepoint < CHAR_LIMIT
                    and codepoint not in SURROGATES
                    and escape[2] != "_"):
                raise token.error("invalid Unicode escape")
            parts.append(chr(codepoint))
        else:
            raise token.error("invalid string escape")
        start = match.end()
    parts.append(source[start:])
    return "".join(parts)


@dataclass
class Binding:
    value: object
    mutable: bool


@dataclass
class Scope:
    parent: Scope | None = None
    bindings: dict = field(default_factory=dict)
    owned: set = field(default_factory=set)
    definitions: dict | None = None
    captures: CaptureContext | None = None

    def __post_init__(self):
        self.definitions = self.parent.definitions if self.parent else {}
        if self.parent:
            self.captures = self.parent.captures

    def remember(self, token):
        frames = []
        scope = self
        while scope:
            frames.append(scope.bindings)
            scope = scope.parent
        snapshot = {}
        for bindings in reversed(frames):
            snapshot.update(bindings)
        self.definitions[token.definition] = snapshot

    def find(self, token):
        scope = self
        while scope:
            if token.key in scope.bindings:
                return scope.bindings[token.key]
            scope = scope.parent
        context = token.context
        while context:
            definition, _ = context[-1]
            context = context[:-1]
            key = (token.text, context) if context else token.text
            snapshot = self.definitions.get(definition, {})
            if key in snapshot:
                return snapshot[key]
        if self.captures:
            return self.captures.find(token)
        raise token.error(f"unknown binding {token.text!r}")


@dataclass
class CaptureContext:
    enclosing: Scope
    by_move: bool
    captures: dict = field(default_factory=dict)

    def find(self, token):
        binding = self.enclosing.find(token)
        self.captures.setdefault(token.key, Capture(token, binding.value, self.by_move))
        return Binding(binding.value, binding.mutable)

    @staticmethod
    def mark_mutable(scope, expr):
        root = place_root(expr)
        while root.kind == "deref":
            root = place_root(root.args[0])
        if root.kind != "name" or scope.captures is None:
            return
        current = scope
        while current:
            if root.token.key in current.bindings:
                return
            current = current.parent
        old = scope.captures.captures.get(root.token.key)
        if old:
            scope.captures.captures[root.token.key] = replace(old, mutable=True)


def require(actual, expected, token):
    if actual not in {expected, NEVER}:
        raise token.error(f"expected {expected}, found {actual}")


def ungroup(expr):
    while expr.kind == "group":
        expr = expr.args[0]
    return expr


def place_root(expr):
    root = ungroup(expr)
    while root.kind in {"field", "index", "range_index"}:
        root = ungroup(root.args[0])
    return root


def reference_source(expr):
    expr = ungroup(expr)
    return expr.kind in {"name", "borrow", "call", "indirect_call", "if", "match", "dyn_reference",
                         "slice_reference", "byte_string"} or (
        expr.kind == "block" and not expr.label
    )


def shared_coercion(actual, expected):
    return (isinstance(actual, ReferenceType) and actual.mutable
            and isinstance(expected, ReferenceType) and not expected.mutable
            and actual.inner == expected.inner)


@dataclass(frozen=True)
class Address:
    binding: Binding
    fields: tuple = ()

    def child(self, field):
        return Address(self.binding, (*self.fields, field))

    def read(self):
        value = self.binding.value
        for name in self.fields:
            value = value.at(name)
        return value

    def write(self, value):
        if self.fields:
            value = self.binding.value.replace(self.fields, value)
        self.binding.value = value


@dataclass(frozen=True)
class SliceAddress:
    address: Address
    start: int
    stop: int

    @property
    def length(self):
        return self.stop - self.start

    def child(self, token):
        index = int(token.text)
        if not 0 <= index < self.length:
            raise RustPanic(f"index out of bounds: the len is {self.length} but the index is {index}")
        return self.address.child(replace(token, text=str(self.start + index)))

    def read(self):
        return ArrayValue(self.address.read().items[self.start:self.stop])


@dataclass(frozen=True)
class DynAddress:
    address: Address
    concrete: object

    def child(self, field):
        return self.address.child(field)

    def read(self):
        return self.address.read()

    def write(self, value):
        self.address.write(value)


@dataclass(frozen=True)
class Value:
    constructor: Constructor
    fields: tuple

    def at(self, name):
        return self.fields[self.constructor.index(name)]

    def replace(self, path, value):
        if not path:
            return value
        index = self.constructor.index(path[0])
        fields = list(self.fields)
        fields[index] = self.fields[index].replace(path[1:], value) if path[1:] else value
        return Value(self.constructor, tuple(fields))


@dataclass(frozen=True)
class ArrayValue:
    items: tuple

    def index(self, token):
        index = int(token.text)
        if not 0 <= index < len(self.items):
            raise RustPanic(f"index out of bounds: the len is {len(self.items)} but the index is {index}")
        return index

    def at(self, token):
        return self.items[self.index(token)]

    def replace(self, path, value):
        index = self.index(path[0])
        items = list(self.items)
        items[index] = items[index].replace(path[1:], value) if path[1:] else value
        return ArrayValue(tuple(items))


@dataclass(frozen=True)
class IntegerRange:
    start: int
    stop: int

    def contains(self, value):
        return self.start <= ordinal(value) < self.stop


def ordinal(value):
    return ord(value) if isinstance(value, Character) else value


@dataclass(frozen=True)
class SlicePattern:
    annotation: SliceType
    prefix: int
    suffix: int | None

    @property
    def fields(self):
        return tuple((str(index), self.annotation.element) for index in range(self.prefix + (self.suffix or 0)))

    def positions(self, length):
        if self.suffix is None:
            return tuple(range(length)) if length == self.prefix else None
        if length < self.prefix + self.suffix:
            return None
        return (*range(self.prefix), *range(length - self.suffix, length))


@dataclass(frozen=True)
class Pattern:
    tag: object = None
    children: tuple = ()
    binding: str | None = None
    alternatives: tuple = ()
    by_reference: bool = False

    def bindings(self, value, address=None):
        if self.alternatives:
            for alternative in self.alternatives:
                yield from alternative.bindings(value, address)
            return
        if self.tag is None:
            yield {self.binding: address if self.by_reference else value} if self.binding else {}
        elif isinstance(self.tag, Constructor):
            if self.tag.style == "reference":
                yield from self.children[0].bindings(value.read(), value)
            elif isinstance(value, ArrayValue) and self.tag.style == "array":
                names = tuple(name for name, _ in self.tag.fields)
                yield from self.field_bindings(tuple(value.items[int(name)] for name in names), address, names)
            elif isinstance(value, Value) and value.constructor.path == self.tag.path:
                names = tuple(name for name, _ in self.tag.fields)
                yield from self.field_bindings(value.fields, address, names)
        elif isinstance(self.tag, SlicePattern):
            positions = self.tag.positions(len(value.items))
            if positions is not None:
                yield from self.field_bindings(tuple(value.items[index] for index in positions),
                                               address, tuple(map(str, positions)))
        elif isinstance(self.tag, IntegerRange):
            if self.tag.contains(value):
                yield {}
        elif value == self.tag[1]:
            yield {}

    def field_bindings(self, values, address, names, index=0):
        if index == len(self.children):
            yield {}
            return
        child = address.child(Token(names[index], 1, 1)) if address is not None else None
        for first in self.children[index].bindings(values[index], child):
            for rest in self.field_bindings(values, address, names, index + 1):
                yield first | rest


def uncovered(matrix, types, program):
    """Find a missing case by specializing one constructor column at a time."""
    if not types:
        return None if matrix else ()
    annotation, *remaining_types = types
    if annotation == NEVER:
        return None
    declaration = program.layout(annotation)
    if declaration is not None:
        constructors = declaration[1]
        signatures = [(ctor, tuple(kind for _, kind in ctor.fields)) for ctor in constructors]
    elif isinstance(annotation, ReferenceType):
        signatures = [(Constructor("&", annotation, "reference", (("0", annotation.inner),)), (annotation.inner,))]
    elif annotation == "bool":
        signatures = [(("bool", value), ()) for value in (False, True)]
    elif annotation == "()":
        signatures = [(("()", None), ())]
    else:
        signatures = None

    explicit = []
    default = []
    for first, *tail in matrix:
        for option in first.alternatives or (first,):
            if option.tag is None:
                default.append(tuple(tail))
            else:
                explicit.append((option, tuple(tail)))
    if annotation in SCALAR_BOUNDS and explicit:
        return uncovered_scalar(explicit, default, annotation, remaining_types, program)
    if isinstance(annotation, SliceType) and explicit:
        return uncovered_slice(explicit, default, annotation, remaining_types, program)
    if isinstance(annotation, ArrayType) and explicit:
        # Only positions mentioned by a pattern can distinguish its arms.
        positions = sorted({int(name) for first, _ in explicit for name, _ in first.tag.fields})
        fields = tuple((str(index), annotation.element) for index in positions)
        signatures = [(Constructor("[]", annotation, "array", fields), (annotation.element,) * len(fields))]
    if signatures is None or (not explicit and signatures):
        missing = uncovered(default, remaining_types, program)
        return None if missing is None else (None, *missing)
    for signature, fields in signatures:
        wildcards = (Pattern(),) * len(fields)
        rows = [wildcards + tail for tail in default]
        tag = signature.path if isinstance(signature, Constructor) else signature
        rows.extend(specialized_pattern(first, signature) + tail for first, tail in explicit
                    if (first.tag.path if isinstance(first.tag, Constructor) else first.tag) == tag)
        missing = uncovered(rows, (*fields, *remaining_types), program)
        if missing is not None:
            nested, tail = missing[:len(fields)], missing[len(fields):]
            return ((signature, nested), *tail)
    return None


def specialized_pattern(pattern, signature):
    if isinstance(signature, Constructor) and signature.style == "array":
        selected = dict(zip((name for name, _ in pattern.tag.fields), pattern.children))
        return tuple(selected.get(name, Pattern()) for name, _ in signature.fields)
    return pattern.children


def uncovered_slice(explicit, default, annotation, types, program):
    exact = [first.tag.prefix for first, _ in explicit if first.tag.suffix is None]
    open_ended = [first.tag for first, _ in explicit if first.tag.suffix is not None]
    # Beyond the final exact length, the longest prefix and suffix determine
    # when the ends stop overlapping. All larger lengths have the same tests.
    limit = max(max(exact, default=-1) + 1,
                max((shape.prefix for shape in open_ended), default=0)
                + max((shape.suffix for shape in open_ended), default=0))
    inhabited = uncovered([], (annotation.element,), program) is not None
    for length in range(limit + 1 if inhabited else 1):
        concrete = ArrayType(annotation.element, ConstValue(length, "usize"))
        rows = [(Pattern(), *tail) for tail in default]
        for pattern, tail in explicit:
            positions = pattern.tag.positions(length)
            if positions is None:
                continue
            fields = tuple((str(index), annotation.element) for index in positions)
            ctor = Constructor("[]", concrete, "array", fields)
            rows.append((replace(pattern, tag=ctor), *tail))
        missing = uncovered(rows, (concrete, *types), program)
        if missing is not None:
            if missing[0] is None:
                fields = tuple((str(index), annotation.element) for index in range(length))
                return ((Constructor("[]", concrete, "array", fields), (None,) * length), *missing[1:])
            return missing
    return None


def uncovered_scalar(explicit, default, annotation, types, program):
    boundaries = set(SCALAR_BOUNDS[annotation])
    if annotation == "char":
        boundaries.update((SURROGATES.start, SURROGATES.stop))
    ranges = []
    for first, tail in explicit:
        tag = first.tag
        interval = tag if isinstance(tag, IntegerRange) else IntegerRange(ordinal(tag[1]), ordinal(tag[1]) + 1)
        boundaries.update((interval.start, interval.stop))
        ranges.append((interval, tail))
    # A pattern's membership is constant between adjacent boundaries.
    for value in sorted(boundaries)[:-1]:
        if annotation == "char" and value in SURROGATES:
            continue
        rows = default + [tail for interval, tail in ranges if interval.contains(value)]
        missing = uncovered(rows, types, program)
        if missing is not None:
            shown = Character(chr(value)) if annotation == "char" else value
            return (((annotation, shown), ()), *missing)
    return None


def show_missing(witness):
    if witness is None:
        return "_"
    tag, children = witness
    if not isinstance(tag, Constructor):
        return display(tag[1], True)
    if tag.style == "unit":
        return tag.path
    fields = [show_missing(child) for child in children]
    if tag.style == "reference":
        return f"&{fields[0]}"
    if tag.style == "array":
        if len(fields) == tag.annotation.length.value:
            return f"[{', '.join(fields)}]"
        entries = ", ".join(f"{name}: {value}" for (name, _), value in zip(tag.fields, fields))
        return f"array with elements {{{entries}}}"
    if tag.style == "tuple":
        return f"{tag.path}({', '.join(fields)})" if tag.path else tuple_text(fields)
    names = (name for name, _ in tag.fields)
    entries = ", ".join(f"{name}: {value}" for name, value in zip(names, fields))
    return f"{tag.path} {{ {entries} }}" if entries else f"{tag.path} {{}}"


PLACEHOLDER = re.compile(r'\{([A-Za-z_][A-Za-z_0-9]*)?(?::(\?))?\}')


def format_parts(text, token):
    parts = []
    literal = []
    index = 0
    while index < len(text):
        char = text[index]
        if char not in "{}":
            literal.append(char)
            index += 1
        elif text[index:index + 2] in {"{{", "}}"}:
            literal.append(char)
            index += 2
        else:
            parts.append(("text", "".join(literal)))
            literal.clear()
            match = PLACEHOLDER.match(text, index)
            if not match:
                raise token.error("unsupported format placeholder")
            name, debug = match.groups()
            parts.append(("debug" if debug else "display", name))
            index = match.end()
    parts.append(("text", "".join(literal)))
    return parts


ARITHMETIC = {"+": operator.add, "-": operator.sub, "*": operator.mul, **BITWISE}
COMPARE = {
    "==": operator.eq, "!=": operator.ne, "<": operator.lt,
    "<=": operator.le, ">": operator.gt, ">=": operator.ge,
}


@dataclass
class ControlTarget:
    expr: Expr
    exit_types: list = field(default_factory=list)
    in_condition: bool = False
    restart: FlowNode | None = None
    finish: FlowNode | None = None


def find_control(targets, jump, definitions=None):
    bound = next((target for target in reversed(targets)
                  if jump.label is not None and target.expr.label is jump.label), None)
    if bound:
        if jump.kind == "continue" and bound.expr.kind == "block":
            raise jump.token.error("continue requires a loop label")
        return bound
    for target in reversed(targets):
        if jump.label:
            label = target.expr.label
            if label is None or label.key != jump.label.key:
                if not label or not definitions:
                    continue
                context = jump.label.context
                visible = False
                while context:
                    definition, _ = context[-1]
                    context = context[:-1]
                    if (target in definitions.get(definition, ())
                            and label.text == jump.label.text and label.context == context):
                        visible = True
                        break
                if not visible:
                    continue
        elif target.expr.kind == "block":
            raise jump.token.error(f"unlabeled {jump.kind} inside a labeled block")
        elif target.in_condition:
            raise jump.token.error(f"unlabeled {jump.kind} in a while condition")
        if jump.kind == "continue" and target.expr.kind == "block":
            raise jump.token.error("continue requires a loop label")
        return target
    if jump.label:
        raise jump.label.error(f"unknown label {jump.label.text}")
    raise jump.token.error(f"{jump.kind} outside a loop")


def unify_exits(exits, expect=require):
    result = NEVER
    for actual, token in exits:
        if result == NEVER:
            result = actual
        else:
            expect(actual, result, token)
    return result


class Checker:
    def __init__(self, program):
        self.program = program
        self.functions = program.functions
        self.result = "()"
        self.loops = []
        self.macro_labels = {}
        self.substitutions = {}
        self.variables = []
        self.obligations = []
        self.generic_arguments = []
        self.numeric_operands = []
        self.unary_results = []
        self.array_integers = []
        self.parameters = {}
        self.ascriptions = []
        self.closures = []
        self.pointers = []
        self.indirect_calls = []

    def check(self):
        for alias in self.program.type_aliases.values():
            target, token = alias.target, alias.name
            self.check_bounds(alias.parameters, token)
            if isinstance(target, (SliceType, DynType)):
                self.check_type(ReferenceType(target, False), token)
            else:
                self.check_type(target, token)
            for parameter in alias.parameters:
                if not parameter.const_type and substitute(target, {parameter: object()}) == target:
                    raise token.error(f"unused type parameter {parameter}")
        finished = set()
        for name, (token, _) in self.program.types.items():
            self.check_bounds(self.program.type_parameters[name], token)
            self.check_layout(name, token, set(), finished)
            self.check_derives(name)
        self.check_implementations()
        main = self.functions.get("main")
        if main is None:
            raise RustError("missing fn main()")
        if main.params or main.result != "()" or main.type_parameters:
            raise main.name.error("main must take no arguments and return ()")
        for function in tuple(self.functions.values()):
            self.check_bounds(function.type_parameters, function.name)
            self.parameters = {parameter.name: parameter for parameter in function.type_parameters}
            constructor = self.program.constructors.get(function.name.text)
            if constructor and constructor.style != "record":
                raise function.name.error(f"duplicate value {function.name.text!r}")
            scope = Scope()
            for name, mutable, annotation in function.params:
                self.check_type(annotation, name)
                if name.text == "_":
                    continue
                if name.key in scope.bindings:
                    raise name.error(f"duplicate parameter {name.text!r}")
                scope.bindings[name.key] = Binding(annotation, mutable)
            self.result = function.result
            self.check_type(self.result, function.name, allow_never=True)
            concrete = self.effective_function(function, tuple(kind for _, _, kind in function.params),
                                               self.resolve(self.result))
            function.return_inputs = concrete.return_inputs
            result = self.type_of(function.body, scope, function.result)
            self.require_argument(result, function.result, function.body.token)
        self.finish_inference()

    def check_bounds(self, parameters, token):
        for parameter in parameters:
            if isinstance(parameter, LifetimeArgument):
                continue
            for name in parameter.bounds:
                if name not in TRAIT_MODULES and name not in self.program.traits:
                    raise token.error(f"unknown trait {name!r}")

    def check_implementations(self):
        for trait in self.program.traits.values():
            self.check_bounds((trait.self_type,), trait.name)
            if trait.name.text in self.program.trait_closure(trait.bounds):
                raise trait.name.error("cyclic supertrait relationship")
            for associated in trait.associated.values():
                self.check_bounds(associated.parameters, trait.name)
                for bound in associated.bounds:
                    if bound not in TRAIT_MODULES and bound not in self.program.traits:
                        raise trait.name.error(f"unknown trait {bound!r}")
            for method in trait.methods.values():
                self.check_bounds(method.type_parameters, method.name)
                by_value = (method.result is trait.self_type
                            or any(kind is trait.self_type for _, _, kind in method.params))
                if (method.body and by_value and not method.self_sized
                        and "Sized" not in self.program.trait_closure(trait.self_type.bounds)):
                    raise method.name.error("default body using Self by value requires Self: Sized")
                for name, _, annotation in method.params:
                    self.check_type(annotation, name)
                self.check_type(method.result, method.name, allow_never=True)
        for index, impl in enumerate(self.program.impls):
            self.check_bounds(impl.parameters, impl.token)
            self.check_type(impl.target, impl.token)
            target_parameters = set(self.type_parts(impl.target))
            if any(parameter not in target_parameters for parameter in impl.parameters):
                raise impl.token.error("unconstrained impl parameter")
            if impl.trait is None and (isinstance(impl.target, TupleType) or self.program.layout(impl.target) is None):
                raise impl.token.error("inherent impl requires a declared struct or enum")
            for previous in self.program.impls[:index]:
                if previous.trait == impl.trait and overlapping_types(previous.target, impl.target):
                    if impl.trait or previous.methods.keys() & impl.methods.keys():
                        raise impl.token.error("conflicting implementations")
            if impl.trait is None:
                continue
            trait = self.program.traits.get(impl.trait)
            if trait is None:
                raise impl.token.error(f"unknown or unsupported implemented trait {impl.trait!r}")
            for bound in sorted(trait.bounds):
                self.obligation(impl.target, bound, impl.token)
            for name in impl.associated.keys() ^ trait.associated.keys():
                raise impl.token.error(f"missing or unexpected associated type {name!r}")
            for name, definition in impl.associated.items():
                declaration = trait.associated[name]
                if (len(definition.parameters) != len(declaration.parameters)
                        or any(not self.same_parameter_kind(wanted, actual)
                               for wanted, actual in zip(declaration.parameters, definition.parameters))):
                    raise impl.token.error(f"associated type {name!r} has incompatible generic parameters")
                kind = definition.body
                self.check_type(kind, impl.token)
                resolved = self.resolve(kind)
                if isinstance(resolved, AssociatedType):
                    raise impl.token.error("associated type cannot resolve to itself")
                if isinstance(resolved, ReferenceType) and LifetimeArgument(resolved.lifetime) not in definition.parameters:
                    raise impl.token.error("associated reference types require a lifetime parameter")
                for bound in sorted(declaration.bounds):
                    self.obligation(kind, bound, impl.token)
            for name in impl.methods.keys() - trait.methods.keys():
                raise impl.methods[name].name.error(f"{name!r} is not a method of {impl.trait}")
            for name, required in trait.methods.items():
                method = impl.methods.get(name)
                if not method:
                    if required.body is None:
                        raise impl.token.error(f"missing method {name!r} for {impl.trait}")
                    continue
                bindings = {trait.self_type: impl.target}
                if (len(required.method_parameters) != len(method.method_parameters)
                        or any(first.bounds != second.bounds or first.const_type != second.const_type
                               for first, second in zip(required.method_parameters, method.method_parameters))):
                    raise method.name.error(f"method {name!r} has incompatible generic parameters")
                bindings.update(zip(required.method_parameters, method.method_parameters))
                required_types = tuple(self.resolve(substitute(kind, bindings)) for _, _, kind in required.params)
                actual_types = tuple(self.resolve(kind) for _, _, kind in method.params)
                required_result = self.resolve(substitute(required.result, bindings))
                actual_result = self.resolve(method.result)
                required_signature = self.effective_function(required, required_types, required_result)
                actual_signature = self.effective_function(method, actual_types, actual_result)
                if (required_types != actual_types
                        or actual_result != required_result
                        or actual_signature.return_inputs != required_signature.return_inputs
                        or method.associated_const != required.associated_const
                        or self.has_receiver(method) != self.has_receiver(required)):
                    raise method.name.error(f"method {name!r} has an incompatible trait signature")

    @staticmethod
    def same_parameter_kind(first, second):
        if isinstance(first, LifetimeArgument) or isinstance(second, LifetimeArgument):
            return isinstance(first, LifetimeArgument) and isinstance(second, LifetimeArgument)
        return first.const_type == second.const_type and first.bounds == second.bounds

    def effective_function(self, function, types, result):
        if not isinstance(function.result, AssociatedType) or not isinstance(result, ReferenceType):
            return function
        params = [(name, mutable, kind) for (name, mutable, _), kind in zip(function.params, types)]
        if self.has_receiver(function) and isinstance(types[0], ReferenceType) and not result.lifetime:
            inputs = (0,)
        else:
            inputs = Parser.return_inputs(function.name, params, result, function.lifetimes)
        return replace(function, params=params, result=result, return_inputs=inputs)

    def resolve(self, annotation):
        if isinstance(annotation, TypeVariable) and annotation in self.substitutions:
            result = self.resolve(self.substitutions[annotation])
            self.substitutions[annotation] = result
            return result
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.resolve(annotation.inner))
        if isinstance(annotation, FunctionType):
            return FunctionType(tuple(map(self.resolve, annotation.parameters)), self.resolve(annotation.result),
                                annotation.closure)
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.resolve(arg) for arg in annotation.arguments))
        if isinstance(annotation, TupleType):
            return TupleType(tuple(map(self.resolve, annotation.elements)))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.resolve(annotation.element), self.resolve(annotation.length))
        if isinstance(annotation, SliceType):
            return SliceType(self.resolve(annotation.element))
        if isinstance(annotation, ConstExpression):
            return fold_const(replace(annotation, operands=tuple(map(self.resolve, annotation.operands))))
        if isinstance(annotation, DynType):
            return DynType(annotation.trait, tuple((name, self.resolve(value)) for name, value in annotation.bindings))
        if isinstance(annotation, AssociatedType):
            projected = replace(annotation, owner=self.resolve(annotation.owner),
                                arguments=tuple(map(self.resolve, annotation.arguments)))
            return self.program.normalize(projected)
        return annotation

    def occurs(self, variable, annotation):
        annotation = self.resolve(annotation)
        if isinstance(annotation, ReferenceType):
            return self.occurs(variable, annotation.inner)
        if isinstance(annotation, FunctionType):
            return any(self.occurs(variable, kind) for kind in (*annotation.parameters, annotation.result))
        if isinstance(annotation, AppliedType):
            return any(self.occurs(variable, arg) for arg in annotation.arguments)
        if isinstance(annotation, TupleType):
            return any(self.occurs(variable, kind) for kind in annotation.elements)
        if isinstance(annotation, ArrayType):
            return self.occurs(variable, annotation.element) or self.occurs(variable, annotation.length)
        if isinstance(annotation, SliceType):
            return self.occurs(variable, annotation.element)
        if isinstance(annotation, ConstExpression):
            return any(self.occurs(variable, value) for value in annotation.operands)
        if isinstance(annotation, AssociatedType):
            return self.occurs(variable, annotation.owner) or any(
                self.occurs(variable, arg) for arg in annotation.arguments
            )
        return annotation is variable

    def require(self, actual, expected, token):
        actual, expected = self.resolve(actual), self.resolve(expected)
        if actual in {NEVER, expected}:
            return
        if isinstance(expected, TypeVariable) and not self.occurs(expected, actual):
            self.check_variable_binding(expected, actual, token)
            self.substitutions[expected] = actual
            return
        if isinstance(actual, TypeVariable) and expected != NEVER and not self.occurs(actual, expected):
            self.check_variable_binding(actual, expected, token)
            self.substitutions[actual] = expected
            return
        if isinstance(actual, ReferenceType) and isinstance(expected, ReferenceType):
            if actual.mutable == expected.mutable:
                self.require(actual.inner, expected.inner, token)
                return
        if isinstance(actual, FunctionType) and isinstance(expected, FunctionType):
            if actual.closure == expected.closure and len(actual.parameters) == len(expected.parameters):
                for left, right in zip(actual.parameters, expected.parameters):
                    self.require(left, right, token)
                self.require(actual.result, expected.result, token)
                return
        if isinstance(actual, AppliedType) and isinstance(expected, AppliedType):
            if actual.name == expected.name and len(actual.arguments) == len(expected.arguments):
                for left, right in zip(actual.arguments, expected.arguments):
                    self.require(left, right, token)
                return
        if isinstance(actual, TupleType) and isinstance(expected, TupleType):
            if len(actual.elements) == len(expected.elements):
                for left, right in zip(actual.elements, expected.elements):
                    self.require(left, right, token)
                return
        if isinstance(actual, ArrayType) and isinstance(expected, ArrayType):
            self.require(actual.element, expected.element, token)
            self.require(actual.length, expected.length, token)
            return
        if isinstance(actual, SliceType) and isinstance(expected, SliceType):
            self.require(actual.element, expected.element, token)
            return
        if isinstance(actual, AssociatedType) and isinstance(expected, AssociatedType):
            if (actual.item, actual.trait, actual.owner, len(actual.arguments)) == (
                expected.item, expected.trait, expected.owner, len(expected.arguments)
            ):
                for left, right in zip(actual.arguments, expected.arguments):
                    self.require(left, right, token)
                return
        raise token.error(f"expected {expected}, found {actual}")

    def require_argument(self, actual, expected, token):
        actual, expected = self.resolve(actual), self.resolve(expected)
        if (isinstance(actual, ReferenceType) and isinstance(expected, ReferenceType)
                and actual.mutable and not expected.mutable):
            self.require(actual.inner, expected.inner, token)
        else:
            self.require(actual, expected, token)

    def obligation(self, annotation, trait, token):
        self.obligations.append((annotation, trait, token))

    def check_variable_binding(self, variable, value, token):
        if variable.const_type:
            self.check_const(value, variable.const_type, token)
        elif isinstance(value, (ConstValue, ConstExpression)) or getattr(value, "const_type", None):
            raise token.error("expected a type argument, found a constant")

    def check_const(self, value, kind, token):
        check_constant_kind(self.resolve(value), kind, token)

    def specialize(self, parameters, explicit, token):
        if explicit is not None and (not parameters or len(explicit) != len(parameters)):
            raise token.error(f"{token.text} expects {len(parameters)} type arguments")
        arguments = {}
        for index, parameter in enumerate(parameters):
            argument = explicit[index] if explicit is not None else None
            if argument is None:
                argument = TypeVariable(parameter.name, token, parameter.const_type)
                self.variables.append(argument)
            elif parameter.const_type:
                self.check_const(argument, parameter.const_type, token)
            else:
                argument = self.fill_holes(argument, token)
                self.check_type(argument, token)
            arguments[parameter] = argument
            if not parameter.const_type:
                self.generic_arguments.append((argument, token))
            for trait in sorted(parameter.bounds):
                self.obligation(argument, trait, token)
        return arguments

    def fill_holes(self, annotation, token):
        if annotation is None:
            variable = TypeVariable("_", token)
            self.variables.append(variable)
            return variable
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.fill_holes(annotation.inner, token))
        if isinstance(annotation, TupleType):
            return TupleType(tuple(self.fill_holes(kind, token) for kind in annotation.elements))
        if isinstance(annotation, SliceType):
            return SliceType(self.fill_holes(annotation.element, token))
        if isinstance(annotation, AppliedType):
            parameters = self.program.type_parameters.get(annotation.name, ())
            arguments = []
            for index, arg in enumerate(annotation.arguments):
                if arg is None and index < len(parameters) and parameters[index].const_type:
                    variable = TypeVariable(parameters[index].name, token, parameters[index].const_type)
                    self.variables.append(variable)
                    arguments.append(variable)
                else:
                    arguments.append(self.fill_holes(arg, token))
            return AppliedType(annotation.name, tuple(arguments))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.fill_holes(annotation.element, token), annotation.length)
        return annotation

    def finish_inference(self):
        for annotation, value, token in self.array_integers:
            resolved = self.resolve(annotation)
            if isinstance(resolved, TypeVariable):
                self.require(resolved, "i32", token)
                resolved = "i32"
            if resolved not in INTEGER_TYPES:
                raise token.error(f"expected an integer, found {resolved}")
            checked(value, token, resolved)
        for operand, result, token in self.unary_results:
            operand = self.resolve(operand)
            if not isinstance(operand, TypeVariable):
                if operand != "bool" and operand not in INTEGER_TYPES:
                    raise token.error(f"cannot negate {operand} with '!'")
                self.require(result, operand, token)
        for variable in self.variables:
            if isinstance(self.resolve(variable), TypeVariable):
                raise variable.token.error(f"type annotations needed for {variable.name}")
        for function in self.closures:
            function.captures = tuple(replace(capture, annotation=self.resolve(capture.annotation))
                                      for capture in function.captures)
        for annotation, token in self.generic_arguments:
            annotation = self.resolve(annotation)
            if isinstance(annotation, ReferenceType):
                raise token.error("references as generic type arguments are unsupported")
            if isinstance(annotation, FunctionType) and annotation.closure:
                raise token.error("capturing closures as generic type arguments are unsupported")
        for annotation, trait, token in self.obligations:
            annotation = self.resolve(annotation)
            if annotation != NEVER and not self.implements(annotation, trait):
                raise token.error(f"{annotation} does not implement {trait}")
        for annotation, expected, token in self.numeric_operands:
            if expected is None:
                annotation = self.resolve(annotation)
                if annotation != NEVER and annotation not in INTEGER_TYPES:
                    raise token.error(f"expected an integer, found {annotation}")
            else:
                self.require(annotation, expected, token)
        for key, value in self.program.expr_types.items():
            self.program.expr_types[key] = self.resolve(value)
        for key, (types, arguments) in self.program.call_types.items():
            self.program.call_types[key] = (tuple(self.resolve(kind) for kind in types),
                                            {p: self.resolve(arg) for p, arg in arguments.items()})
        for key, (trait, receiver, method) in self.program.dispatches.items():
            self.program.dispatches[key] = (trait, self.resolve(receiver), method)
        for key, (pattern, bindings) in self.program.patterns.items():
            self.program.patterns[key] = (
                self.resolve_pattern(pattern),
                {name: (self.resolve(kind), mutable) for name, (kind, mutable) in bindings.items()},
            )
        for expr in self.ascriptions:
            source, annotation, value, otherwise = expr.args
            expr.args = (source, self.resolve(annotation), value, otherwise)
        for function in self.closures:
            function.params = [(name, mutable, self.resolve(kind)) for name, mutable, kind in function.params]
            function.result = self.resolve(function.result)
            for capture in function.captures:
                kind = capture.annotation
                if isinstance(kind, FunctionType) and kind.closure:
                    raise capture.name.error("capturing another capturing closure is unsupported")
            if isinstance(function.result, ReferenceType):
                if function.captures:
                    raise function.name.error("reference results from capturing closures are unsupported")
                function.return_inputs = Parser.return_inputs(function.name, function.params, function.result, {})
        for expr in self.pointers:
            name, arguments = expr.args
            expr.args = (name, {parameter: self.resolve(kind) for parameter, kind in arguments.items()})
        for expr in self.indirect_calls:
            function = self.program.call_functions[id(expr)]
            function.params = [(name, mutable, self.resolve(kind)) for name, mutable, kind in function.params]
            function.result = self.resolve(function.result)
            function.return_inputs = Parser.return_inputs(function.name, function.params, function.result, {})

    def resolve_pattern(self, pattern):
        tag = pattern.tag
        if isinstance(tag, Constructor):
            tag = replace(tag, annotation=self.resolve(tag.annotation),
                          fields=tuple((name, self.resolve(kind)) for name, kind in tag.fields))
        elif isinstance(tag, SlicePattern):
            tag = replace(tag, annotation=self.resolve(tag.annotation))
        return replace(pattern, tag=tag, children=tuple(map(self.resolve_pattern, pattern.children)),
                       alternatives=tuple(map(self.resolve_pattern, pattern.alternatives)))

    def check_derives(self, annotation):
        derives = self.program.derives[annotation]
        parameters = self.program.type_parameters[annotation]
        if ("Default" in derives and annotation not in self.program.constructors
                and annotation not in self.program.default_variants):
            raise derives["Default"].error("Default derive on enums requires a #[default] unit variant")
        for trait, token in derives.items():
            if trait == "Default" and annotation in self.program.default_variants:
                continue
            for required in sorted(SUPERTRAITS.get(trait, ())):
                if required not in derives:
                    raise token.error(f"{trait} requires {required}")
            for ctor in self.program.types[annotation][1]:
                for _, kind in ctor.fields:
                    if not self.program.implements(kind, trait, parameters):
                        raise token.error(f"{kind} does not implement {trait}")

    def implements(self, annotation, trait):
        return self.program.implements(annotation, trait)

    def check_type(self, annotation, token, *, allow_never=False):
        if annotation is None:
            raise token.error("type placeholders are unsupported in item signatures")
        if isinstance(annotation, (TypeParameter, TypeVariable)):
            if annotation.const_type:
                raise token.error("expected a type, found a const parameter")
            return
        if isinstance(annotation, FunctionType):
            for kind in annotation.parameters:
                self.check_type(kind, token)
            self.check_type(annotation.result, token, allow_never=True)
            if isinstance(annotation.result, ReferenceType):
                params = [(token, False, kind) for kind in annotation.parameters]
                Parser.return_inputs(token, params, annotation.result, {})
            return
        if isinstance(annotation, TupleType):
            for kind in annotation.elements:
                if isinstance(kind, ReferenceType) or isinstance(kind, FunctionType) and kind.closure:
                    raise token.error("reference and capturing closure elements in tuples are unsupported")
                self.check_type(kind, token)
            return
        if isinstance(annotation, ArrayType):
            if isinstance(annotation.element, ReferenceType):
                raise token.error("reference elements in arrays are unsupported")
            self.check_type(annotation.element, token)
            self.check_const(annotation.length, "usize", token)
            return
        if isinstance(annotation, AssociatedType):
            self.check_type(annotation.owner, token)
            trait = self.program.projection_trait(annotation)
            if not trait:
                raise token.error(f"unknown or ambiguous associated type {annotation}")
            self.obligation(annotation.owner, trait.name.text, token)
            parameters = trait.associated[annotation.item].parameters
            if len(parameters) != len(annotation.arguments):
                raise token.error(f"{annotation.item} expects {len(parameters)} generic arguments")
            for parameter, argument in zip(parameters, annotation.arguments):
                if isinstance(parameter, LifetimeArgument):
                    if not isinstance(argument, LifetimeArgument):
                        raise token.error(f"{annotation.item} expects a lifetime argument")
                elif parameter.const_type:
                    self.check_const(argument, parameter.const_type, token)
                else:
                    self.check_type(argument, token)
                    for bound in parameter.bounds:
                        self.obligation(argument, bound, token)
            return
        if isinstance(annotation, ReferenceType):
            if isinstance(annotation.inner, ReferenceType):
                raise token.error("nested reference types are unsupported")
            if isinstance(annotation.inner, DynType):
                self.check_dyn(annotation.inner, token)
            elif isinstance(annotation.inner, SliceType):
                element = annotation.inner.element
                if isinstance(element, ReferenceType):
                    raise token.error("reference elements in slices are unsupported")
                self.check_type(element, token)
            else:
                self.check_type(annotation.inner, token)
            return
        if isinstance(annotation, DynType):
            raise token.error("trait objects require a reference")
        if isinstance(annotation, SliceType):
            raise token.error("slice type requires a reference")
        if isinstance(annotation, AppliedType):
            parameters = self.program.type_parameters.get(annotation.name)
            if not parameters or len(parameters) != len(annotation.arguments):
                raise token.error(f"{annotation.name} expects {len(parameters or ())} type arguments")
            for parameter, argument in zip(parameters, annotation.arguments):
                if parameter.const_type:
                    self.check_const(argument, parameter.const_type, token)
                    continue
                self.check_type(argument, token)
                self.generic_arguments.append((argument, token))
                for trait in sorted(parameter.bounds):
                    self.obligation(argument, trait, token)
            return
        if self.program.type_parameters.get(annotation):
            count = len(self.program.type_parameters[annotation])
            raise token.error(f"{annotation} expects {count} type arguments")
        if annotation in PRIMITIVES or annotation in self.program.types:
            return
        if annotation != NEVER or not allow_never:
            raise token.error(f"unsupported type {annotation!r}")

    def check_dyn(self, annotation, token):
        if annotation.trait not in self.program.traits:
            raise token.error(f"unknown object trait {annotation.trait!r}")
        traits = self.program.trait_closure({annotation.trait})
        if "Sized" in traits:
            raise token.error(f"{annotation.trait} is not dyn-compatible")
        required = set()
        for name in traits:
            trait = self.program.traits.get(name)
            if not trait:
                continue
            required.update(trait.associated)
            if any(item.parameters for item in trait.associated.values()):
                raise token.error(f"{name} is not dyn-compatible")
            for method in trait.methods.values():
                if method.self_sized:
                    continue
                if (method.associated_const or method.method_parameters or not self.has_receiver(method)
                        or self.mentions_self(method.result, trait.self_type)
                        or any(self.mentions_self(kind, trait.self_type) for _, _, kind in method.params[1:])):
                    raise token.error(f"{name} is not dyn-compatible")
        given = dict(annotation.bindings)
        if len(given) != len(annotation.bindings) or required != given.keys():
            raise token.error(f"{annotation.trait} needs bindings for its associated types")
        for kind in given.values():
            self.check_type(kind, token)

    @staticmethod
    def mentions_self(kind, self_type):
        if kind is self_type:
            return True
        if isinstance(kind, ReferenceType):
            return Checker.mentions_self(kind.inner, self_type)
        if isinstance(kind, (ArrayType, SliceType)):
            return Checker.mentions_self(kind.element, self_type)
        if isinstance(kind, TupleType):
            return any(Checker.mentions_self(element, self_type) for element in kind.elements)
        if isinstance(kind, AppliedType):
            return any(Checker.mentions_self(arg, self_type) for arg in kind.arguments)
        return False

    def check_layout(self, name, token, visiting, finished):
        if name in finished:
            return
        if name in visiting:
            raise token.error(f"recursive type {name!r} has infinite size")
        visiting.add(name)
        used = set()
        for ctor in self.program.types[name][1]:
            for _, kind in ctor.fields:
                self.check_type(kind, token)
                for part in self.type_parts(kind):
                    if part in self.program.types:
                        self.check_layout(part, token, visiting, finished)
                    if isinstance(part, TypeParameter):
                        used.add(part)
        for param in self.program.type_parameters[name]:
            if param not in used and not param.const_type:
                raise token.error(f"unused type parameter {param}")
        visiting.remove(name)
        finished.add(name)

    @staticmethod
    def type_parts(annotation):
        if isinstance(annotation, AppliedType):
            yield annotation.name
            for argument in annotation.arguments:
                yield from Checker.type_parts(argument)
        elif isinstance(annotation, ArrayType):
            yield from Checker.type_parts(annotation.element)
            yield annotation.length
        elif isinstance(annotation, TupleType):
            for kind in annotation.elements:
                yield from Checker.type_parts(kind)
        else:
            yield annotation

    def constructor(self, expr, style, expected=None):
        token = expr.token
        ctor = self.program.constructors.get(token.text)
        if not ctor or ctor.style != style:
            raise token.error(f"unknown {style} constructor {token.text!r}")
        if style == "tuple" and any(token.module[:len(scope)] != scope for _, scope in ctor.visibility):
            raise token.error(f"constructor {token.text!r} has private fields")
        parameters = self.program.type_parameters[ctor.annotation]
        arguments = self.specialize(parameters, expr.type_arguments, token)
        if not arguments:
            return ctor
        result = AppliedType(ctor.annotation, tuple(arguments.values()))
        if expected is not None:
            self.require(result, expected, token)
        return self.program.apply_constructor(ctor, self.resolve(result), arguments)

    def field_type(self, annotation, token):
        if annotation == NEVER:
            return NEVER
        declaration = self.program.layout(annotation)
        if not declaration or len(declaration[1]) != 1 or "::" in declaration[1][0].path:
            raise token.error(f"{annotation} has no accessible fields")
        ctor = declaration[1][0]
        self.visible_field(ctor, token)
        return ctor.fields[ctor.index(token)][1]

    @staticmethod
    def visible_field(ctor, token):
        scope = dict(ctor.visibility).get(token.text, ())
        if token.module[:len(scope)] != scope:
            raise token.error(f"field {token.text!r} is private")

    @staticmethod
    def field_indices(ctor, fields, rest, token):
        indices = []
        for name, _ in fields:
            Checker.visible_field(ctor, name)
            index = ctor.index(name)
            if index in indices:
                raise name.error(f"duplicate field {name.text!r}")
            indices.append(index)
        if not rest and len(indices) != len(ctor.fields):
            missing = [name for i, (name, _) in enumerate(ctor.fields) if i not in indices]
            raise token.error(f"missing fields: {', '.join(missing)}")
        return indices

    @staticmethod
    def positional_patterns(items, size, token, label):
        rest = [index for index, child in enumerate(items) if child.kind == "rest"]
        if len(rest) > 1:
            raise items[rest[1]].token.error("a positional pattern can contain only one '..'")
        if not rest:
            if len(items) != size:
                raise token.error(f"{label} takes {size} fields")
            return tuple(enumerate(items))
        if len(items) - 1 > size:
            raise token.error(f"{label} takes {size} fields")
        index = rest[0]
        suffix = index + 1
        return (*enumerate(items[:index]), *enumerate(items[suffix:], start=size - len(items) + suffix))

    def prepare_pattern(self, expr, annotation, *, borrowed=False):
        kind, token, args = expr.kind, expr.token, expr.args
        annotation = self.resolve(annotation)
        unit = self.program.constructors.get(token.text) if kind == "bind" else None
        structural = kind not in {"bind", "wild", "or"} or (unit and unit.style == "unit")
        if isinstance(annotation, ReferenceType) and structural:
            if annotation.mutable:
                raise token.error("matching a mutable reference requires explicit dereferencing")
            inner, bindings = self.prepare_pattern(expr, annotation.inner, borrowed=True)
            ctor = Constructor("&", annotation, "reference", (("0", annotation.inner),))
            return Pattern(tag=ctor, children=(inner,)), bindings
        if kind == "wild":
            return Pattern(), {}
        if kind == "rest":
            raise token.error("'..' is only allowed within a tuple, array, or record pattern")
        if kind == "literal":
            actual, value = args
            hint = self.resolve(annotation)
            if expr.unsuffixed and hint in INTEGER_TYPES:
                actual = hint
            if actual in INTEGER_TYPES:
                checked(value, token, actual)
            self.require(actual, annotation, token)
            return Pattern(tag=(actual, value)), {}
        if kind == "pattern_range":
            first, last, inclusive = args
            scalar = self.resolve(annotation)
            if scalar not in SCALAR_BOUNDS:
                scalar = next((endpoint.args[0] for endpoint in (first, last)
                               if endpoint and not endpoint.unsuffixed and endpoint.args[0] in SCALAR_BOUNDS), "i32")
            self.require(scalar, annotation, token)
            for endpoint in (first, last):
                if endpoint:
                    actual = scalar if endpoint.unsuffixed and scalar in INTEGER_TYPES else endpoint.args[0]
                    self.require(actual, scalar, endpoint.token)
                    if scalar in INTEGER_TYPES:
                        checked(endpoint.args[1], endpoint.token, scalar)
            lower, upper = SCALAR_BOUNDS[scalar]
            start = ordinal(first.args[1]) if first else lower
            stop = ordinal(last.args[1]) + int(inclusive) if last else upper
            if start >= stop:
                raise token.error("empty integer range pattern")
            return Pattern(tag=IntegerRange(start, stop)), {}
        if kind == "or":
            pattern, bindings = self.prepare_pattern(args[0], annotation, borrowed=borrowed)
            alternatives = list(pattern.alternatives or (pattern,))
            for alternative in args[1:]:
                option, other = self.prepare_pattern(alternative, annotation, borrowed=borrowed)
                resolved = {key: (self.resolve(t), m) for key, (t, m) in bindings.items()}
                incoming = {key: (self.resolve(t), m) for key, (t, m) in other.items()}
                if resolved != incoming:
                    raise alternative.token.error("all alternatives must bind the same names and types")
                alternatives.extend(option.alternatives or (option,))
            return Pattern(alternatives=tuple(alternatives)), bindings
        if kind == "bind":
            if not unit or unit.style != "unit":
                if isinstance(self.resolve(annotation), SliceType):
                    raise token.error("cannot bind an unsized slice value")
                if borrowed and not args[0]:
                    return (Pattern(binding=token.key, by_reference=True),
                            {token.key: (ReferenceType(annotation, False), False)})
                return Pattern(binding=token.key), {token.key: (annotation, args[0])}
            if args[0]:
                raise token.error(f"cannot shadow unit struct {token.text!r}")
            kind = "pattern_unit"
            args = ()
        style = kind.removeprefix("pattern_")
        if style == "array":
            annotation = self.resolve(annotation)
            if isinstance(annotation, SliceType):
                rest = [index for index, child in enumerate(args) if child.kind == "rest"]
                if len(rest) > 1:
                    raise args[rest[1]].token.error("a slice pattern can contain only one '..'")
                prefix = rest[0] if rest else len(args)
                suffix = len(args) - prefix - 1 if rest else None
                ctor = SlicePattern(annotation, prefix, suffix)
                assignments = enumerate(child for child in args if child.kind != "rest")
            elif not isinstance(annotation, ArrayType):
                raise token.error(f"expected an array pattern target, found {annotation}")
            else:
                if not isinstance(annotation.length, ConstValue):
                    raise token.error("array pattern requires a fixed length")
                pairs = self.positional_patterns(args, annotation.length.value, token, "array pattern")
                pairs = tuple((index, child) for index, child in pairs if child.kind != "wild")
                if not pairs:
                    return Pattern(), {}
                fields = tuple((str(index), annotation.element) for index, _ in pairs)
                ctor = Constructor("[]", annotation, "array", fields)
                assignments = enumerate(child for _, child in pairs)
        elif style == "anonymous_tuple":
            annotation = self.resolve(annotation)
            if annotation == "()" and len(args) == 1 and args[0].kind == "rest":
                return Pattern(tag=("()", None)), {}
            if isinstance(annotation, TypeVariable) and all(child.kind != "rest" for child in args):
                elements = tuple(TypeVariable("tuple element", child.token) for child in args)
                self.variables.extend(elements)
                self.require(TupleType(elements), annotation, token)
                annotation = self.resolve(annotation)
            if not isinstance(annotation, TupleType):
                raise token.error(f"expected a tuple pattern target, found {annotation}")
            ctor = self.program.layout(annotation)[1][0]
            assignments = self.positional_patterns(args, len(ctor.fields), token, "tuple pattern")
        else:
            ctor = self.constructor(expr, style, annotation)
            self.require(ctor.annotation, annotation, token)
            if style == "record":
                fields, rest = args
                indices = self.field_indices(ctor, fields, rest, token)
                assignments = zip(indices, (value for _, value in fields))
            else:
                assignments = self.positional_patterns(args, len(ctor.fields), token, ctor.path)
        fields = ctor.fields
        children = [Pattern()] * len(fields)
        bindings = {}
        for index, child in assignments:
            child_pattern, added = self.prepare_pattern(child, fields[index][1], borrowed=borrowed)
            duplicate = bindings.keys() & added.keys()
            if duplicate:
                raise child.token.error(f"duplicate pattern binding {sorted(duplicate, key=str)[0]!r}")
            bindings.update(added)
            children[index] = child_pattern
        if isinstance(ctor, SlicePattern):
            for annotation, _ in bindings.values():
                self.obligation(annotation, "Copy", token)
        return Pattern(tag=ctor, children=tuple(children)), bindings

    def match_type(self, expr, scope, expected=None):
        scrutinee, arms = expr.args
        annotation = self.type_of(scrutinee, scope)
        if isinstance(annotation, ReferenceType) and annotation.mutable:
            raise scrutinee.token.error("matching a mutable reference requires explicit dereferencing")
        coverage = []
        result = NEVER
        for source, guard, body in arms:
            pattern, bindings = self.prepare_pattern(source, annotation)
            self.program.patterns[id(source)] = (pattern, bindings)
            inner = Scope(scope)
            for name, (kind, mutable) in bindings.items():
                inner.bindings[name] = Binding(kind, mutable if guard is None else False)
            if guard:
                self.require(self.type_of(guard, inner), "bool", guard.token)
                for name, (_, mutable) in bindings.items():
                    inner.bindings[name].mutable = mutable
            else:
                coverage.append((pattern,))
            actual = self.type_of(body, inner, expected)
            if result == NEVER or shared_coercion(result, actual):
                result = actual
            elif not shared_coercion(actual, result):
                self.require(actual, result, body.token)
        missing = uncovered(coverage, (self.resolve(annotation),), self.program)
        if missing is not None:
            raise expr.token.error(f"non-exhaustive match, missing {show_missing(missing[0])}")
        return NEVER if annotation == NEVER else result

    def loop_type(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        target = ControlTarget(expr)
        test = "()"
        if kind == "for":
            name, mutable, start, end, _, _ = args
            first, last = self.operand_types(start, end, scope, same_type=True)
            integer = self.integer_pair(first, last)
            bounds = (first, last)
            for bound in bounds:
                self.require(bound, integer, token)
            if NEVER in bounds:
                test = NEVER
            scope = Scope(scope, {name.key: Binding(integer, mutable)})
        self.loops.append(target)
        try:
            if kind in {"while", "while_let"}:
                condition = args[0]
                target.in_condition = True
                test = self.type_of(condition, scope)
                target.in_condition = False
                if kind == "while":
                    self.require(test, "bool", condition.token)
                else:
                    source = args[1]
                    if isinstance(test, ReferenceType) and test.mutable:
                        raise condition.token.error("matching a mutable reference requires explicit dereferencing")
                    pattern, bindings = self.prepare_pattern(source, test)
                    self.program.patterns[id(source)] = (pattern, bindings)
                    scope = Scope(scope, {name: Binding(kind, mutable)
                                          for name, (kind, mutable) in bindings.items()})
            body = args[-1]
            self.require(self.type_of(body, scope), "()", body.token)
        finally:
            self.loops.pop()
        if kind == "loop":
            return unify_exits(target.exit_types, self.require)
        if test == NEVER and not target.exit_types:
            return NEVER
        return "()"

    def type_of(self, expr, scope, expected=None):
        if id(expr) in self.program.call_functions or expr.kind in {"length", "is_empty"}:
            cached = self.resolve(self.program.expr_types[id(expr)])
            result = cached
        elif expr.kind == "block" and expr.label:
            target = ControlTarget(expr)
            self.loops.append(target)
            try:
                tail = self.check_expr(expr, scope, expected)
            finally:
                self.loops.pop()
            result = unify_exits([*target.exit_types, (tail, expr.token)], self.require)
        else:
            result = self.check_expr(expr, scope, expected)
        result = self.resolve(result)
        result = self.coerce_reference(expr, result, self.resolve(expected))
        if isinstance(result, ReferenceType) and not reference_source(expr):
            raise expr.token.error(f"reference result from {expr.kind} is unsupported")
        self.program.expr_types[id(expr)] = result
        return result

    def coerce_reference(self, expr, actual, expected):
        if (not isinstance(expected, ReferenceType) or not isinstance(actual, ReferenceType)
                or actual.inner == expected.inner
                or (expected.mutable and not actual.mutable)):
            return actual
        target = expected.inner
        if isinstance(target, SliceType) and isinstance(actual.inner, ArrayType):
            self.require(actual.inner.element, target.element, expr.token)
            conversion = "slice_reference"
        elif isinstance(target, DynType):
            self.check_dyn(target, expr.token)
            self.obligation(actual.inner, target.trait, expr.token)
            traits = self.program.trait_closure({target.trait})
            for name, kind in target.bindings:
                owners = [trait for trait in traits if trait in self.program.traits
                          and name in self.program.traits[trait].associated]
                if len(owners) != 1:
                    raise expr.token.error(f"ambiguous associated type {name!r}")
                self.require(AssociatedType(actual.inner, name, owners[0]), kind, expr.token)
            conversion = "dyn_reference"
        else:
            return actual
        original = replace(expr)
        for table in (self.program.call_types, self.program.call_functions, self.program.dispatches):
            if id(expr) in table:
                table[id(original)] = table.pop(id(expr))
        self.program.expr_types[id(original)] = actual
        expr.kind = conversion
        expr.args = (original, actual.inner)
        return expected

    def require_place(self, expr, *, write=False, allow_temporary=False):
        place = place_root(expr)
        if place.kind == "deref":
            pointer = self.program.expr_types[id(place.args[0])]
            if pointer == NEVER:
                return
            if write and not pointer.mutable:
                raise expr.token.error("cannot write through a shared reference")
            return
        if place.kind != "name":
            if allow_temporary:
                return
            raise expr.token.error("expected a local place")
        ctor = self.program.constructors.get(place.token.text)
        if ctor and ctor.style == "unit" and not allow_temporary:
            raise expr.token.error("expected a local place")

    def check_expr(self, expr, scope, expected=None):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "closure":
            return self.closure_type(expr, scope, expected)
        if kind == "indirect_call":
            callee = self.type_of(args[0], scope)
            return self.indirect_type(expr, scope, expected, callee)
        if kind in {"function_pointer", "closure_value"}:
            return self.program.expr_types[id(expr)]
        if kind == "try":
            value = args[0]
            enclosing = self.resolve(self.result)
            hint = None
            if expected is not None and isinstance(enclosing, AppliedType) and enclosing.name in {OPTION, RESULT}:
                hint = AppliedType(enclosing.name, (expected, *enclosing.arguments[1:]))
            actual = self.type_of(value, scope, hint)
            if actual == NEVER:
                return NEVER
            if not isinstance(actual, AppliedType) or actual.name not in {OPTION, RESULT}:
                raise token.error("? requires Option or Result")
            if not isinstance(enclosing, AppliedType) or actual.name != enclosing.name:
                raise token.error("? requires a function returning the same enum family")
            if actual.name == RESULT:
                self.require(actual.arguments[1], enclosing.arguments[1], token)
            expr.args = (value, enclosing)
            return self.resolve(actual.arguments[0])
        if kind == "macro_definition":
            scope.remember(token)
            self.macro_labels[token.definition] = tuple(self.loops)
            return "()"
        if kind == "cast":
            value, target = args
            if isinstance(target, ReferenceType) and isinstance(target.inner, (DynType, SliceType)):
                result = self.type_of(value, scope, target)
                self.require_argument(result, target, token)
                expr.kind, expr.args = "group", (value,)
                return result
            if target in SCALAR_BOUNDS:
                hint = ("u8" if target == "char" else target) if self.unsuffixed_integer(value) else None
                actual = self.type_of(value, scope, hint)
                if actual == NEVER:
                    expr.kind, expr.args = "group", (value,)
                    return NEVER
                if not scalar_cast_supported(actual, target):
                    raise token.error(f"cannot cast {actual} to {target}")
                expr.kind = "scalar_cast"
                return target
            raise token.error("supported casts convert numbers, characters, or borrowed references")
        if kind in {"dyn_reference", "slice_reference"}:
            return self.program.expr_types[id(expr)]
        if kind == "literal":
            annotation, value = args
            hint = self.resolve(expected)
            if expr.unsuffixed and hint in INTEGER_TYPES:
                annotation = hint
            elif expr.unsuffixed and isinstance(hint, TypeVariable) and hint.integer_literal:
                self.array_integers.append((hint, value, token))
                return hint
            if annotation in INTEGER_TYPES:
                checked(value, token, annotation)
            return annotation
        if kind == "byte_string":
            return ReferenceType(ArrayType("u8", ConstValue(len(args), "usize")), False, "'static")
        if kind == "const_value":
            return args[0].const_type
        if kind in {"array", "array_repeat"}:
            return self.array_type(expr, scope, expected)
        if kind == "tuple":
            hint = self.resolve(expected)
            types = hint.elements if isinstance(hint, TupleType) and len(hint.elements) == len(args) else (None,) * len(args)
            elements = tuple(self.type_of(item, scope, kind) for item, kind in zip(args, types))
            if NEVER in elements:
                return NEVER
            result = TupleType(elements)
            self.check_type(result, token)
            if hint is not None:
                self.require(result, hint, token)
            return self.resolve(result)
        if kind in {"index", "range_index"}:
            base, *bounds = args
            annotation = self.type_of(base, scope)
            if isinstance(annotation, ReferenceType):
                base = Expr("deref", base.token, (base, False))
                expr.args = (base, *bounds)
                annotation = self.type_of(base, scope)
            if annotation != NEVER and not isinstance(annotation, (ArrayType, SliceType)):
                raise token.error(f"cannot index {annotation}")
            diverges = annotation == NEVER
            for index in (bounds if kind == "index" else bounds[:2]):
                if index is not None:
                    actual = self.type_of(index, scope, "usize")
                    self.require(actual, "usize", index.token)
                    diverges |= actual == NEVER
            if diverges:
                return NEVER
            return annotation.element if kind == "index" else SliceType(annotation.element)
        if kind == "default":
            return self.program.expr_types[id(expr)]
        if kind == "method":
            return self.method_type(expr, scope, expected)
        if kind == "group":
            return self.type_of(args[0], scope, expected)
        if kind == "name":
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "unit":
                return self.constructor(expr, "unit", expected).annotation
            if expr.type_arguments is not None:
                if "::" not in token.text:
                    raise token.error("type arguments require a generic call or constructor")
            if expr.qualified or "::" in token.text:
                if token.text not in self.functions:
                    return self.constant_type(expr, scope, expected)
            binding = self.local_binding(scope, token)
            if binding is not None:
                return binding.value
            name = self.program.values_in_scope.get(token.module, {}).get(token.text, token.text)
            function = self.functions.get(name)
            if not function:
                return scope.find(token).value
            arguments = self.specialize(function.type_parameters, expr.type_arguments, token)
            annotation = FunctionType(tuple(substitute(kind, arguments) for _, _, kind in function.params),
                                      substitute(function.result, arguments))
            if isinstance(expected, FunctionType):
                self.require(annotation, expected, token)
            expr.kind, expr.args = "function_pointer", (function.name.text, arguments)
            self.pointers.append(expr)
            return self.resolve(annotation)
        if kind == "field":
            value, name = args
            annotation = self.type_of(value, scope)
            if isinstance(annotation, ReferenceType):
                value = Expr("deref", value.token, (value, False))
                expr.args = (value, name)
                annotation = self.type_of(value, scope)
            return self.field_type(annotation, name)
        if kind == "borrow":
            value, mutable = args
            hint = expected.inner if isinstance(expected, ReferenceType) else None
            annotation = self.type_of(value, scope, hint)
            if annotation == NEVER:
                return NEVER
            self.require_place(value, write=mutable, allow_temporary=True)
            if mutable:
                CaptureContext.mark_mutable(scope, value)
            result = ReferenceType(annotation, mutable)
            self.check_type(result, token)
            return result
        if kind == "deref":
            value = args[0]
            annotation = self.type_of(value, scope)
            if annotation == NEVER:
                return NEVER
            if not isinstance(annotation, ReferenceType):
                raise token.error(f"cannot dereference {annotation}")
            return annotation.inner
        if kind == "clone":
            annotation = self.type_of(args[0], scope, expected)
            if isinstance(annotation, ReferenceType) and self.implements(annotation.inner, "Clone"):
                operand = Expr("deref", args[0].token, (args[0], False))
                expr.args = (operand,)
                annotation = self.type_of(operand, scope)
            self.obligation(annotation, "Clone", token)
            return annotation
        if kind == "record":
            ctor = self.constructor(expr, "record", expected)
            fields = args[0]
            indices = self.field_indices(ctor, fields, False, token)
            actual = [self.type_of(value, scope, ctor.fields[index][1])
                      for index, (_, value) in zip(indices, fields)]
            for index, kind in zip(indices, actual):
                self.require(kind, ctor.fields[index][1], token)
            return NEVER if NEVER in actual else ctor.annotation
        if kind == "match":
            return self.match_type(expr, scope, expected)
        if kind == "block":
            statements, tail = args
            inner = Scope(scope)
            diverges = False
            for statement in statements:
                diverges |= self.type_of(statement, inner) == NEVER
            result = self.type_of(tail, inner, expected) if tail else "()"
            return NEVER if diverges else result
        if kind == "let":
            source, annotation, value, otherwise = args
            if annotation is not None:
                annotation = self.fill_holes(annotation, token)
                self.check_type(annotation, token)
            actual = self.type_of(value, scope, annotation) if value else annotation
            if annotation:
                self.require_argument(actual, annotation, token)
                expr.args = (source, annotation, value, otherwise)
                self.ascriptions.append(expr)
            annotation = self.resolve(annotation or actual)
            if isinstance(annotation, ReferenceType):
                if value is None or (otherwise and source.kind in {"bind", "wild"}):
                    raise token.error("reference let requires a simple initialized binding")
            pattern, bindings = self.prepare_pattern(source, annotation)
            if value is None and not bindings:
                raise token.error("uninitialized let requires a local binding")
            self.program.patterns[id(source)] = (pattern, bindings)
            if otherwise:
                self.require(self.type_of(otherwise, scope), NEVER, otherwise.token)
            else:
                missing = uncovered([(pattern,)], (self.resolve(annotation),), self.program)
                if missing is not None:
                    raise token.error(f"refutable let pattern, missing {show_missing(missing[0])}")
            for name, (kind, mutable) in bindings.items():
                scope.bindings[name] = Binding(kind, mutable)
            return NEVER if actual == NEVER else "()"
        if kind == "discard":
            value, requires_unit = args
            actual = self.type_of(value, scope)
            if requires_unit:
                self.require(actual, "()", token)
            return NEVER if actual == NEVER else "()"
        if kind == "if":
            condition, yes, no = args
            test = self.type_of(condition, scope, "bool")
            self.require(test, "bool", condition.token)
            left = self.type_of(yes, scope, expected)
            right_hint = expected if expected is not None else (left if left != NEVER else None)
            right = self.type_of(no, scope, right_hint) if no else "()"
            if left != NEVER and not (shared_coercion(left, right) or shared_coercion(right, left)):
                self.require(right, left, token)
            if test == NEVER:
                return NEVER
            return right if left == NEVER or shared_coercion(left, right) else left
        if kind in {"while", "while_let", "loop", "for"}:
            return self.loop_type(expr, scope)
        if kind == "return":
            actual = self.type_of(args[0], scope, self.result) if args[0] else "()"
            self.require_argument(actual, self.result, token)
            return NEVER
        if kind in {"break", "continue"}:
            target = find_control(self.loops, expr, self.macro_labels)
            if expr.label:
                expr.label = target.expr.label
            if kind == "break":
                if args[0] and target.expr.kind not in {"loop", "block"}:
                    raise token.error("break values require loop or a labeled block")
                actual = self.type_of(args[0], scope) if args[0] else "()"
                target.exit_types.append((actual, token))
            return NEVER
        if kind == "unary":
            operand = args[0]
            hint = self.resolve(expected)
            hint = hint if hint in CONST_TYPES else None
            literal = ungroup(operand)
            if token.text == "-" and literal.kind == "literal" and literal.args[0] in INTEGER_TYPES:
                kind, magnitude = literal.args
                if literal.unsuffixed and hint in INTEGER_TYPES:
                    kind = hint
                lower, _ = INTEGER_BOUNDS[kind]
                if lower < 0 and magnitude == -lower:
                    expr.kind, expr.args = "literal", (kind, lower)
                    expr.unsuffixed = literal.unsuffixed
                    return kind
            actual = self.type_of(operand, scope, hint)
            if actual == NEVER:
                return NEVER
            if token.text == "!" and actual == "bool":
                return "bool"
            if actual in INTEGER_TYPES:
                if token.text == "-" and INTEGER_BOUNDS[actual][0] == 0:
                    raise token.error(f"cannot negate {actual} with '-'")
                return actual
            if token.text == "!" and isinstance(actual, TypeVariable):
                result = TypeVariable("operator result", token)
                self.variables.append(result)
                self.unary_results.append((actual, result, token))
                return result
            self.numeric_operands.append((actual, "i32", token))
            return "i32"
        if kind == "assign":
            left, right = args
            annotation = self.type_of(left, scope)
            self.require_place(left, write=True)
            CaptureContext.mark_mutable(scope, left)
            if isinstance(annotation, ReferenceType):
                raise token.error("reassigning reference bindings is unsupported")
            if isinstance(annotation, SliceType):
                raise token.error("cannot assign an unsized slice value")
            operator = token.text.removesuffix("=")
            hint = annotation if token.text == "=" or (
                operator not in SHIFTS and self.unsuffixed_integer(right)
            ) else None
            actual = self.type_of(right, scope, hint)
            if token.text == "=":
                self.require(actual, annotation, token)
            else:
                self.binary_type(replace(token, text=operator), annotation, actual)
            return NEVER if NEVER in {annotation, actual} else "()"
        if kind == "binary":
            hint = self.resolve(expected)
            if token.text in SHIFTS:
                left, right = args
                left_hint = hint if hint in INTEGER_TYPES and self.unsuffixed_integer(left) else None
                return self.binary_type(token, self.type_of(left, scope, left_hint), self.type_of(right, scope))
            if token.text in {"&&", "||"}:
                hint = "bool"
            elif (hint not in INTEGER_TYPES and not (hint == "bool" and token.text in BITWISE)) or token.text in COMPARE:
                hint = None
            left, right = self.operand_types(*args, scope, hint, same_type=token.text in COMPARE)
            return self.binary_type(token, left, right)
        if kind == "call":
            return self.call_type(expr, scope, expected)
        if kind in {"print", "panic"}:
            if not args:
                return NEVER if kind == "panic" else "()"
            template, *values = args
            template = ungroup(template)
            expr.args = (template, *values)
            if template.kind != "literal" or template.args[0] != "&str":
                raise template.token.error("println! requires a format string literal")
            parts = format_parts(template.args[1], template.token)
            placeholders = [(mode, name) for mode, name in parts if mode != "text"]
            if sum(name is None for _, name in placeholders) != len(values):
                raise token.error("println! argument count does not match format string")
            types = [self.type_of(value, scope) for value in values]
            positional = iter(types)
            for mode, name in placeholders:
                if name:
                    actual = scope.find(replace(template.token, text=name)).value
                else:
                    actual = next(positional)
                if isinstance(actual, ReferenceType):
                    actual = actual.inner
                elif isinstance(actual, SliceType):
                    raise token.error("formatting an unsized slice requires a reference")
                if mode == "display" and actual == "()":
                    raise token.error("() requires debug formatting")
                self.obligation(actual, "Debug" if mode == "debug" else "Display", token)
            return NEVER if kind == "panic" or NEVER in types else "()"
        raise AssertionError(kind)

    @staticmethod
    def unsuffixed_integer(expr):
        expr = ungroup(expr)
        if expr.kind == "unary" and expr.token.text == "-":
            return Checker.unsuffixed_integer(expr.args[0])
        if expr.kind == "binary" and expr.token.text in SHIFTS:
            return Checker.unsuffixed_integer(expr.args[0])
        if expr.kind == "binary" and expr.token.text in {*ARITHMETIC, "/", "%"}:
            return all(map(Checker.unsuffixed_integer, expr.args))
        return expr.unsuffixed

    def operand_types(self, left, right, scope, hint=None, *, same_type=False):
        left_literal, right_literal = self.unsuffixed_integer(left), self.unsuffixed_integer(right)
        if hint is None and left_literal and not right_literal:
            right_type = self.type_of(right, scope)
            left_type = self.type_of(left, scope, right_type if right_type in INTEGER_TYPES else None)
        else:
            left_type = self.type_of(left, scope, hint if hint == "bool" or left_literal else None)
            right_hint = left_type if left_type != NEVER else hint
            right_type = self.type_of(right, scope, right_hint if hint == "bool" or same_type or right_literal else None)
        return left_type, right_type

    @staticmethod
    def integer_pair(left, right):
        if left in INTEGER_TYPES:
            return left
        return right if right in INTEGER_TYPES else "i32"

    def array_type(self, expr, scope, expected):
        kind, token, args = expr.kind, expr.token, expr.args
        expected = self.resolve(expected)
        element = expected.element if isinstance(expected, (ArrayType, SliceType)) else None
        if args and args[0].unsuffixed and (element is None or isinstance(element, TypeVariable)):
            literal = TypeVariable("integer literal", token, integer_literal=True)
            self.variables.append(literal)
            if element is not None:
                self.require(literal, element, token)
            element = literal
        diverges = False
        if kind == "array_repeat":
            value, length = args
            element = self.type_of(value, scope, element)
            if not isinstance(length, ConstValue) or length.value > 1:
                self.obligation(element, "Copy", token)
            diverges = element == NEVER
        else:
            length = ConstValue(len(args), "usize")
            for value in args:
                found = self.type_of(value, scope, element)
                diverges |= found == NEVER
                if found != NEVER:
                    if element is None:
                        element = found
                    else:
                        self.require(found, element, value.token)
            if element is None and not diverges:
                element = TypeVariable("array element", token)
                self.variables.append(element)
        if diverges:
            return NEVER
        result = ArrayType(element, length)
        self.check_type(result, token)
        if expected is not None and not isinstance(expected, SliceType):
            self.require(result, expected, token)
        return self.resolve(result)

    def call_type(self, expr, scope, expected):
        token, args = expr.token, expr.args
        if expr.local_callee:
            local = self.local_binding(scope, expr.local_callee)
            if local is not None:
                pointer = Expr("name", expr.local_callee)
                self.program.expr_types[id(pointer)] = self.resolve(local.value)
                expr.kind, expr.args = "indirect_call", (pointer, *args)
                return self.indirect_type(expr, scope, expected, local.value)
        if expr.qualified:
            owner, trait = expr.qualified
            self.check_type(owner, token)
            if trait:
                return self.trait_call(expr, scope, expected, owner, trait, token.text)
            return self.call_on_type(expr, scope, expected, owner, token.text)
        ctor = self.program.constructors.get(token.text)
        if ctor and ctor.style == "tuple":
            ctor = self.constructor(expr, "tuple", expected)
            params = tuple(kind for _, kind in ctor.fields)
            result = ctor.annotation
            arguments = {}
        elif token.text.endswith("::default"):
            name = token.text.removesuffix("::default")
            if args:
                raise token.error("default takes no arguments")
            if name == "Default":
                if expr.type_arguments is not None:
                    raise token.error("Default::default takes no explicit type arguments")
                result = TypeVariable("Default", token)
                self.variables.append(result)
            else:
                parameters = self.program.type_parameters.get(name, ())
                specialized = self.specialize(parameters, expr.type_arguments, token)
                result = (AppliedType(name, tuple(specialized.values())) if specialized
                          else self.parameters.get(name, name))
                self.check_type(result, token)
            if expected is not None:
                self.require(result, expected, token)
            self.obligation(result, "Default", token)
            expr.kind = "default"
            return result
        else:
            function = self.functions.get(token.text)
            if function is None:
                return self.associated_type(expr, scope, expected)
            arguments = self.specialize(function.type_parameters, expr.type_arguments, token)
            return self.function_call(expr, scope, expected, function, arguments)
        return self.check_call(expr, scope, expected, params, result, arguments)

    def function_call(self, expr, scope, expected, function, arguments, first=None):
        if function.associated_const != (expr.kind == "name"):
            raise expr.token.error("associated constants and methods use different call syntax")
        expr.kind = "call"
        if function.method_parameters:
            local = self.specialize(function.method_parameters, expr.type_arguments, expr.token)
            arguments = arguments | local
        params = tuple(substitute(self.resolve(kind), arguments) for _, _, kind in function.params)
        result = substitute(self.resolve(function.result), arguments)
        annotation = self.check_call(expr, scope, expected, params, result, arguments, first)
        self.program.call_functions[id(expr)] = self.effective_function(
            function, tuple(map(self.resolve, params)), self.resolve(result)
        )
        return annotation

    def check_call(self, expr, scope, expected, params, result, arguments, first=None):
        token, args = expr.token, expr.args
        if expr.kind == "indirect_call":
            args = args[1:]
        if len(args) != len(params):
            raise token.error(f"{token.text} takes {len(params)} arguments")
        defer_projection = isinstance(self.resolve(result), AssociatedType)
        if expected is not None and arguments and not defer_projection:
            self.require_argument(result, expected, token)
        actual_types = []
        for index, (expr_arg, parameter) in enumerate(zip(args, params)):
            hint = self.resolve(parameter)
            actual = first if index == 0 and first is not None else self.type_of(expr_arg, scope, hint)
            self.require_argument(actual, parameter, expr_arg.token)
            if isinstance(hint, ReferenceType) and hint.mutable:
                CaptureContext.mark_mutable(scope, expr_arg)
            actual_types.append(actual)
        if expected is not None and defer_projection:
            self.require_argument(result, expected, token)
        self.program.call_types[id(expr)] = (params, arguments)
        return NEVER if NEVER in actual_types else self.resolve(result)

    @staticmethod
    def local_binding(scope, token):
        try:
            return scope.find(token)
        except RustError:
            return None

    def indirect_type(self, expr, scope, expected, callee):
        callee = self.resolve(callee)
        if callee == NEVER:
            expr.kind, expr.args = "group", (expr.args[0],)
            return NEVER
        if not isinstance(callee, FunctionType):
            raise expr.token.error(f"expected a callable value, found {callee}")
        token = expr.token
        params = [(replace(token, text="_"), False, kind) for kind in callee.parameters]
        inputs = Parser.return_inputs(token, params, callee.result, {})
        self.program.call_functions[id(expr)] = Function(token, params, callee.result, None, inputs)
        self.indirect_calls.append(expr)
        return self.check_call(expr, scope, expected, callee.parameters, callee.result, {})

    def closure_type(self, expr, scope, expected):
        raw, declared, body, by_move = expr.args
        hint = self.resolve(expected)
        if isinstance(hint, FunctionType) and len(hint.parameters) != len(raw):
            raise expr.token.error("closure has a different parameter count")
        params = []
        captures = CaptureContext(scope, by_move)
        local = Scope(captures=captures)
        for index, (name, mutable, annotation) in enumerate(raw):
            if annotation is None and isinstance(hint, FunctionType):
                annotation = hint.parameters[index]
            if annotation is None:
                annotation = TypeVariable(name.text, name)
                self.variables.append(annotation)
            self.check_type(annotation, name)
            if name.text != "_":
                if name.key in local.bindings:
                    raise name.error(f"duplicate closure parameter {name.text!r}")
                local.bindings[name.key] = Binding(annotation, mutable)
            params.append((name, mutable, annotation))
        result = declared if declared is not None else hint.result if isinstance(hint, FunctionType) else None
        if result is None:
            result = TypeVariable("closure result", expr.token)
            self.variables.append(result)
        previous_result, previous_loops = self.result, self.loops
        self.result, self.loops = result, []
        try:
            actual = self.type_of(body, local, result)
        finally:
            self.result, self.loops = previous_result, previous_loops
        self.require_argument(actual, result, expr.token)
        if actual == NEVER and isinstance(self.resolve(result), TypeVariable):
            self.require(result, "()", expr.token)
        name = replace(expr.token, text=f"@closure:{len(self.closures)}")
        arguments = {parameter: parameter for parameter in self.parameters.values()}
        function = Function(name, params, result, body, type_parameters=tuple(arguments))
        function.captures = tuple(captures.captures.values())
        self.closures.append(function)
        self.program.functions[name.text] = function
        expr.kind, expr.args = ("closure_value" if function.captures else "function_pointer"), (name.text, arguments)
        self.pointers.append(expr)
        return FunctionType(tuple(kind for _, _, kind in params), self.resolve(result),
                            name.text if function.captures else None)

    @staticmethod
    def has_receiver(method):
        return bool(method.params and method.params[0][0].text == "self")

    def method_type(self, expr, scope, expected):
        name = expr.token.text
        original, *rest = expr.args
        actual = self.type_of(original, scope)
        if actual == NEVER:
            expr.kind, expr.args = "group", (original,)
            return NEVER
        base = actual.inner if isinstance(actual, ReferenceType) else actual
        if isinstance(base, AppliedType) and base.name in {OPTION, RESULT}:
            result = self.prelude_method(expr, scope, original, actual, base, rest)
            if result is not None:
                return result
        if isinstance(base, (ArrayType, SliceType)) and name in {"len", "is_empty"} and not rest:
            if isinstance(actual, ReferenceType):
                original = Expr("deref", expr.token, (original, False))
                self.type_of(original, scope)
            expr.kind, expr.args = ("length" if name == "len" else name), (original,)
            return "usize" if name == "len" else "bool"
        # There can be multiple inherent impl blocks on one target.
        for impl in self.program.impls:
            if impl.trait or name not in impl.methods:
                continue
            arguments = {}
            if match_types(impl.target, base, impl.parameters, arguments) and all(
                self.implements(arguments[p], bound) for p in impl.parameters for bound in p.bounds
            ):
                method = impl.methods[name]
                if self.has_receiver(method):
                    self.visible_method(method, expr.token)
                    return self.receiver_call(expr, scope, expected, method, arguments, actual)
        candidates = [trait for trait in self.program.traits.values()
                      if name in trait.methods and self.has_receiver(trait.methods[name])
                      and self.implements(base, trait.name.text)
                      and self.trait_available(base, trait.name.text, expr.token)]
        if len(candidates) > 1:
            raise expr.token.error(f"multiple traits provide method {name!r}")
        if candidates:
            trait = candidates[0]
            if isinstance(base, DynType) and trait.methods[name].self_sized:
                raise expr.token.error(f"method {name!r} requires Self: Sized")
            self.program.dispatches[id(expr)] = (trait.name.text, base, name)
            return self.receiver_call(expr, scope, expected, trait.methods[name],
                                      {trait.self_type: base}, actual)
        if name == "clone" and not rest:
            expr.kind = "clone"
            return self.check_expr(expr, scope, expected)
        raise expr.token.error(f"no method {name!r} for {base}")

    def prelude_method(self, expr, scope, original, actual, base, rest):
        name = expr.token.text
        checks = {"is_some": "Some", "is_none": "None"} if base.name == OPTION else {
            "is_ok": "Ok", "is_err": "Err"
        }
        unwrap = name in {"unwrap", "expect", "unwrap_or"}
        if not unwrap and name not in checks:
            return None
        if base.name == RESULT and name in {"unwrap", "expect"}:
            self.obligation(base.arguments[1], "Debug", expr.token)
        count = int(name in {"expect", "unwrap_or"})
        if len(rest) != count:
            raise expr.token.error(f"{name} takes {count} arguments")
        if isinstance(actual, ReferenceType):
            original = Expr("deref", original.token, (original, False))
            self.type_of(original, scope)
        expr.kind = "enum_method"
        success = ("Some" if base.name == OPTION else "Ok") if unwrap else checks[name]
        expr.args = (original, tuple(rest), success, unwrap)
        result = base.arguments[0] if unwrap else "bool"
        if rest:
            hint = "&str" if name == "expect" else result
            actual = self.type_of(rest[0], scope, hint)
            self.require(actual, hint, rest[0].token)
            if actual == NEVER:
                return NEVER
        return self.resolve(result)

    def trait_available(self, base, name, token):
        if isinstance(base, DynType):
            return name in self.program.trait_closure({base.trait})
        if isinstance(base, TypeParameter) and name in self.program.trait_closure(base.bounds):
            return True
        return name in self.program.traits_in_scope.get(token.module, ())

    @staticmethod
    def visible_method(method, token):
        scope = method.visible_from
        if token.module[:len(scope)] != scope:
            raise token.error(f"method {token.text!r} is private")

    def receiver_call(self, expr, scope, expected, method, arguments, actual):
        original, *rest = expr.args
        if expr.type_arguments is not None and not method.method_parameters:
            raise expr.token.error("method takes no generic arguments")
        wanted = substitute(method.params[0][2], arguments)
        if isinstance(wanted, ReferenceType) and not isinstance(actual, ReferenceType):
            original = Expr("borrow", expr.token, (original, wanted.mutable), auto_borrow=True)
            actual = self.type_of(original, scope)
        elif isinstance(actual, ReferenceType) and not isinstance(wanted, ReferenceType):
            original = Expr("deref", expr.token, (original, False))
            actual = self.type_of(original, scope)
        expr.kind = "call"
        expr.args = (original, *rest)
        return self.function_call(expr, scope, expected, method, arguments, actual)

    def associated_type(self, expr, scope, expected):
        owner, separator, name = expr.token.text.partition("::")
        if not separator:
            raise expr.token.error(f"unknown function {owner!r}")
        trait = self.program.traits.get(owner)
        if trait and name in trait.methods:
            actual = None
            if self.has_receiver(trait.methods[name]) and expr.args:
                actual = self.type_of(expr.args[0], scope)
                base = actual.inner if isinstance(actual, ReferenceType) else actual
            else:
                base = TypeVariable("Self", expr.token)
                self.variables.append(base)
            return self.trait_call(expr, scope, expected, base, owner, name, actual)
        candidates = [impl for impl in self.program.impls if impl.trait is None and name in impl.methods
                      and (impl.target.name if isinstance(impl.target, AppliedType) else impl.target) == owner]
        if len(candidates) != 1:
            if owner in self.program.types and not self.program.type_parameters[owner]:
                return self.call_on_type(expr, scope, expected, owner, name)
            raise expr.token.error(f"unknown or ambiguous associated function {expr.token.text!r}")
        impl = candidates[0]
        method = impl.methods[name]
        self.visible_method(method, expr.token)
        explicit = None if method.method_parameters else expr.type_arguments
        arguments = self.specialize(impl.parameters, explicit, expr.token)
        return self.function_call(expr, scope, expected, method, arguments)

    def constant_type(self, expr, scope, expected):
        if expr.qualified:
            owner, trait = expr.qualified
            self.check_type(owner, expr.token)
            if trait:
                return self.trait_call(expr, scope, expected, owner, trait, expr.token.text)
            return self.call_on_type(expr, scope, expected, owner, expr.token.text)
        return self.associated_type(expr, scope, expected)

    def trait_call(self, expr, scope, expected, owner, trait_name, name, first=None):
        trait = self.program.traits.get(trait_name)
        if not trait or name not in trait.methods:
            raise expr.token.error(f"trait {trait_name!r} has no function {name!r}")
        if isinstance(owner, DynType) and trait.methods[name].self_sized:
            raise expr.token.error(f"method {name!r} requires Self: Sized")
        self.obligation(owner, trait_name, expr.token)
        self.program.dispatches[id(expr)] = (trait_name, owner, name)
        return self.function_call(expr, scope, expected, trait.methods[name], {trait.self_type: owner}, first)

    def call_on_type(self, expr, scope, expected, owner, name):
        for impl in self.program.impls:
            if impl.trait is not None or name not in impl.methods:
                continue
            args = {}
            if match_types(impl.target, owner, impl.parameters, args):
                self.visible_method(impl.methods[name], expr.token)
                for parameter, argument in args.items():
                    for trait in parameter.bounds:
                        self.obligation(argument, trait, expr.token)
                return self.function_call(expr, scope, expected, impl.methods[name], args)
        traits = [trait.name.text for trait in self.program.traits.values()
                  if name in trait.methods and self.implements(owner, trait.name.text)
                  and self.trait_available(owner, trait.name.text, expr.token)]
        if len(traits) == 1:
            return self.trait_call(expr, scope, expected, owner, traits[0], name)
        raise expr.token.error(f"unknown or ambiguous associated function {name!r} for {owner}")

    def binary_type(self, token, left, right):
        word = token.text
        if word in {"&&", "||"}:
            self.require(left, "bool", token)
            self.require(right, "bool", token)
            return NEVER if left == NEVER else "bool"
        if word in COMPARE:
            if left != NEVER and right != NEVER:
                self.require(right, left, token)
                self.obligation(left, "PartialEq" if word in {"==", "!="} else "PartialOrd", token)
            return NEVER if NEVER in {left, right} else "bool"
        if word in SHIFTS:
            self.numeric_operands.extend(((left, None, token), (right, None, token)))
            return NEVER if NEVER in {left, right} else left
        if word in BITWISE and "bool" in {left, right}:
            self.require(left, "bool", token)
            self.require(right, "bool", token)
            return NEVER if NEVER in {left, right} else "bool"
        integer = self.integer_pair(left, right)
        self.numeric_operands.extend(((left, integer, token), (right, integer, token)))
        return NEVER if NEVER in {left, right} else integer


@dataclass(frozen=True, eq=False)
class Local:
    token: Token
    mutable: bool
    loan: Loan | None = None
    static: bool = False
    shared_capture: bool = False


@dataclass(frozen=True)
class Place:
    local: Local
    fields: tuple = ()
    holder: Local | None = field(default=None, compare=False)
    indexed: bool = field(default=False, compare=False)

    def child(self, field):
        if self.indexed:
            return self
        return Place(self.local, (*self.fields, field), self.holder)

    def contains(self, other):
        return self.local is other.local and other.fields[:len(self.fields)] == self.fields

    def overlaps(self, other):
        return any(left.contains(right) or right.contains(left)
                   for left in self.regions() for right in other.regions())

    def regions(self):
        if self.holder and self.holder.loan.possible:
            return self.holder.loan.possible
        return (self,)

    @property
    def name(self):
        return ".".join((self.local.token.text, *self.fields))


@dataclass(frozen=True, eq=False)
class Loan:
    place: Place
    mutable: bool
    parents: tuple = ()
    two_phase: bool = False
    possible: tuple = ()
    expiring: frozenset = frozenset()

    def ancestors(self):
        visited = set()
        pending = [self]
        while pending:
            loan = pending.pop()
            if loan in visited:
                continue
            visited.add(loan)
            yield loan
            pending.extend(loan.parents)

    def regions(self):
        return self.possible or (self.place,)


@dataclass(frozen=True)
class Effect:
    kind: str
    place: Place
    token: Token
    loan: Loan | None = None
    roots: frozenset = frozenset()

    def reference_uses(self):
        if self.kind in {"declare", "uninitialized", "end_temporaries"}:
            return set()
        if self.place.holder:
            return {self.place.holder}
        if self.place.local.loan:
            return {self.place.local}
        return set()


@dataclass(eq=False)
class FlowNode:
    effect: Effect | None = None
    edges: list = field(default_factory=list)


@dataclass(frozen=True)
class MoveState:
    missing: frozenset = frozenset()
    written: frozenset = frozenset()

    def merge(self, other):
        return MoveState(self.missing | other.missing, self.written | other.written)

    def transfer(self, effect):
        if effect is None:
            return self
        kind, place = effect.kind, effect.place
        if kind == "write" and place.holder:
            return self
        if kind == "move":
            return MoveState(self.missing | {(place, "moved")}, self.written)
        if kind in {"declare", "uninitialized", "write"}:
            missing = frozenset(item for item in self.missing if not place.contains(item[0]))
            if kind == "uninitialized":
                return MoveState(missing | {(place, "uninitialized")}, self.written - {place.local})
            return MoveState(missing, self.written | {place.local})
        return self

    def validate(self, effect):
        if effect is None:
            return
        kind, place, token = effect.kind, effect.place, effect.token
        if kind == "return_reference":
            for owner in place.local.loan.regions():
                if owner.local not in effect.roots and not owner.local.static:
                    raise token.error(f"cannot return a reference to {owner.name!r} with the declared lifetime")
        if kind == "static_reference":
            if any(not owner.local.static for owner in place.local.loan.regions()):
                raise token.error("reference does not live for 'static")
        if kind == "block_reference":
            for owner in place.local.loan.regions():
                if owner.local in effect.roots:
                    raise token.error(f"reference to {owner.name!r} outlives its block")
        if place.holder and any(item == Place(place.holder) for item, _ in self.missing):
            raise token.error(f"use of moved reference {place.holder.token.text!r}")
        if kind == "deref_move":
            raise token.error(f"cannot move out of borrowed place {place.name!r}")
        if kind == "indexed_move":
            raise token.error(f"cannot move out of array index in {place.name!r}")
        if kind == "capture_move":
            raise token.error(f"cannot move out of shared capture {place.name!r}")
        if kind == "guard_move":
            raise token.error(f"cannot move out of {place.name!r} in a pattern guard")
        if kind == "guard_write":
            raise token.error(f"cannot assign to {place.name!r} in a pattern guard")
        # The referent was initialized when borrowed. Only the holder can
        # have moved since then. Checking its candidate owners after a branch
        # would mix the referent on one path with moves from another path.
        if kind in {"read", "move", "borrow"} and not place.holder:
            unavailable = [(item, reason) for item, reason in self.missing if place.overlaps(item)]
            if unavailable:
                item, reason = min(unavailable, key=lambda entry: (len(entry[0].fields), entry[0].name, entry[1]))
                if item == place or item.contains(place):
                    raise token.error(f"use of {reason} value {item.name!r}")
                raise token.error(f"use of partially {reason} value {place.name!r} ({item.name})")
        if kind == "write":
            if not place.holder and not place.local.mutable and (place.fields or place.local in self.written):
                raise token.error(f"cannot assign to immutable binding {place.local.token.text!r}")
            if not place.holder:
                for item, reason in sorted(self.missing, key=lambda entry: (entry[0].name, entry[1])):
                    if item != place and item.contains(place):
                        raise token.error(f"cannot assign to a field of {reason} value {item.name!r}")


class Ownership:
    """Lower checked expressions to a control-flow graph, then check move paths."""

    def __init__(self, program):
        self.program = program
        self.loops = []
        self.guarded = set()
        self.guard_borrows = ()
        self.guard_tests = ()

    def check(self):
        functions = tuple(self.program.functions.values())
        ordered = tuple(function for function in functions if function.name.text.startswith("@closure:"))
        ordered += tuple(function for function in functions if not function.name.text.startswith("@closure:"))
        for function in ordered:
            scope = Scope()
            entry = FlowNode()
            cursor = entry
            self.return_roots = set()
            self.function = function
            self.owned_captures = set()
            for capture in function.captures:
                loan = None
                annotation = capture.annotation
                if isinstance(annotation, ReferenceType):
                    mutable = annotation.mutable and (capture.by_move or capture.mutable)
                    referent = Local(replace(capture.name, text=f"*{capture.name.text}"), mutable,
                                     static=annotation.lifetime == "'static")
                    loan = Loan(Place(referent), mutable)
                    cursor = self.emit(cursor, "declare", loan.place, capture.name)
                local = Local(capture.name, capture.mutable, loan, shared_capture=not capture.by_move)
                if capture.by_move:
                    self.owned_captures.add(local)
                scope.bindings[capture.name.key] = local
                cursor = self.emit(cursor, "declare", Place(local), capture.name)
            for index, (token, mutable, annotation) in enumerate(function.params):
                if token.text == "_":
                    continue
                loan = None
                if isinstance(annotation, ReferenceType):
                    referent = Local(Token(f"*{token.text}", token.line, token.column), annotation.mutable,
                                     static=annotation.lifetime == "'static")
                    loan = Loan(Place(referent), annotation.mutable)
                    if index in function.return_inputs:
                        self.return_roots.add(referent)
                    cursor = self.emit(cursor, "declare", loan.place, token)
                local = Local(token, mutable, loan)
                scope.bindings[token.key] = local
                cursor = self.emit(cursor, "declare", Place(local), token)
            self.lower_return(function.body, scope, cursor)
            self.check_graph(entry)

    @staticmethod
    def check_graph(entry):
        incoming = {entry: MoveState()}
        pending = deque([entry])
        queued = {entry}
        while pending:
            node = pending.popleft()
            queued.remove(node)
            output = incoming[node].transfer(node.effect)
            for successor in node.edges:
                previous = incoming.get(successor)
                combined = output if previous is None else previous.merge(output)
                if combined != previous:
                    incoming[successor] = combined
                    if successor not in queued:
                        pending.append(successor)
                        queued.add(successor)
        for node, state in incoming.items():
            state.validate(node.effect)
        Ownership.check_loans(incoming)

    @staticmethod
    def check_loans(nodes):
        if not any(node.effect and (node.effect.kind == "borrow" or node.effect.reference_uses())
                   for node in nodes):
            return
        predecessors = {node: [] for node in nodes}
        for node in nodes:
            for successor in node.edges:
                predecessors[successor].append(node)
        live = {node: set() for node in nodes}
        # Reference bindings carry loans back to their last reachable uses.
        pending = deque(reversed(nodes))
        queued = set(nodes)
        while pending:
            node = pending.popleft()
            queued.remove(node)
            before = set().union(*(live[successor] for successor in node.edges))
            effect = node.effect
            if effect:
                if effect.kind in {"declare", "uninitialized"}:
                    before.discard(effect.place.local)
                before.update(effect.reference_uses())
            if before != live[node]:
                live[node] = before
                for previous in predecessors[node]:
                    if previous not in queued:
                        pending.append(previous)
                        queued.add(previous)
        for node, holders in live.items():
            if node.effect:
                active = {loan for local in holders for loan in local.loan.ancestors()}
                Ownership.validate_loans(node.effect, active)

    @staticmethod
    def validate_loans(effect, live):
        kind, place, token = effect.kind, effect.place, effect.token
        if kind == "end_temporaries":
            if place.local.loan in live:
                raise token.error("temporary value dropped while borrowed")
            return
        if kind not in {"read", "write", "move", "borrow", "activate"}:
            return
        if kind == "borrow" and effect.loan.mutable and not place.holder and not place.local.mutable:
            raise token.error(f"cannot borrow immutable place {place.name!r} as mutable")
        if place.holder and not place.holder.loan.mutable and (
            kind == "write" or kind == "borrow" and effect.loan.mutable
        ):
            raise token.error(f"cannot modify {place.name!r} through a shared reference")
        if kind == "activate":
            parent = place.local.loan
            place = parent.place
            regions = parent.regions()
        else:
            parent = place.holder.loan if place.holder else None
            regions = place.regions()
        permitted = set(parent.ancestors()) if parent else set()
        reference = place.local.loan
        if kind == "move" and reference:
            if any(loan is not reference and reference in loan.ancestors() for loan in live):
                raise token.error(f"cannot move reference {place.name!r} while it is reborrowed")
        exclusive = kind in {"move", "write", "activate"} or (
            kind == "borrow" and effect.loan.mutable and not effect.loan.two_phase
        )
        for loan in live - permitted:
            active_mutable = loan.mutable and not loan.two_phase
            overlapping = next((origin for origin in loan.regions()
                                if any(region.overlaps(origin) for region in regions)), None)
            if overlapping is not None and (exclusive or active_mutable):
                action = {"read": "read", "write": "assign to", "move": "move",
                          "borrow": "borrow", "activate": "borrow"}[kind]
                mode = "mutably" if active_mutable else "immutably"
                raise token.error(f"cannot {action} {place.name!r} while {overlapping.name!r} is {mode} borrowed")

    @staticmethod
    def connect(source, target):
        if source is not None:
            source.edges.append(target)

    def emit(self, cursor, kind, place, token, *, loan=None, roots=frozenset()):
        node = FlowNode(Effect(kind, place, token, loan, roots))
        self.connect(cursor, node)
        return node

    def join(self, *cursors):
        node = FlowNode()
        for cursor in cursors:
            self.connect(cursor, node)
        return node

    def lower_address(self, expr, scope, cursor):
        if expr.kind == "group":
            return self.lower_address(expr.args[0], scope, cursor)
        if expr.kind == "name":
            ctor = self.program.constructors.get(expr.token.text)
            if not ctor or ctor.style != "unit":
                return cursor, Place(scope.find(expr.token))
        if expr.kind == "field":
            base, name = expr.args
            cursor, place = self.lower_address(base, scope, cursor)
            if place is not None:
                return cursor, place.child(name.text)
        if expr.kind in {"index", "range_index"}:
            base, *bounds = expr.args
            cursor, place = self.lower_address(base, scope, cursor)
            if place is not None:
                for index in (bounds if expr.kind == "index" else bounds[:2]):
                    if index is not None:
                        cursor = self.lower(index, scope, cursor)
                return cursor, replace(place, indexed=True)
        if expr.kind == "deref":
            value = ungroup(expr.args[0])
            if value.kind == "name":
                return cursor, self.referent(scope.find(value.token))
            if isinstance(self.annotation(value), ReferenceType):
                cursor, loan = self.lower_reference(value, scope, cursor)
                cursor, holder = self.keep_loan(cursor, loan, value.token)
                return cursor, self.referent(holder.local)
        return cursor, None

    def annotation(self, expr):
        return self.program.expr_types[id(expr)]

    def access(self, cursor, place, annotation, token, *, consume):
        if consume and not self.program.is_copy(annotation):
            action = self.move_action(place)
        else:
            action = "read"
        return self.emit(cursor, action, place, token)

    @staticmethod
    def referent(holder):
        owner = holder.loan.place
        return Place(owner.local, owner.fields, holder)

    def start_loan(self, cursor, place, mutable, token, *, two_phase=False, expiring=frozenset()):
        parent = place.holder.loan if place.holder else None
        parents = (parent,) if parent else ()
        possible = parent.possible if parent else ()
        if parent:
            expiring = expiring | parent.expiring
        loan = Loan(Place(place.local, place.fields), mutable, parents, two_phase, possible, expiring)
        protected = self.guard_borrows + self.guard_tests
        if mutable and (place.local in self.guarded or any(place.overlaps(item) for item in protected)):
            return self.emit(cursor, "guard_write", place, token), loan
        return self.emit(cursor, "borrow", place, token, loan=loan), loan

    def lower_reference(self, expr, scope, cursor, *, reborrow=None, two_phase=False, extend_to=None):
        expr = ungroup(expr)
        annotation = self.annotation(expr)
        if annotation == NEVER:
            return self.lower(expr, scope, cursor), None
        if expr.kind in {"dyn_reference", "slice_reference"}:
            return self.lower_reference(expr.args[0], scope, cursor, reborrow=reborrow,
                                        two_phase=two_phase, extend_to=extend_to)
        if expr.kind in {"call", "indirect_call"}:
            return self.lower_call(expr, scope, cursor)
        if expr.kind == "byte_string":
            place = Place(Local(expr.token, False, static=True))
            cursor = self.emit(cursor, "declare", place, expr.token)
            return self.start_loan(cursor, place, False, expr.token)
        if expr.kind == "block":
            statements, tail = expr.args
            inner = Scope(scope)
            for statement in statements:
                cursor = self.lower(statement, inner, cursor)
            cursor, loan = self.lower_reference(tail, inner, cursor, extend_to=extend_to)
            return self.leave_reference_scope(cursor, loan, inner, tail.token), loan
        if expr.kind == "if":
            condition, yes, no = expr.args
            cursor = self.lower(condition, scope, cursor)
            yes_end, yes_loan = self.lower_reference(yes, scope, cursor, extend_to=extend_to)
            no_end, no_loan = self.lower_reference(no, scope, cursor, extend_to=extend_to)
            loans = tuple(loan for loan in (yes_loan, no_loan) if loan)
            return self.join(yes_end, no_end), self.merge_loans(loans, annotation.mutable)
        if expr.kind == "match":
            return self.lower_match(expr, scope, cursor, result_type=annotation, extend_to=extend_to)
        if expr.kind == "borrow":
            target, mutable = expr.args
            cursor, place = self.lower_address(target, scope, cursor)
            expiring = frozenset()
            if place is None:
                cursor = self.lower(target, scope, cursor)
                promoted = not mutable and self.promotable(target)
                local = Local(Token("<temporary value>", expr.token.line, expr.token.column),
                              not promoted, static=promoted)
                place = Place(local)
                cursor = self.emit(cursor, "declare", place, expr.token)
                if not promoted:
                    if extend_to is None:
                        expiring = frozenset({local})
                    else:
                        extend_to.owned.add(local)
            return self.start_loan(cursor, place, mutable, expr.token, expiring=expiring,
                                   two_phase=two_phase and expr.auto_borrow)
        holder = scope.find(expr.token)
        if reborrow:
            return self.start_loan(cursor, self.referent(holder), reborrow.mutable, expr.token,
                                   two_phase=two_phase)
        return self.lower(expr, scope, cursor), holder.loan

    def promotable(self, expr):
        expr = ungroup(expr)
        if expr.kind == "literal":
            return True
        if expr.kind in {"name", "call", "record"}:
            ctor = self.program.constructors.get(expr.token.text)
            style = {"name": "unit", "call": "tuple", "record": "record"}[expr.kind]
            if not ctor or ctor.style != style:
                return False
            values = (value for _, value in expr.args[0]) if expr.kind == "record" else expr.args
            return all(self.promotable(value) for value in values)
        if expr.kind in {"unary", "binary", "tuple"}:
            return all(self.promotable(operand) for operand in expr.args)
        if expr.kind in {"array", "array_repeat"}:
            elements = expr.args[:1] if expr.kind == "array_repeat" else expr.args
            return all(self.promotable(value) for value in elements)
        if expr.kind == "block":
            statements, tail = expr.args
            return not statements and (tail is None or self.promotable(tail))
        return False

    @staticmethod
    def merge_loans(loans, mutable, *, from_call=False):
        if not loans:
            return None
        if not from_call and all(loan is loans[0] for loan in loans):
            return loans[0]
        origins = tuple(dict.fromkeys(
            place for loan in loans if not mutable or loan.mutable for place in loan.regions()
        ))
        # A diverging callee can declare a mutable result tied only to shared
        # inputs. A conservative placeholder keeps dead continuations lowerable.
        if not origins:
            origins = loans[0].regions()
        # At a call boundary a reserved argument borrow has already activated.
        parents = tuple(replace(loan, two_phase=False) if loan.two_phase else loan for loan in loans)
        expiring = frozenset().union(*(loan.expiring for loan in loans))
        return Loan(origins[0], mutable, parents, possible=origins, expiring=expiring)

    def leave_reference_scope(self, cursor, loan, scope, token):
        if loan and any(origin.local in scope.owned for origin in loan.regions()):
            cursor, holder = self.keep_loan(cursor, loan, token)
            cursor = self.emit(cursor, "block_reference", holder, token, roots=frozenset(scope.owned))
        return cursor

    def lower_return(self, expr, scope, cursor):
        if not isinstance(self.annotation(expr), ReferenceType):
            return self.lower(expr, scope, cursor)
        cursor, loan = self.lower_reference(expr, scope, cursor)
        if loan:
            cursor, holder = self.keep_loan(cursor, loan, expr.token)
            cursor = self.emit(cursor, "return_reference", holder, expr.token,
                               roots=frozenset(self.return_roots))
        return cursor

    def keep_loan(self, cursor, loan, token):
        holder = Local(Token("<temporary reference>", token.line, token.column), False, loan)
        place = Place(holder)
        return self.emit(cursor, "declare", place, token), place

    def lower_call(self, expr, scope, cursor):
        token, args = expr.token, expr.args
        temporaries = []
        if expr.kind == "indirect_call":
            pointer, *args = args
            annotation = self.annotation(pointer)
            if isinstance(annotation, FunctionType) and annotation.closure:
                closure = self.program.functions[annotation.closure]
                cursor, lease = self.lower_closure(pointer, scope, cursor, consume=closure.consumes_captures)
                if not closure.consumes_captures and any(capture.mutable for capture in closure.captures):
                    receiver = ungroup(pointer)
                    if receiver.kind == "name":
                        cursor = self.emit(cursor, "write", Place(scope.find(receiver.token)), receiver.token)
                if lease:
                    cursor, holder = self.keep_loan(cursor, lease, pointer.token)
                    temporaries.append((holder, pointer.token))
            else:
                cursor = self.lower(pointer, scope, cursor)
        function = self.program.call_functions.get(id(expr))
        expected, _ = self.program.call_types[id(expr)]
        passed = {}
        for index, (expr, annotation) in enumerate(zip(args, expected)):
            if isinstance(annotation, ReferenceType) and self.annotation(expr) != NEVER:
                cursor, loan = self.lower_reference(
                    expr, scope, cursor, reborrow=annotation, two_phase=annotation.mutable
                )
                cursor, holder = self.keep_loan(cursor, loan, expr.token)
                if annotation.lifetime == "'static":
                    cursor = self.emit(cursor, "static_reference", holder, expr.token)
                temporaries.append((holder, expr.token))
                passed[index] = loan
            else:
                cursor = self.lower(expr, scope, cursor)
        # Activate implicit mutable reborrows after all arguments have been
        # evaluated. Keep each argument live through the activation checks.
        for holder, at in temporaries:
            if holder.local.loan.two_phase:
                cursor = self.emit(cursor, "activate", holder, at)
        for holder, at in temporaries:
            cursor = self.emit(cursor, "read", holder, at)
        if function and function.return_inputs and all(index in passed for index in function.return_inputs):
            loans = tuple(passed[index] for index in function.return_inputs)
            return cursor, self.merge_loans(loans, function.result.mutable, from_call=True)
        if function and isinstance(function.result, ReferenceType) and function.result.lifetime == "'static":
            place = Place(Local(token, function.result.mutable, static=True))
            cursor = self.emit(cursor, "declare", place, token)
            return cursor, Loan(place, function.result.mutable)
        return cursor, None

    def lower_print(self, token, args, scope, cursor):
        if not args:
            return cursor
        template, *values = args
        temporaries = []
        for value in values:
            if isinstance(self.annotation(value), ReferenceType):
                reference = ungroup(value)
                if reference.kind != "name":
                    cursor, loan = self.lower_reference(reference, scope, cursor)
                else:
                    holder = scope.find(reference.token)
                    cursor, loan = self.start_loan(cursor, self.referent(holder), False, reference.token)
            else:
                cursor, place = self.lower_address(value, scope, cursor)
                if place is None:
                    cursor = self.lower(value, scope, cursor, consume=False)
                    continue
                cursor, loan = self.start_loan(cursor, place, False, value.token)
            cursor, holder = self.keep_loan(cursor, loan, value.token)
            temporaries.append(holder)
        for mode, name in format_parts(template.args[1], template.token):
            if mode == "text" or not name:
                continue
            local = scope.find(replace(template.token, text=name))
            place = self.referent(local) if local.loan else Place(local)
            cursor, loan = self.start_loan(cursor, place, False, token)
            cursor, holder = self.keep_loan(cursor, loan, token)
            temporaries.append(holder)
        for holder in temporaries:
            cursor = self.emit(cursor, "read", holder, token)
        return cursor

    def move_action(self, place):
        if place.indexed:
            return "indexed_move"
        if place.holder:
            return "deref_move"
        if place.local.shared_capture:
            return "capture_move"
        if place.local in self.owned_captures:
            self.function.consumes_captures = True
        if place.local in self.guarded or any(place.overlaps(item) for item in self.guard_borrows):
            return "guard_move"
        return "move"

    def lower(self, expr, scope, cursor, *, consume=True):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "closure_value":
            cursor, _ = self.lower_closure(expr, scope, cursor)
            return cursor
        if kind == "try":
            return self.lower(args[0], scope, cursor)
        if kind == "enum_method":
            receiver, arguments, _, consuming = args
            cursor = self.lower(receiver, scope, cursor, consume=consuming)
            for argument in arguments:
                cursor = self.lower(argument, scope, cursor)
            return cursor
        if kind == "macro_definition":
            scope.remember(token)
            return cursor
        if kind in {"dyn_reference", "slice_reference"}:
            return self.lower(args[0], scope, cursor, consume=consume)
        cursor, place = self.lower_address(expr, scope, cursor)
        if place is not None:
            return self.access(cursor, place, self.annotation(expr), token, consume=consume)
        if kind in {"literal", "name", "default", "const_value", "function_pointer", "byte_string"}:
            return cursor
        if kind == "borrow":
            cursor, _ = self.lower_reference(expr, scope, cursor)
            return cursor
        if kind in {"group", "field", "unary", "deref", "scalar_cast"}:
            return self.lower(args[0], scope, cursor)
        if kind == "clone":
            return self.lower(args[0], scope, cursor, consume=False)
        if kind in {"length", "is_empty"}:
            return self.lower(args[0], scope, cursor, consume=False)
        if kind == "range_index":
            for item in args[:3]:
                if item is not None:
                    cursor = self.lower(item, scope, cursor)
            return cursor
        if kind in {"array", "tuple", "index", "array_repeat"}:
            for item in (args[:1] if kind == "array_repeat" else args):
                cursor = self.lower(item, scope, cursor)
            return cursor
        if kind == "block":
            statements, tail = args
            inner = Scope(scope)
            target = ControlTarget(expr, finish=FlowNode()) if expr.label else None
            if target:
                self.loops.append(target)
            try:
                for item in statements:
                    cursor = self.lower(item, inner, cursor)
                if tail:
                    cursor = self.lower(tail, inner, cursor)
            finally:
                if target:
                    self.loops.pop()
            if target:
                self.connect(cursor, target.finish)
                return target.finish
            return cursor
        if kind == "let":
            source, annotation, value, otherwise = args
            pattern, types = self.program.patterns[id(source)]
            if value is None:
                return self.declare(source, types, scope, cursor, initialized=False)
            value_type = self.annotation(value)
            if isinstance(value_type, FunctionType) and value_type.closure:
                cursor, lease = self.lower_closure(value, scope, cursor)
                return self.declare(source, types, scope, cursor, loan=lease)
            if isinstance(value_type, ReferenceType) and source.kind in {"bind", "wild"}:
                static = isinstance(annotation, ReferenceType) and annotation.lifetime == "'static"
                if not types and not static and ungroup(value).kind == "name":
                    return cursor
                coercion = annotation if shared_coercion(self.annotation(value), annotation) else None
                cursor, loan = self.lower_reference(value, scope, cursor, reborrow=coercion,
                                                   extend_to=scope)
                cursor = self.declare(source, types, scope, cursor, loan=loan)
                if loan and static:
                    cursor, holder = self.keep_loan(cursor, loan, source.token)
                    cursor = self.emit(cursor, "static_reference", holder, source.token)
                if loan and loan.expiring and types:
                    holder = Place(scope.find(source.token))
                    cursor = self.emit(cursor, "end_temporaries", holder, source.token)
                return cursor
            if self.annotation(value) == NEVER and isinstance(annotation, ReferenceType):
                cursor = self.lower(value, scope, cursor)
                # Keep an address for type-checked expressions in the dead tail.
                loan = Loan(Place(Local(source.token, annotation.mutable)), annotation.mutable)
                return self.declare(source, types, scope, cursor, loan=loan)
            cursor, origin, loan = self.lower_pattern_input(value, scope, cursor, extend_to=scope)
            reads, moves, tests = self.pattern_accesses(pattern, origin, value_type)
            cursor = self.read_pattern(cursor, tests | reads, source.token)
            if otherwise:
                self.lower(otherwise, Scope(scope), cursor)
            cursor = self.move_pattern(cursor, moves, source.token)
            cursor = self.declare(source, types, scope, cursor, loan=loan)
            if loan and isinstance(annotation, ReferenceType) and annotation.lifetime == "'static":
                cursor = self.emit(cursor, "static_reference", origin, source.token)
            if loan and loan.expiring:
                for name, (kind, _) in types.items():
                    if isinstance(kind, ReferenceType):
                        holder = Place(scope.find(source.token.binding(name)))
                        cursor = self.emit(cursor, "end_temporaries", holder, source.token)
            return cursor
        if kind == "match":
            end, _ = self.lower_match(expr, scope, cursor)
            return end
        if kind == "discard":
            return self.lower(args[0], scope, cursor)
        if kind == "if":
            condition, yes, no = args
            cursor = self.lower(condition, scope, cursor)
            yes_end = self.lower(yes, scope, cursor)
            no_end = self.lower(no, scope, cursor) if no else cursor
            return self.join(yes_end, no_end)
        if kind in {"while", "while_let", "loop", "for"}:
            return self.lower_loop(expr, scope, cursor)
        if kind == "return":
            if args[0]:
                self.lower_return(args[0], scope, cursor)
            return None
        if kind in {"break", "continue"}:
            target = find_control(self.loops, expr)
            if args[0]:
                cursor = self.lower(args[0], scope, cursor)
            self.connect(cursor, target.finish if kind == "break" else target.restart)
            return None
        if kind == "assign":
            left, right = args
            cursor = self.lower(right, scope, cursor)
            cursor, target_place = self.lower_address(left, scope, cursor)
            if target_place is None:
                return self.lower(left, scope, cursor)
            if token.text != "=":
                cursor = self.emit(cursor, "read", target_place, token)
            protected = self.guard_borrows + self.guard_tests
            forbidden = target_place.local in self.guarded or any(
                target_place.overlaps(item) for item in protected
            )
            action = "guard_write" if forbidden else "write"
            if target_place.indexed:
                cursor = self.emit(cursor, "read", target_place, token)
            return self.emit(cursor, action, target_place, token)
        if kind == "binary":
            if token.text in COMPARE and self.annotation(args[0]) not in CONST_TYPES:
                return self.lower_comparison(args, scope, cursor)
            left, right = args
            cursor = self.lower(left, scope, cursor)
            right_end = self.lower(right, scope, cursor)
            return self.join(cursor, right_end) if token.text in {"&&", "||"} else right_end
        if kind == "record":
            for _, value in args[0]:
                cursor = self.lower(value, scope, cursor)
            return cursor
        if kind in {"call", "indirect_call"}:
            cursor, _ = self.lower_call(expr, scope, cursor)
            return None if self.annotation(expr) == NEVER else cursor
        if kind in {"print", "panic"}:
            if kind == "panic":
                self.lower_print(token, args, scope, cursor)
                return None
            return self.lower_print(token, args, scope, cursor)
        raise AssertionError(kind)

    def lower_closure(self, expr, scope, cursor, *, consume=True):
        expr = ungroup(expr)
        if expr.kind == "name":
            return self.lower(expr, scope, cursor, consume=consume), scope.find(expr.token).loan
        if expr.kind == "clone":
            return self.lower_closure(expr.args[0], scope, cursor, consume=False)
        if expr.kind != "closure_value":
            raise expr.token.error(f"capturing closure from {expr.kind} is unsupported")
        function = self.program.functions[expr.args[0]]
        leases = []
        for capture in function.captures:
            local = scope.find(capture.name)
            place = Place(local)
            if capture.by_move:
                cursor = self.access(cursor, place, capture.annotation, capture.name, consume=True)
                if local.loan:
                    leases.append(local.loan)
            elif isinstance(capture.annotation, ReferenceType):
                cursor, lease = self.start_loan(cursor, self.referent(local), capture.mutable, capture.name)
                leases.append(lease)
            else:
                cursor, lease = self.start_loan(cursor, place, capture.mutable, capture.name)
                leases.append(lease)
        return cursor, self.merge_loans(tuple(leases), False)

    def lower_comparison(self, operands, scope, cursor):
        temporaries = []
        for value in operands:
            annotation = self.annotation(value)
            if isinstance(annotation, ReferenceType):
                cursor, loan = self.lower_reference(value, scope, cursor,
                                                    reborrow=ReferenceType(annotation.inner, False))
            else:
                cursor, place = self.lower_address(value, scope, cursor)
                if place is None:
                    cursor = self.lower(value, scope, cursor)
                    continue
                cursor, loan = self.start_loan(cursor, place, False, value.token)
            cursor, holder = self.keep_loan(cursor, loan, value.token)
            temporaries.append((holder, value.token))
        for holder, token in temporaries:
            cursor = self.emit(cursor, "read", holder, token)
        return cursor

    def lower_loop(self, expr, scope, cursor):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "for":
            name, mutable, start, end, _, body = args
            cursor = self.lower(end, scope, self.lower(start, scope, cursor))
        else:
            body = args[-1]
        restart = self.join(cursor)
        finish = FlowNode()
        target = ControlTarget(expr, restart=restart, finish=finish)
        self.loops.append(target)
        try:
            body_entry = restart
            if kind == "while":
                target.in_condition = True
                body_entry = self.lower(args[0], scope, restart)
                target.in_condition = False
                self.connect(body_entry, finish)
            elif kind == "while_let":
                condition, source, _ = args
                target.in_condition = True
                body_entry, origin, loan = self.lower_pattern_input(condition, scope, restart)
                target.in_condition = False
                pattern, types = self.program.patterns[id(source)]
                annotation = self.annotation(condition)
                reads, moves, tests = self.pattern_accesses(pattern, origin, annotation)
                body_entry = self.read_pattern(body_entry, tests | reads, source.token)
                if uncovered([(pattern,)], (annotation,), self.program) is not None:
                    self.connect(body_entry, finish)
                body_entry = self.move_pattern(body_entry, moves, source.token)
                scope = Scope(scope)
                body_entry = self.declare(source, types, scope, body_entry, loan=loan)
            if kind == "for":
                self.connect(restart, finish)
                local = Local(name, mutable)
                scope = Scope(scope, {name.key: local})
                body_entry = self.emit(restart, "declare", Place(local), token)
            self.connect(self.lower(body, scope, body_entry), restart)
        finally:
            self.loops.pop()
        return finish

    def lower_match(self, expr, scope, cursor, *, result_type=None, extend_to=None):
        scrutinee, arms = expr.args
        cursor, origin, scrutinee_loan = self.lower_pattern_input(scrutinee, scope, cursor)
        finishes = []
        results = []
        prepared = []
        protected = {}
        for source, guard, body in arms:
            pattern, types = self.program.patterns[id(source)]
            reads, moves, tests = self.pattern_accesses(pattern, origin, self.annotation(scrutinee))
            protected.update(tests)
            prepared.append((source, guard, body, types, reads, moves, tests))
        for source, guard, body, types, reads, moves, tests in prepared:
            tested = self.read_pattern(cursor, tests | reads, source.token)
            inner = Scope(scope)
            ready = self.declare(source, types, inner, tested, loan=scrutinee_loan)
            if guard:
                earlier = self.guarded, self.guard_borrows, self.guard_tests
                self.guarded = self.guarded | set(inner.bindings.values())
                self.guard_borrows = (*self.guard_borrows, *reads, *moves)
                self.guard_tests = (*self.guard_tests, *protected)
                try:
                    ready = self.lower(guard, inner, ready)
                finally:
                    self.guarded, self.guard_borrows, self.guard_tests = earlier
                cursor = self.join(tested, ready)
            else:
                cursor = tested
            ready = self.move_pattern(ready, moves, source.token)
            if result_type:
                end, loan = self.lower_reference(body, inner, ready, extend_to=extend_to)
                finishes.append(self.leave_reference_scope(end, loan, inner, body.token))
                if loan:
                    results.append(loan)
            else:
                finishes.append(self.lower(body, inner, ready))
        loan = self.merge_loans(tuple(results), result_type.mutable) if result_type else None
        return self.join(*finishes), loan

    def lower_pattern_input(self, expr, scope, cursor, *, extend_to=None):
        cursor, origin = self.lower_address(expr, scope, cursor)
        if isinstance(self.annotation(expr), ReferenceType):
            if origin is not None:
                return cursor, origin, origin.local.loan
            cursor, loan = self.lower_reference(expr, scope, cursor, extend_to=extend_to)
            if loan:
                cursor, origin = self.keep_loan(cursor, loan, expr.token)
            return cursor, origin, loan
        if origin is None:
            cursor = self.lower(expr, scope, cursor)
        return cursor, origin, None

    def declare(self, source, types, scope, cursor, *, initialized=True, loan=None):
        action = "declare" if initialized else "uninitialized"
        for name, (annotation, mutable) in types.items():
            carries_loan = isinstance(annotation, ReferenceType) or (isinstance(annotation, FunctionType) and annotation.closure)
            local = Local(source.token.binding(name), mutable, loan if carries_loan else None)
            scope.bindings[name] = local
            scope.owned.add(local)
            cursor = self.emit(cursor, action, Place(local), source.token)
        return cursor

    def pattern_accesses(self, pattern, origin, annotation):
        reads = {}
        moves = {}
        tests = {}
        if origin is not None:
            self.collect_pattern(pattern, origin, reads, moves, tests, annotation)
        return reads, moves, tests

    def collect_pattern(self, pattern, place, reads, moves, tests, annotation, *, opaque=False):
        if pattern.alternatives:
            for option in pattern.alternatives:
                self.collect_pattern(option, place, reads, moves, tests, annotation, opaque=opaque)
        elif pattern.binding:
            if pattern.by_reference or self.program.is_copy(annotation):
                reads[place] = None
            else:
                moves[place] = annotation
        elif isinstance(pattern.tag, Constructor):
            ctor = pattern.tag
            if ctor.style == "reference":
                self.collect_pattern(pattern.children[0], self.referent(place.local), reads, moves, tests, annotation.inner)
                return
            variant = "::" in ctor.path
            if variant and len(self.program.layout(ctor.annotation)[1]) > 1:
                tests[place] = None
            for child, (field, kind) in zip(pattern.children, ctor.fields):
                projected = place if opaque or variant else place.child(field)
                self.collect_pattern(child, projected, reads, moves, tests, kind, opaque=opaque or variant)
        elif isinstance(pattern.tag, SlicePattern):
            projected = replace(place, indexed=True)
            tests[projected] = None
            for child in pattern.children:
                self.collect_pattern(child, projected, reads, moves, tests, annotation.element, opaque=True)
        elif pattern.tag is not None:
            tests[place] = None

    def read_pattern(self, cursor, reads, token):
        for place in reads:
            cursor = self.emit(cursor, "read", place, token)
        return cursor

    def move_pattern(self, cursor, moves, token):
        # Check all paths before moving any of them. Two bindings can project
        # different payload fields from the same opaque enum storage.
        for place in moves:
            cursor = self.emit(cursor, "read", place, token)
        for place in moves:
            cursor = self.emit(cursor, self.move_action(place), place, token)
        return cursor


def checked(value, token, integer="i32", *, runtime=False):
    lower, upper = INTEGER_BOUNDS[integer]
    if not lower <= value < upper:
        message = f"{integer} arithmetic overflow"
        raise RustPanic(message) if runtime else token.error(message)
    return value


def truncate_integer(value, annotation):
    lower, upper = INTEGER_BOUNDS[annotation]
    return (value - lower) % (upper - lower) + lower


def scalar_cast_supported(actual, target):
    return actual == "u8" if target == "char" else actual in CONST_TYPES


def convert_scalar(value, target):
    number = int(ordinal(value))
    return Character(chr(number)) if target == "char" else truncate_integer(number, target)


def complement(value, annotation):
    return not value if annotation == "bool" else truncate_integer(~value, annotation)


def binary(word, left, right, token, integer="i32", *, runtime=False):
    if word in BITWISE and integer == "bool":
        return BITWISE[word](left, right)
    if word in SHIFTS:
        if not 0 <= right < INTEGER_WIDTHS[integer]:
            message = "shift amount is outside the integer width"
            raise RustPanic(message) if runtime else token.error(message)
        if not runtime:
            checked(left, token, integer)
        return truncate_integer(left << right, integer) if word == "<<" else left >> right
    if not runtime:
        checked(left, token, integer)
        checked(right, token, integer)
    if word in ARITHMETIC:
        return checked(ARITHMETIC[word](left, right), token, integer, runtime=runtime)
    if right == 0:
        raise RustPanic("division by zero") if runtime else token.error("division by zero")
    quotient = abs(left) // abs(right)
    if (left < 0) != (right < 0):
        quotient = -quotient
    checked(quotient, token, integer, runtime=runtime)
    return quotient if word == "/" else left - quotient * right


def display(value, debug):
    if isinstance(value, (Address, DynAddress, SliceAddress)):
        return display(value.read(), debug)
    if isinstance(value, ArrayValue):
        return "[" + ", ".join(display(item, True) for item in value.items) + "]"
    if isinstance(value, Value):
        ctor = value.constructor
        name = ctor.path.rsplit("::", 1)[-1].rsplit(MODULE_SEPARATOR, 1)[-1]
        if ctor.style == "unit" or not ctor.fields:
            return name
        fields = [display(item, True) for item in value.fields]
        if ctor.style == "tuple":
            return f"{name}({', '.join(fields)})" if name else tuple_text(fields)
        entries = ", ".join(f"{field}: {item}" for (field, _), item in zip(ctor.fields, fields))
        return f"{name} {{ {entries} }}"
    if value is None:
        return "()"
    if isinstance(value, bool):
        return "true" if value else "false"
    if debug and isinstance(value, str):
        quote = "'" if isinstance(value, Character) else '"'
        parts = [quote]
        for char in value:
            if char in {quote, '\\', '\n', '\r', '\t', '\0'}:
                escape = next(key for key, item in ESCAPES.items() if item == char)
                parts.append("\\" + escape)
            elif not char.isprintable():
                parts.append(f"\\u{{{ord(char):x}}}")
            else:
                parts.append(char)
        return "".join(parts) + quote
    return str(value)


class Return(Exception):
    pass


class Break(Exception):
    pass


class Continue(Exception):
    pass


def handles_jump(expr, signal):
    label = signal.args[0]
    return label is None or expr.label is label


class Interpreter:
    def __init__(self, program, stdout):
        self.program = program
        self.functions = program.functions
        self.stdout = stdout
        self.type_arguments = {}

    def match_bindings(self, source, value):
        pattern, types = self.program.patterns[id(source)]
        for bindings in pattern.bindings(value):
            yield {name: Binding(item, types[name][1]) for name, item in bindings.items()}

    def placeholder(self, annotation):
        # Wildcard patterns can inspect an aggregate's shape without reading
        # its fields. Ownership prevents an uninitialized field from escaping.
        declaration = self.program.layout(substitute(annotation, self.type_arguments))
        if declaration and declaration[1]:
            ctor = declaration[1][0]
            return Value(ctor, tuple(self.placeholder(kind) for _, kind in ctor.fields))
        return None

    def call(self, name, values, arguments=None, environment=None):
        function = self.functions[name]
        bindings = dict(environment or {}) | {
            param.key: Binding(value, mutable)
            for (param, mutable, _), value in zip(function.params, values)
            if param.text != "_"
        }
        previous = self.type_arguments
        self.type_arguments = {p: self.program.normalize(substitute(t, previous))
                               for p, t in (arguments or {}).items()}
        try:
            return self.eval(function.body, Scope(bindings=bindings))
        except Return as signal:
            return signal.args[0]
        finally:
            self.type_arguments = previous

    def default_value(self, annotation):
        if annotation in INTEGER_TYPES:
            return 0
        if annotation in PRIMITIVES:
            return {"bool": False, "char": Character("\0"), "&str": "", "()": None}[annotation]
        if isinstance(annotation, ArrayType):
            return ArrayValue(tuple(self.default_value(annotation.element)
                                    for _ in range(annotation.length.value)))
        constructors = self.program.layout(annotation)[1]
        name = annotation.name if isinstance(annotation, AppliedType) else annotation
        path = self.program.default_variants.get(name)
        ctor = next(ctor for ctor in constructors if not path or ctor.path == path)
        return Value(ctor, tuple(self.default_value(kind) for _, kind in ctor.fields))

    def compare(self, left, right):
        if isinstance(left, (Address, SliceAddress)):
            return self.compare(left.read(), right.read())
        if isinstance(left, ArrayValue):
            for a, b in zip(left.items, right.items):
                result = self.compare(a, b)
                if result:
                    return result
            return len(left.items) - len(right.items)
        if isinstance(left, FunctionValue):
            return int(left != right)
        if isinstance(left, Value):
            if left.constructor.path != right.constructor.path:
                variants = self.program.layout(left.constructor.annotation)[1]
                order = {ctor.path: index for index, ctor in enumerate(variants)}
                return order[left.constructor.path] - order[right.constructor.path]
            for a, b in zip(left.fields, right.fields):
                result = self.compare(a, b)
                if result:
                    return result
            return 0
        if left is None:
            return 0
        return (left > right) - (left < right)

    def runtime_constructor(self, expr, template):
        annotation = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
        layout = self.program.layout(annotation)
        if layout:
            return next(ctor for ctor in layout[1] if ctor.path == template.path)
        return template

    def address(self, expr, scope):
        expr = ungroup(expr)
        if expr.kind == "name":
            ctor = self.program.constructors.get(expr.token.text)
            if not ctor or ctor.style != "unit":
                return Address(scope.find(expr.token))
        if expr.kind == "deref":
            return self.eval(expr.args[0], scope)
        if expr.kind == "field":
            base, name = expr.args
            return self.address(base, scope).child(name)
        if expr.kind == "index":
            base, index = expr.args
            root = self.address(base, scope)
            offset = self.eval(index, scope)
            return root.child(replace(expr.token, text=str(offset)))
        if expr.kind == "range_index":
            base, first, last, inclusive = expr.args
            root = self.address(base, scope)
            length = root.length if isinstance(root, SliceAddress) else len(root.read().items)
            start = self.eval(first, scope) if first else 0
            stop = self.eval(last, scope) + int(inclusive) if last else length
            if not 0 <= start <= stop <= length:
                raise RustPanic(f"slice range {start}..{stop} is out of bounds for length {length}")
            if isinstance(root, SliceAddress):
                return SliceAddress(root.address, root.start + start, root.start + stop)
            return SliceAddress(root, start, stop)
        return Address(Binding(self.eval(expr, scope), True))

    def eval(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind in {"function_pointer", "closure_value"}:
            name, arguments = args
            environment = {}
            for capture in self.program.functions[name].captures:
                binding = scope.find(capture.name)
                environment[capture.name.key] = Binding(binding.value, capture.mutable) if capture.by_move else binding
            return FunctionValue(name, {parameter: self.program.normalize(substitute(value, self.type_arguments))
                                        for parameter, value in arguments.items()}, environment)
        if kind == "indirect_call":
            callee = self.eval(args[0], scope)
            values = tuple(self.eval(argument, scope) for argument in args[1:])
            return self.call(callee.name, values, callee.arguments, callee.environment)
        if kind == "try":
            value, enclosing = args
            value = self.eval(value, scope)
            variant = value.constructor.path.rsplit("::", 1)[-1]
            if variant in {"Some", "Ok"}:
                return value.fields[0]
            target = self.program.normalize(substitute(enclosing, self.type_arguments))
            ctor = next(item for item in self.program.layout(target)[1]
                        if item.path.endswith(f"::{variant}"))
            raise Return(Value(ctor, value.fields))
        if kind == "enum_method":
            receiver, arguments, wanted, consuming = args
            value = self.eval(receiver, scope)
            actual = value.constructor.path.rsplit("::", 1)[-1]
            supplied = [self.eval(item, scope) for item in arguments]
            if not consuming:
                return actual == wanted
            if actual == wanted:
                return value.fields[0]
            if token.text == "unwrap_or":
                return supplied[0]
            message = supplied[0] if token.text == "expect" else f"called {token.text} on {actual}"
            if actual == "Err":
                message += f": {display(value.fields[0], True)}"
            raise RustPanic(message)
        if kind == "macro_definition":
            scope.remember(token)
            return None
        if kind == "dyn_reference":
            value = self.eval(args[0], scope)
            if isinstance(value, DynAddress):
                return value
            concrete = self.program.normalize(substitute(args[1], self.type_arguments))
            return DynAddress(value, concrete)
        if kind == "slice_reference":
            return self.eval(args[0], scope)
        if kind == "literal":
            return args[1]
        if kind == "byte_string":
            return Address(Binding(ArrayValue(args), False))
        if kind == "const_value":
            return substitute(args[0], self.type_arguments).value
        if kind == "array":
            return ArrayValue(tuple(self.eval(item, scope) for item in args))
        if kind == "tuple":
            fields = tuple(self.eval(item, scope) for item in args)
            kind = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
            return Value(self.program.layout(kind)[1][0], fields)
        if kind == "array_repeat":
            value, length = args
            item = self.eval(value, scope)
            count = substitute(length, self.type_arguments).value
            if count > 1_000_000:
                raise token.error("array exceeds interpreter allocation limit")
            return ArrayValue((item,) * count)
        if kind in {"index", "range_index"}:
            return self.address(expr, scope).read()
        if kind in {"length", "is_empty"}:
            root = self.address(args[0], scope)
            length = root.length if isinstance(root, SliceAddress) else len(root.read().items)
            return length if kind == "length" else length == 0
        if kind == "default":
            annotation = self.program.expr_types[id(expr)]
            return self.default_value(self.program.normalize(substitute(annotation, self.type_arguments)))
        if kind == "group":
            return self.eval(args[0], scope)
        if kind == "borrow":
            return self.address(args[0], scope)
        if kind == "deref":
            return self.eval(args[0], scope).read()
        if kind == "clone":
            # Aggregate values are immutable records, so a clone can share storage.
            return self.eval(args[0], scope)
        if kind == "name":
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "unit":
                return Value(self.runtime_constructor(expr, ctor), ())
            return scope.find(token).value
        if kind == "field":
            value, name = args
            aggregate = self.eval(value, scope)
            return aggregate.fields[aggregate.constructor.index(name)]
        if kind == "record":
            ctor = self.runtime_constructor(expr, self.program.constructors[token.text])
            fields = [None] * len(ctor.fields)
            for name, value in args[0]:
                fields[ctor.index(name)] = self.eval(value, scope)
            return Value(ctor, tuple(fields))
        if kind == "match":
            scrutinee, arms = args
            value = self.eval(scrutinee, scope)
            for source, guard, body in arms:
                for bindings in self.match_bindings(source, value):
                    inner = Scope(scope, bindings)
                    if guard is None or self.eval(guard, inner):
                        return self.eval(body, inner)
            raise AssertionError("type checker accepted a non-exhaustive match")
        if kind == "block":
            statements, tail = args
            inner = Scope(scope)
            try:
                for statement in statements:
                    self.eval(statement, inner)
                return self.eval(tail, inner) if tail else None
            except Break as signal:
                if not expr.label or not handles_jump(expr, signal):
                    raise
                return signal.args[1]
        if kind == "let":
            source, annotation, value, otherwise = args
            if value is None:
                scope.bindings[source.token.key] = Binding(self.placeholder(annotation), source.args[0])
                return None
            result = self.eval(value, scope)
            bindings = next(self.match_bindings(source, result), None)
            if bindings is None:
                if otherwise:
                    self.eval(otherwise, scope)
                raise AssertionError("type checker accepted a fall-through let pattern")
            scope.bindings.update(bindings)
            return None
        if kind == "discard":
            self.eval(args[0], scope)
            return None
        if kind == "if":
            condition, yes, no = args
            branch = yes if self.eval(condition, scope) else no
            return self.eval(branch, scope) if branch else None
        if kind in {"while", "while_let", "loop"}:
            body = args[-1]
            while True:
                try:
                    inner = scope
                    if kind == "while" and not self.eval(args[0], scope):
                        return None
                    if kind == "while_let":
                        condition, source, _ = args
                        value = self.eval(condition, scope)
                        bindings = next(self.match_bindings(source, value), None)
                        if bindings is None:
                            return None
                        inner = Scope(scope, bindings)
                    self.eval(body, inner)
                except Continue as signal:
                    if not handles_jump(expr, signal):
                        raise
                except Break as signal:
                    if not handles_jump(expr, signal):
                        raise
                    return signal.args[1]
        if kind == "for":
            name, mutable, start, end, inclusive, body = args
            first = self.eval(start, scope)
            stop = self.eval(end, scope) + int(inclusive)
            for value in range(first, stop):
                inner = Scope(scope, {name.key: Binding(value, mutable)})
                try:
                    self.eval(body, inner)
                except Continue as signal:
                    if not handles_jump(expr, signal):
                        raise
                    continue
                except Break as signal:
                    if not handles_jump(expr, signal):
                        raise
                    break
            return None
        if kind == "return":
            raise Return(self.eval(args[0], scope) if args[0] else None)
        if kind == "break":
            raise Break(expr.label, self.eval(args[0], scope) if args[0] else None)
        if kind == "continue":
            raise Continue(expr.label)
        if kind == "unary":
            value = self.eval(args[0], scope)
            annotation = self.program.expr_types[id(expr)]
            if token.text == "-":
                return checked(-value, token, annotation, runtime=True)
            return complement(value, annotation)
        if kind == "scalar_cast":
            value, target = args
            return convert_scalar(self.eval(value, scope), target)
        if kind == "assign":
            left, right = args
            value = self.eval(right, scope)
            address = self.address(left, scope)
            if token.text != "=":
                value = binary(token.text[:-1], address.read(), value, token,
                               self.program.expr_types[id(left)], runtime=True)
            address.write(value)
            return None
        if kind == "binary":
            left, right = args
            value = self.eval(left, scope)
            if token.text == "&&":
                return value and self.eval(right, scope)
            if token.text == "||":
                return value or self.eval(right, scope)
            if token.text in COMPARE:
                other = self.eval(right, scope)
                return COMPARE[token.text](self.compare(value, other), 0)
            return binary(token.text, value, self.eval(right, scope), token,
                          self.program.expr_types[id(expr)], runtime=True)
        if kind == "call":
            values = tuple(self.eval(arg, scope) for arg in args)
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "tuple":
                return Value(self.runtime_constructor(expr, ctor), values)
            _, arguments = self.program.call_types[id(expr)]
            function = self.program.call_functions[id(expr)]
            dispatch = self.program.dispatches.get(id(expr))
            if dispatch:
                trait, receiver, name = dispatch
                receiver = self.program.normalize(substitute(receiver, self.type_arguments))
                if isinstance(receiver, DynType):
                    receiver = values[0].concrete
                    values = (values[0].address, *values[1:])
                impl, specialized = self.program.implementation(receiver, trait)
                if name in impl.methods:
                    prototype = function
                    function = impl.methods[name]
                    arguments = specialized | {actual: arguments[declared] for declared, actual in
                                               zip(prototype.method_parameters, function.method_parameters)}
                else:
                    arguments = arguments | {self.program.traits[trait].self_type: receiver}
            return self.call(function.name.text, values, arguments)
        if kind in {"print", "panic"}:
            if not args:
                if kind == "panic":
                    raise RustPanic(token.text)
                print(file=self.stdout)
                return None
            template, *values = args
            evaluated = iter([self.eval(value, scope) for value in values])
            parts = format_parts(template.args[1], template.token)
            rendered = []
            for mode, value in parts:
                if mode == "text":
                    rendered.append(value)
                    continue
                if value:
                    item = scope.find(replace(template.token, text=value)).value
                else:
                    item = next(evaluated)
                rendered.append(display(item, mode == "debug"))
            rendered = "".join(rendered)
            if kind == "panic":
                raise RustPanic(rendered)
            print(rendered, file=self.stdout)
            return None
        raise AssertionError(kind)


DEMO = """\
fn factorial(n: i32) -> i32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn main() {
    for n in 1..=6 {
        println!("{}! = {}", n, factorial(n));
    }
}
"""


def render_source(trees):
    return " ".join(token.text for token in flatten_trees(trees))


def rustc_source(source, libraries, source_path=None):
    # rustc's expanded text lowers standard derives and println into internal
    # library calls. Preserve those two interpreter entry points through it.
    prefix = "__rust_in_python_host"
    while prefix in source:
        prefix += "_"
    marker = prefix + ":derive:"
    shim = prefix + "_print"
    trees, _ = token_trees(lex(source))

    def keep_derives(items):
        result = []
        index = 0
        while index < len(items):
            item = items[index]
            attribute = items[index + 1] if index + 1 < len(items) else None
            if (tree_text(item) == "#" and isinstance(attribute, TokenGroup)
                    and len(attribute.items) == 2 and tree_text(attribute.items[0]) == "derive"
                    and isinstance(attribute.items[1], TokenGroup)):
                derives = attribute.items[1]
                pieces = []
                current = []
                for part in derives.items:
                    if tree_text(part) == ",":
                        if current:
                            pieces.append(tuple(current))
                        current = []
                    else:
                        current.append(part)
                if current:
                    pieces.append(tuple(current))
                builtin = [part[0].text for part in pieces if len(part) == 1 and tree_text(part[0]) in DERIVABLE]
                if builtin:
                    replacements, _ = token_trees(lex("#[doc=" + json.dumps(marker + ",".join(builtin)) + "]"))
                    result.extend(replacements)
                other = [part for part in pieces if not (len(part) == 1 and tree_text(part[0]) in DERIVABLE)]
                if other:
                    flattened = []
                    for part in other:
                        if flattened:
                            flattened.append(replace(item, text=","))
                        flattened.extend(part)
                    result.extend((item, replace(attribute, items=(attribute.items[0],
                                                               replace(derives, items=tuple(flattened))))))
                index += 2
                continue
            if isinstance(item, TokenGroup):
                item = replace(item, items=keep_derives(item.items))
            result.append(item)
            index += 1
        return tuple(result)

    source = render_source(keep_derives(trees))
    macro = ("fn " + shim + "() {}\n"
             "macro_rules! println { () => { " + shim + "() }; "
             "($fmt:expr $(, $arg:expr)* $(,)?) => { " + shim + "($fmt $(,$arg)*) }; }\n")
    # Inner attributes belong first in a crate.
    trees, _ = token_trees(lex(source))
    start = 0
    while (start + 2 < len(trees) and tree_text(trees[start]) == "#" and tree_text(trees[start + 1]) == "!"
           and isinstance(trees[start + 2], TokenGroup)):
        start += 3
    source = render_source(trees[:start]) + "\n" + macro + render_source(trees[start:])
    with TemporaryDirectory(prefix="rust-in-python-") as folder:
        directory = Path(source_path).resolve().parent if source_path is not None else folder
        args = ["rustc", "--edition=2021", "--crate-name", "rust_in_python_input",
                "--crate-type", "lib", "-Zunpretty=expanded", "-"]
        for name, library in libraries.items():
            args.extend(("--extern", f"{name}={library}"))
        try:
            compiler = subprocess.run(args, cwd=directory, input=source, text=True, capture_output=True, timeout=30,
                                      env={**os.environ, "RUSTC_BOOTSTRAP": "1"}, check=False)
        except subprocess.TimeoutExpired as error:
            raise RustError("procedural macro expansion exceeded 30 seconds") from error
        if compiler.returncode:
            raise RustError("rustc macro expansion failed\n" + compiler.stderr.strip())
    output, _ = token_trees(lex(compiler.stdout))
    return render_source(strip_host_items(output, marker, shim, libraries))


def strip_host_items(items, marker, shim, libraries, *, top=True):
    result = []
    index = 0
    while index < len(items):
        current = items[index]
        words = tuple(tree_text(t) for t in items[index:index + 4])
        if top and words[:2] == ("#", "!") and index + 2 < len(items):
            index += 3
            continue
        if words and words[0] == "#" and index + 1 < len(items) and isinstance(items[index + 1], TokenGroup):
            attribute = items[index + 1].items
            name = tree_text(attribute[0]) if attribute else ""
            if name == "doc" and len(attribute) == 3 and isinstance(attribute[-1], Token):
                content = decode_string(attribute[-1])
                if content.startswith(marker):
                    derives = content.removeprefix(marker)
                    restored, _ = token_trees(lex(f"#[derive({derives})]"))
                    result.extend(restored)
                index += 2
                continue
            if name in {"prelude_import", "allow", "warn", "deny", "forbid", "automatically_derived"}:
                index += 2
                continue
        if top and (words[:3] == ("extern", "crate", "std")
                    or (len(words) > 1 and words[0] == "use"
                        and (words[1] in libraries or words[1:4] == ("std", "::", "prelude")))):
            while index < len(items) and tree_text(items[index]) != ";":
                index += 1
            index += 1
            continue
        if top and (words[:3] == ("macro_rules", "!", "println") or words[:2] == ("fn", shim)):
            index += 4
            continue
        if isinstance(current, Token) and current.text == shim:
            result.extend((replace(current, text="println"), replace(current, text="!")))
        elif isinstance(current, TokenGroup):
            result.append(replace(current, items=strip_host_items(current.items, marker, shim, libraries, top=False)))
        else:
            result.append(current)
        index += 1
    return tuple(result)


def proc_libraries(values):
    libraries = {}
    for item in values:
        name, separator, file = item.partition("=")
        if not separator or not IDENT.fullmatch(name) or not file:
            raise RustError("expected a procedural macro library as NAME=PATH")
        path = Path(file).resolve()
        if not path.is_file():
            raise RustError(f"procedural macro library does not exist: {path}")
        if name in libraries:
            raise RustError(f"duplicate procedural macro crate {name!r}")
        libraries[name] = path
    return libraries


def compile_program(source, *, proc_extern=(), source_path=None):
    if proc_extern:
        source = rustc_source(source, proc_libraries(proc_extern), source_path)
    trees, end = token_trees(lex(source))
    modules = Modules(trees, source_path)
    modules.expand()
    program = Program(type_aliases=modules.type_aliases)
    program.add_prelude()
    for unit in modules.units.values():
        Parser("", tokens=(*flatten_trees(unit.items), end), modules=modules, unit=unit).parse(program)
        symbols = unit.glob_aliases | unit.aliases | unit.symbols
        program.traits_in_scope[unit.path] = unit.hidden_traits | {
            symbol.name for symbol in symbols.values() if symbol and symbol.kind == "trait"
        }
        program.values_in_scope[unit.path] = {name: symbol.name for name, symbol in symbols.items()
                                              if symbol and symbol.kind == "fn"}
    Checker(program).check()
    Ownership(program).check()
    return program


def run(source, stdout=None, *, proc_extern=(), source_path=None):
    program = compile_program(source, proc_extern=proc_extern, source_path=source_path)
    Interpreter(program, sys.stdout if stdout is None else stdout).call("main", ())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", help="Rust source file, or - for stdin")
    parser.add_argument("--demo", action="store_true", help="run a built-in Rust program")
    parser.add_argument("--check", action="store_true", help="check the source without running it")
    parser.add_argument("--proc-extern", action="append", default=[], metavar="NAME=PATH",
                        help="load a compiled Rust procedural macro crate through rustc")
    args = parser.parse_args()
    if args.demo == bool(args.source):
        parser.error("provide either a source file or --demo")
    label = "<demo>" if args.demo else "<stdin>" if args.source == "-" else args.source
    try:
        if args.demo:
            source = DEMO
        elif args.source == "-":
            source = sys.stdin.read()
        else:
            source = Path(args.source).read_text(encoding="utf-8")
        source_path = None if args.demo or args.source == "-" else args.source
        operation = compile_program if args.check else run
        operation(source, proc_extern=args.proc_extern, source_path=source_path)
    except (RustError, OSError, UnicodeError) as error:
        print(f"{label}: {str(error).replace(MODULE_SEPARATOR, '::')}", file=sys.stderr)
        return 1
    except RustPanic as error:
        print(f"{label}: {error}", file=sys.stderr)
        return 101
    except RecursionError:
        print(f"{label}: Python recursion limit reached", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
