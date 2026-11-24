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
from dataclasses import dataclass, field
from dataclasses import replace as dataclass_replace
from decimal import Decimal
import functools
import itertools
import json
import math
import operator
import os
from pathlib import Path
import re
import string
import struct
import subprocess
import sys
import unicodedata
from tempfile import TemporaryDirectory


RUST_EDITIONS = ("2015", "2018", "2021", "2024")


class RustError(Exception):
    pass


class RustPanic(Exception):
    pass


class ProcessExit(Exception):
    def __init__(self, status):
        self.status = status


@dataclass(frozen=True)
class Token:
    text: str
    line: int
    column: int
    context: tuple = ()
    definition: int | None = None
    module: tuple = ()
    child_module: tuple | None = None
    end: tuple[int, int] | None = field(default=None, compare=False, repr=False)
    origin: object | None = field(default=None, compare=False, repr=False)
    source_file: str | None = field(default=None, compare=False, repr=False)
    expansion_site: Token | None = field(default=None, compare=False, repr=False)

    @functools.cached_property
    def key(self):
        name = identifier_text(self.text)
        return (name, self.context) if self.context else name

    def binding(self, key):
        if isinstance(key, tuple):
            name, context = key
            return replace(self, text=name, context=context)
        return replace(self, text=key, context=())

    def error(self, message):
        return RustError(f"{self.line}:{self.column}: {message}")

    def followed_by(self, token):
        return (self.origin is not None and self.origin is token.origin
                and self.end == (token.line, token.column))


INTEGER_WIDTHS = {f"{sign}{width}": width for sign in ("i", "u") for width in (8, 16, 32, 64, 128)}
INTEGER_WIDTHS.update(isize=64, usize=64)
INTEGER_TYPES = set(INTEGER_WIDTHS)
INTEGER_SUFFIX = re.compile(f"(?:{'|'.join(INTEGER_WIDTHS)})$")
INTEGER_BOUNDS = {kind: (-(1 << (width - 1)), 1 << (width - 1)) if kind.startswith("i") else (0, 1 << width)
                  for kind, width in INTEGER_WIDTHS.items()}
CONST_TYPES = INTEGER_TYPES | {"bool", "char"}
FLOAT_TYPES = {"f32", "f64"}
NUMBER_TYPES = INTEGER_TYPES | FLOAT_TYPES
CONST_VALUE_TYPES = CONST_TYPES | FLOAT_TYPES
FLOAT_OPERATORS = {"+", "-", "*", "/", "%"}
FLOAT_PREDICATES = {"is_nan": math.isnan, "is_finite": math.isfinite, "is_infinite": math.isinf,
                    "is_sign_negative": lambda value: math.copysign(1.0, value) < 0}
CHAR_LIMIT = 0x110000
SURROGATES = range(0xD800, 0xE000)
SCALAR_BOUNDS = {**INTEGER_BOUNDS, "char": (0, CHAR_LIMIT)}


FAST_REPLACE = set()


def replace(instance, **changes):
    """dataclasses.replace, with a fast path for the plain frozen dataclasses the interpreter copies often."""
    if type(instance) in FAST_REPLACE:
        copy = object.__new__(type(instance))
        copy.__dict__.update(instance.__dict__)
        copy.__dict__.update(changes)
        copy.__dict__.pop("key", None)
        return copy
    return dataclass_replace(instance, **changes)


FAST_REPLACE.add(Token)


class Character(str):
    """A Unicode scalar value, with its own Rust debug representation."""


class Float32(float):
    """A binary32 value stored exactly in a Python float."""


DECIMAL_DIGITS = r"[0-9][0-9_]*"
DECIMAL_EXPONENT = rf"[eE][+-]?{DECIMAL_DIGITS}"
FLOAT_TOKEN = (rf"(?:{DECIMAL_DIGITS}\.{DECIMAL_DIGITS}(?:{DECIMAL_EXPONENT})?(?:f32|f64)?"
               rf"|{DECIMAL_DIGITS}{DECIMAL_EXPONENT}(?:f32|f64)?|{DECIMAL_DIGITS}(?:f32|f64)"
               rf"|{DECIMAL_DIGITS}\.(?![\w.]))")
FLOAT_LITERAL = re.compile(FLOAT_TOKEN)
FLOAT_SUFFIX = re.compile(r"f(?:32|64)$")


TOKEN = re.compile(
    r'\s+|//[^\n]*|/\*|b?r(?P<hashes>#{0,255})"[\s\S]*?"(?P=hashes)|'
    r'b?"(?:(?:\\[\s\S])|[^"\\])*"|'
    r"b?'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[^\r\n])|[^'\\\r\n\t])'|"
    r"(?P<lifetime>'[A-Za-z_][A-Za-z_0-9]*)|"
    rf'{FLOAT_TOKEN}|'
    rf'(?:0x[0-9a-fA-F_]+|0o[0-7_]+|0b[01_]+|[0-9][0-9_]*)(?:{"|".join(INTEGER_WIDTHS)})?|'
    r'(?P<identifier>[A-Za-z_][A-Za-z_0-9]*)|'
    r'\.\.=|\.\.|::|->|=>|==|!=|<=|>=|&&|\|\||[+*/%&|^\-]=|[{}()[\]#$@?.:,;=!+*/%&<>|^\-]'
)
COMMENT_EDGE = re.compile(r'/\*|\*/')
RESERVED = set("""
    as async await break const continue crate dyn else enum extern false fn for
    gen if impl in let loop match mod move mut pub ref return self Self static
    struct super trait true type unsafe use where while abstract become box do
    final macro override priv try typeof unsized virtual yield _
""".split())
RESERVED_BY_EDITION = {
    "2015": RESERVED - {"async", "await", "dyn", "gen", "try"},
    "2018": RESERVED - {"gen"},
    "2021": RESERVED - {"gen"},
    "2024": RESERVED,
}
PRECEDENCE = {
    "=": 1, "+=": 1, "-=": 1, "*=": 1, "/=": 1, "%=": 1, "&=": 1, "|=": 1, "^=": 1, "<<=": 1, ">>=": 1,
    "||": 2, "&&": 3, "==": 4, "!=": 4, "<": 4, "<=": 4, ">": 4, ">=": 4,
    "|": 5, "^": 6, "&": 7, "<<": 8, ">>": 8, "+": 9, "-": 9, "*": 10, "/": 10, "%": 10,
}
BITWISE = {"&": operator.and_, "|": operator.or_, "^": operator.xor}
SHIFTS = {"<<", ">>"}
BLOCK_EXPRESSIONS = {"block", "inline_const", "if", "if_chain", "while", "while_let", "while_chain", "loop",
                     "for", "match"}
PRIMITIVES = CONST_VALUE_TYPES | {"()"}
NEVER = "!"
SCALAR_CONSTANTS = {f"{kind}::{name}": (result, value)
                    for kind, (lo, hi) in INTEGER_BOUNDS.items()
                    for name, result, value in (("MIN", kind, lo), ("MAX", kind, hi - 1),
                                                ("BITS", "u32", INTEGER_WIDTHS[kind]))}
SCALAR_CONSTANTS.update({"char::MIN": ("char", Character("\0")),
                         "char::MAX": ("char", Character(chr(CHAR_LIMIT - 1)))})
for _kind, _precision, _max_exponent in (("f32", 24, 128), ("f64", 53, 1024)):
    _maximum = math.ldexp(2 - 2.0 ** (1 - _precision), _max_exponent - 1)
    SCALAR_CONSTANTS.update({f"{_kind}::{name}": (_kind, value) for name, value in (
        ("MIN", -_maximum), ("MAX", _maximum), ("MIN_POSITIVE", math.ldexp(1.0, 2 - _max_exponent)),
        ("EPSILON", 2.0 ** (1 - _precision)), ("NAN", math.nan),
        ("INFINITY", math.inf), ("NEG_INFINITY", -math.inf),
    )})


def is_identifier(text):
    return identifier_text(text).isidentifier()


def identifier_text(text):
    if text.startswith("r#") and text[2:].isidentifier():
        return text[2:]
    return text


def is_string_literal(text):
    return text.startswith(('"', 'r"')) or text.startswith("r#") and '"' in text


def is_lifetime(text):
    return text.startswith("'") and is_identifier(text[1:])


HOST_CONTEXT = re.compile(r"\s*/\*\s*\d+#(\d+)\s*\*/")


def lex(source, *, edition="2021", source_file=None, host_contexts=None):
    offset, line, column = 0, 1, 1
    origin = object()
    while offset < len(source):
        match = TOKEN.match(source, offset)
        if match is None:
            start = offset + int(source[offset] == "'")
            if start == len(source) or not is_identifier(source[start]):
                raise Token(source[offset], line, column).error(f"unexpected character {source[offset]!r}")
            end = start + 1
        else:
            end = match.end()
        identifier = match is None or match.lastgroup in {"identifier", "lifetime"}
        name_start = offset + int(source[offset] == "'")
        raw = (identifier and (name_start == offset or edition >= "2021")
               and source.startswith("r#", name_start) and name_start + 2 < len(source)
               and is_identifier(source[name_start + 2]))
        if raw:
            end = name_start + 3
        if identifier:
            while end < len(source) and is_identifier("_" + source[end]):
                end += 1
        word = source[offset:end]
        if identifier:
            word = unicodedata.normalize("NFC", word)
            if raw and identifier_text(word[name_start - offset:]) in {"_", "crate", "self", "Self", "super"}:
                raise Token(word, line, column).error(f"{word!r} is reserved")
        if word == "/*":
            depth = 1
            for edge in COMMENT_EDGE.finditer(source, end):
                depth += 1 if edge.group() == "/*" else -1
                if depth == 0:
                    end = edge.end()
                    break
            if depth:
                raise Token(word, line, column).error("unclosed block comment")
        consumed = source[offset:end]
        lines = consumed.count("\n")
        next_column = len(consumed.rsplit("\n", 1)[1]) + 1 if lines else column + len(consumed)
        if word != "/*" and not (word.isspace() or word.startswith("//")):
            context = ()
            if host_contexts is not None and (mark := HOST_CONTEXT.match(source, end)):
                context = host_contexts.get(int(mark[1]), ())
            yield Token(word, line, column, context=context, end=(line + lines, next_column), origin=origin,
                        source_file=source_file)
        line += lines
        column = next_column
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


def flatten_trees(trees, *, group_fragments=True):
    for tree in trees:
        if isinstance(tree, TokenGroup):
            yield tree.opening
            yield from flatten_trees(tree.items, group_fragments=group_fragments)
            yield tree.closing
        elif isinstance(tree, OpaqueFragment):
            grouped = group_fragments and tree.fragment in {"expr", "expr_2021", "pat", "pat_param"}
            if grouped:
                yield replace(tree.token, text="(")
            yield from flatten_trees(tree.items, group_fragments=group_fragments)
            if grouped:
                yield replace(tree.token, text=")")
        else:
            yield tree


def tree_text(tree):
    if isinstance(tree, TokenGroup):
        return tree.opening.text
    return "@opaque" if isinstance(tree, OpaqueFragment) else tree.text


def expose_fragments(trees, fragments):
    for tree in trees:
        if isinstance(tree, OpaqueFragment) and tree.fragment in fragments:
            yield from expose_fragments(tree.items, fragments)
        else:
            yield tree


@dataclass(frozen=True)
class MacroCapture:
    name: Token
    fragment: str = ""


@dataclass(frozen=True)
class MacroRepeat:
    body: tuple
    separator: Token | None
    count: str


MACRO_FOLLOW = {
    "expr": {"=>", ",", ";"},
    "expr_2021": {"=>", ",", ";"},
    "stmt": {"=>", ",", ";"},
    "pat": {"=>", ",", "=", "if", "in"},
    "pat_param": {"=>", ",", "=", "|", "if", "in"},
    "path": {"=>", ",", "=", "|", ";", ":", ">", ">>", "[", "{", "as", "where", "$block"},
    "ty": {"=>", ",", "=", "|", ";", ":", ">", ">>", "[", "{", "as", "where", "$block"},
}


def matcher_starts(nodes, following):
    """Return every token or fragment that can begin this matcher suffix."""
    choices = set()
    for node in nodes:
        if isinstance(node, MacroRepeat):
            choices.update(matcher_starts(node.body, ()))
            if node.count == "+":
                return choices
        elif isinstance(node, MacroCapture):
            choices.add("$" + node.fragment)
            if node.fragment != "vis":
                return choices
        else:
            choices.add(tree_text(node))
            return choices
    return choices | set(following)


def validate_matcher(nodes, token, edition, *, following=(), bound=None):
    if bound is None:
        bound = set()
    for index, node in enumerate(nodes):
        if isinstance(node, TokenGroup):
            validate_matcher(node.items, token, edition, bound=bound)
            continue
        if not isinstance(node, (MacroCapture, MacroRepeat)):
            continue
        successors = matcher_starts(nodes[index + 1:], following)
        if isinstance(node, MacroRepeat):
            if node.separator is None and None in matcher_starts(node.body, (None,)):
                raise token.error("macro repetition can match an empty token sequence")
            if node.count != "?":
                successors.update({tree_text(node.separator)} if node.separator
                                  else matcher_starts(node.body, ()))
            validate_matcher(node.body, token, edition, following=successors, bound=bound)
            continue
        if node.name.text in bound:
            raise node.name.error(f"duplicate macro matcher binding {node.name.text!r}")
        bound.add(node.name.text)
        permitted = MACRO_FOLLOW.get(node.fragment)
        if node.fragment == "pat" and edition < "2021":
            permitted = MACRO_FOLLOW["pat_param"]
        for successor in sorted(successors):
            if node.fragment == "vis":
                allowed = (successor in {",", "$ident", "$ty", "$path", "&", "*", "!", "[", "(", "<"}
                           or (successor != "priv" and is_identifier(successor)))
            else:
                allowed = permitted is None or successor in permitted
            if not allowed:
                raise node.name.error(f"{successor!r} cannot follow macro fragment {node.fragment!r}")


def macro_sequence(items, *, matcher):
    result = []
    index = 0
    while index < len(items):
        tree = items[index]
        index += 1
        if isinstance(tree, TokenGroup):
            result.append(replace(tree, items=macro_sequence(tree.items, matcher=matcher)))
            continue
        if not isinstance(tree, Token) or tree.text != "$":
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
        if not isinstance(next_item, Token) or not is_identifier(next_item.text):
            raise tree.error("invalid macro metavariable")
        fragment = ""
        if matcher:
            if index + 1 >= len(items) or tree_text(items[index]) != ":":
                raise next_item.error("macro matcher requires a fragment specifier")
            fragment = tree_text(items[index + 1])
            index += 2
            if fragment not in {"block", "expr", "expr_2021", "ident", "item", "lifetime", "literal",
                                "meta", "pat", "pat_param", "path", "stmt", "tt", "ty", "vis"}:
                raise next_item.error(f"unsupported macro fragment {fragment!r}")
        result.append(MacroCapture(replace(next_item, text=identifier_text(next_item.text)), fragment))
    return tuple(result)


class MacroExpander:
    def __init__(self, edition, crate_name):
        self.edition = edition
        self.crate_name = crate_name
        self.module_path = ()
        self.steps = 0
        self.match_steps = 0
        self.definitions = 0
        self.qualified = {}
        self.module_macros = {}
        self.local_inner_macros = set()
        self.import_module = None

    def expand(self, items, inherited=None, depth=0, *, item_scope=False, definitions=None):
        if depth > 64:
            raise next(flatten_trees(items), Token("macro", 1, 1)).error("macro expansion recursion limit reached")
        macros = dict(inherited or {})
        if definitions is None:
            definitions = {}
        result = []
        index = 0
        while index < len(items):
            item = items[index]
            if isinstance(item, Token) and item.child_module is not None:
                self.module_macros[item.child_module] = dict(macros)
                if self.import_module:
                    inherited_macros = self.import_module(item.child_module)
                    macros.update(inherited_macros)
                    definitions.update(inherited_macros)
                index += 1
                continue
            if tree_text(item) == "macro_rules" and index + 3 < len(items):
                marker, name, body = items[index + 1:index + 4]
                if tree_text(marker) != "!" or not isinstance(name, Token) or not isinstance(body, TokenGroup):
                    raise item.error("invalid macro_rules definition")
                self.definitions += 1
                key = identifier_text(name.text)
                definition = self.definitions, self.rules(body), id(body) in self.local_inner_macros
                macros[key] = definitions[key] = definition
                result.append(replace(name, text="@macro_definition", definition=self.definitions))
                index += 4
                if index < len(items) and tree_text(items[index]) == ";":
                    index += 1
                continue
            macro = macros.get(identifier_text(item.text)) if isinstance(item, Token) else None
            builtin = identifier_text(item.text) if isinstance(item, Token) else None
            namespace = None
            argument = index + 2
            if isinstance(item, Token) and index + 4 < len(items) and tree_text(items[index + 1]) == "::":
                path = [identifier_text(item.text)]
                position = index + 1
                while position + 1 < len(items) and tree_text(items[position]) == "::":
                    path.append(identifier_text(tree_text(items[position + 1])))
                    position += 2
                macro = self.qualified.get(tuple(path))
                namespace = path[0]
                builtin = path[1] if len(path) == 2 and namespace in {"std", "core", "alloc"} else None
                argument = position + 1
            if (macro and argument < len(items) and tree_text(items[argument - 1]) == "!"
                    and isinstance(items[argument], TokenGroup)):
                self.steps += 1
                if self.steps > 10_000:
                    raise item.error("macro expansion limit reached")
                definition, rules, local_inner = macro
                mark = definition, self.steps
                group = items[argument]
                site = item.expansion_site or group.opening.expansion_site or item
                expanded = self.invoke(item, rules, group.items, mark, site)
                if local_inner:
                    expanded = self.qualify_inner(expanded, mark)
                index = argument + 1
                added = {}
                result.extend(self.expand(expanded, macros, depth + 1, item_scope=item_scope, definitions=added))
                macros.update(added)
                definitions.update(added)
                expanded_items = expose_fragments(expanded, {"item", "vis"})
                first = next(expanded_items, None)
                item_start = first and tree_text(first) in {
                    "#", "pub", "fn", "struct", "enum", "trait", "impl", "type", "const", "static", "mod", "use",
                }
                if item_start and tree_text(first) == "const":
                    second = next(expanded_items, None)
                    item_start = second is not None and tree_text(second) != "{"
                if (item_start or (item_scope and first is None)) and index < len(items) and tree_text(items[index]) == ";":
                    index += 1
                continue
            if (builtin and argument < len(items) and tree_text(items[argument - 1]) == "!"
                    and isinstance(items[argument], TokenGroup)):
                group = items[argument]
                if builtin in {"vec", "@host:vec"}:
                    result.append(replace(item, text="@vec"))
                    result.append(replace(group, items=self.expand(group.items, macros, depth)))
                    index = argument + 1
                    continue
                if builtin in {"concat", "stringify"}:
                    if builtin == "concat":
                        parts = self.expand(group.items, macros, depth + 1)
                        value = self.concatenate(parts, item)
                    else:
                        value = self.stringify(group.items)
                    result.append(replace(item, text=display(value, True), end=group.closing.end))
                    index = argument + 1
                    continue
                if namespace != "alloc" and builtin in {"file", "line", "column", "module_path",
                                                        "include_str", "include_bytes"}:
                    site = item.expansion_site or group.opening.expansion_site or item
                    if builtin in {"include_str", "include_bytes"}:
                        parts = self.expand(group.items, macros, depth + 1)
                        value = self.include_literal(builtin, parts, site)
                    else:
                        if group.items:
                            raise item.error(f"{builtin}! takes no arguments")
                        value = self.location_literal(builtin, site)
                    result.append(replace(item, text=value, end=group.closing.end))
                    index = argument + 1
                    continue
            if isinstance(item, (TokenGroup, OpaqueFragment)):
                item = replace(item, items=self.expand(item.items, macros, depth))
            result.append(item)
            index += 1
        return tuple(result)

    def location_literal(self, name, site):
        if name == "line":
            return f"{site.line}u32"
        if name == "column":
            return f"{site.column}u32"
        if name == "file":
            return display(site.source_file or "<anon>", True)
        return display("::".join((self.crate_name, *self.module_path)), True)

    @staticmethod
    def include_literal(name, items, site):
        parts = list(flatten_trees(items, group_fragments=False))
        if parts and parts[-1].text == ",":
            parts.pop()
        if len(parts) != 1 or not is_string_literal(parts[0].text):
            raise site.error(f"{name}! requires one string literal path")
        path = Path(decode_string(parts[0]))
        if not path.is_absolute() and site.source_file is not None:
            path = Path(site.source_file).parent / path
        try:
            data = path.read_bytes()
            if name == "include_str":
                return display(data.decode("utf-8"), True)
        except (OSError, UnicodeError, ValueError) as error:
            raise site.error(f"{name}! could not read {str(path)!r}: {error}") from error
        return 'b"' + "".join(f"\\x{byte:02x}" for byte in data) + '"'

    @staticmethod
    def qualify_inner(items, mark):
        result = []
        for index, item in enumerate(items):
            if (isinstance(item, Token) and is_identifier(item.text) and item.context[-1:] == (mark,)
                    and (index == 0 or tree_text(items[index - 1]) != "::")
                    and index + 2 < len(items) and tree_text(items[index + 1]) == "!"
                    and isinstance(items[index + 2], TokenGroup)):
                result.extend((replace(item, text="crate"), replace(item, text="::")))
            elif isinstance(item, (TokenGroup, OpaqueFragment)):
                item = replace(item, items=MacroExpander.qualify_inner(item.items, mark))
            result.append(item)
        return tuple(result)

    @staticmethod
    def stringify(items):
        text = []
        previous = None
        for token in flatten_trees(items, group_fragments=False):
            adjacent = previous is not None and previous.followed_by(token) and previous.context == token.context
            if previous is not None and not adjacent:
                text.append(" ")
            text.append(token.text)
            previous = token
        return "".join(text)

    @staticmethod
    def concatenate(items, token):
        values = []
        tokens = iter(flatten_trees(items, group_fragments=False))
        for literal_token in tokens:
            negative = literal_token.text == "-"
            if negative:
                literal_token = next(tokens, token)
            if literal_token.text.startswith(("b'", 'b"', 'br"', 'br#')):
                raise literal_token.error("concat! cannot concatenate a byte literal")
            literal = Parser.literal(literal_token, negative=negative)
            if literal is None:
                raise literal_token.error("concat! requires a literal")
            kind, value = literal.args
            if kind in FLOAT_TYPES:
                number = FLOAT_SUFFIX.sub("", literal_token.text).replace("_", "")
                values.append(("-" if negative else "") + number)
            else:
                values.append(display(value, False))
            comma = next(tokens, None)
            if comma is not None and comma.text != ",":
                raise comma.error("expected ',' after concat! argument")
        return "".join(values)

    def rules(self, group):
        items = group.items
        rules = []
        index = 0
        while index < len(items):
            if index + 2 >= len(items):
                raise group.opening.error("incomplete macro rule")
            matcher, arrow, body = items[index:index + 3]
            if not isinstance(matcher, TokenGroup) or tree_text(arrow) != "=>" or not isinstance(body, TokenGroup):
                raise group.opening.error("expected a macro matcher and transcriber")
            pattern = macro_sequence(matcher.items, matcher=True)
            validate_matcher(pattern, matcher.opening, self.edition)
            rules.append((pattern, macro_sequence(body.items, matcher=False)))
            index += 3
            if index < len(items):
                if tree_text(items[index]) not in {";", ","}:
                    raise group.opening.error("expected ';' between macro rules")
                index += 1
        return rules

    def invoke(self, token, rules, items, mark, site):
        self.match_steps = 0
        for matcher, body in rules:
            for offset, captures in self.match(matcher, items, 0, {}, ()):
                if offset == len(items):
                    return tuple(self.render(body, captures, (), mark, site))
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
                    if end > cursor or (node.separator is not None and end == start):
                        matches.append((end, found))
                        stack.append((end, found, count + 1))
            if node.count == "+":
                matches = matches[1:]
            for end, found in sorted(matches, key=lambda match: -match[0]):
                yield from self.match(rest, items, end, found, path)
            return
        if isinstance(node, MacroCapture):
            remaining = len(items) - pos
            minimum = 0 if node.fragment == "vis" and remaining else 1
            length = remaining
            if node.fragment in {"tt", "ident", "literal", "block", "lifetime"}:
                negative = remaining and node.fragment == "literal" and tree_text(items[pos]) == "-"
                length = min(2 if negative else 1, remaining)
            elif node.fragment == "vis":
                length = min(2, remaining)
            for end in range(pos + length, pos + minimum - 1, -1):
                part = items[pos:end]
                if self.fragment(node.fragment, part):
                    key = (node.name.text, path)
                    if key in captures:
                        raise node.name.error(f"duplicate macro matcher binding {node.name.text!r}")
                    yield from self.match(rest, items, end, captures | {key: (node.fragment, part)}, path)
            return
        if pos == len(items):
            return
        item = items[pos]
        if isinstance(node, TokenGroup):
            if isinstance(item, TokenGroup) and node.opening.text == item.opening.text:
                for end, found in self.match(node.items, item.items, 0, captures, path):
                    if end == len(item.items):
                        yield from self.match(rest, items, pos + 1, found, path)
        elif isinstance(node, Token) and isinstance(item, Token) and node.text == item.text:
            yield from self.match(rest, items, pos + 1, captures, path)

    def fragment(self, fragment, items):
        if not items:
            return fragment == "vis"
        item = items[0]
        if fragment == "tt":
            return True
        if len(items) == 1 and isinstance(item, OpaqueFragment):
            return fragment == item.fragment or {fragment, item.fragment} <= {"expr", "expr_2021"} or (
                fragment == "pat" and item.fragment == "pat_param"
            )
        if fragment == "ident":
            return isinstance(item, Token) and item.text != "_" and is_identifier(item.text)
        if fragment == "lifetime":
            return isinstance(item, Token) and bool(is_lifetime(item.text))
        if fragment == "literal":
            token = items[-1]
            try:
                return isinstance(token, Token) and Parser.literal(token, negative=len(items) == 2) is not None
            except (ValueError, RustError):
                return False
        if fragment == "block":
            return isinstance(item, TokenGroup) and item.opening.text == "{"
        if fragment == "vis":
            if tree_text(item) != "pub":
                return False
            if len(items) == 1:
                return True
            group = items[1]
            return (isinstance(group, TokenGroup) and group.opening.text == "(" and bool(group.items)
                    and tree_text(group.items[0]) in {"self", "super", "crate", "in"})
        if fragment in {"item", "stmt"} and self.item_fragment(items):
            return True
        if fragment == "item":
            return False
        tokens = list(flatten_trees(items))
        if not tokens:
            return False
        first = tokens[0]
        end = replace(first, text="")
        if fragment == "stmt" and first.text == "let":
            if tokens[-1].text == ";":
                return False
            tokens = [replace(first, text="{"), *tokens, replace(end, text=";"), replace(end, text="}")]
            first = tokens[0]
        parser = Parser("", tokens=(*tokens, end), fragment=True, edition=self.edition)
        try:
            if fragment in {"expr", "expr_2021"}:
                modern = fragment == "expr" and self.edition >= "2024"
                if first.text == "_" and (not modern or len(tokens) == 1):
                    return modern
                if first.text == "const" and not modern:
                    return False
                parser.expression()
            elif fragment == "ty":
                parser.type_name()
            elif fragment == "pat" and self.edition >= "2021":
                parser.pattern()
            elif fragment in {"pat", "pat_param"}:
                parser.range_pattern()
            elif fragment == "path":
                word = parser.advance()
                if not is_identifier(word.text):
                    return False
                parser.expression_path(word)
            elif fragment == "meta":
                parser.expression_path(parser.identifier())
                if parser.accept("="):
                    parser.expression()
                elif parser.current.text == "(":
                    parser.skip_group()
            elif fragment == "stmt":
                if first.text == "{":
                    parser.block()
                elif not parser.accept(";"):
                    parser.expression()
            return not parser.current.text
        except RustError:
            return False

    @staticmethod
    def item_fragment(items):
        index = 0
        while index + 1 < len(items) and tree_text(items[index]) == "#":
            attribute = items[index + 1]
            if not isinstance(attribute, TokenGroup) or attribute.opening.text != "[":
                return False
            index += 2
        if index < len(items) and tree_text(items[index]) == "pub":
            index += 1
            if index < len(items) and tree_text(items[index]) == "(":
                index += 1
        if index == len(items):
            return False
        if (tree_text(items[index]) == "const" and index + 1 < len(items)
                and tree_text(items[index + 1]) == "fn"):
            index += 1
        kind = tree_text(items[index])
        if kind not in {"fn", "mod", "use", "struct", "enum", "trait", "const", "static", "type",
                        "impl", "macro_rules"}:
            return False
        if index + 2 >= len(items):
            return False
        if kind == "macro_rules" and (tree_text(items[index + 1]) != "!" or index + 3 >= len(items)):
            return False
        closing = tree_text(items[-1])
        return (closing == ";" or (closing == "{" and kind not in {"use", "type", "const", "static"})) and (
            Modules.item_end(items, index, kind) == len(items)
        )

    @staticmethod
    def variables(nodes):
        for node in nodes:
            if isinstance(node, MacroCapture):
                yield node.name.text
            elif isinstance(node, MacroRepeat):
                yield from MacroExpander.variables(node.body)
            elif isinstance(node, TokenGroup):
                yield from MacroExpander.variables(node.items)

    def render(self, nodes, captures, path, mark, site):
        for index, node in enumerate(nodes):
            if isinstance(node, MacroCapture):
                if node.name.text == "crate":
                    following = nodes[index + 1] if index + 1 < len(nodes) else None
                    if not isinstance(following, Token) or following.text != "::":
                        raise node.name.error("expected a path after $crate")
                    yield replace(node.name, text="crate", expansion_site=site)
                    continue
                for level in range(len(path), -1, -1):
                    key = (node.name.text, path[:level])
                    if key in captures:
                        fragment, items = captures[key]
                        if fragment not in {"ident", "lifetime", "tt"}:
                            yield OpaqueFragment(node.name, fragment, items)
                        else:
                            yield from items
                        break
                else:
                    raise node.name.error(f"metavariable {node.name.text!r} has no value at this repetition depth")
            elif isinstance(node, TokenGroup):
                opening = replace(node.opening, context=(*node.opening.context, mark), expansion_site=site)
                closing = replace(node.closing, context=(*node.closing.context, mark), expansion_site=site)
                yield replace(node, opening=opening, closing=closing,
                              items=tuple(self.render(node.items, captures, path, mark, site)))
            elif isinstance(node, MacroRepeat):
                names = set(self.variables(node.body))
                if not names:
                    token = next((node for node in node.body if isinstance(node, Token)), Token("macro", 1, 1))
                    raise token.error("macro repetition needs a metavariable")
                indices = {key[len(path)] for name, key in captures
                           if name in names and len(key) > len(path) and key[:len(path)] == path}
                for position, index_in_path in enumerate(sorted(indices)):
                    if position and node.separator is not None:
                        yield replace(node.separator, expansion_site=site)
                    yield from self.render(node.body, captures, (*path, index_in_path), mark, site)
            elif isinstance(node, Token):
                yield replace(node, context=(*node.context, mark), expansion_site=site)
            else:
                yield node


MODULE_SEPARATOR = "§"
OPTION = MODULE_SEPARATOR.join(("core", "option", "Option"))
RESULT = MODULE_SEPARATOR.join(("core", "result", "Result"))
VEC = MODULE_SEPARATOR.join(("alloc", "vec", "Vec"))
BOX = MODULE_SEPARATOR.join(("alloc", "boxed", "Box"))
STRING = MODULE_SEPARATOR.join(("alloc", "string", "String"))
EXIT_CODE = MODULE_SEPARATOR.join(("std", "process", "ExitCode"))
TERMINATION = MODULE_SEPARATOR.join(("std", "process", "Termination"))
PROCESS_EXIT = MODULE_SEPARATOR.join(("std", "process", "exit"))
TRAIT_MODULE_NAMES = {"Display": "fmt", "Debug": "fmt", "PartialEq": "cmp", "Eq": "cmp", "PartialOrd": "cmp",
                      "Ord": "cmp", "Clone": "clone", "Default": "default", "Copy": "marker", "Hash": "hash"}
STD_ROOT = ("std",)
STD_BUILTIN_TRAITS = {MODULE_SEPARATOR.join(("std", TRAIT_MODULE_NAMES[name], name)): name
                      for name in TRAIT_MODULE_NAMES}
STD_PRELUDE = ("std", "prelude")
STD_CRATES = {"std", "core", "alloc"}
INTEGER_METHODS = """
impl {T} {
    pub fn pow(self, exp: u32) -> {T};
    pub fn abs(self) -> {T};
    pub fn signum(self) -> {T};
    pub fn is_positive(self) -> bool { self > 0 }
    pub fn is_negative(self) -> bool { self < 0 }
    pub fn checked_add(self, rhs: {T}) -> Option<{T}>;
    pub fn checked_sub(self, rhs: {T}) -> Option<{T}>;
    pub fn checked_mul(self, rhs: {T}) -> Option<{T}>;
    pub fn checked_div(self, rhs: {T}) -> Option<{T}>;
    pub fn checked_rem(self, rhs: {T}) -> Option<{T}>;
    pub fn checked_pow(self, exp: u32) -> Option<{T}>;
    pub fn checked_neg(self) -> Option<{T}>;
    pub fn checked_abs(self) -> Option<{T}>;
    pub fn wrapping_add(self, rhs: {T}) -> {T};
    pub fn wrapping_sub(self, rhs: {T}) -> {T};
    pub fn wrapping_mul(self, rhs: {T}) -> {T};
    pub fn wrapping_div(self, rhs: {T}) -> {T};
    pub fn wrapping_rem(self, rhs: {T}) -> {T};
    pub fn wrapping_neg(self) -> {T};
    pub fn wrapping_pow(self, exp: u32) -> {T};
    pub fn wrapping_abs(self) -> {T};
    pub fn saturating_add(self, rhs: {T}) -> {T};
    pub fn saturating_sub(self, rhs: {T}) -> {T};
    pub fn saturating_mul(self, rhs: {T}) -> {T};
    pub fn saturating_pow(self, exp: u32) -> {T};
    pub fn overflowing_add(self, rhs: {T}) -> ({T}, bool);
    pub fn overflowing_sub(self, rhs: {T}) -> ({T}, bool);
    pub fn overflowing_mul(self, rhs: {T}) -> ({T}, bool);
    pub fn rem_euclid(self, rhs: {T}) -> {T};
    pub fn div_euclid(self, rhs: {T}) -> {T};
    pub fn abs_diff(self, other: {T}) -> {U};
    pub fn unsigned_abs(self) -> {U};
    pub fn count_ones(self) -> u32;
    pub fn count_zeros(self) -> u32;
    pub fn leading_zeros(self) -> u32;
    pub fn trailing_zeros(self) -> u32;
    pub fn leading_ones(self) -> u32;
    pub fn trailing_ones(self) -> u32;
    pub fn checked_shl(self, rhs: u32) -> Option<{T}>;
    pub fn checked_shr(self, rhs: u32) -> Option<{T}>;
    pub fn wrapping_shl(self, rhs: u32) -> {T};
    pub fn wrapping_shr(self, rhs: u32) -> {T};
    pub fn rotate_left(self, n: u32) -> {T};
    pub fn rotate_right(self, n: u32) -> {T};
    pub fn swap_bytes(self) -> {T};
    pub fn reverse_bits(self) -> {T};
    pub fn is_power_of_two(self) -> bool;
    pub fn next_power_of_two(self) -> {T};
    pub fn isqrt(self) -> {T};
    pub fn ilog2(self) -> u32;
    pub fn ilog10(self) -> u32;
    pub fn ilog(self, base: {T}) -> u32;
    pub fn midpoint(self, rhs: {T}) -> {T};
    pub fn to_be_bytes(self) -> Vec<u8>;
    pub fn to_le_bytes(self) -> Vec<u8>;
    pub fn from_str_radix(src: &str, radix: u32) -> Result<{T}, crate::num::ParseIntError>;
}
impl crate::str::FromStr for {T} {
    type Err = crate::num::ParseIntError;
    fn from_str(s: &str) -> Result<{T}, crate::num::ParseIntError> { crate::intrinsics::parse_int(s) }
}
"""
FLOAT_METHODS = """
impl {T} {
    pub fn floor(self) -> {T};
    pub fn ceil(self) -> {T};
    pub fn round(self) -> {T};
    pub fn round_ties_even(self) -> {T};
    pub fn trunc(self) -> {T};
    pub fn fract(self) -> {T};
    pub fn signum(self) -> {T};
    pub fn cbrt(self) -> {T};
    pub fn powi(self, n: i32) -> {T};
    pub fn powf(self, n: {T}) -> {T};
    pub fn exp(self) -> {T};
    pub fn exp2(self) -> {T};
    pub fn exp_m1(self) -> {T};
    pub fn ln(self) -> {T};
    pub fn ln_1p(self) -> {T};
    pub fn log(self, base: {T}) -> {T};
    pub fn log10(self) -> {T};
    pub fn log2(self) -> {T};
    pub fn sin(self) -> {T};
    pub fn cos(self) -> {T};
    pub fn tan(self) -> {T};
    pub fn asin(self) -> {T};
    pub fn acos(self) -> {T};
    pub fn atan(self) -> {T};
    pub fn atan2(self, other: {T}) -> {T};
    pub fn sinh(self) -> {T};
    pub fn cosh(self) -> {T};
    pub fn tanh(self) -> {T};
    pub fn sin_cos(self) -> ({T}, {T});
    pub fn hypot(self, other: {T}) -> {T};
    pub fn to_degrees(self) -> {T};
    pub fn to_radians(self) -> {T};
    pub fn max(self, other: {T}) -> {T};
    pub fn min(self, other: {T}) -> {T};
    pub fn clamp(self, min: {T}, max: {T}) -> {T};
    pub fn mul_add(self, a: {T}, b: {T}) -> {T};
    pub fn recip(self) -> {T};
    pub fn rem_euclid(self, rhs: {T}) -> {T};
    pub fn div_euclid(self, rhs: {T}) -> {T};
    pub fn copysign(self, sign: {T}) -> {T};
    pub fn abs_sub(self, other: {T}) -> {T};
    pub fn is_sign_positive(self) -> bool;
    pub fn is_sign_negative(self) -> bool;
    pub fn is_normal(self) -> bool;
    pub fn total_cmp(&self, other: &{T}) -> crate::cmp::Ordering;
    pub fn to_bits(self) -> {B};
    pub fn from_bits(bits: {B}) -> {T};
}
impl crate::str::FromStr for {T} {
    type Err = crate::num::ParseFloatError;
    fn from_str(s: &str) -> Result<{T}, crate::num::ParseFloatError> { crate::intrinsics::parse_float(s) }
}
"""
SCALAR_STD = "pub mod num {\n    pub use crate::num_wrapping::Wrapping;\n" + """
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParseIntError { kind: IntErrorKind }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum IntErrorKind { Empty, InvalidDigit, PosOverflow, NegOverflow, Zero }
    impl ParseIntError {
        pub fn kind(&self) -> &IntErrorKind { &self.kind }
    }
    impl crate::fmt::Display for ParseIntError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
            match self.kind {
                IntErrorKind::Empty => write!(f, "cannot parse integer from empty string"),
                IntErrorKind::InvalidDigit => write!(f, "invalid digit found in string"),
                IntErrorKind::PosOverflow => write!(f, "number too large to fit in target type"),
                IntErrorKind::NegOverflow => write!(f, "number too small to fit in target type"),
                IntErrorKind::Zero => write!(f, "number would be zero for non-zero type"),
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParseFloatError { empty: bool }
    impl crate::fmt::Display for ParseFloatError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
            if self.empty { write!(f, "cannot parse float from empty string") } else { write!(f, "invalid float literal") }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParseBoolError;
    impl crate::fmt::Display for ParseBoolError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
            write!(f, "provided string was not `true` or `false`")
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParseCharError;
    impl crate::fmt::Display for ParseCharError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "invalid character") }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TryFromIntError;
    impl crate::fmt::Display for TryFromIntError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
            write!(f, "out of range integral type conversion attempted")
        }
    }
    impl crate::str::FromStr for bool {
        type Err = ParseBoolError;
        fn from_str(s: &str) -> Result<bool, ParseBoolError> {
            if s == "true" { Ok(true) } else if s == "false" { Ok(false) } else { Err(ParseBoolError) }
        }
    }
    impl crate::str::FromStr for char {
        type Err = ParseCharError;
        fn from_str(s: &str) -> Result<char, ParseCharError> {
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(c),
                _ => Err(ParseCharError),
            }
        }
    }
    impl crate::str::FromStr for String {
        type Err = crate::convert::Infallible;
        fn from_str(s: &str) -> Result<String, crate::convert::Infallible> { Ok(s.to_string()) }
    }
""" + "".join(INTEGER_METHODS.replace("{T}", kind).replace("{U}", "u" + kind[1:])
              for kind in sorted(INTEGER_TYPES)) + "".join(
    FLOAT_METHODS.replace("{T}", kind).replace("{B}", "u" + kind[1:]) for kind in sorted(FLOAT_TYPES)) + "}\n"
def integer_conversions():
    """From impls for lossless integer conversions and TryFrom for the rest, as Rust's std provides."""
    lines = []
    pointer = {"usize": "u64", "isize": "i64"}
    for source in sorted(INTEGER_TYPES):
        for target in sorted(INTEGER_TYPES):
            if source == target:
                continue
            low, high = INTEGER_BOUNDS[source]
            target_low, target_high = INTEGER_BOUNDS[target]
            lossless = target_low <= low and high <= target_high
            if lossless and (source in pointer or target in pointer):
                # std only promises usize and isize conversions from 8 and 16 bit types.
                lossless = INTEGER_WIDTHS[source] <= 16 and target in pointer and source not in pointer
            if lossless:
                lines.append(f"impl From<{source}> for {target} {{ fn from(value: {source}) -> {target} "
                             f"{{ value as {target} }} }}")
            else:
                lines.append(f"impl TryFrom<{source}> for {target} {{ type Error = TryFromIntError; "
                             f"fn try_from(value: {source}) -> Result<{target}, TryFromIntError> "
                             f"{{ crate::intrinsics::try_integer(value) }} }}")
        for kind in sorted(FLOAT_TYPES):
            if INTEGER_WIDTHS[source] <= (16 if kind == "f32" else 32) and source not in pointer:
                lines.append(f"impl From<{source}> for {kind} {{ fn from(value: {source}) -> {kind} "
                             f"{{ value as {kind} }} }}")
    return ("pub mod integer_conversions {\n    use crate::convert::{From, TryFrom};\n"
            "    use crate::num::TryFromIntError;\n    " + "\n    ".join(lines) + "\n}\n")


STD_SOURCE = Path(__file__).with_name("std.rs").read_text(encoding="utf-8") + SCALAR_STD + integer_conversions()
ITERATOR = MODULE_SEPARATOR.join(("std", "iter", "Iterator"))
INTO_ITERATOR = MODULE_SEPARATOR.join(("std", "iter", "IntoIterator"))
LAZY = MODULE_SEPARATOR.join(("std", "iter", "Lazy"))
PATTERN = MODULE_SEPARATOR.join(("std", "str", "Pattern"))
CONVERT_FROM = MODULE_SEPARATOR.join(("std", "convert", "From"))
DEREF = MODULE_SEPARATOR.join(("std", "ops", "Deref"))
DEREF_MUT = MODULE_SEPARATOR.join(("std", "ops", "DerefMut"))
GUARD_TYPES = {MODULE_SEPARATOR.join(("std", "cell", "RefMut"))}
GUARDS = GUARD_TYPES | {MODULE_SEPARATOR.join(("std", "cell", "Ref"))}
DROP = MODULE_SEPARATOR.join(("std", "ops", "Drop"))
RC = MODULE_SEPARATOR.join(("std", "rc", "Rc"))
WEAK = MODULE_SEPARATOR.join(("std", "rc", "Weak"))
POINTER_TYPES = {BOX, RC, WEAK}
INDIRECT_TYPES = {VEC, BOX, RC, WEAK, *(MODULE_SEPARATOR.join(("std", "collections", name)) for name in (
    "HashMap", "BTreeMap", "HashSet", "BTreeSet", "VecDeque", "BinaryHeap"))}
MAP_TYPES = {MODULE_SEPARATOR.join(("std", "collections", name)): name
             for name in ("HashMap", "BTreeMap", "HashSet", "BTreeSet")}
DEQUE = MODULE_SEPARATOR.join(("std", "collections", "VecDeque"))
HEAP = MODULE_SEPARATOR.join(("std", "collections", "BinaryHeap"))
OPERATOR_TRAITS = {
    "+": ("Add", "add"), "-": ("Sub", "sub"), "*": ("Mul", "mul"), "/": ("Div", "div"), "%": ("Rem", "rem"),
    "&": ("BitAnd", "bitand"), "|": ("BitOr", "bitor"), "^": ("BitXor", "bitxor"), "<<": ("Shl", "shl"),
    ">>": ("Shr", "shr"),
}
UNARY_TRAITS = {"-": ("Neg", "neg"), "!": ("Not", "not")}
BUILTIN_OPERATORS = {MODULE_SEPARATOR.join(("std", "ops", trait)): symbol
                     for table in (OPERATOR_TRAITS, UNARY_TRAITS) for symbol, (trait, _) in table.items()}
BUILTIN_OPERATORS.update({MODULE_SEPARATOR.join(("std", "ops", trait + "Assign")): symbol + "="
                          for symbol, (trait, _) in OPERATOR_TRAITS.items()})

BUILTIN_METHOD_ARITIES = {name: 1 for name in (
    "is_some", "is_none", "is_ok", "is_err", "unwrap", "len", "is_empty", "to_string", "to_owned", "clone",
    "abs", "sqrt", "is_nan", "is_finite", "chars", "bytes", "iter", "into_iter", "trim", "to_uppercase",
    "to_lowercase", "as_str", "pop")}
BUILTIN_METHOD_ARITIES.update({"expect": 2, "unwrap_or": 2, "push": 2, "push_str": 2, "contains": 2})
PRELUDE_OWNERS = {MODULE_SEPARATOR.join(parts) for parts in (
    ("core", "option", "Option"), ("core", "result", "Result"), ("alloc", "vec", "Vec"), ("alloc", "string", "String"),
    ("alloc", "boxed", "Box"))}
PRELUDE_TYPES = {"Option": OPTION, "Result": RESULT, "Vec": VEC, "Box": BOX, "String": STRING}
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
macro_rules! debug_assert {
    ($($argument:tt)*) => { assert!($($argument)*) };
}
macro_rules! debug_assert_eq {
    ($($argument:tt)*) => { assert_eq!($($argument)*) };
}
macro_rules! debug_assert_ne {
    ($($argument:tt)*) => { assert_ne!($($argument)*) };
}
macro_rules! write {
    ($destination:expr, $($argument:tt)*) => { $destination.write_fmt(format!($($argument)*)) };
}
macro_rules! writeln {
    ($destination:expr $(,)?) => { $destination.write_str("\n") };
    ($destination:expr, $($argument:tt)*) => {
        match $destination.write_fmt(format!($($argument)*)) {
            Ok(()) => $destination.write_str("\n"),
            Err(error) => Err(error),
        }
    };
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
    macro_use: bool = False


class Modules:
    def __init__(self, trees, source_path, edition):
        self.source_path = os.fspath(source_path) if source_path is not None else None
        self.directory = Path(source_path).absolute().parent if source_path is not None else None
        root = ModuleUnit((), self.directory)
        self.units = {(): root}
        self.edition = edition
        self.exported = {}
        self.macro_paths = {}
        self.constants = {}
        self.statics = {}
        self.const_functions = {}
        self.active_const_functions = set()
        self.type_aliases = {}
        self.alias_names = set()
        self.lazy_items = {}
        self.lazy_fallback = False
        self.enum_variants = {}
        root.items = self.scan(root, trees)
        std = ModuleUnit(STD_ROOT, None)
        self.units[STD_ROOT] = std
        std.items = self.scan(std, token_trees(lex(STD_SOURCE, edition=edition, source_file="<std>"))[0])

    def register(self, unit, token, kind, boundary):
        if not is_identifier(token.text) or token.text in RESERVED_BY_EDITION[self.edition]:
            raise token.error("expected an item identifier")
        name = identifier_text(token.text)
        if name in unit.symbols:
            raise token.error(f"duplicate item {token.text!r}")
        symbol = ModuleSymbol((*unit.path, name), kind, boundary, token)
        unit.symbols[name] = symbol
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
            if kind not in {"const", "static"}:
                if word == "<":
                    angles += 1
                elif word in {">", ">="}:
                    angles = max(0, angles - 1)
            if angles == 0:
                if word == ";":
                    return position + 1
                if kind not in {"const", "static", "type", "use"} and isinstance(item, TokenGroup) and word == "{":
                    return position + 1
        return len(items)

    def scan(self, unit, items, *, expanded=False):
        items = tuple(expose_fragments(items, {"item", "vis"}))
        if not expanded:
            items = self.hoist_inner_items(items)
        output = []
        index = 0
        while index < len(items):
            start = index
            kept_attributes = []
            export = False
            local_inner = False
            macro_use = False
            custom_path = None
            while index + 1 < len(items) and tree_text(items[index]) == "#":
                inner = tree_text(items[index + 1]) == "!"
                position = index + (2 if inner else 1)
                if position >= len(items) or not isinstance(items[position], TokenGroup):
                    break
                attribute = items[position]
                attribute = replace(attribute, items=tuple(expose_fragments(attribute.items, {"meta"})))
                if not inner and attribute.items and tree_text(attribute.items[0]) == "macro_export":
                    export = True
                    if len(attribute.items) > 1:
                        option = attribute.items[1]
                        if (len(attribute.items) != 2 or not isinstance(option, TokenGroup)
                                or option.opening.text != "("
                                or tuple(map(tree_text, option.items)) not in {("local_inner_macros",),
                                                                                ("local_inner_macros", ",")}):
                            raise attribute.opening.error("unsupported macro_export option")
                        local_inner = True
                elif not inner and len(attribute.items) == 1 and tree_text(attribute.items[0]) == "macro_use":
                    macro_use = True
                elif (not inner and len(attribute.items) == 3
                      and tuple(map(tree_text, attribute.items[:2])) == ("path", "=")
                      and isinstance(attribute.items[2], Token)):
                    if not is_string_literal(attribute.items[2].text):
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
            if (tree_text(items[index]) == "unsafe" and index + 1 < len(items)
                    and tree_text(items[index + 1]) in {"fn", "impl", "trait"}):
                index += 1
                if isinstance(items[index], Token) and items[index].text == "fn":
                    UNSAFE_FUNCTIONS.add(token_position(items[index]))
            word = tree_text(items[index])
            if macro_use and word != "mod":
                raise items[index].error("macro_use requires a module")
            if word == "const" and index + 1 < len(items) and tree_text(items[index + 1]) == "fn":
                end = self.item_end(items, index + 1, "fn")
                if index + 2 >= len(items):
                    raise items[index].error("expected a const function name")
                name = items[index + 2]
                if not isinstance(name, Token):
                    raise items[index].error("expected a const function name")
                original = unit.symbols.get(identifier_text(name.text))
                if expanded and original and original.token is name:
                    symbol = original
                else:
                    symbol = self.register(unit, name, "fn", boundary)
                if expanded:
                    self.const_functions[symbol.name] = (unit, items[index:end])
                output.extend(kept_attributes)
                output.extend(items[index:end])
                index = end
                continue
            if word == "mod" and index + 2 < len(items):
                token, body = items[index + 1:index + 3]
                if not isinstance(token, Token):
                    raise items[index].error("expected a module name")
                symbol = self.register(unit, token, "module", boundary)
                directory = unit.directory / symbol.path[-1] if unit.directory is not None else None
                if custom_path is not None and unit.directory is not None:
                    directory = (unit.directory / custom_path).resolve().parent
                child = ModuleUnit(symbol.path, directory, macro_use=macro_use)
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
            if word in {"fn", "struct", "enum", "trait", "const", "static", "type", "impl", "macro_rules"}:
                end = self.item_end(items, index, word)
                if word not in {"impl", "macro_rules"} and index + 1 < len(items):
                    name = items[index + 1]
                    if word == "static" and tree_text(name) == "mut" and index + 2 < len(items):
                        name = items[index + 2]
                    if isinstance(name, Token):
                        kind = word
                        if word == "struct" and not any(isinstance(part, TokenGroup)
                                                        for part in items[index + 2:end]):
                            kind = "unit_struct"
                        original = unit.symbols.get(identifier_text(name.text))
                        if not expanded or not original or original.token is not name:
                            self.register(unit, name, kind, boundary)
                        if word == "enum":
                            body = next((part for part in items[index + 2:end]
                                         if isinstance(part, TokenGroup) and tree_text(part) == "{"), None)
                            if body is not None:
                                symbol = unit.symbols[identifier_text(name.text)]
                                self.enum_variants[symbol.name] = self.variant_names(body.items)
                if expanded and word in {"const", "static", "type"}:
                    unit.declarations.append(items[index:end])
                else:
                    if export and word == "macro_rules":
                        name, body = items[index + 2:index + 4]
                        key = identifier_text(name.text)
                        if key in self.exported and not expanded:
                            raise name.error(f"duplicate exported macro {name.text!r}")
                        self.exported[key] = body, local_inner
                    output.extend(kept_attributes)
                    output.extend(items[index:end])
                index = end
                continue
            output.extend(kept_attributes)
            output.append(items[index])
            index += 1
        return tuple(output)

    @staticmethod
    def variant_names(items):
        names = []
        expect_name = True
        index = 0
        while index < len(items):
            item = items[index]
            if expect_name and tree_text(item) == "#" and index + 1 < len(items):
                index += 2
                continue
            if expect_name and isinstance(item, Token) and is_identifier(item.text):
                names.append(identifier_text(item.text))
                expect_name = False
            elif tree_text(item) == ",":
                expect_name = True
            index += 1
        return names

    INNER_ITEMS = {"fn", "struct", "enum", "impl", "trait", "use", "type", "static", "mod"}

    def hoist_inner_items(self, items):
        """Move items declared inside function bodies to module level, renaming any that collide."""
        output = []
        taken = {tree_text(items[index + 1]) for index in range(len(items) - 1)
                 if tree_text(items[index]) in {"fn", "struct", "enum", "trait", "type", "const", "static"}}
        taken |= {alias for index in range(len(items)) if tree_text(items[index]) == "use"
                  for alias in [self.use_alias(items[index:self.item_end(items, index, "use")])[0]] if alias}
        index = 0
        while index < len(items):
            item = items[index]
            word = tree_text(item)
            if word in {"fn", "impl", "trait"} and index + 1 < len(items):
                end = self.item_end(items, index, "fn" if word == "fn" else word)
                body_index = end - 1
                if body_index > index and isinstance(items[body_index], TokenGroup) and tree_text(items[body_index]) == "{":
                    hoisted = []
                    if word == "fn":
                        body = self.hoist_from_function(items[body_index], tree_text(items[index + 1]), taken, hoisted)
                    else:
                        body = self.hoist_from_members(items[body_index], taken, hoisted)
                    output.extend(items[index:body_index])
                    output.append(body)
                    output.extend(self.hoist_inner_items(tuple(hoisted)))
                    index = end
                    continue
            output.append(item)
            index += 1
        return tuple(output)

    def hoist_from_members(self, group, taken, hoisted):
        """Hoist the inner items of each method in an impl or trait body."""
        items = group.items
        output = []
        index = 0
        while index < len(items):
            if tree_text(items[index]) == "fn" and index + 1 < len(items):
                end = self.item_end(items, index, "fn")
                body_index = end - 1
                if isinstance(items[body_index], TokenGroup) and tree_text(items[body_index]) == "{":
                    output.extend(items[index:body_index])
                    output.append(self.hoist_from_function(items[body_index], tree_text(items[index + 1]),
                                                           taken, hoisted))
                    index = end
                    continue
            output.append(items[index])
            index += 1
        return replace(group, items=tuple(output))

    def hoist_from_function(self, body, function_name, taken, hoisted):
        return self.hoist_block(body, function_name, taken, hoisted)

    def hoist_block(self, group, function_name, taken, hoisted):
        """Hoist one block's items, renaming collisions within that block only, as Rust scopes them."""
        items = group.items
        kept, own = [], []
        index = 0
        while index < len(items):
            start = index
            statement_start = index == 0 or tree_text(items[index - 1]) in {";", "}"} or (
                isinstance(items[index - 1], TokenGroup) and tree_text(items[index - 1]) == "{")
            if statement_start:
                cursor = index
                while (cursor + 1 < len(items) and tree_text(items[cursor]) == "#"
                       and isinstance(items[cursor + 1], TokenGroup)):
                    cursor += 2
                if cursor < len(items) and tree_text(items[cursor]) == "pub":
                    cursor += 1
                word = tree_text(items[cursor]) if cursor < len(items) else ""
                is_const_item = (word == "const" and cursor + 2 < len(items)
                                 and isinstance(items[cursor + 1], Token) and tree_text(items[cursor + 2]) == ":")
                if word == "fn" and (cursor + 1 >= len(items) or not isinstance(items[cursor + 1], Token)
                                     or not is_identifier(tree_text(items[cursor + 1]))):
                    word = ""
                if word in self.INNER_ITEMS or is_const_item:
                    end = self.item_end(items, cursor, word if word != "mod" else "fn")
                    own.append((word, cursor - start, items[start:end]))
                    index = end
                    continue
            item = items[index]
            if isinstance(item, TokenGroup):
                item = self.hoist_block(item, function_name, taken, hoisted)
            kept.append(item)
            index += 1
        renames = {}
        rewritten = []
        for kind, offset, inner in own:
            if kind == "use":
                alias, position = self.use_alias(inner)
                if alias and alias in taken:
                    fresh = f"{alias}_in_{function_name}_{len(taken)}"
                    renames[alias] = fresh
                    renamed = replace(inner[position], text=fresh)
                    if position > 0 and tree_text(inner[position - 1]) == "as":
                        inner = (*inner[:position], renamed, *inner[position + 1:])
                    else:
                        inner = (*inner[:position + 1], replace(inner[position], text="as"), renamed,
                                 *inner[position + 1:])
                    alias = fresh
                if alias:
                    taken.add(alias)
            elif kind not in {"impl", "mod"}:
                name_index = offset + 1 + (1 if kind == "static" and tree_text(inner[offset + 1]) == "mut" else 0)
                name = tree_text(inner[name_index])
                if name in taken:
                    renames[name] = f"{name}_in_{function_name}_{len(taken)}"
                taken.add(renames.get(name, name))
            rewritten.append((kind, inner))
        if renames:
            kept = [self.rename_tokens(item, renames) for item in kept]
        for kind, inner in rewritten:
            if kind == "fn":
                # Items nested in a hoisted function body are handled when it is hoisted in turn.
                pass
            hoisted.extend(inner if kind == "use" else (self.rename_tokens(tree, renames) for tree in inner))
        return replace(group, items=tuple(kept))

    @staticmethod
    def use_alias(item):
        """The name a simple `use` binds and the index of its token, or (None, None) for groups and globs."""
        trees = [tree for tree in item if tree_text(tree) != ";"]
        if not trees or any(isinstance(tree, TokenGroup) for tree in trees):
            return None, None
        last = len(trees) - 1
        name = tree_text(trees[last])
        if name == "*" or not is_identifier(name):
            return None, None
        return identifier_text(name), item.index(trees[last])

    def extract_items(self, group, hoisted):
        items = group.items
        kept = []
        index = 0
        while index < len(items):
            start = index
            statement_start = index == 0 or tree_text(items[index - 1]) in {";", "}"} or (
                isinstance(items[index - 1], TokenGroup) and tree_text(items[index - 1]) == "{")
            if statement_start:
                cursor = index
                while (cursor + 1 < len(items) and tree_text(items[cursor]) == "#"
                       and isinstance(items[cursor + 1], TokenGroup)):
                    cursor += 2
                if cursor < len(items) and tree_text(items[cursor]) == "pub":
                    cursor += 1
                word = tree_text(items[cursor]) if cursor < len(items) else ""
                is_const_item = (word == "const" and cursor + 2 < len(items)
                                 and isinstance(items[cursor + 1], Token) and tree_text(items[cursor + 2]) == ":")
                if word in self.INNER_ITEMS or is_const_item:
                    if word == "fn" and (cursor + 1 >= len(items) or not isinstance(items[cursor + 1], Token)
                                         or not is_identifier(tree_text(items[cursor + 1]))):
                        word = ""
                if word in self.INNER_ITEMS or is_const_item:
                    end = self.item_end(items, cursor, word if word != "mod" else "fn")
                    hoisted.append(items[start:end])
                    index = end
                    continue
            item = items[index]
            if isinstance(item, TokenGroup):
                item = replace(item, items=self.extract_items(item, hoisted).items)
            kept.append(item)
            index += 1
        return replace(group, items=tuple(kept))

    def rename_tokens(self, tree, renames):
        if isinstance(tree, TokenGroup):
            return replace(tree, items=tuple(self.rename_tokens(item, renames) for item in tree.items))
        if isinstance(tree, Token) and tree.text in renames:
            return replace(tree, text=renames[tree.text])
        return tree

    def load_module(self, unit, token, custom_path=None):
        if unit.directory is None:
            raise token.error("an external module requires a source file path")
        if custom_path is not None:
            filename = unit.directory / custom_path
            if not filename.is_file():
                raise token.error(f"module {token.text!r} needs {filename}")
            filenames = (filename,)
        else:
            name = identifier_text(token.text)
            if not name.isascii():
                raise token.error("a non-ASCII file-backed module requires a path attribute")
            filenames = (unit.directory / f"{name}.rs", unit.directory / name / "mod.rs")
        found = [path for path in filenames if path.is_file()]
        if len(found) != 1:
            if found:
                raise token.error(f"module {token.text!r} exists in both {filenames[0]} and {filenames[1]}")
            raise token.error(f"module {token.text!r} needs {filenames[0]} or {filenames[1]}")
        try:
            filename = found[0]
            source_file = os.path.join(os.path.dirname(self.source_path), os.path.relpath(filename, self.directory))
            return token_trees(lex(filename.read_text(encoding="utf-8"), edition=self.edition,
                                   source_file=source_file))[0]
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
            name = identifier_text(item.text)
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
                alias = identifier_text(tree_text(items[index + 1]))
                index += 2
            if index != len(items):
                raise item.error("unexpected token in import")
            return [Import(tuple(parts), alias, item, boundary)]
        return []

    def resolve(self, path, unit, token, *, origin=None):
        symbol = self.resolve_symbol(path, unit, token, origin=origin)
        if symbol is not None and symbol.name in STD_BUILTIN_TRAITS:
            return ModuleSymbol((STD_BUILTIN_TRAITS[symbol.name],), "trait", (), token)
        return symbol

    def resolve_symbol(self, path, unit, token, *, origin=None):
        origin = unit.path if origin is None else origin
        path = list(map(identifier_text, path))
        root = unit
        absolute = False
        if path[0] in {"crate", "self", "super"}:
            head = path.pop(0)
            absolute = True
            if head == "crate":
                root = self.units[STD_ROOT if unit.path[:1] == STD_ROOT else ()]
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
        if not symbol and not absolute and first in STD_CRATES:
            symbol = ModuleSymbol(STD_ROOT, "module", (), token)
        if not symbol and not absolute:
            symbol = self.lookup(self.units[STD_PRELUDE], first, origin, token)
        if not symbol:
            if absolute:
                raise token.error("unresolved path")
            return self.prelude((first, *path), token)
        while path and symbol.kind == "module":
            nested = self.units[symbol.path]
            name = path.pop(0)
            found = self.lookup(nested, name, origin, token)
            if found and found.name in STD_BUILTIN_TRAITS:
                found = ModuleSymbol((STD_BUILTIN_TRAITS[found.name],), "trait", (), token)
            if not found and symbol.path[:1] == STD_ROOT:
                if not path and TRAIT_MODULES.get(name) == symbol.path[-1]:
                    return ModuleSymbol((name,), "trait", (), token)
                return self.prelude(("std", *symbol.path[1:], name, *path), token)
            symbol = found
            if not symbol:
                raise token.error("unresolved path")
        if not path:
            return symbol
        if len(path) == 1 and symbol.kind in {"struct", "unit_struct", "enum", "trait"}:
            return ModuleSymbol((*symbol.path[:-1], f"{symbol.path[-1]}::{path[0]}"), "associated", (), token)
        if len(path) == 1 and symbol.kind == "type" and symbol.name in self.type_aliases:
            # `Alias::Variant` names the variant of the aliased enum.
            target = self.type_aliases[symbol.name].target
            name = target.name if isinstance(target, AppliedType) else target
            if isinstance(name, str):
                parts = name.split(MODULE_SEPARATOR)
                return ModuleSymbol((*parts[:-1], f"{parts[-1]}::{path[0]}"), "associated", (), token)
        raise token.error("unresolved associated path")

    @staticmethod
    def prelude(path, token):
        head, *tail = path
        if tuple(path[:2]) == ("std", "process") and len(path) in {3, 4}:
            name = path[2]
            if name == "exit" and len(path) == 3:
                return ModuleSymbol(tuple(path), "fn", (), token)
            if name in {"ExitCode", "Termination"}:
                if len(path) == 4:
                    return ModuleSymbol(("std", "process", f"{name}::{path[3]}"), "associated", (), token)
                kind = "struct" if name == "ExitCode" else "trait"
                return ModuleSymbol(tuple(path), kind, (), token)
        if head in {"std", "core"} and len(tail) >= 2:
            crate = head
            module, head, *tail = tail
            if (module, head) not in {("option", "Option"), ("result", "Result")}:
                if crate != "std" or (module, head) not in {("vec", "Vec"), ("boxed", "Box"), ("string", "String")}:
                    return None
        if head in PRELUDE_TYPES and len(tail) <= 1:
            parts = tuple(PRELUDE_TYPES[head].split(MODULE_SEPARATOR))
            if tail:
                return ModuleSymbol((*parts[:-1], f"{parts[-1]}::{tail[0]}"), "associated", (), token)
            return ModuleSymbol(parts, "struct" if head in {"Vec", "Box", "String"} else "enum", (), token)
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
                if item.alias == "*" and target.kind == "enum" and target.name in self.enum_variants:
                    for variant in self.enum_variants[target.name]:
                        symbol = ModuleSymbol((*target.path[:-1], f"{target.path[-1]}::{variant}"),
                                              "associated", (), item.token)
                        unit.glob_aliases.setdefault(variant, symbol)
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
            if bare and len(parts) == 1 and symbol.kind not in {"unit_struct", "associated", "const", "static"}:
                return token
            return replace(token, text=symbol.name)
        if len(parts) > 1:
            return replace(token, text="::".join(parts))
        if type_name and unit.path:
            return replace(token, text=MODULE_SEPARATOR.join((*unit.path, token.text)))
        return token

    def expand(self):
        crate_name = Path(self.source_path).stem.replace("-", "_") if self.source_path else "rust_out"
        expander = MacroExpander(self.edition, crate_name)
        builtin_items, _ = token_trees(lex(PRELUDE_MACROS))
        builtins = {}
        for index in range(0, len(builtin_items), 4):
            _, _, name, body = builtin_items[index:index + 4]
            expander.definitions += 1
            builtins[name.text] = (expander.definitions, expander.rules(body), False)
        exports = {}
        for name, (body, local_inner) in self.exported.items():
            expander.definitions += 1
            exports[name] = (expander.definitions, expander.rules(body), local_inner)
            if local_inner:
                expander.local_inner_macros.add(id(body))
        self.link_macros(exports)
        self.link_imports(strict=False)
        self.link_macros(exports)
        pending = deque(self.units)
        declared = {}

        def expand_unit(path):
            if path in declared:
                return declared[path]
            definitions = declared[path] = {}
            unit = self.units[path]
            macros = builtins | expander.module_macros.get(unit.path, {})
            macros.update({item.alias: self.macro_paths[self.macro_path(unit, item.path)][0]
                           for item in unit.imports if self.is_macro_import(unit, item)})
            previous = expander.qualified
            previous_path = expander.module_path
            expander.qualified = self.macro_qualifiers(unit)
            expander.module_path = unit.path
            try:
                expanded = expander.expand(unit.items, macros, item_scope=True, definitions=definitions)
            finally:
                expander.qualified = previous
                expander.module_path = previous_path
            unit.items = self.scan(unit, expanded, expanded=True)
            return definitions

        expander.import_module = lambda path: expand_unit(path) if self.units[path].macro_use else {}
        while pending:
            path = pending.popleft()
            if path in declared:
                continue
            expand_unit(path)
            pending.extend(path for path in self.units if path not in declared)
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
                    elif parser.accept("static"):
                        parser.constant_item(static=True)
                    else:
                        parser.expect("type")
                        alias = parser.type_alias()
                        self.type_aliases[alias.name.text] = alias
                except RustError as error:
                    remaining.append((unit, segment, error))
            if len(remaining) == len(pending):
                if self.lazy_fallback:
                    raise remaining[0][2]
                # Nothing resolved this round: evaluate what const evaluation cannot at first use instead.
                self.lazy_fallback = True
            pending = [(unit, segment) for unit, segment, _ in remaining]

    def read_const_function(self, name):
        unit, segment = self.const_functions[name]
        parser = Parser("", tokens=(*flatten_trees(segment), replace(segment[0], text="")),
                        modules=self, unit=unit)
        parser.expect("const")
        parser.expect("fn")
        return parser, parser.function(parser.declare_token(parser.identifier()))


@dataclass(frozen=True)
class LazyItem:
    name: Token
    kind: str
    annotation: object
    expression: object
    mutable: bool


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
    where_bounds: tuple = ()


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


STR_REFERENCE = ReferenceType("str", False)
STATIC_STR = replace(STR_REFERENCE, lifetime="'static")


def is_str_reference(annotation):
    return isinstance(annotation, ReferenceType) and annotation.inner == "str"


def is_static_str(annotation):
    return is_str_reference(annotation) and not annotation.mutable and annotation.lifetime == "'static"


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
class IteratorType:
    item: object
    source: tuple
    borrowed: bool = False

    def __str__(self):
        return f"iterator with item type {self.item}"


FAST_REPLACE.update({ReferenceType, FunctionType})


def borrowing_iterator(annotation):
    return isinstance(annotation, IteratorType) and annotation.borrowed


@dataclass(frozen=True)
class IteratorValue:
    collection: object
    borrowed: bool
    reverse: bool = False
    mappers: tuple = ()
    string_mode: str | None = None
    # A started iterator keeps its Python iterator in a one-element list.
    cursor: list | None = field(default=None, compare=False)


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
    anonymous: bool = False
    associated_bindings: dict = field(default_factory=dict)
    unsized: bool = False

    def __str__(self):
        return self.name


@dataclass(frozen=True, eq=False)
class TypeVariable:
    name: str
    token: Token
    const_type: str | None = None
    integer_literal: bool = False
    float_literal: bool = False

    def __str__(self):
        return self.name


@dataclass(frozen=True)
class AppliedType:
    name: str
    arguments: tuple

    def __str__(self):
        return f"{self.name}<{', '.join(map(str, self.arguments))}>"


def is_vector(annotation):
    return isinstance(annotation, AppliedType) and annotation.name == VEC


def is_box(annotation):
    return isinstance(annotation, AppliedType) and annotation.name == BOX


@dataclass(frozen=True)
class OpaqueType:
    function: str
    bounds: frozenset
    arguments: tuple
    bindings: tuple = ()

    def __str__(self):
        return f"impl {' + '.join(sorted(self.bounds))} returned by {self.function}"


@dataclass(frozen=True)
class ConstValue:
    value: int | bool | float | Character
    kind: str | None = field(default=None, compare=False)
    boolean: bool = field(init=False)

    def __post_init__(self):
        object.__setattr__(self, "boolean", type(self.value) is bool)

    def __str__(self):
        return display(self.value, True)


@dataclass(frozen=True)
class ExitCodeValue:
    status: int


@dataclass(frozen=True)
class ConstExpression:
    operator: str
    operands: tuple
    kind: str
    token: Token = field(compare=False)
    integer: str | None = None

    def __str__(self):
        if self.operator == "block":
            return "{ " + "; ".join(map(str, self.operands)) + " }"
        if self.operator == "if":
            condition, yes, no = self.operands
            return f"(if {condition} {{ {yes} }} else {{ {no} }})"
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
    if kind in FLOAT_TYPES and isinstance(value.value, float):
        return
    if kind in INTEGER_TYPES and type(value.value) is int:
        checked(value.value, token, kind)
        return
    raise token.error(f"expected a {kind} constant")


def fold_const(expression, *, defer_const=False):
    if defer_const or not all(isinstance(value, ConstValue) for value in expression.operands):
        return expression
    values = tuple(value.value for value in expression.operands)
    word, token, kind = expression.operator, expression.token, expression.kind
    if word == "block":
        return ConstValue(values[-1], kind)
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
        result = rounded_float(-values[0], kind) if kind in FLOAT_TYPES else checked(-values[0], token, kind)
    else:
        result = binary(word, *values, token, expression.integer or kind)
    return ConstValue(result, kind)


def resolve_const_operands(expression, resolve, *, defer_const=False):
    operands = expression.operands
    word = expression.operator
    if word in {"if", "&&", "||"}:
        first = resolve(operands[0], defer_const=defer_const)
        if isinstance(first, ConstValue) and not defer_const:
            if word == "if":
                value = resolve(operands[1] if first.value else operands[2])
                return replace(value, kind=expression.kind) if isinstance(value, ConstValue) else value
            if first.value == (word == "||"):
                return first
            return resolve(operands[1])
        operands = (first, *(resolve(value, defer_const=True) for value in operands[1:]))
        return replace(expression, operands=operands)
    operands = tuple(resolve(value, defer_const=defer_const) for value in operands)
    return fold_const(replace(expression, operands=operands), defer_const=defer_const)


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
    trait_arguments: tuple = ()

    def __str__(self):
        suffix = f"<{', '.join(map(str, self.arguments))}>" if self.arguments else ""
        return f"{self.owner}::{self.item}{suffix}"


@dataclass(frozen=True)
class AssociatedBinding:
    bound: str
    item: str
    arguments: tuple = ()
    lifetimes: tuple = ()

    def map_arguments(self, transform):
        return replace(self, arguments=tuple(map(transform, self.arguments)))

    def project(self, owner, trait):
        return AssociatedType(owner, self.item, trait.name.text, self.arguments)

    def match_arguments(self, arguments):
        if len(arguments) != len(self.arguments):
            return None
        replacements = {}
        for declared, actual in zip(self.arguments, arguments):
            if isinstance(declared, LifetimeArgument) and declared in self.lifetimes:
                if not isinstance(actual, LifetimeArgument) or replacements.setdefault(declared, actual) != actual:
                    return None
            elif declared != actual:
                return None
        return replacements

    def canonicalize(self, expected):
        if not self.lifetimes:
            return self, expected
        names = {lifetime: LifetimeArgument(f"'@bound:{index}")
                 for index, lifetime in enumerate(self.lifetimes)}
        binding = self.map_arguments(functools.partial(substitute, arguments=names))
        return replace(binding, lifetimes=tuple(names.values())), substitute(expected, names)


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
    parameters: tuple = ()
    defaults: tuple = ()


@dataclass(frozen=True)
class Implementation:
    token: Token
    target: object
    trait: str | None
    parameters: tuple
    methods: dict
    associated: dict
    trait_arguments: tuple = ()


def match_types(template, actual, parameters, result):
    if isinstance(actual, TypeVariable) and (
            actual.integer_literal and template in INTEGER_TYPES or actual.float_literal and template in FLOAT_TYPES):
        return True
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
    if isinstance(template, ReferenceType) and isinstance(actual, ReferenceType):
        return template.mutable == actual.mutable and match_types(template.inner, actual.inner, parameters, result)
    if isinstance(template, SliceType) and isinstance(actual, SliceType):
        return match_types(template.element, actual.element, parameters, result)
    if isinstance(template, FunctionType) and isinstance(actual, FunctionType):
        return len(template.parameters) == len(actual.parameters) and all(
            match_types(left, right, parameters, result)
            for left, right in zip((*template.parameters, template.result), (*actual.parameters, actual.result))
        )
    return template == actual


def literal_compatible(variable, other):
    """Whether an {integer} or {float} inference variable could become the other type."""
    if isinstance(other, (TypeVariable, TypeParameter)):
        return True
    if variable.integer_literal:
        return other in INTEGER_TYPES
    if variable.float_literal:
        return other in FLOAT_TYPES
    return True


def loose_match(template, actual, parameters, result):
    """match_types, except that inference variables in the actual type match anything."""
    if isinstance(actual, TypeVariable):
        if template in parameters:
            if template in result:
                return loose_equal(result[template], actual)
            result[template] = actual
            return True
        return isinstance(template, TypeParameter) or literal_compatible(actual, template)
    if template in parameters:
        if template in result:
            return loose_equal(result[template], actual)
        result[template] = actual
        return True
    if isinstance(template, AppliedType) and isinstance(actual, AppliedType):
        return (template.name == actual.name and len(template.arguments) == len(actual.arguments)
                and all(loose_match(left, right, parameters, result)
                        for left, right in zip(template.arguments, actual.arguments)))
    if isinstance(template, TupleType) and isinstance(actual, TupleType):
        return len(template.elements) == len(actual.elements) and all(
            loose_match(left, right, parameters, result) for left, right in zip(template.elements, actual.elements))
    if isinstance(template, ReferenceType) and isinstance(actual, ReferenceType):
        return template.mutable == actual.mutable and loose_match(template.inner, actual.inner, parameters, result)
    if isinstance(template, (ArrayType, SliceType)) and type(template) is type(actual):
        return loose_match(template.element, actual.element, parameters, result)
    return template == actual


def loose_equal(first, second):
    if isinstance(first, TypeVariable):
        return literal_compatible(first, second)
    if isinstance(second, TypeVariable):
        return literal_compatible(second, first)
    if isinstance(first, AppliedType) and isinstance(second, AppliedType):
        return first.name == second.name and all(map(loose_equal, first.arguments, second.arguments))
    return first == second


def bind_signature_parameters(parameters, result):
    """Infer impl parameters that only appear in Fn-bound outputs, such as B in F: FnMut(T) -> B."""
    for parameter in parameters:
        if parameter in result or not isinstance(parameter, TypeParameter):
            continue
        for owner, actual in tuple(result.items()):
            template = callable_signature(owner) if isinstance(owner, TypeParameter) else None
            signature = callable_signature(actual) if template is not None else None
            if signature is None or len(signature.parameters) != len(template.parameters):
                continue
            for left, right in zip((*template.parameters, template.result), (*signature.parameters, signature.result)):
                match_types(left, right, parameters, result)


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


FAST_REPLACE.update({Expr, IteratorType, AppliedType, AssociatedType})
TRAIT_MODULES = {
    "Clone": "clone", "Copy": "marker", "Sized": "marker", "Default": "default",
    "Debug": "fmt", "Display": "fmt", "PartialEq": "cmp", "Eq": "cmp",
    "PartialOrd": "cmp", "Ord": "cmp", "Fn": "ops", "FnMut": "ops", "FnOnce": "ops", "Hash": "hash",
    "Send": "marker", "Sync": "marker", "Unpin": "marker",
}
CALLABLE_TRAITS = ("Fn", "FnMut", "FnOnce")
CALL_ITEM = "@call"
# Bounds declared in where clauses on projections such as `C::Item: Display`, keyed by owner and item.
PROJECTION_BOUNDS = {}
# `#![allow(overflowing_literals)]` makes out-of-range literals wrap instead of failing to compile.
OVERFLOWING_LITERALS = [False]
# Deny-by-default lints for operations on constant operands that rustc rejects unless allowed.
ALLOWED_LINTS = set()
# Positions of the `fn` keywords of `unsafe fn` items, whose bodies may use mutable statics.
UNSAFE_FUNCTIONS = set()


def token_position(token):
    return token.line, token.column, token.source_file


TYPE_BOUNDS = set()
ARGS_ITEM = "@args"
SUPERTRAITS = {
    "Copy": {"Clone"}, "Eq": {"PartialEq"}, "PartialOrd": {"PartialEq"},
    "Ord": {"Eq", "PartialOrd", "PartialEq"}, "Fn": {"FnMut", "FnOnce"}, "FnMut": {"FnOnce"},
}
DERIVABLE = set(TRAIT_MODULES) - {"Display", "Sized", "Send", "Sync", "Unpin", *CALLABLE_TRAITS}


def type_leaves(annotation):
    """Yield every type nested in an annotation, including the annotation itself."""
    yield annotation
    if isinstance(annotation, ReferenceType):
        yield from type_leaves(annotation.inner)
    elif isinstance(annotation, FunctionType):
        for kind in (*annotation.parameters, annotation.result):
            yield from type_leaves(kind)
    elif isinstance(annotation, (AppliedType, OpaqueType)):
        for kind in annotation.arguments:
            yield from type_leaves(kind)
    elif isinstance(annotation, TupleType):
        for kind in annotation.elements:
            yield from type_leaves(kind)
    elif isinstance(annotation, (ArrayType, SliceType)):
        yield from type_leaves(annotation.element)
        if isinstance(annotation, ArrayType):
            yield annotation.length
    elif isinstance(annotation, AssociatedType):
        yield from type_leaves(annotation.owner)
        for kind in annotation.arguments:
            yield from type_leaves(kind)


def callable_signature(annotation):
    """Return the call signature a type promises through an Fn-family bound."""
    if isinstance(annotation, FunctionType):
        return annotation
    if isinstance(annotation, TypeParameter):
        bindings = annotation.associated_bindings.items()
    elif isinstance(annotation, OpaqueType):
        bindings = annotation.bindings
    elif isinstance(annotation, DynType):
        return dict(annotation.bindings).get(CALL_ITEM)
    else:
        return None
    for binding, kind in bindings:
        if binding.item == CALL_ITEM:
            return kind
    return None


def substitute(annotation, arguments, *, defer_const=False):
    if isinstance(annotation, ConstExpression):
        return resolve_const_operands(annotation, functools.partial(substitute, arguments=arguments),
                                      defer_const=defer_const)
    if isinstance(annotation, ReferenceType):
        lifetime = arguments.get(LifetimeArgument(annotation.lifetime), None) if annotation.lifetime else None
        return replace(annotation, inner=substitute(annotation.inner, arguments),
                       lifetime=(None if lifetime.name == "'_" else lifetime.name)
                       if lifetime else annotation.lifetime)
    if isinstance(annotation, FunctionType):
        return FunctionType(tuple(substitute(kind, arguments) for kind in annotation.parameters),
                            substitute(annotation.result, arguments), annotation.closure)
    if isinstance(annotation, IteratorType):
        return replace(annotation, item=substitute(annotation.item, arguments),
                       source=tuple(substitute(part, arguments) for part in annotation.source))
    if isinstance(annotation, AppliedType):
        return AppliedType(annotation.name, tuple(substitute(arg, arguments) for arg in annotation.arguments))
    if isinstance(annotation, OpaqueType):
        return replace(annotation, arguments=tuple(substitute(arg, arguments) for arg in annotation.arguments),
                       bindings=tuple((binding.map_arguments(functools.partial(substitute, arguments=arguments)),
                                       substitute(kind, arguments)) for binding, kind in annotation.bindings))
    if isinstance(annotation, TupleType):
        return TupleType(tuple(substitute(kind, arguments) for kind in annotation.elements))
    if isinstance(annotation, ArrayType):
        return ArrayType(substitute(annotation.element, arguments), substitute(annotation.length, arguments))
    if isinstance(annotation, SliceType):
        return SliceType(substitute(annotation.element, arguments))
    if isinstance(annotation, AssociatedType):
        return replace(annotation, owner=substitute(annotation.owner, arguments),
                       arguments=tuple(substitute(arg, arguments) for arg in annotation.arguments),
                       trait_arguments=tuple(substitute(arg, arguments) for arg in annotation.trait_arguments))
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
    constants_in_scope: dict = field(default_factory=dict)
    statics: dict = field(default_factory=dict)
    opaque_witnesses: dict = field(default_factory=dict)
    formats: dict = field(default_factory=dict)
    type_lifetimes: dict = field(default_factory=dict)
    try_conversions: dict = field(default_factory=dict)
    moves: set = field(default_factory=set)
    cross_compares: dict = field(default_factory=dict)
    discriminants: dict = field(default_factory=dict)
    crate_name: str = "main"

    def discriminant(self, ctor):
        """The integer a C-like enum variant casts to: its explicit value, or one past the previous variant."""
        value = -1
        for candidate in self.layout(ctor.annotation)[1]:
            value = self.discriminants.get(candidate.path, value + 1)
            if candidate.path == ctor.path:
                return value
        raise AssertionError(ctor.path)
    lazy_items: dict = field(default_factory=dict)
    normalized: dict | None = None
    dispatch_cache: dict = field(default_factory=dict)

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
        self.type_parameters[VEC] = (TypeParameter("T"),)
        self.type_parameters[BOX] = (TypeParameter("T"),)
        token = Token(TERMINATION, 1, 1)
        self_type = TypeParameter("Self", {TERMINATION})
        report = Function(replace(token, text="@termination_report"), [(replace(token, text="self"), False, self_type)],
                          EXIT_CODE, None, type_parameters=(self_type,))
        self.traits[TERMINATION] = Trait(token, self_type, set(), {"report": report}, {})

    def projection_trait(self, annotation):
        if annotation.trait:
            trait = self.traits.get(annotation.trait)
            if trait and annotation.item in trait.associated:
                return trait
            matches = [self.traits[name] for name in self.trait_closure({annotation.trait})
                       if name in self.traits and annotation.item in self.traits[name].associated]
            return matches[0] if len(matches) == 1 else None
        if isinstance(annotation.owner, (TypeParameter, OpaqueType)):
            bounds = self.trait_closure(annotation.owner.bounds)
            matches = [trait for name, trait in self.traits.items()
                       if name in bounds and annotation.item in trait.associated]
            if len(matches) == 1:
                return matches[0]
        elif isinstance(annotation.owner, IteratorType) and annotation.item == "Item":
            return self.traits.get(ITERATOR)
        elif annotation.item == "IntoIter" and not isinstance(annotation.owner, TypeVariable):
            return self.traits.get(INTO_ITERATOR)
        elif not isinstance(annotation.owner, TypeVariable):
            matches = [trait for name, trait in self.traits.items()
                       if annotation.item in trait.associated and self.implementation(annotation.owner, name)]
            if len(matches) == 1:
                return matches[0]
            if annotation.item == "Item" and ITERATOR in self.traits and self.implements(annotation.owner, ITERATOR):
                return self.traits[ITERATOR]
            if annotation.item == "Item" and INTO_ITERATOR in self.traits and (
                    iteration_item(annotation.owner) is not None):
                return self.traits[INTO_ITERATOR]
        return None

    def pin_projections(self, annotation):
        if isinstance(annotation, AssociatedType):
            owner = self.pin_projections(annotation.owner)
            annotation = replace(annotation, owner=owner,
                                 arguments=tuple(map(self.pin_projections, annotation.arguments)))
            if annotation.trait is None and isinstance(owner, TypeParameter):
                trait = self.projection_trait(annotation)
                if trait is not None:
                    annotation = replace(annotation, trait=trait.name.text)
            return annotation
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.pin_projections(annotation.inner))
        if isinstance(annotation, FunctionType):
            return FunctionType(tuple(map(self.pin_projections, annotation.parameters)),
                                self.pin_projections(annotation.result), annotation.closure)
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(map(self.pin_projections, annotation.arguments)))
        if isinstance(annotation, TupleType):
            return TupleType(tuple(map(self.pin_projections, annotation.elements)))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.pin_projections(annotation.element), annotation.length)
        if isinstance(annotation, SliceType):
            return SliceType(self.pin_projections(annotation.element))
        return annotation

    def associated_trait(self, bound, item):
        matches = [self.traits[name] for name in self.trait_closure({bound})
                   if name in self.traits and item in self.traits[name].associated]
        return matches[0] if len(matches) == 1 else None

    def normalize(self, annotation, seen=()):
        if self.normalized is not None and not seen:
            try:
                return self.normalized[annotation]
            except KeyError:
                result = self.normalized[annotation] = self.normalize_type(annotation)
                return result
            except TypeError:
                pass
        return self.normalize_type(annotation, seen)

    def normalize_type(self, annotation, seen=()):
        if isinstance(annotation, AssociatedType):
            owner = self.normalize(annotation.owner, seen)
            annotation = replace(annotation, owner=owner,
                                 arguments=tuple(self.normalize(arg, seen) for arg in annotation.arguments),
                                 trait_arguments=tuple(self.normalize(arg, seen) for arg in annotation.trait_arguments))
            if isinstance(owner, DynType):
                return dict(owner.bindings).get(annotation.item, annotation)
            if isinstance(owner, IteratorType) and annotation.item == "Item":
                return owner.item
            if (annotation.item == "Item" and isinstance(owner, AssociatedType) and owner.item == "IntoIter"
                    and owner.trait in {None, INTO_ITERATOR} and annotation.trait in {None, ITERATOR}):
                # IntoIterator declares `type IntoIter: Iterator<Item = Self::Item>`.
                return self.normalize(AssociatedType(owner.owner, "Item", INTO_ITERATOR), seen)
            primitive = owner.inner if isinstance(owner, ReferenceType) else owner
            if (annotation.item == "Output" and annotation.trait in BUILTIN_OPERATORS
                    and (primitive in PRIMITIVES or primitive == STRING)
                    and not any(isinstance(kind, AppliedType) or kind not in PRIMITIVES and isinstance(kind, str)
                                for kind in annotation.trait_arguments)):
                # Built-in arithmetic yields the operand type; impls like Mul<Vec2> for f64 are user-defined.
                return primitive
            if (annotation.item == "Item" and annotation.trait == INTO_ITERATOR
                    and not isinstance(owner, (TypeParameter, TypeVariable, AssociatedType, OpaqueType))
                    and self.implementation(owner, INTO_ITERATOR) is None):
                builtin = iteration_item(owner)
                if builtin is not None:
                    return builtin
                if ITERATOR in self.traits and self.implements(owner, ITERATOR):
                    return self.normalize(AssociatedType(owner, "Item", ITERATOR), seen)
            if (annotation.item == "IntoIter" and annotation.trait in {None, INTO_ITERATOR}
                    and not isinstance(owner, (TypeParameter, TypeVariable, AssociatedType, OpaqueType))
                    and self.implementation(owner, INTO_ITERATOR) is None):
                builtin = self.builtin_iterator_type(owner)
                if builtin is not None:
                    return builtin
                if ITERATOR in self.traits and self.implements(owner, ITERATOR):
                    return owner
            if annotation in seen:
                return annotation
            trait = self.projection_trait(annotation)
            if trait:
                annotation = replace(annotation, trait=trait.name.text)
                if annotation in seen:
                    return annotation
                if isinstance(owner, OpaqueType):
                    bindings = owner.bindings
                elif isinstance(owner, TypeParameter):
                    bindings = owner.associated_bindings.items()
                else:
                    bindings = ()
                for binding, kind in bindings:
                    if binding.item == annotation.item and self.associated_trait(binding.bound, binding.item) is trait:
                        arguments = binding.match_arguments(annotation.arguments)
                        if arguments is not None:
                            return self.normalize(substitute(kind, arguments), (*seen, annotation))
                if not isinstance(owner, (TypeParameter, TypeVariable)):
                    wanted = annotation.trait_arguments
                    implementation = self.implementation(
                        owner, trait.name.text,
                        arguments_wanted=wanted if wanted and not any(isinstance(kind, (TypeVariable, TypeParameter))
                                                                      for kind in wanted) else None)
                    if implementation:
                        impl, arguments = implementation
                        if annotation.item in impl.associated:
                            definition = impl.associated[annotation.item]
                            if any(part in impl.parameters and part not in arguments
                                   for part in type_leaves(definition.body)):
                                return annotation
                            if len(definition.parameters) == len(annotation.arguments):
                                arguments = arguments | dict(zip(definition.parameters, annotation.arguments))
                                return self.normalize(substitute(definition.body, arguments), (*seen, annotation))
            return annotation
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.normalize(annotation.inner, seen))
        if isinstance(annotation, FunctionType):
            return FunctionType(tuple(self.normalize(kind, seen) for kind in annotation.parameters),
                                self.normalize(annotation.result, seen), annotation.closure)
        if isinstance(annotation, IteratorType):
            return replace(annotation, item=self.normalize(annotation.item, seen),
                           source=tuple(self.normalize(part, seen) for part in annotation.source))
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.normalize(arg, seen) for arg in annotation.arguments))
        if isinstance(annotation, OpaqueType):
            return replace(annotation, arguments=tuple(self.normalize(arg, seen) for arg in annotation.arguments),
                           bindings=tuple((binding.map_arguments(functools.partial(self.normalize, seen=seen)),
                                           self.normalize(kind, seen)) for binding, kind in annotation.bindings))
        if isinstance(annotation, TupleType):
            return TupleType(tuple(self.normalize(kind, seen) for kind in annotation.elements))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.normalize(annotation.element, seen), annotation.length)
        if isinstance(annotation, SliceType):
            return SliceType(self.normalize(annotation.element, seen))
        return annotation

    def concrete_type(self, annotation):
        seen = set()
        while isinstance(annotation, OpaqueType):
            function = self.functions[annotation.function]
            if annotation in seen:
                raise function.name.error("cannot resolve recursive opaque return type")
            seen.add(annotation)
            arguments = dict(zip(function.type_parameters, annotation.arguments))
            annotation = self.normalize(substitute(self.opaque_witnesses[function.name.text], arguments))
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

    def trait_arguments(self, trait_name, given, owner):
        """Positional arguments of a generic trait, with defaults such as Rhs = Self filled in."""
        trait = self.traits.get(trait_name)
        if not trait or not trait.parameters:
            return ()
        given = tuple(given or ())
        result = []
        for index, parameter in enumerate(trait.parameters):
            if index < len(given):
                result.append(given[index])
            elif trait.defaults[index] is not None:
                result.append(substitute(trait.defaults[index],
                                         {trait.self_type: owner, **dict(zip(trait.parameters, result))}))
            else:
                result.append(None)
        return tuple(result)

    def implementation(self, annotation, trait, seen=(), arguments_wanted=None):
        key = (annotation, trait, arguments_wanted)
        if key in seen:
            return None
        for impl in self.impls:
            if impl.trait != trait:
                continue
            arguments = {}
            matched = match_types(impl.target, annotation, impl.parameters, arguments)
            if matched and arguments_wanted is not None:
                declared = self.trait_arguments(trait, impl.trait_arguments, impl.target)
                for template, actual in zip(declared, arguments_wanted):
                    if actual is None or template is None or isinstance(actual, TypeVariable):
                        continue
                    if not match_types(substitute(template, arguments) if not isinstance(template, TypeParameter)
                                       else template, actual, impl.parameters, arguments):
                        matched = False
                        break
            if matched and any(isinstance(value, (SliceType, DynType)) or value == "str"
                               for parameter, value in arguments.items() if isinstance(parameter, TypeParameter)
                               and not parameter.const_type and "?Sized" not in parameter.bounds):
                matched = False
            if matched:
                bind_signature_parameters(impl.parameters, arguments)
            # Parameters fixed only by trait arguments the caller has not determined yet stay open.
            open_parameters = {part for argument in impl.trait_arguments for part in type_leaves(argument)
                               if part in impl.parameters and part not in arguments}
            if matched and all(
                parameter in arguments or parameter in open_parameters for parameter in impl.parameters
            ) and all(
                self.implements(arguments[parameter], bound, seen=(*seen, key))
                for parameter in impl.parameters if parameter in arguments for bound in parameter.bounds
                if not isinstance(arguments[parameter], TypeVariable)
            ) and self.matches_associated_bounds(arguments):
                return impl, arguments
        return None

    def matches_associated_bounds(self, arguments):
        for parameter, argument in arguments.items():
            for binding, expected in parameter.associated_bindings.items():
                if binding.item in {CALL_ITEM, ARGS_ITEM}:
                    continue
                trait = self.associated_trait(binding.bound, binding.item)
                if trait is None:
                    return False
                actual = self.normalize(substitute(binding.project(argument, trait), arguments))
                if actual != self.normalize(substitute(expected, arguments)):
                    return False
        return True

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
        if trait in {"?Sized", "Send", "Sync", "Unpin"}:
            return True
        if trait in BUILTIN_OPERATORS:
            primitive = annotation.inner if isinstance(annotation, ReferenceType) else annotation
            if isinstance(primitive, TypeVariable) and (primitive.integer_literal or primitive.float_literal):
                return True
            symbol = BUILTIN_OPERATORS[trait].removesuffix("=")
            if primitive in NUMBER_TYPES and (symbol in FLOAT_OPERATORS or symbol == "-" or primitive in INTEGER_TYPES):
                return True
            if primitive == "bool" and symbol in {"!", "&", "|", "^"}:
                return True
            if primitive == STRING and symbol == "+":
                return True
        if isinstance(annotation, TypeVariable) and (annotation.integer_literal or annotation.float_literal):
            annotation = "f64" if annotation.float_literal else "i32"
        if (trait in TRAIT_MODULES and trait in self.traits
                and not isinstance(annotation, (TypeParameter, OpaqueType, DynType, AssociatedType, TypeVariable))
                and self.implementation(annotation, trait, seen) is not None):
            return True
        if isinstance(annotation, DynType):
            return trait in self.trait_closure({annotation.trait})
        if annotation == "str" and (trait in TRAIT_MODULES or trait not in self.traits):
            return trait in {"Debug", "Display", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"}
        if annotation in FLOAT_TYPES and trait == "Hash":
            return False
        if annotation in FLOAT_TYPES and trait in {"Eq", "Ord"}:
            return False
        if trait == "Sized":
            return not isinstance(annotation, SliceType)
        if trait == TERMINATION:
            if annotation in {"()", NEVER, EXIT_CODE}:
                return True
            if isinstance(annotation, AppliedType) and annotation.name == RESULT:
                result, error = annotation.arguments
                return (self.implements(result, TERMINATION, assumed, seen=seen)
                        and self.implements(error, "Debug", assumed, seen=seen))
        if annotation == EXIT_CODE and trait in {"Copy", "Clone", "Debug", "PartialEq"}:
            return True
        if isinstance(annotation, AppliedType) and trait == "Default" and (
                annotation.name in MAP_TYPES or annotation.name in {DEQUE, HEAP}):
            return True
        if isinstance(annotation, AppliedType) and annotation.name in {RC, WEAK} and trait in TRAIT_MODULES:
            if trait == "Clone":
                return True
            if trait in {"Copy", "Sized"}:
                return trait == "Sized"
            if annotation.name == WEAK:
                return trait == "Debug"
            return self.implements(annotation.arguments[0], trait, assumed, seen=seen)
        if is_box(annotation):
            if trait in {"Clone", "Debug", "Display", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash", *CALLABLE_TRAITS}:
                return self.implements(annotation.arguments[0], trait, assumed, seen=seen)
            if trait in {"Copy", "Default"}:
                return False
        if isinstance(annotation, FunctionType):
            if trait in self.traits and trait not in TRAIT_MODULES:
                return self.implementation(annotation, trait, seen) is not None
            if trait in CALLABLE_TRAITS:
                if not annotation.closure or trait == "FnOnce":
                    return True
                closure = self.functions[annotation.closure]
                if closure.consumes_captures:
                    return False
                return trait == "FnMut" or not any(capture.mutable for capture in closure.captures)
            if annotation.closure:
                captures = self.functions[annotation.closure].captures
                return trait in {"Copy", "Clone"} and all(
                    self.implements(capture.annotation, trait, assumed) if capture.by_move else not capture.mutable
                    for capture in captures
                )
            return trait in {"Copy", "Clone", "PartialEq", "Eq"}
        if trait == INTO_ITERATOR and not isinstance(annotation, (TypeVariable, AssociatedType)) and (
                iteration_item(annotation) is not None
                or (ITERATOR in self.traits and self.implements(annotation, ITERATOR, assumed, seen=seen))):
            return True
        if isinstance(annotation, IteratorType):
            if trait == ITERATOR:
                return True
            source = annotation.source[1] if annotation.source[0] in {"map", "rev"} else annotation.item
            return (trait == "Clone" and self.implements(source, "Clone", assumed)
                    and (annotation.source[0] != "map" or self.implements(annotation.source[3], "Clone", assumed)))
        if isinstance(annotation, AssociatedType):
            normalized = self.normalize(annotation)
            if not isinstance(normalized, AssociatedType):
                return self.implements(normalized, trait, assumed, seen=seen)
            declared = PROJECTION_BOUNDS.get((id(normalized.owner), normalized.item), ())
            if trait in self.trait_closure(declared):
                return True
            declaration = self.projection_trait(normalized)
            return bool(declaration and trait in self.trait_closure(
                declaration.associated[normalized.item].bounds
            ))
        if isinstance(annotation, (TypeParameter, OpaqueType)):
            if annotation in assumed or trait in self.trait_closure(annotation.bounds):
                return True
            return trait in self.traits and self.implementation(annotation, trait, seen) is not None
        if trait in self.traits and trait not in TRAIT_MODULES:
            return self.implementation(annotation, trait, seen) is not None
        if isinstance(annotation, ReferenceType):
            if trait in {"Copy", "Clone"}:
                return not annotation.mutable
            if trait == "Default" and is_str_reference(annotation):
                return not annotation.mutable
            return trait != "Default" and self.implements(annotation.inner, trait, assumed)
        if isinstance(annotation, AppliedType):
            if annotation.name == VEC:
                return trait == "Default" or (
                    trait in {"Clone", "Debug", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"}
                    and self.implements(annotation.arguments[0], trait, assumed)
                )
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
            return trait in {"Debug", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"} and self.implements(
                annotation.element, trait, assumed)
        if annotation == STRING:
            return trait in {"Clone", "Debug", "Display", "Default", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"}
        if annotation in PRIMITIVES:
            return trait != "Display" or annotation != "()"
        return trait in self.derives.get(annotation, {})

    def is_copy(self, annotation):
        return self.implements(annotation, "Copy")

    @staticmethod
    def builtin_iteration_item(annotation):
        return iteration_item(annotation)

    @staticmethod
    def builtin_iterator_type(annotation):
        """The IteratorType that into_iter produces for a built-in iterable."""
        if isinstance(annotation, IteratorType):
            return annotation
        item = iteration_item(annotation)
        if item is None:
            return None
        borrowed = isinstance(annotation, ReferenceType)
        base = annotation.inner if borrowed else annotation
        if borrowed and (isinstance(base, (ArrayType, SliceType)) or is_vector(base)):
            origin = ("slice", annotation.mutable)
        elif isinstance(base, ArrayType):
            origin = ("array", base.length)
        else:
            origin = (base.name, annotation.mutable if borrowed else None)
        return IteratorType(item, origin, borrowed)

    def iteration_item(self, annotation):
        """The item type a for loop yields, through built-in collections or the Iterator traits."""
        item = iteration_item(annotation)
        if item is not None:
            return item
        owner = annotation
        if isinstance(owner, ReferenceType) and owner.mutable and ITERATOR in self.traits:
            owner = owner.inner
        if ITERATOR in self.traits and self.implements(owner, ITERATOR):
            return self.normalize(AssociatedType(owner, "Item", ITERATOR))
        if INTO_ITERATOR in self.traits and self.implements(annotation, INTO_ITERATOR):
            return self.normalize(AssociatedType(annotation, "Item", INTO_ITERATOR))
        return None


class Parser:
    def __init__(self, source, *, tokens=None, fragment=False, modules=None, unit=None, edition="2021"):
        self.edition = modules.edition if modules else edition
        self.tokens = iter(lex(source, edition=self.edition) if tokens is None else tokens)
        self.fragment = fragment
        self.reserved = RESERVED_BY_EDITION[self.edition]
        self.lookahead = deque()
        self.self_type = None
        self.current_trait = None
        self.sized_receiver = False
        self.modules = modules
        self.unit = unit
        self.constants = modules.constants if modules else {}
        self.statics = modules.statics if modules else {}
        self.current = next(self.tokens)
        self.unsafe_depth = 0
        self.type_parameters = {}
        self.argument_parameters = None
        self.lifetime_scope = {}
        self.parameter_defaults = {}
        self.current_trait_parameters = ()
        self.method_outer_parameters = set()
        self.method_where_bounds = []
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
        if is_identifier(token.text) and token.text not in self.reserved:
            token = self.advance()
            return replace(token, text=identifier_text(token.text)) if token.text.startswith("r#") else token
        raise token.error("expected an identifier")

    def lifetime_token(self, token):
        if token.text not in {"'static", "'_"} and token.text[1:] in self.reserved:
            raise token.error("lifetimes cannot use keyword names")
        if token.text.startswith("'r#"):
            return replace(token, text="'" + identifier_text(token.text[1:]))
        return token

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

    def type_name(self, *, static_string=False, signature=None):
        if self.lifetime_scope:
            signature = self.lifetime_scope if signature is None else {**self.lifetime_scope, **signature}
        token = self.advance()
        if token.text == "&&":
            self.lookahead.appendleft(self.current)
            self.current = replace(token, text="&", column=token.column + 1)
            token = replace(token, text="&")
        if token.text == "_":
            return None
        if token.text == "fn":
            outer = self.argument_parameters
            self.argument_parameters = None
            self.expect("(")
            arguments = []
            while self.current.text != ")":
                arguments.append(self.type_name(signature=signature))
                if not self.accept(","):
                    break
            self.expect(")")
            result = self.type_name(signature=signature) if self.accept("->") else "()"
            self.argument_parameters = outer
            if static_string and isinstance(result, ReferenceType) and not is_static_str(result):
                raise token.error("reference-returning function pointer fields are unsupported")
            return FunctionType(tuple(arguments), result)
        if token.text == "impl":
            if self.argument_parameters is None and not self.fragment:
                raise token.error("impl Trait requires a function argument in this position")
            bindings = {}
            bounds = self.trait_bounds(bindings=bindings, signature=signature)
            if self.fragment:
                return TypeParameter("impl Trait", bounds, anonymous=True, associated_bindings=bindings)
            name = f"@argument:{len(self.argument_parameters)}"
            parameter = TypeParameter(name, bounds, anonymous=True, associated_bindings=bindings)
            self.argument_parameters[name] = parameter
            self.type_parameters[name] = parameter
            return parameter
        if token.text == "dyn":
            trait = self.trait_path()
            bindings = []
            if trait in CALLABLE_TRAITS and self.current.text == "(":
                outer = self.argument_parameters
                self.argument_parameters = None
                bindings.append((CALL_ITEM, self.call_signature(signature)))
                self.argument_parameters = outer
            elif self.accept("<"):
                while self.current.text != ">":
                    name = self.identifier()
                    self.expect("=")
                    bindings.append((name.text, self.type_name(signature=signature)))
                    if not self.accept(","):
                        break
                self.expect(">")
            while self.current.text == "+" and (is_lifetime(self.peek()) or self.peek() in {"Send", "Sync", "Unpin"}):
                # Auto traits and lifetime bounds on trait objects do not change behaviour here.
                self.advance()
                self.advance()
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
        if token.text in CONST_VALUE_TYPES or token.text == "str":
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
            lifetime = self.lifetime_token(self.advance()) if is_lifetime(self.current.text) else None
            if lifetime and lifetime.text == "'_":
                lifetime = None
            mutable = bool(self.accept("mut"))
            if static_string and not mutable and self.current.text == "str" and (
                    not lifetime or lifetime.text == "'static"):
                if not lifetime:
                    raise token.error("this position requires &'static str")
                self.advance()
                return STATIC_STR
            if static_string and not lifetime:
                raise token.error("missing lifetime specifier")
            if lifetime and lifetime.text == "'static" and mutable:
                raise lifetime.error("mutable 'static references are unsupported")
            if lifetime and lifetime.text != "'static" and (signature is None or lifetime.text not in signature):
                raise lifetime.error(f"undeclared or unsupported lifetime {lifetime.text}")
            return ReferenceType(self.type_name(signature=signature), mutable, lifetime.text if lifetime else None)
        if token.text == "(":
            if self.accept(")"):
                return "()"
            first = self.type_name(static_string=static_string, signature=signature)
            if not self.accept(","):
                self.expect(")")
                return first
            kinds = [first]
            while self.current.text != ")":
                kinds.append(self.type_name(static_string=static_string, signature=signature))
                if not self.accept(","):
                    break
            self.expect(")")
            return TupleType(tuple(kinds))
        if token.text == "!":
            return NEVER
        if ((is_identifier(token.text) and token.text not in self.reserved)
                or token.text in {"crate", "self", "super"}):
            name = identifier_text(token.text)
            if self.current.text == "::" and name in self.type_parameters:
                self.advance()
                return self.associated_projection(self.type_parameters[name])
            if name in self.type_parameters:
                return self.type_parameters[name]
            parts = [name]
            while self.accept("::"):
                parts.append(self.advance().text if self.current.text == "super" else self.identifier().text)
            name = self.qualified_token(token, parts, type_name=True).text
            is_alias = self.modules and name in self.modules.alias_names
            arguments = self.type_arguments(lifetimes=is_alias) if self.accept("<") else None
            if arguments == () and not is_alias:
                arguments = None
            if is_alias:
                alias = self.modules.type_aliases.get(name)
                if alias is None:
                    raise token.error(f"cyclic or unresolved type alias {name!r}")
                target = alias.specialize(arguments or (), token)
                if isinstance(target, ReferenceType):
                    if static_string and not is_static_str(target):
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
        trait_arguments = ()
        if owner is self.self_type and trait is not None and trait == self.current_trait:
            trait_arguments = tuple(self.current_trait_parameters)
        return AssociatedType(owner, name.text, trait, arguments, trait_arguments)

    def parse(self, program=None):
        program = Program() if program is None else program
        self.current_program = program
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
                    if attribute.text == "allow" and self.current.text == "(" and self.peek() == "overflowing_literals":
                        OVERFLOWING_LITERALS[0] = True
                    self.skip_group()
                else:
                    while self.current.text not in {"]", ""}:
                        if self.current.text in {"(", "[", "{"}:
                            self.skip_group()
                        else:
                            self.advance()
                self.expect("]")
                continue
            if self.accept("use"):
                self.trait_import()
                continue
            if self.current.text == "const" and self.peek() != "fn":
                self.advance()
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
            constant = self.accept("const")
            unsafe = token_position(self.expect("fn")) in UNSAFE_FUNCTIONS
            name = self.declare_token(self.identifier())
            if name.text in program.functions:
                raise name.error(f"duplicate function {name.text!r}")
            self.unsafe_depth += unsafe
            try:
                function = self.function(name, declaration=self.in_std)
            finally:
                self.unsafe_depth -= unsafe
            if constant:
                self.validate_const_function(function)
            program.functions[name.text] = function
        return program

    @property
    def in_std(self):
        return bool(self.unit and self.unit.path[:1] == STD_ROOT)

    def function(self, name, *, method=False, declaration=False):
        outer_parameters = self.type_parameters
        outer_lifetimes = self.lifetime_scope
        lifetimes, parameters = self.generic_parameters()
        lifetimes = {**outer_lifetimes, **lifetimes}
        self.lifetime_scope = lifetimes
        self.type_parameters = outer_parameters | parameters
        self.argument_parameters = parameters
        self.expect("(")
        params = []
        pattern_bindings = []
        receiver = False
        while self.current.text != ")":
            if method and not params and self.is_self_parameter():
                param, mutable, annotation = self.self_parameter(lifetimes)
                receiver = True
            else:
                pattern = self.range_pattern()
                param, mutable = self.parameter_binding(pattern, params, pattern_bindings)
                self.expect(":")
                annotation = self.type_name(signature=lifetimes)
            params.append((param, mutable, annotation))
            if not self.accept(","):
                break
        self.expect(")")
        self.argument_parameters = None
        result = "()"
        if self.accept("->"):
            if opaque := self.accept("impl"):
                if self.current_trait is not None:
                    raise opaque.error("opaque return types in trait methods are unsupported")
                bindings = {}
                bounds = self.trait_bounds(bindings=bindings, signature=lifetimes)
                result = OpaqueType(name.text, frozenset(bounds), tuple(self.type_parameters.values()),
                                    tuple(bindings.items()))
            else:
                result = self.type_name(signature=lifetimes)
        self.sized_receiver = False
        self.method_outer_parameters = set(outer_parameters) - set(parameters) if method else set()
        self.method_where_bounds = []
        self.where_clause(lifetimes)
        self.method_outer_parameters = set()
        where_bounds = tuple(self.method_where_bounds)
        sized_receiver = self.sized_receiver
        if (self.in_std and receiver and isinstance(params[0][2], ReferenceType) and not params[0][2].mutable
                and isinstance(result, ReferenceType) and result.mutable):
            # Interior mutability (RefCell, Cell): a &mut from &self is checked at runtime in Rust, not here.
            return_inputs = ()
        elif receiver and isinstance(params[0][2], ReferenceType) and isinstance(result, ReferenceType) and not result.lifetime:
            return_inputs = (0,)
        else:
            return_inputs = self.return_inputs(name, params, result, lifetimes)
        body = None if declaration and self.accept(";") else self.block()
        if pattern_bindings:
            if body is None:
                raise pattern_bindings[0].token.error("parameter patterns require a function body")
            statements, tail = body.args
            body.args = ([*pattern_bindings, *statements], tail)
        function = Function(name, params, result, body, return_inputs, tuple(self.type_parameters.values()),
                            method_parameters=tuple(parameters.values()) if method else (),
                            lifetimes=lifetimes, self_sized=sized_receiver)
        function.where_bounds = where_bounds
        self.type_parameters = outer_parameters
        self.lifetime_scope = outer_lifetimes
        return function

    def constant_item(self, *, static=False):
        mutable = bool(static and self.accept("mut"))
        name = self.declare_token(self.identifier())
        values = self.statics if static else self.constants
        if name.text in values:
            raise name.error(f"duplicate constant {name.text!r}")
        self.expect(":")
        annotation = self.type_name(static_string=False)
        self.expect("=")
        expression = self.expression()
        if (annotation not in CONST_VALUE_TYPES or mutable) and self.modules is not None:
            # Non-scalar constants are re-evaluated at each use; statics are initialized once on first use.
            self.expect(";")
            self.modules.lazy_items[name.text] = LazyItem(name, "static" if static else "const", annotation,
                                                          expression, mutable)
            return
        if annotation not in CONST_VALUE_TYPES:
            raise name.error("unsupported constant type")
        try:
            value = self.const_expression(expression, annotation)
        except RustError as error:
            if ("unsupported constant expression" not in str(error) or self.modules is None
                    or not self.modules.lazy_fallback):
                raise
            # Initializers like `"hello".len()` run at first use instead of at compile time.
            self.expect(";")
            self.modules.lazy_items[name.text] = LazyItem(name, "static" if static else "const", annotation,
                                                          expression, mutable)
            return
        if not isinstance(value, ConstValue) or (value.kind and value.kind != annotation):
            raise name.error(f"expected a {annotation} constant")
        if ((annotation == "bool") != value.boolean
                or (annotation == "char") != isinstance(value.value, Character)
                or (annotation in FLOAT_TYPES) != isinstance(value.value, float)):
            raise name.error(f"expected a {annotation} constant")
        if annotation in INTEGER_TYPES:
            checked(value.value, name, annotation)
        self.expect(";")
        values[name.text] = ConstValue(value.value, annotation)

    def type_alias(self):
        name = self.declare_token(self.identifier())
        if name.text in PRIMITIVES:
            raise name.error(f"duplicate type {name.text!r}")
        lifetimes, parameters = self.generic_parameters()
        self.type_parameters = parameters
        self.where_clause(lifetimes)
        self.expect("=")
        outer_lifetimes = self.lifetime_scope
        self.lifetime_scope = {**outer_lifetimes, **lifetimes}
        try:
            target = self.type_name(signature=lifetimes)
        finally:
            self.lifetime_scope = outer_lifetimes
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
        offset = int(is_lifetime(self.peek()))
        if self.peek(offset) == "mut":
            offset += 1
        return self.peek(offset) == "self"

    def self_parameter(self, lifetimes):
        borrowed = self.accept("&")
        lifetime = self.lifetime_token(self.advance()) if borrowed and is_lifetime(self.current.text) else None
        if lifetime and lifetime.text != "'static" and lifetime.text not in lifetimes:
            raise lifetime.error("undeclared receiver lifetime")
        mutable = bool(self.accept("mut"))
        name = self.expect("self")
        annotation = self.self_type
        if self.accept(":"):
            if borrowed:
                raise name.error("an explicitly typed receiver cannot also use '&self' syntax")
            annotation = self.type_name(signature=lifetimes)
            allowed = (annotation == self.self_type
                       or isinstance(annotation, ReferenceType) and annotation.inner == self.self_type
                       or is_box(annotation) and annotation.arguments[0] == self.self_type)
            if not allowed:
                raise name.error("receiver type must be Self, a reference to Self, or Box<Self>")
        elif borrowed:
            annotation = ReferenceType(annotation, mutable, lifetime.text if lifetime else None)
        return name, mutable and not borrowed, annotation

    def trait_item(self, program):
        name = self.declare_token(self.identifier())
        name = replace(name, text=STD_BUILTIN_TRAITS.get(name.text, name.text))
        if name.text in program.traits or (name.text in TRAIT_MODULES and not self.in_std):
            raise name.error(f"duplicate trait {name.text!r}")
        self_type = TypeParameter("Self", {name.text})
        old_self = self.self_type
        self.self_type = self_type
        self.type_parameters = {"Self": self_type}
        self.parameter_defaults = {}
        trait_lifetimes, trait_parameters = self.generic_parameters()
        outer_trait_lifetimes = self.lifetime_scope
        self.lifetime_scope = {**outer_trait_lifetimes, **trait_lifetimes}
        defaults = tuple(self.parameter_defaults.get(parameter) for parameter in trait_parameters)
        self.type_parameters = {"Self": self_type, **trait_parameters}
        bounds = self.trait_bounds(bindings={}) if self.accept(":") else set()
        self_type.bounds.update(bounds)
        self.where_clause({})
        self.current_trait = name.text
        self.current_trait_parameters = tuple(trait_parameters.values())
        self.expect("{")
        methods, associated = self.methods(program, declaration=True, prefix=f"@trait:{name.text}")
        self.current_trait_parameters = ()
        self.lifetime_scope = outer_trait_lifetimes
        self.self_type = old_self
        self.current_trait = None
        self.type_parameters = {}
        program.traits[name.text] = Trait(name, self_type, bounds, methods, associated,
                                          tuple(trait_parameters.values()), defaults)

    def impl_item(self, program, token):
        lifetimes, parameters = self.generic_parameters()
        outer_lifetimes = self.lifetime_scope
        self.lifetime_scope = {**outer_lifetimes, **lifetimes}
        self.type_parameters = parameters
        target = self.type_name()
        trait = None
        trait_arguments = ()
        if self.accept("for"):
            if isinstance(target, AppliedType):
                trait, trait_arguments = target.name, target.arguments
            elif isinstance(target, str):
                trait = target
            else:
                raise token.error("expected a trait name in impl")
            target = self.type_name()
        self.where_clause(lifetimes)
        self.expect("{")
        old_self = self.self_type
        self.self_type = target
        self.current_trait = trait
        methods, associated = self.methods(program, declaration=False, prefix=f"@impl:{len(program.impls)}")
        self.self_type = old_self
        self.current_trait = None
        program.impls.append(Implementation(token, target, trait, tuple(parameters.values()), methods, associated,
                                            trait_arguments))
        self.type_parameters = {}
        self.lifetime_scope = outer_lifetimes

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
                    if annotation in CONST_VALUE_TYPES:
                        try:
                            self.const_expression(body)
                        except RustError:
                            pass
                elif not declaration:
                    raise name.error("associated constant needs a value")
                self.expect(";")
                function = Function(internal, [], annotation, body,
                                    type_parameters=tuple(self.type_parameters.values()), associated_const=True)
            else:
                function = self.function(internal, method=True, declaration=declaration or self.in_std)
            function.visible_from = visible
            methods[name.text] = function
            if function.body is not None or (self.in_std and not declaration):
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
        outer = self.type_parameters
        self.type_parameters = dict(outer)
        while self.current.text != ">":
            token = self.advance()
            if is_lifetime(token.text) and token.text not in {"'static", "'_"}:
                token = self.lifetime_token(token)
                if token.text in {"'static", "'_"}:
                    raise token.error(f"reserved lifetime name {token.text}")
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
                self.type_parameters[name.text] = types[name.text]
            elif is_identifier(token.text) and token.text not in self.reserved:
                name = identifier_text(token.text)
                if name in types:
                    raise token.error(f"duplicate type parameter {token.text}")
                parameter = TypeParameter(name)
                types[name] = self.type_parameters[name] = parameter
                if self.accept(":"):
                    parameter.bounds.update(self.trait_bounds(bindings=parameter.associated_bindings,
                                                            signature=lifetimes))
                if self.accept("="):
                    self.parameter_defaults[name] = self.type_name()
            else:
                raise token.error("expected a lifetime or type parameter")
            if not self.accept(","):
                break
        self.expect(">")
        self.type_parameters = outer
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

    def trait_bounds(self, *, bindings=None, signature=None, outer_lifetimes=()):
        bounds = set()
        outer_signature = (signature or {}) | {lifetime.name: set() for lifetime in outer_lifetimes}
        while True:
            if self.accept("?"):
                self.expect("Sized")
                bounds.add("?Sized")
                if not self.accept("+"):
                    break
                continue
            if is_lifetime(self.current.text):
                self.advance()
                if not self.accept("+"):
                    break
                continue
            local_lifetimes = self.bound_lifetimes(outer_signature) if self.accept("for") else ()
            lifetimes = (*outer_lifetimes, *local_lifetimes)
            bound_signature = (signature or {}) | {lifetime.name: set() for lifetime in lifetimes}
            bound = self.trait_path()
            bounds.add(bound)
            if bound in CALLABLE_TRAITS and self.current.text == "(":
                signature_type = self.call_signature(bound_signature)
                if bindings is not None:
                    bindings[AssociatedBinding(bound, CALL_ITEM)] = signature_type
                if not self.accept("+"):
                    break
                continue
            if lifetimes and bindings is None:
                raise self.current.error("higher-ranked bounds require an associated type constraint here")
            if bindings is not None and self.accept("<"):
                positional = []
                while self.current.text != ">":
                    if not (is_identifier(self.current.text) and self.peek() == "=") and not (
                            is_identifier(self.current.text) and self.peek() == "<" and self.generic_item_binding()):
                        positional.append(self.type_name(signature=bound_signature))
                        if not self.accept(","):
                            break
                        continue
                    name = self.identifier()
                    arguments = self.type_arguments(lifetimes=True) if self.accept("<") else ()
                    for argument in arguments:
                        if isinstance(argument, LifetimeArgument) and argument.name not in {"'static", "'_"} and (
                            argument.name not in bound_signature
                        ):
                            raise name.error(f"undeclared lifetime {argument.name}")
                    self.expect("=")
                    key = AssociatedBinding(bound, name.text, arguments, lifetimes)
                    if key in bindings:
                        raise name.error(f"duplicate associated type binding {name.text!r}")
                    bindings[key] = self.type_name(signature=bound_signature)
                    if not self.accept(","):
                        break
                self.expect(">")
                if positional:
                    bindings[AssociatedBinding(bound, ARGS_ITEM)] = TupleType(tuple(positional))
            if not self.accept("+"):
                break
        return bounds

    def generic_item_binding(self):
        """Whether `Name<...> =` follows, which binds a generic associated type rather than naming a type."""
        depth = 0
        index = 0
        while True:
            text = self.peek(index)
            if text == "<":
                depth += 1
            elif text in {">", ">>"}:
                depth -= len(text)
                if depth <= 0:
                    return self.peek(index + 1) == "="
            elif text in {"", ";", "{"}:
                return False
            index += 1

    def call_signature(self, signature):
        self.expect("(")
        arguments = []
        while self.current.text != ")":
            arguments.append(self.type_name(signature=signature))
            if not self.accept(","):
                break
        self.expect(")")
        result = self.type_name(signature=signature) if self.accept("->") else "()"
        return FunctionType(tuple(arguments), result)

    def bound_lifetimes(self, signature):
        self.expect("<")
        lifetimes = {}
        while self.current.text != ">":
            token = self.advance()
            if not is_lifetime(token.text):
                raise token.error("expected a bound lifetime parameter")
            token = self.lifetime_token(token)
            if token.text in {"'static", "'_"}:
                raise token.error("expected a bound lifetime parameter")
            if token.text in lifetimes or token.text in (signature or {}):
                raise token.error(f"duplicate or shadowed lifetime {token.text}")
            lifetimes[token.text] = LifetimeArgument(token.text)
            if not self.accept(","):
                break
        self.expect(">")
        return tuple(lifetimes.values())

    def lifetime_bounds(self):
        bounds = set()
        while True:
            token = self.advance()
            if not is_lifetime(token.text):
                raise token.error("expected a lifetime bound")
            token = self.lifetime_token(token)
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
            quantified = self.bound_lifetimes(lifetimes) if self.accept("for") else ()
            if self.current.text == "<" or (self.peek() == "::" and self.current.text != "Self") or (
                    self.current.text == "Self" and self.peek() == "::") or (
                    self.peek() in {"<", "("} and identifier_text(self.current.text) not in self.type_parameters) or (
                    self.current.text in {"&", "[", "("}):
                # A bound on a projection, like `C::Item: Into<u64>`.
                projection = self.type_name(signature=lifetimes)
                self.expect(":")
                bounds = self.trait_bounds(bindings={}, signature=lifetimes)
                if isinstance(projection, AssociatedType):
                    PROJECTION_BOUNDS.setdefault((id(projection.owner), projection.item), set()).update(bounds)
                # Bounds on other types, like `Option<T>: Debug`, are assumed rather than checked.
                for bound in bounds:
                    TYPE_BOUNDS.add((projection, bound))
                if not self.accept(","):
                    break
                continue
            lhs = self.advance()
            parameter_name = identifier_text(lhs.text)
            self.expect(":")
            if quantified and (lhs.text == "Self" or parameter_name not in self.type_parameters):
                raise lhs.error("a higher-ranked where bound requires a type parameter")
            if is_lifetime(lhs.text):
                lhs = self.lifetime_token(lhs)
                self.require_lifetimes((lhs,), lifetimes)
                bounds = self.lifetime_bounds()
                self.require_lifetimes(bounds, lifetimes)
                lifetimes[lhs.text].update(bounds)
            elif lhs.text == "Self" and self.self_type is not None:
                if self.trait_bounds() != {"Sized"}:
                    raise lhs.error("method Self bounds currently support Sized")
                self.sized_receiver = True
            elif parameter_name in self.type_parameters:
                parameter = self.type_parameters[parameter_name]
                if parameter_name in self.method_outer_parameters:
                    # A method's where clause on an impl parameter only holds inside that method.
                    bindings = {}
                    bounds = self.trait_bounds(bindings=bindings, signature=lifetimes, outer_lifetimes=quantified)
                    self.method_where_bounds.append((parameter, bounds, bindings))
                else:
                    parameter.bounds.update(self.trait_bounds(bindings=parameter.associated_bindings,
                                                              signature=lifetimes, outer_lifetimes=quantified))
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
        if not inputs and result.lifetime not in lifetimes:
            raise name.error("return lifetime must be tied to a reference input")
        # A lifetime from an enclosing struct or impl ties the result to no tracked input.
        return tuple(inputs)

    def attributes(self):
        derives = {}
        while self.current.text == "#" and self.peek() == "[":
            self.expect("#")
            self.expect("[")
            if self.current.text != "derive":
                # repr, allow, inline, must_use, doc, and similar attributes do not change behaviour here.
                depth = 1
                while depth:
                    token = self.advance()
                    depth += {"[": 1, "]": -1}.get(token.text, 0)
                continue
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
        program.type_lifetimes[name.text] = tuple(lifetimes)
        outer_lifetimes = self.lifetime_scope
        self.lifetime_scope = {**outer_lifetimes, **lifetimes}
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
                if self.accept("="):
                    discriminant = self.const_expression(self.expression(), "i128")
                    program.discriminants[ctor.path] = discriminant.value
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
        self.lifetime_scope = outer_lifetimes
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
            if self.current.text == "#" and self.peek() == "[":
                self.attributes()
                continue
            let = self.accept("let")
            if let:
                pattern = self.pattern()
                annotation = None
                if self.accept(":") and not self.accept("_"):
                    annotation = self.type_name()
                value = self.expression() if self.accept("=") else None
                otherwise = self.block() if value and self.accept("else") else None
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
                if index is not None and index.kind == "range_value" and index.args[0] is not None:
                    start, stop, inclusive = index.args
                    left = Expr("range_index", opening, (left, start, stop, inclusive))
                    self.expect("]")
                    continue
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
            if re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", name.text):
                first, second = name.text.split(".")
                left = Expr("field", dot, (left, replace(name, text=first)))
                dot = replace(name, text=".", column=name.column + len(first))
                name = replace(name, text=second, column=dot.column + 1)
            if not (is_identifier(name.text) or re.fullmatch(r'0|[1-9][0-9]*', name.text)):
                raise name.error("expected a field name or tuple field index")
            name = replace(name, text=identifier_text(name.text)) if name.text.startswith("r#") else name
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
        while minimum <= PRECEDENCE["*"] + 1 and (cast := self.accept("as")):
            left = Expr("cast", cast, (left, self.type_name()))
        compared = False
        while True:
            if self.current.text in {"..", "..="} and minimum <= 2:
                token = self.advance()
                end = self.range_end(token, allow_record)
                left = Expr("range_value", token, (left, end, token.text == "..="))
                break
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

    def prefix_borrow(self, token, allow_record):
        mutable = bool(self.accept("mut"))
        operand = self.expression(PRECEDENCE["*"] + 2, allow_record=allow_record)
        return Expr("borrow", replace(token, text="&"), (operand, mutable))

    def range_end(self, token, allow_record):
        if self.current.text in {")", "]", "}", ",", ";", "=>", ""} or (
                self.current.text == "{" and not allow_record):
            if token.text == "..=":
                raise token.error("inclusive range requires an end")
            return None
        return self.expression(3, allow_record=allow_record)

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
        if self.current.text == "unsafe" and self.peek() == "{":
            self.advance()
            self.unsafe_depth += 1
            try:
                return self.block()
            finally:
                self.unsafe_depth -= 1
        token = self.advance()
        word = token.text
        if word.startswith("@host:"):
            word = word.removeprefix("@host:")
            token = replace(token, text=word)
        if word == ".." and self.current.text not in {")", "]", "}", ",", ";", "=>", "="}:
            return Expr("range_value", token, (None, self.range_end(token, allow_record), False))
        if word == "..=":
            return Expr("range_value", token, (None, self.range_end(token, allow_record), True))
        if word in {"_", ".."}:
            return Expr("wild" if word == "_" else "rest", token)
        if word == "const" and self.current.text == "{":
            return Expr("inline_const", token, (self.block(),))
        if word == "@vec":
            close = DELIMITERS[self.advance().text]
            if self.accept(close):
                return Expr("vector", token)
            first = self.expression()
            if self.accept(";"):
                count = self.expression()
                self.expect(close)
                return Expr("vector_repeat", token, (first, count))
            elements = [first]
            while self.accept(",") and self.current.text != close:
                elements.append(self.expression())
            self.expect(close)
            return Expr("vector", token, tuple(elements))
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
                    return Expr("record", path, self.record_fields(self.expression, "name"),
                                type_arguments=arguments)
                return Expr("name", path, type_arguments=arguments)
            self.expect("::")
            member = self.identifier()
            if self.current.text == "(":
                return Expr("call", member, self.arguments(), qualified=(self.self_type, self.current_trait))
            return Expr("name", member, qualified=(self.self_type, self.current_trait))
        if is_lifetime(word):
            token = self.lifetime_token(token)
            if token.text in {"'static", "'_"}:
                raise token.error("invalid label")
            self.expect(":")
            if self.current.text not in {"{", "loop", "while", "for"}:
                raise token.error("a label must precede a block or loop")
            labeled = self.prefix(allow_record)
            labeled.label = token
            return labeled
        if word == "if":
            conditions = self.condition()
            yes = self.block()
            no = None
            if self.accept("else"):
                no = self.prefix() if self.current.text == "if" else self.block()
            if len(conditions) > 1:
                return Expr("if_chain", token, (conditions, yes, no))
            pattern, condition = conditions[0]
            if pattern:
                return self.conditional_match(token, condition, pattern, yes, no)
            return Expr("if", token, (condition, yes, no))
        if word == "while":
            conditions = self.condition()
            body = self.block()
            if len(conditions) > 1:
                return Expr("while_chain", token, (conditions, body))
            pattern, condition = conditions[0]
            if pattern:
                return Expr("while_let", token, (condition, pattern, body))
            return Expr("while", token, (condition, body))
        if word == "loop":
            return Expr("loop", token, (self.block(),))
        if word == "for":
            pattern = self.pattern()
            self.expect("in")
            iterable = self.expression(allow_record=False)
            if iterable.kind == "range_value" and None not in iterable.args[:2]:
                start, stop, inclusive = iterable.args
                iterable = Expr("range", start.token, (start, stop, inclusive))
            inclusive = bool(self.accept("..="))
            if inclusive or self.accept(".."):
                stop = self.expression(allow_record=False)
                iterable = Expr("range", iterable.token, (iterable, stop, inclusive))
            return Expr("for", token, (pattern, iterable, self.block()))
        if word == "match":
            return self.match_expression(token)
        if word in {"return", "break", "continue"}:
            label = None
            if word != "return" and is_lifetime(self.current.text):
                label = self.lifetime_token(self.advance())
            value = None
            if word != "continue" and self.current.text not in {";", "}", ")", "]", ",", "=>", "else"}:
                value = self.expression(allow_record=allow_record)
            return Expr(word, token, (value,), label)
        if word in {"-", "!"}:
            return Expr("unary", token, (self.expression(PRECEDENCE["*"] + 2, allow_record=allow_record),))
        if word == "&&":
            inner = self.prefix_borrow(token, allow_record)
            return Expr("borrow", token, (inner, False))
        if word in {"&", "*"}:
            mutable = bool(self.accept("mut")) if word == "&" else False
            operand = self.expression(PRECEDENCE["*"] + 2, allow_record=allow_record)
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
        if (parameter := self.type_parameters.get(identifier_text(word))) and parameter.const_type:
            return Expr("const_value", token, (parameter,))
        if word in self.constants:
            constant = self.constants[word]
            return Expr("literal", token, (constant.kind, constant.value))
        literal = self.literal(token)
        if literal:
            return literal
        if word == "self" and self.current.text != "::" and (self.self_type is not None or self.fragment):
            return Expr("name", token)
        if (is_identifier(word) and word not in self.reserved) or word in {"crate", "self", "super"}:
            if self.accept("!"):
                word = identifier_text(word)
                token = replace(token, text=word)
                opening = self.current.text
                if opening not in DELIMITERS:
                    raise self.current.error("expected a macro delimiter")
                if word in {"panic", "unreachable", "unimplemented", "todo"}:
                    return Expr("panic", token, self.arguments(opening))
                if word in {"println", "print", "eprintln", "eprint", "format"}:
                    return Expr("format" if word == "format" else "print", token, self.arguments(opening))
                if not self.fragment:
                    raise token.error(f"unknown macro {word!r}")
                self.skip_group()
                return Expr("literal", token, ("i32", 0))
            local = token if self.current.text != "::" else None
            path, type_arguments = self.expression_path(token, allow_record=allow_record)
            if path.text in SCALAR_CONSTANTS:
                return Expr("literal", path, SCALAR_CONSTANTS[path.text])
            if path.text in {f"{EXIT_CODE}::SUCCESS", f"{EXIT_CODE}::FAILURE"}:
                status = int(path.text.endswith("::FAILURE"))
                return Expr("literal", path, (EXIT_CODE, ExitCodeValue(status)))
            if path.text in self.constants:
                constant = self.constants[path.text]
                return Expr("literal", path, (constant.kind, constant.value))
            if path.text in self.statics:
                return Expr("static_item", path, (self.statics[path.text],))
            if self.modules is not None and path.text in self.modules.lazy_items:
                item = self.modules.lazy_items[path.text]
                if item.kind == "static" and item.mutable and not self.unsafe_depth and not self.in_std:
                    raise path.error("use of mutable static is unsafe and requires an unsafe block or function")
                return Expr("lazy_" + item.kind, path, (item,))
            if allow_record and self.current.text == "{":
                return Expr("record", path, self.record_fields(self.expression, "name"),
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
        pattern_bindings = []
        if opening == "|":
            while self.current.text != "|":
                pattern = self.range_pattern()
                name, mutable = self.parameter_binding(pattern, params, pattern_bindings)
                annotation = self.type_name() if self.accept(":") else None
                params.append((name, mutable, annotation))
                if not self.accept(","):
                    break
            self.expect("|")
        result = self.type_name() if self.accept("->") else None
        body = self.block() if result is not None else self.expression()
        if pattern_bindings:
            body = Expr("block", body.token, (pattern_bindings, body))
        return Expr("closure", token, (params, result, body, token.text == "move"))

    def parameter_binding(self, pattern, params, pattern_bindings):
        if pattern.kind == "wild":
            return pattern.token, False
        if pattern.kind == "bind" and len(pattern.args) == 1:
            symbol = self.modules.resolve((pattern.token.text,), self.unit, pattern.token) if self.unit else None
            if not symbol or symbol.kind != "unit_struct":
                return pattern.token, pattern.args[0]
        hidden = replace(pattern.token, text=f"@param:{len(params)}")
        initializer = Expr("name", hidden)
        pattern_bindings.append(Expr("parameter_let", pattern.token, (pattern, None, initializer, None)))
        return hidden, True

    @staticmethod
    def literal(token, negative=False):
        word = token.text
        if FLOAT_LITERAL.fullmatch(word):
            suffix = FLOAT_SUFFIX.search(word)
            kind = suffix.group() if suffix else "f64"
            digits = (word[:suffix.start()] if suffix else word).replace("_", "")
            value = float(digits)
            return Expr("literal", token, (kind, -value if negative else value), unsuffixed=suffix is None)
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
        if is_string_literal(word):
            return Expr("literal", token, (STATIC_STR, decode_string(token)))
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
        parts = [identifier_text(token.text)]
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
            parameter = self.type_parameters.get(identifier_text(word))
            if self.accept("_"):
                arguments.append(None)
            elif is_lifetime(word):
                lifetime = LifetimeArgument(self.lifetime_token(self.advance()).text)
                if lifetimes:
                    arguments.append(lifetime)
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
        if self.constants and is_identifier(token.text) and self.peek() != "::":
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
        if not negative and self.constants and is_identifier(token.text):
            path = self.qualified_token(token, (token.text,), bare=True)
            if path.text in self.constants:
                return self.constants[path.text]
        parameter = self.type_parameters.get(identifier_text(token.text))
        if parameter and parameter.const_type and not negative:
            return parameter
        literal = self.literal(token, negative=bool(negative))
        if literal is None or literal.args[0] not in CONST_TYPES:
            raise token.error("expected a constant value or parameter")
        kind, value = literal.args
        return ConstValue(value, None if literal.unsuffixed else kind)

    def validate_const_function(self, function):
        if function.name.text == "main":
            raise function.name.error("main cannot be const")
        if (function.type_parameters or function.result not in CONST_VALUE_TYPES
                or any(kind not in CONST_VALUE_TYPES or name.text.startswith("@param:")
                       for name, _, kind in function.params)):
            raise function.name.error("const functions currently require scalar parameters and a scalar result")
        arguments = tuple(TypeParameter(name.text, const_type=kind) for name, _, kind in function.params)
        self.const_function_body(function, arguments)

    def const_function_body(self, function, arguments, *, defer_const=False):
        token = function.name
        if function.result not in CONST_VALUE_TYPES:
            raise token.error("const functions currently require a scalar result")
        if len(arguments) != len(function.params):
            raise token.error(f"expected {len(function.params)} const function arguments")
        active = self.modules.active_const_functions
        if token.text in active:
            raise token.error("recursive const functions are unsupported")
        locals = {}
        for (name, _, kind), argument in zip(function.params, arguments):
            check_constant_kind(argument, kind, name)
            if name.text != "_":
                locals[name.key] = replace(argument, kind=kind) if isinstance(argument, ConstValue) else argument
        active.add(token.text)
        try:
            value = self.const_expression(function.body, function.result, locals=locals, defer_const=defer_const)
            check_constant_kind(value, function.result, token)
            effects = tuple(arg for arg in arguments if not isinstance(arg, ConstValue))
            if effects:
                return ConstExpression("block", (*effects, value), function.result, token)
            return replace(value, kind=function.result) if isinstance(value, ConstValue) else value
        finally:
            active.remove(token.text)

    def const_expression(self, expr, hint=None, *, locals=None, defer_const=False):
        expr = ungroup(expr)
        read = functools.partial(self.const_expression, locals=locals, defer_const=defer_const)
        if expr.kind == "inline_const":
            return read(expr.args[0], hint, locals=None)
        if expr.kind == "name":
            if locals is not None and expr.token.key in locals:
                return locals[expr.token.key]
            if expr.token.text in self.constants:
                return self.constants[expr.token.text]
            if expr.token.text in self.statics:
                return self.statics[expr.token.text]
            owner, separator, member = expr.token.text.rpartition("::")
            program = getattr(self, "current_program", None)
            if separator and program is not None:
                # `Config::MAX_USERS` reads an associated constant of an inherent impl.
                for impl in program.impls:
                    target = impl.target.name if isinstance(impl.target, AppliedType) else impl.target
                    method = impl.methods.get(member)
                    if impl.trait is None and target == owner and method is not None and method.associated_const:
                        return read(method.body, method.result)
        if expr.kind == "call" and expr.qualified is None and expr.type_arguments is None:
            if self.modules is None or expr.token.text not in self.modules.const_functions:
                raise expr.token.error("constant evaluation requires a const function")
            parser, function = self.modules.read_const_function(expr.token.text)
            if (len(expr.args) != len(function.params) or function.type_parameters
                    or any(kind not in CONST_VALUE_TYPES for _, _, kind in function.params)):
                raise expr.token.error("incompatible const function arguments")
            arguments = tuple(read(arg, kind)
                              for arg, (_, _, kind) in zip(expr.args, function.params))
            return parser.const_function_body(function, arguments, defer_const=defer_const)
        if expr.kind == "block":
            statements, tail = expr.args
            if tail is None:
                raise expr.token.error("constant block requires a result")
            scope = {} if locals is None else dict(locals)
            read = functools.partial(self.const_expression, locals=scope, defer_const=defer_const)
            effects = []
            for statement in statements:
                if statement.kind == "macro_definition":
                    continue
                if statement.kind == "discard" and not statement.args[1]:
                    effects.append(read(statement.args[0]))
                    continue
                if statement.kind != "let":
                    raise statement.token.error("unsupported statement in a constant block")
                pattern, annotation, initializer, fallback = statement.args
                if (pattern.kind not in {"bind", "wild"} or pattern.args not in {(), (False,)}
                        or initializer is None or fallback or annotation not in CONST_VALUE_TYPES | {None}):
                    raise statement.token.error("constant bindings require an immutable scalar value")
                value = read(initializer, annotation)
                kind = annotation or const_kind(value)
                if kind is None and isinstance(value, ConstValue):
                    kind = "f64" if isinstance(value.value, float) else "i32"
                check_constant_kind(value, kind, statement.token)
                if isinstance(value, ConstValue):
                    value = replace(value, kind=kind)
                effects.append(value)
                if pattern.kind == "bind":
                    scope[pattern.token.key] = value
            value = read(tail, hint)
            effects = tuple(effect for effect in effects if not isinstance(effect, ConstValue))
            if effects:
                kind = const_kind(value) or hint or "i32"
                check_constant_kind(value, kind, tail.token)
                return ConstExpression("block", (*effects, value), kind, expr.token)
            return value
        if expr.kind == "if":
            test, yes, no = expr.args
            if no is None:
                raise expr.token.error("scalar constant conditionals require an else branch")
            condition = read(test, "bool")
            check_constant_kind(condition, "bool", test.token)
            left = read(yes, hint, defer_const=True)
            right = read(no, hint or const_kind(left), defer_const=True)
            kind = const_kind(left) or const_kind(right) or hint
            if kind is None:
                kind = "f64" if isinstance(left.value, float) else "i32"
            check_constant_kind(left, kind, yes.token)
            check_constant_kind(right, kind, no.token)
            choice = ConstExpression("if", (condition, left, right), kind, expr.token)
            return substitute(choice, {}, defer_const=defer_const)
        if expr.kind in {"const_value", "static_item"}:
            return expr.args[0]
        if expr.kind == "literal" and expr.args[0] in CONST_VALUE_TYPES:
            kind, value = expr.args
            if kind in FLOAT_TYPES:
                target = hint if expr.unsuffixed and hint in FLOAT_TYPES else kind
                value = float_literal_value(value, target, expr.token)
            return ConstValue(value, None if expr.unsuffixed else kind)
        if expr.kind == "cast":
            operand, target = expr.args
            if target not in SCALAR_BOUNDS and target not in FLOAT_TYPES:
                raise expr.token.error("constant cast requires a numeric or char target")
            preferred = "u8" if target == "char" else target
            value = read(operand, preferred)
            actual = const_kind(value)
            if actual is None and isinstance(value, ConstValue) and isinstance(value.value, float):
                actual = "f32" if isinstance(value.value, Float32) else "f64"
            actual = actual or (preferred if preferred in INTEGER_TYPES else "i32")
            if not scalar_cast_supported(actual, target):
                raise expr.token.error(f"cannot cast {actual} to {target}")
            return fold_const(ConstExpression("as", (value,), target, expr.token, actual),
                              defer_const=defer_const)
        if expr.kind == "unary" and expr.token.text == "!":
            value = read(expr.args[0], hint)
            kind = const_kind(value) or hint or "i32"
            if kind != "bool" and kind not in INTEGER_TYPES:
                raise expr.token.error(f"cannot negate {kind} with '!'")
            return fold_const(ConstExpression("!", (value,), kind, expr.token, kind),
                              defer_const=defer_const)
        if expr.kind == "unary" and expr.token.text == "-":
            operand = expr.args[0]
            value = read(operand, hint)
            kind = const_kind(value) or hint
            if kind in INTEGER_TYPES and INTEGER_BOUNDS[kind][0] == 0:
                raise expr.token.error(f"cannot negate {kind} with '-'")
            signed_literal = ungroup(operand).kind == "literal"
            if isinstance(value, ConstValue) and type(value.value) is int and (not defer_const or signed_literal):
                result = -value.value
                if kind in INTEGER_TYPES:
                    checked(result, expr.token, kind)
                return ConstValue(result, value.kind)
            if isinstance(value, ConstValue) and isinstance(value.value, float) and not defer_const:
                width = kind if kind in FLOAT_TYPES else "f32" if isinstance(value.value, Float32) else "f64"
                return ConstValue(rounded_float(-value.value, width), value.kind)
            if kind is None and isinstance(value, ConstValue):
                kind = "f64" if isinstance(value.value, float) else "i32"
            if kind in FLOAT_TYPES or kind in INTEGER_TYPES and kind.startswith("i"):
                check_constant_kind(value, kind, expr.token)
                return ConstExpression("-", (value,), kind, expr.token)
        if expr.kind == "binary":
            word = expr.token.text
            if word in {"&&", "||"}:
                left = read(expr.args[0], "bool")
                right = read(expr.args[1], "bool", defer_const=True)
                check_constant_kind(left, "bool", expr.args[0].token)
                check_constant_kind(right, "bool", expr.args[1].token)
                branch = ConstExpression(word, (left, right), "bool", expr.token)
                return substitute(branch, {}, defer_const=defer_const)
            left, right = (read(arg, hint) for arg in expr.args)
            left_kind, right_kind = const_kind(left), const_kind(right)
            if word in SHIFTS:
                kind = left_kind or (hint if hint in INTEGER_TYPES else "i32")
                if kind not in INTEGER_TYPES or (right_kind and right_kind not in INTEGER_TYPES):
                    raise expr.token.error("shift operands must be integers")
                return fold_const(ConstExpression(word, (left, right), kind, expr.token, kind),
                                  defer_const=defer_const)
            if defer_const or not all(isinstance(arg, ConstValue) for arg in (left, right)):
                kind = left_kind or right_kind
                if kind is None:
                    floating = isinstance(left, ConstValue) and isinstance(left.value, float)
                    kind = hint if hint in NUMBER_TYPES else "f64" if floating else "i32"
                if left_kind and right_kind and left_kind != right_kind:
                    raise expr.token.error("constant operands have different types")
                if kind == "char" and (left_kind != kind or right_kind != kind):
                    raise expr.token.error("character comparison requires two characters")
                check_constant_kind(left, kind, expr.args[0].token)
                check_constant_kind(right, kind, expr.args[1].token)
                if word in COMPARE or (kind == "bool" and word in BITWISE):
                    return ConstExpression(word, (left, right), "bool", expr.token, kind)
                if ((kind in INTEGER_TYPES and word in {*ARITHMETIC, "/", "%"})
                        or (kind in FLOAT_TYPES and word in FLOAT_OPERATORS)):
                    return ConstExpression(word, (left, right), kind, expr.token, kind)
            if all(isinstance(arg, ConstValue) for arg in (left, right)):
                if left.boolean and right.boolean and word in BITWISE:
                    return ConstValue(BITWISE[word](left.value, right.value), "bool")
                if word in COMPARE and left.boolean == right.boolean and (
                    not left.kind or not right.kind or left.kind == right.kind
                ):
                    if isinstance(left.value, Character) != isinstance(right.value, Character):
                        raise expr.token.error("character comparison requires two characters")
                    if isinstance(left.value, float) != isinstance(right.value, float):
                        raise expr.token.error("float comparison requires two floats")
                    if type(left.value) is int:
                        kind = left.kind or right.kind or (hint if hint in INTEGER_TYPES else "i32")
                        checked(left.value, expr.token, kind)
                        checked(right.value, expr.token, kind)
                    return ConstValue(COMPARE[word](left.value, right.value), "bool")
                kind = left.kind or right.kind
                if (isinstance(left.value, float) and isinstance(right.value, float) and word in FLOAT_OPERATORS
                        and (not left.kind or not right.kind or left.kind == right.kind)):
                    width = kind or (hint if hint in FLOAT_TYPES else "f64")
                    return ConstValue(float_binary(word, left.value, right.value, width), kind)
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
            if self.accept(".."):
                rest = True if shorthand == "bind" or self.current.text == "}" else parse_value()
                break
            by_reference = shorthand == "bind" and bool(self.accept("ref"))
            mutable = shorthand == "bind" and bool(self.accept("mut"))
            name = self.identifier()
            if self.accept(":"):
                if mutable or by_reference:
                    raise name.error("put the binding modifier after ':'")
                value = parse_value()
            else:
                args = (False, mutable) if by_reference else (mutable,) if shorthand == "bind" else ()
                value = Expr(shorthand, name, args)
            fields.append((name, value))
            if not self.accept(","):
                break
        self.expect("}")
        return tuple(fields), rest

    def pattern(self, *, array_item=False):
        self.accept("|")
        first = self.range_pattern(array_item=array_item)
        alternatives = [first]
        while self.accept("|"):
            alternatives.append(self.range_pattern(array_item=array_item))
        if len(alternatives) > 1:
            return Expr("or", first.token, tuple(alternatives))
        return first

    def range_pattern(self, *, array_item=False):
        first = None if self.current.text in {"..", "..="} else self.simple_pattern()
        if first is not None and first.kind == "bind" and self.accept("@"):
            return Expr("pattern_binding", first.token, (first, self.range_pattern(array_item=array_item)))
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
        if token.text == "&&":
            self.lookahead.appendleft(self.current)
            self.current = replace(token, text="&", column=token.column + 1)
            token = replace(token, text="&")
        if token.text == "_":
            return Expr("wild", token)
        if token.text == "&":
            mutable = bool(self.accept("mut"))
            return Expr("pattern_reference", token, (mutable, self.simple_pattern()))
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
        if token.text == "ref":
            mutable = bool(self.accept("mut"))
            return Expr("bind", self.identifier(), (False, mutable))
        if token.text == "Self" and self.self_type is not None and self.current.text == "::":
            owner = self.self_type.name if isinstance(self.self_type, AppliedType) else self.self_type
            self.expect("::")
            variant = self.identifier()
            path, type_arguments = replace(variant, text=f"{owner}::{variant.text}"), None
        elif token.text not in {"crate", "self", "super"} and (
            not is_identifier(token.text) or token.text in self.reserved
        ):
            raise token.error("expected a pattern")
        else:
            path, type_arguments = self.expression_path(token)
        if path.text in SCALAR_CONSTANTS:
            return Expr("literal", path, SCALAR_CONSTANTS[path.text])
        if path.text in self.constants:
            constant = self.constants[path.text]
            return Expr("literal", path, (constant.kind, constant.value))
        if path.text in self.statics:
            raise path.error("a pattern binding cannot shadow a static item")
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
            body = self.expression(statement=self.current.text == "{")
            arms.append((pattern, guard, body))
            if not self.accept(",") and body.kind not in BLOCK_EXPRESSIONS:
                break
        self.expect("}")
        return Expr("match", token, (scrutinee, tuple(arms)))

    def condition(self):
        terms = []
        delimiters = []
        while True:
            pattern = None
            if self.accept("let"):
                pattern = self.pattern()
                self.expect("=")
            term = self.expression(PRECEDENCE["&&"] + 1, allow_record=False)
            terms.append((pattern, term))
            separator = self.accept("&&")
            if separator is None:
                break
            delimiters.append(separator)
        if any(pattern is not None for pattern, _ in terms):
            if len(terms) > 1 and self.edition != "2024":
                raise delimiters[0].error("let chains require Rust 2024")
            if self.current.text == "||":
                raise self.current.error("let chain conditions cannot use unparenthesized '||'")
            return tuple(terms)
        condition = terms[0][1]
        for separator, (_, right) in zip(delimiters, terms[1:]):
            condition = Expr("binary", separator, (condition, right))
        while separator := self.accept("||"):
            right = self.expression(PRECEDENCE["||"] + 1, allow_record=False)
            condition = Expr("binary", separator, (condition, right))
        return ((None, condition),)

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
    moved: bool = False


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
            # A closure nested in another closure makes the outer capture mutable too.
            CaptureContext.mark_mutable(scope.captures.enclosing, expr)


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


def reference_source(expr, annotation):
    expr = ungroup(expr)
    if is_static_str(annotation) or (is_str_reference(annotation) and expr.kind in {"literal", "default"}):
        return True
    return expr.kind in {"name", "borrow", "call", "indirect_call", "if", "if_chain", "match", "dyn_reference",
                         "slice_reference", "string_reference", "byte_string", "vector_method",
                         "string_method", "clone"} or (
        expr.kind == "block" and not expr.label
    )


def shared_coercion(actual, expected):
    return (isinstance(actual, ReferenceType) and actual.mutable
            and isinstance(expected, ReferenceType) and not expected.mutable
            and actual.inner == expected.inner)


def iteration_item(annotation):
    if isinstance(annotation, IteratorType):
        return annotation.item
    base = annotation.inner if isinstance(annotation, ReferenceType) else annotation
    if isinstance(base, ArrayType) or (isinstance(base, SliceType) and isinstance(annotation, ReferenceType)):
        element = base.element
    elif isinstance(base, AppliedType) and base.name in {VEC, OPTION, RESULT}:
        element = base.arguments[0]
    else:
        return None
    return ReferenceType(element, annotation.mutable) if isinstance(annotation, ReferenceType) else element


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

    def write(self, value):
        if len(value.items) != self.length:
            raise AssertionError("slice mutation changed the length of a borrowed view")
        items = self.address.read().items
        self.address.write(ArrayValue((*items[:self.start], *value.items, *items[self.stop:])))


@dataclass(frozen=True)
class StringAddress:
    address: Address
    start: int
    stop: int

    def read(self):
        return self.address.read().encode("utf-8")[self.start:self.stop].decode("utf-8")

    def write(self, value):
        data = self.address.read().encode("utf-8")
        replacement = value.encode("utf-8")
        if len(replacement) != self.stop - self.start:
            raise AssertionError("str mutation changed the length of a borrowed view")
        self.address.write((data[:self.start] + replacement + data[self.stop:]).decode("utf-8"))


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


class MapValue:
    """A HashMap, BTreeMap, HashSet, or BTreeSet: keys map to [key, value] pairs, sets store None values."""

    __slots__ = ("entries", "ordered")

    def __init__(self, entries=None, ordered=False):
        self.entries = {} if entries is None else entries
        self.ordered = ordered

    def __eq__(self, other):
        return isinstance(other, MapValue) and self.entries.keys() == other.entries.keys()

    __hash__ = None


class RcValue:
    """An Rc or Arc: clones share one cell and one pair of strong and weak counts."""

    __slots__ = ("cell", "counts", "concrete")

    def __init__(self, cell, counts, concrete=None):
        self.cell = cell
        self.counts = counts
        self.concrete = concrete

    def read(self):
        return self.cell.value


class WeakValue:
    __slots__ = ("cell", "counts", "concrete")

    def __init__(self, cell, counts, concrete=None):
        self.cell = cell
        self.counts = counts
        self.concrete = concrete


class MapSlot:
    """A binding-like view of one map value, so Address can read and write through it."""

    __slots__ = ("map", "key")

    def __init__(self, map_value, key):
        self.map = map_value
        self.key = key

    @property
    def value(self):
        return self.map.entries[self.key][1]

    @value.setter
    def value(self, value):
        self.map.entries[self.key][1] = value

    @property
    def mutable(self):
        return True


@dataclass(frozen=True)
class BoxValue:
    address: Address
    concrete: object

    def read(self):
        return self.address.read()


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


@dataclass(frozen=True)
class FloatRange:
    start: float | None
    stop: float | None
    inclusive: bool

    def contains(self, value):
        return (not math.isnan(value) and (self.start is None or self.start <= value)
                and (self.stop is None or value < self.stop or (self.inclusive and value == self.stop)))


def ordinal(value):
    return ord(value) if isinstance(value, Character) else value


@dataclass(frozen=True)
class SlicePattern:
    annotation: SliceType
    prefix: int
    suffix: int | None
    rest_binding: object = field(default=None, compare=False)
    rest_by_value: bool = field(default=False, compare=False)

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
    aliases: tuple = ()

    def bindings(self, value, address=None):
        aliases = {alias.binding: address if alias.by_reference else value for alias in self.aliases}
        for matched in self.match_value(value, address):
            yield aliases | matched

    def match_value(self, value, address):
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
                extra = {}
                if self.tag.rest_binding is not None:
                    start, stop = self.tag.prefix, len(value.items) - (self.tag.suffix or 0)
                    if self.tag.rest_by_value:
                        extra = {self.tag.rest_binding: ArrayValue(value.items[start:stop])}
                    elif isinstance(address, SliceAddress):
                        extra = {self.tag.rest_binding: SliceAddress(address.address, address.start + start,
                                                                     address.start + stop)}
                    else:
                        extra = {self.tag.rest_binding: SliceAddress(address, start, stop)}
                for matched in self.field_bindings(tuple(value.items[index] for index in positions),
                                                   address, tuple(map(str, positions))):
                    yield matched | extra
        elif isinstance(self.tag, (IntegerRange, FloatRange)):
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
            if (isinstance(option.tag, SlicePattern) and isinstance(annotation, ArrayType)
                    and isinstance(annotation.length, ConstValue)):
                # An array pattern with `rest @ ..` checks the same positions as a plain array pattern.
                length = annotation.length.value
                positions = (*range(option.tag.prefix), *range(length - (option.tag.suffix or 0), length))
                fields = tuple((str(index), annotation.element) for index in positions)
                option = replace(option, tag=Constructor("[]", annotation, "array", fields))
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


FORMAT_SPECIFICATION = re.compile(
    r"(?:(?P<fill>[^{}])(?P<filled_alignment>[<^>])|(?P<alignment>[<^>]))?"
    r"(?P<sign>[+-])?(?P<alternate>#)?(?P<zero>0(?!\$))?"
    r"(?P<width>[^\s$.:]+\$|[0-9]+)?"
    r"(?:\.(?P<precision>[^\s$.:]+\$|[0-9]+|\*))?(?P<kind>[?boxXeE])?"
)


@dataclass(frozen=True)
class FormatCount:
    argument: object


@dataclass(frozen=True)
class FormatSpec:
    kind: str
    fill: str
    align: str | None
    sign: str | None
    alternate: bool
    zero: bool
    width: int | FormatCount
    precision: int | FormatCount | None


@dataclass(frozen=True)
class FormatField:
    argument: object
    spec: FormatSpec
    annotation: object = None


def parse_format_field(text, token):
    argument, _, specification = text.partition(":")
    argument = unicodedata.normalize("NFC", argument.rstrip())
    if argument and not (argument.isidentifier() or re.fullmatch(r"[0-9]+", argument)):
        raise token.error("unsupported format argument")
    match = FORMAT_SPECIFICATION.fullmatch(specification.rstrip())
    if not match:
        raise token.error("unsupported format specification")
    fields = match.groupdict()
    try:
        width = parse_format_count(fields["width"], 0)
        precision = parse_format_count(fields["precision"], None)
        argument = int(argument) if argument.isdecimal() else argument or None
    except ValueError:
        raise token.error("format argument is out of range") from None
    spec = FormatSpec(fields["kind"] or "", fields["fill"] or " ", fields["filled_alignment"] or fields["alignment"],
                      fields["sign"], bool(fields["alternate"]), bool(fields["zero"]), width, precision)
    return FormatField(argument, spec)


def parse_format_count(value, default):
    if value is None:
        return default
    if value == "*":
        return FormatCount(None)
    if value.endswith("$"):
        argument = unicodedata.normalize("NFC", value[:-1])
        if re.fullmatch(r"[0-9]+", argument):
            return FormatCount(int(argument))
        if argument.isidentifier():
            return FormatCount(argument)
        raise ValueError("invalid format count")
    return int(value)


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
            end = text.find("}", index + 1)
            if char != "{" or end < 0:
                raise token.error("unsupported format placeholder")
            if literal:
                parts.append("".join(literal))
                literal.clear()
            parts.append(parse_format_field(text[index + 1:end], token))
            index = end + 1
    if literal:
        parts.append("".join(literal))
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
    def __init__(self, program, modules):
        self.program = program
        self.modules = modules
        self.functions = program.functions
        self.result = "()"
        self.loops = []
        self.macro_labels = {}
        self.substitutions = {}
        self.variables = []
        self.obligations = []
        self.associated_equalities = []
        self.generic_arguments = []
        self.numeric_operands = []
        self.unary_operands = []
        self.integer_literals = []
        self.float_literals = []
        self.constant_lints = []
        self.deferred_binding_types = set()
        self.scalar_casts = []
        self.iterator_results = []
        self.parameters = {}
        self.ascriptions = []
        self.closures = []
        self.pointers = []
        self.indirect_calls = []
        self.callable_hints = {}
        self.callable_requirements = []
        self.assumed_bounds = ()
        self.obligation_assumptions = {}
        self.trait_obligations = []
        self.pending_selections = []
        self.error_conversions = []
        self.mutable_places = set()
        self.pretyped = {}
        self.std_string_search = False

    def pin_all_projections(self):
        """Record the trait of each `T::Item` on a type parameter, so substitution keeps it unambiguous."""
        pin = self.program.pin_projections
        for function in self.functions.values():
            function.params = [(name, mutable, pin(kind)) for name, mutable, kind in function.params]
            function.result = pin(function.result)
        for trait in self.program.traits.values():
            for function in trait.methods.values():
                function.params = [(name, mutable, pin(kind)) for name, mutable, kind in function.params]
                function.result = pin(function.result)
        for impl in self.program.impls:
            for name, definition in tuple(impl.associated.items()):
                impl.associated[name] = replace(definition, body=pin(definition.body))
            for function in impl.methods.values():
                function.params = [(name, mutable, pin(kind)) for name, mutable, kind in function.params]
                function.result = pin(function.result)
        for name, (token, constructors) in tuple(self.program.types.items()):
            pinned = tuple(replace(ctor, fields=tuple((field, pin(kind)) for field, kind in ctor.fields))
                           for ctor in constructors)
            self.program.types[name] = (token, pinned)
            for ctor in pinned:
                self.program.constructors[ctor.path] = ctor

    def check(self):
        self.pin_all_projections()
        for alias in self.program.type_aliases.values():
            target, token = alias.target, alias.name
            self.check_bounds(alias.parameters, token)
            if isinstance(target, (SliceType, DynType)) or target == "str":
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
        if main.params or main.type_parameters:
            raise main.name.error("main must take no arguments and cannot be generic")
        self.obligation(main.result, TERMINATION, main.name)
        for function in tuple(self.functions.values()):
            added = []
            for parameter, bounds, bindings in function.where_bounds:
                added.append((parameter, bounds - parameter.bounds,
                              bindings.keys() - parameter.associated_bindings.keys()))
                parameter.bounds.update(bounds)
                parameter.associated_bindings.update(bindings)
            self.assumed_bounds = function.where_bounds
            try:
                self.check_function(function)
            finally:
                self.assumed_bounds = ()
                for parameter, bounds, keys in added:
                    parameter.bounds.difference_update(bounds)
                    for key in keys:
                        del parameter.associated_bindings[key]
        self.finish_inference()
        self.check_const_initializers()

    def check_const_initializers(self):
        """Reject calls that can never be const in const and static initializers."""
        def visit(expr):
            if isinstance(expr, (tuple, list)):
                for part in expr:
                    visit(part)
                return
            if not isinstance(expr, Expr) or expr.kind == "closure":
                return
            kind, word = expr.kind, expr.token.text
            target = self.program.call_functions.get(id(expr))
            dispatch = self.program.dispatches.get(id(expr))
            if (kind == "string" and word.endswith("String::from")) or kind in {"box", "format"} or (
                    kind in {"vector", "vector_repeat"} and word == "@vec" and expr.args):
                shown = {"box": "Box::new", "format": "format!", "vector": "vec!",
                         "vector_repeat": "vec!"}.get(kind, "String::from")
                raise expr.token.error(f"cannot call non-const function {shown!r} in {item.kind}s")
            if kind in {"call", "method"} and dispatch is not None and dispatch[0]:
                raise expr.token.error(f"cannot call non-const method {dispatch[2]!r} of trait "
                                       f"{dispatch[0].replace(MODULE_SEPARATOR, '::')} in {item.kind}s")
            if (kind == "call" and isinstance(target, Function) and target.name.source_file != "<std>"
                    and self.program.functions.get(target.name.text) is target
                    and target.name.text not in self.modules.const_functions):
                raise expr.token.error(f"cannot call non-const function {word!r} in {item.kind}s")
            visit(expr.args)

        for item in self.modules.lazy_items.values():
            if item.name.source_file == "<std>" or (item.kind == "static" and item.name.expansion_site):
                continue
            visit(item.expression)

    def check_function(self, function):
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
        self.check_type(function.result, function.name, allow_never=True)
        self.result = function.result
        opaque = isinstance(self.result, OpaqueType)
        if opaque:
            self.result = TypeVariable("opaque return type", function.name)
            self.variables.append(self.result)
            for bound in sorted(function.result.bounds):
                self.obligation(self.result, bound, function.name)
            signature = callable_signature(function.result)
            if signature is not None:
                self.callable_hints[self.result] = signature
            self.program.opaque_witnesses[function.name.text] = self.result
        concrete = self.effective_function(function, tuple(kind for _, _, kind in function.params),
                                           self.resolve(self.result))
        function.return_inputs = concrete.return_inputs
        if function.body is None:
            return
        result = self.type_of(function.body, scope, self.result)
        if opaque and result == function.result:
            result = NEVER
        self.require_argument(result, self.result, function.body.token)
        if opaque and result == NEVER and isinstance(self.resolve(self.result), TypeVariable):
            self.require(self.result, "()", function.name)

    def check_bounds(self, parameters, token):
        for parameter in parameters:
            if isinstance(parameter, LifetimeArgument):
                continue
            for name in parameter.bounds:
                if name not in TRAIT_MODULES and name not in self.program.traits and name != "?Sized":
                    raise token.error(f"unknown trait {name!r}")
            if isinstance(parameter, TypeParameter):
                for binding, expected in parameter.associated_bindings.items():
                    self.check_associated_binding(binding, expected, token)

    def check_associated_binding(self, binding, expected, token):
        if binding.item in {CALL_ITEM, ARGS_ITEM}:
            self.check_type(expected, token)
            return None
        trait = self.program.associated_trait(binding.bound, binding.item)
        if trait is None:
            raise token.error(f"unknown or ambiguous associated item {binding.item!r} for {binding.bound}")
        self.check_associated_arguments(trait, binding.item, binding.arguments, token)
        for lifetime in binding.lifetimes:
            if lifetime not in binding.arguments:
                replacement = {lifetime: LifetimeArgument("'@check")}
                if (substitute(expected, replacement) != expected
                        or any(substitute(argument, replacement) != argument for argument in binding.arguments)):
                    raise token.error(f"bound lifetime {lifetime.name} must be a direct associated type argument")
        self.check_type(expected, token)
        return trait

    def check_associated_arguments(self, trait, item, arguments, token):
        parameters = trait.associated[item].parameters
        if len(parameters) != len(arguments):
            raise token.error(f"{item} expects {len(parameters)} generic arguments")
        for parameter, argument in zip(parameters, arguments):
            if isinstance(parameter, LifetimeArgument):
                if not isinstance(argument, LifetimeArgument):
                    raise token.error(f"{item} expects a lifetime argument")
            elif parameter.const_type:
                self.check_const(argument, parameter.const_type, token)
            else:
                self.check_type(argument, token)
                for bound in parameter.bounds:
                    self.obligation(argument, bound, token)

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
            if isinstance(impl.target, SliceType) or impl.target == "str":
                self.check_type(ReferenceType(impl.target, False), impl.token)
            else:
                self.check_type(impl.target, impl.token)
            if impl.trait == TERMINATION:
                owner = impl.target
                while isinstance(owner, ReferenceType) or is_box(owner):
                    owner = owner.inner if isinstance(owner, ReferenceType) else owner.arguments[0]
                name = owner.name if isinstance(owner, AppliedType) else owner
                if name in {OPTION, RESULT} or name not in self.program.types:
                    raise impl.token.error("Termination can only be implemented for a type defined in this crate")
            target_parameters = set(self.type_parts(impl.target))
            for argument in impl.trait_arguments:
                target_parameters.update(type_leaves(argument))
            for parameter in tuple(target_parameters):
                signature = callable_signature(parameter) if isinstance(parameter, TypeParameter) else None
                if signature is not None:
                    target_parameters.update(type_leaves(signature))
            if any(parameter not in target_parameters for parameter in impl.parameters):
                raise impl.token.error("unconstrained impl parameter")
            if impl.trait is None and impl.token.module[:1] != STD_ROOT and (
                    isinstance(impl.target, TupleType) or self.program.layout(impl.target) is None):
                raise impl.token.error("inherent impl requires a declared struct or enum")
            for previous in self.program.impls[:index]:
                if impl.token.module[:1] == STD_ROOT:
                    break
                if previous.trait == impl.trait and overlapping_types(previous.target, impl.target) and all(
                        overlapping_types(first, second) for first, second in zip(
                            self.program.trait_arguments(impl.trait, previous.trait_arguments, previous.target),
                            self.program.trait_arguments(impl.trait, impl.trait_arguments, impl.target))):
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
                if (isinstance(resolved, AssociatedType) and resolved.item == name
                        and resolved.owner == impl.target):
                    raise impl.token.error("associated type cannot resolve to itself")
                if (isinstance(resolved, ReferenceType) and resolved.lifetime != "'static"
                        and impl.token.module[:1] != STD_ROOT and resolved.lifetime is None
                        and LifetimeArgument(resolved.lifetime) not in definition.parameters):
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
                bindings.update(zip(trait.parameters,
                                    self.program.trait_arguments(impl.trait, impl.trait_arguments, impl.target)))
                if (len(required.method_parameters) != len(method.method_parameters)
                        or any(not self.same_parameter_kind(first, second)
                               for first, second in zip(required.method_parameters, method.method_parameters))):
                    raise method.name.error(f"method {name!r} has incompatible generic parameters")
                bindings.update(zip(required.method_parameters, method.method_parameters))
                for declared, provided in zip(required.method_parameters, method.method_parameters):
                    def substitute_parameter(kind):
                        return self.resolve(substitute(kind, bindings))

                    expected = dict(key.map_arguments(substitute_parameter).canonicalize(substitute_parameter(kind))
                                    for key, kind in declared.associated_bindings.items())
                    actual = dict(key.map_arguments(self.resolve).canonicalize(self.resolve(kind))
                                  for key, kind in provided.associated_bindings.items())
                    if actual != expected:
                        raise method.name.error(f"method {name!r} has incompatible associated type bounds")
                required_types = tuple(self.resolve(substitute(kind, bindings)) for _, _, kind in required.params)
                actual_types = tuple(self.resolve(kind) for _, _, kind in method.params)
                required_result = self.resolve(substitute(required.result, bindings))
                actual_result = self.resolve(method.result)
                required_signature = self.effective_function(required, required_types, required_result)
                actual_signature = self.effective_function(method, actual_types, actual_result)
                if impl.token.module[:1] == STD_ROOT:
                    continue
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
        return (first.const_type == second.const_type and first.bounds == second.bounds
                and first.anonymous == second.anonymous)

    def effective_function(self, function, types, result):
        if not isinstance(function.result, AssociatedType) or not isinstance(result, ReferenceType):
            return function
        params = [(name, mutable, kind) for (name, mutable, _), kind in zip(function.params, types)]
        if self.has_receiver(function) and isinstance(types[0], ReferenceType) and not result.lifetime:
            inputs = (0,)
        else:
            inputs = Parser.return_inputs(function.name, params, result, function.lifetimes)
        return replace(function, params=params, result=result, return_inputs=inputs)

    def resolve(self, annotation, *, defer_const=False):
        if isinstance(annotation, TypeVariable) and annotation in self.substitutions:
            result = self.resolve(self.substitutions[annotation], defer_const=defer_const)
            if not defer_const:
                self.substitutions[annotation] = result
            return result
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.resolve(annotation.inner))
        if isinstance(annotation, FunctionType):
            return FunctionType(tuple(map(self.resolve, annotation.parameters)), self.resolve(annotation.result),
                                annotation.closure)
        if isinstance(annotation, IteratorType):
            return replace(annotation, item=self.resolve(annotation.item), source=tuple(map(self.resolve, annotation.source)))
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.resolve(arg) for arg in annotation.arguments))
        if isinstance(annotation, OpaqueType):
            return replace(annotation, arguments=tuple(self.resolve(arg) for arg in annotation.arguments),
                           bindings=tuple((binding.map_arguments(self.resolve), self.resolve(kind))
                                          for binding, kind in annotation.bindings))
        if isinstance(annotation, TupleType):
            return TupleType(tuple(map(self.resolve, annotation.elements)))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.resolve(annotation.element), self.resolve(annotation.length))
        if isinstance(annotation, SliceType):
            return SliceType(self.resolve(annotation.element))
        if isinstance(annotation, ConstExpression):
            return resolve_const_operands(annotation, self.resolve, defer_const=defer_const)
        if isinstance(annotation, DynType):
            return DynType(annotation.trait, tuple((name, self.resolve(value)) for name, value in annotation.bindings))
        if isinstance(annotation, AssociatedType):
            projected = replace(annotation, owner=self.resolve(annotation.owner),
                                arguments=tuple(map(self.resolve, annotation.arguments)),
                                trait_arguments=tuple(map(self.resolve, annotation.trait_arguments)))
            normalized = self.program.normalize(projected)
            return normalized if isinstance(normalized, AssociatedType) else self.resolve(normalized)
        return annotation

    def occurs(self, variable, annotation):
        annotation = self.resolve(annotation)
        if isinstance(annotation, ReferenceType):
            return self.occurs(variable, annotation.inner)
        if isinstance(annotation, FunctionType):
            return any(self.occurs(variable, kind) for kind in (*annotation.parameters, annotation.result))
        if isinstance(annotation, IteratorType):
            return any(self.occurs(variable, part) for part in (annotation.item, *annotation.source))
        if isinstance(annotation, (AppliedType, OpaqueType)):
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
        if (isinstance(actual, TypeVariable) and isinstance(expected, TypeVariable)
                and (expected.integer_literal or expected.float_literal)
                and not (actual.integer_literal or actual.float_literal)):
            # Keep the literal variable as the representative so it still defaults to i32 or f64.
            actual, expected = expected, actual
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
        if isinstance(actual, IteratorType) and isinstance(expected, IteratorType):
            if actual.source == expected.source and actual.borrowed == expected.borrowed:
                self.require(actual.item, expected.item, token)
                return
        if isinstance(actual, IteratorType) and isinstance(expected, AppliedType) and expected.name == LAZY:
            # Built-in iterators share Lazy's runtime representation, so std names like Chars accept them.
            self.require(actual.item, expected.arguments[0], token)
            return
        if isinstance(actual, AppliedType) and isinstance(expected, AppliedType):
            if actual.name == expected.name and len(actual.arguments) == len(expected.arguments):
                for left, right in zip(actual.arguments, expected.arguments):
                    self.require(left, right, token)
                return
        if isinstance(actual, OpaqueType) and isinstance(expected, OpaqueType):
            if actual.function == expected.function:
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
        if self.assumed_bounds:
            self.obligation_assumptions[len(self.obligations)] = self.assumed_bounds
        self.obligations.append((annotation, trait, token))

    def check_variable_binding(self, variable, value, token):
        if variable.const_type:
            self.check_const(value, variable.const_type, token)
        elif isinstance(value, (ConstValue, ConstExpression)) or getattr(value, "const_type", None):
            raise token.error("expected a type argument, found a constant")

    def check_const(self, value, kind, token):
        check_constant_kind(self.resolve(value), kind, token)

    def specialize(self, parameters, explicit, token, *, callables=True, requirements=True):
        visible_count = sum(not parameter.anonymous for parameter in parameters)
        if explicit is not None and (not visible_count or len(explicit) != visible_count):
            raise token.error(f"{token.text} expects {visible_count} type arguments")
        explicit_arguments = iter(explicit or ())
        arguments = {}
        for parameter in parameters:
            argument = None if parameter.anonymous else next(explicit_arguments, None)
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
        if requirements:
            self.associated_equalities.extend(self.associated_requirements(arguments, token, callables=callables))
        return arguments

    def associated_requirements(self, arguments, token, *, callables=False):
        for parameter, owner in arguments.items():
            for binding, expected in parameter.associated_bindings.items():
                if binding.item == ARGS_ITEM:
                    continue
                if binding.item == CALL_ITEM:
                    signature = substitute(expected, arguments)
                    if callables and not any(isinstance(part, TypeParameter) and part.name == "Self"
                                             for part in type_leaves(signature)):
                        self.require_callable(owner, signature, token)
                    continue
                trait = self.program.associated_trait(binding.bound, binding.item)
                if trait is None:
                    raise token.error(f"unknown or ambiguous associated item {binding.item!r} for {binding.bound}")
                projection = substitute(binding.project(owner, trait), arguments)
                yield projection, substitute(expected, arguments), token

    def require_callable(self, owner, signature, token):
        """Tie a value's call signature to an Fn-family bound, now or once it is known."""
        resolved = self.resolve(owner)
        if isinstance(resolved, TypeVariable):
            self.callable_hints[resolved] = signature
            self.callable_requirements.append((resolved, signature, token))
            return
        while isinstance(resolved, ReferenceType) or is_box(resolved):
            resolved = self.resolve(resolved.inner if isinstance(resolved, ReferenceType) else resolved.arguments[0])
        actual = callable_signature(resolved)
        if actual is None:
            if resolved == NEVER:
                return
            raise token.error(f"expected a callable value, found {resolved}")
        if len(actual.parameters) != len(signature.parameters):
            raise token.error(f"expected a callable with {len(signature.parameters)} parameters")
        for left, right in zip(actual.parameters, signature.parameters):
            self.require(left, right, token)
        self.require(actual.result, signature.result, token)

    def solve_callables(self, *, final=False):
        pending, self.callable_requirements = self.callable_requirements, []
        for owner, signature, token in pending:
            if isinstance(self.resolve(owner), TypeVariable):
                if final:
                    raise token.error("type annotations needed for a callable argument")
                self.callable_requirements.append((owner, signature, token))
            else:
                self.require_callable(owner, signature, token)

    def solve_associated_equalities(self, requirements):
        pending = list(requirements)
        while pending:
            remaining = []
            for projection, expected, token in pending:
                if isinstance(self.resolve(projection.owner), TypeVariable):
                    remaining.append((projection, expected, token))
                else:
                    self.require_associated_equality(projection, expected, token)
            if len(remaining) == len(pending):
                return remaining
            pending = remaining
        return pending

    def require_associated_equality(self, projection, expected, token):
        actual, expected = self.resolve(projection), self.resolve(expected)
        if isinstance(actual, ReferenceType) and isinstance(expected, ReferenceType):
            if actual.lifetime and expected.lifetime and actual.lifetime != expected.lifetime:
                raise token.error(f"associated type requires lifetime {expected.lifetime}, found {actual.lifetime}")
        self.require(actual, expected, token)

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
        self.solve_callables()
        settled = set()
        progress = True
        while progress:
            # Collect targets can pin literal element types and nested collect targets, so settle
            # them, outermost first, before literals default.
            progress = False
            for index, (name, item, result, token) in enumerate(self.iterator_results):
                if index in settled or name != "collect" or isinstance(self.resolve(result), TypeVariable):
                    continue
                settled.add(index)
                progress = True
                try:
                    self.check_collect(item, result, token)
                except RustError:
                    pass
        remaining_equalities = self.solve_associated_equalities(self.associated_equalities)
        for annotation, value, token in self.integer_literals:
            resolved = self.resolve(annotation)
            if isinstance(resolved, TypeVariable):
                self.require(resolved, "i32", token)
                resolved = "i32"
            if resolved not in INTEGER_TYPES:
                raise token.error(f"expected an integer, found {resolved}")
            if not OVERFLOWING_LITERALS[0]:
                checked(value, token, resolved)
        for annotation, value, token in self.float_literals:
            resolved = self.resolve(annotation)
            if isinstance(resolved, TypeVariable):
                self.require(resolved, "f64", token)
                resolved = "f64"
            if resolved not in FLOAT_TYPES:
                raise token.error(f"expected a float, found {resolved}")
            float_literal_value(value, resolved, token)
        self.solve_associated_equalities(remaining_equalities)
        self.solve_callables(final=True)
        for operand, token in self.unary_operands:
            operand = self.resolve(operand)
            if not isinstance(operand, TypeVariable):
                if token.text == "-":
                    if not (operand in FLOAT_TYPES or (operand in INTEGER_TYPES and INTEGER_BOUNDS[operand][0] < 0)):
                        raise token.error(f"cannot negate {operand} with '-'")
                elif operand != "bool" and operand not in INTEGER_TYPES:
                    raise token.error(f"cannot negate {operand} with '!'")
        for actual, target, token in self.scalar_casts:
            source = self.resolve(actual)
            if not scalar_cast_supported(source, target):
                raise token.error(f"cannot cast {source} to {target}")
        for name, witness in self.program.opaque_witnesses.items():
            function = self.functions[name]
            for binding, expected in function.result.bindings:
                if binding.item in {CALL_ITEM, ARGS_ITEM} or isinstance(self.resolve(witness), TypeVariable):
                    continue
                trait = self.program.associated_trait(binding.bound, binding.item)
                if trait is not None:
                    try:
                        self.require_associated_equality(binding.project(witness, trait), expected, function.name)
                    except RustError:
                        pass
        for name, item, result, token in self.iterator_results:
            self.check_iterator_result(name, self.resolve(item), self.resolve(result), token)
        for variable in self.variables:
            if isinstance(self.resolve(variable), TypeVariable):
                raise variable.token.error(f"type annotations needed for {variable.name}")
        self.check_opaque_witnesses()
        for key, (token, parts) in self.program.formats.items():
            resolved = []
            for field in parts:
                if isinstance(field, FormatField):
                    annotation = self.resolve(field.annotation)
                    if annotation != NEVER and field.spec.kind in {"b", "o", "x", "X"} and annotation not in INTEGER_TYPES:
                        raise token.error(f"base formatting requires an integer, found {annotation}")
                    if annotation != NEVER and field.spec.kind in {"e", "E"} and annotation not in NUMBER_TYPES:
                        raise token.error(f"exponent formatting requires a number, found {annotation}")
                    field = replace(field, annotation=annotation)
                resolved.append(field)
            self.program.formats[key] = token, tuple(resolved)
        for function in self.closures:
            function.captures = tuple(replace(capture, annotation=self.resolve(capture.annotation))
                                      for capture in function.captures)
        for annotation, token in self.generic_arguments:
            annotation = self.resolve(annotation)
        for index, (annotation, trait, token) in enumerate(self.obligations):
            annotation = self.resolve(annotation)
            assumed = self.obligation_assumptions.get(index, ())
            if any(annotation is parameter and trait in self.program.trait_closure(bounds)
                   for parameter, bounds, _ in assumed):
                continue
            if (annotation, trait) in TYPE_BOUNDS:
                continue
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
        self.check_constant_lints()
        for key, (types, arguments) in self.program.call_types.items():
            self.program.call_types[key] = (tuple(self.resolve(kind) for kind in types),
                                            {p: self.resolve(arg) for p, arg in arguments.items()})
        for key, (trait, receiver, method, arguments) in self.program.dispatches.items():
            self.program.dispatches[key] = (trait, self.resolve(receiver), method,
                                            tuple(map(self.resolve, arguments)))
        for source, target, token, expr in self.error_conversions:
            source, target = self.resolve(source), self.resolve(target)
            if source == target:
                continue
            if self.program.implementation(target, CONVERT_FROM, arguments_wanted=(source,)) is None:
                raise token.error(f"? cannot convert {source} into {target}")
            self.program.try_conversions[id(expr)] = (source, target)
        for receiver, trait, arguments, token in self.trait_obligations:
            receiver, arguments = self.resolve(receiver), tuple(map(self.resolve, arguments))
            if receiver == NEVER or isinstance(receiver, (TypeParameter, OpaqueType, DynType, AssociatedType)):
                continue
            if self.program.implementation(receiver, trait, arguments_wanted=arguments) is None and not (
                    self.builtin_trait_arguments(receiver, trait, arguments)):
                shown = f"{trait}<{', '.join(map(str, arguments))}>"
                raise token.error(f"{receiver} does not implement {shown}")
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
            if isinstance(function.result, ReferenceType):
                try:
                    function.return_inputs = Parser.return_inputs(function.name, function.params,
                                                                  function.result, {})
                except RustError:
                    # Closures infer their result lifetime; leave results the checker cannot tie untracked.
                    function.return_inputs = ()
        for expr in self.pointers:
            name, arguments = expr.args
            expr.args = (name, {parameter: self.resolve(kind) for parameter, kind in arguments.items()})
        for expr in self.indirect_calls:
            function = self.program.call_functions[id(expr)]
            function.params = [(name, mutable, self.resolve(kind)) for name, mutable, kind in function.params]
            function.result = self.resolve(function.result)
            function.return_inputs = Parser.return_inputs(function.name, function.params, function.result, {})

    def constant_integer(self, expr):
        """The value of an integer operand built only from literals, or None."""
        expr = ungroup(expr)
        annotation = self.program.expr_types.get(id(expr))
        if annotation not in INTEGER_TYPES:
            return None
        if expr.kind == "literal" and type(expr.args[1]) is int:
            return expr.args[1]
        if expr.kind == "unary" and expr.token.text == "-":
            value = self.constant_integer(expr.args[0])
            return None if value is None else -value
        if expr.kind == "binary" and expr.token.text in {"+", "-", "*"}:
            left, right = (self.constant_integer(arg) for arg in expr.args)
            if left is None or right is None:
                return None
            value = ARITHMETIC[expr.token.text](left, right)
            lower, upper = INTEGER_BOUNDS[annotation]
            return value if lower <= value < upper else None
        return None

    def check_constant_lints(self):
        # rustc's deny-by-default arithmetic_overflow and unconditional_panic lints, for operands
        # that are literals.
        for expr, length in self.constant_lints:
            word = expr.token.text
            if length is not None:
                if expr.kind != "index" or "unconditional_panic" in ALLOWED_LINTS:
                    continue
                index = self.constant_integer(expr.args[1])
                if (isinstance(length, ConstValue) and type(length.value) is int
                        and index is not None and index >= length.value):
                    raise expr.token.error("this operation will panic at runtime: index out of bounds: "
                                           f"the length is {length.value} but the index is {index}")
                continue
            if expr.kind != "binary":
                continue
            annotation = self.program.expr_types.get(id(expr))
            if annotation not in INTEGER_TYPES:
                continue
            right = self.constant_integer(expr.args[1])
            if right is None:
                continue
            if word in {"/", "%"}:
                if right == 0 and "unconditional_panic" not in ALLOWED_LINTS:
                    action = "divide" if word == "/" else "calculate the remainder of"
                    raise expr.token.error(f"this operation will panic at runtime: attempt to {action} by zero")
                continue
            if "arithmetic_overflow" in ALLOWED_LINTS:
                continue
            if word in SHIFTS:
                if not 0 <= right < INTEGER_WIDTHS[annotation]:
                    raise expr.token.error(f"this arithmetic operation will overflow: attempt to shift "
                                           f"{'left' if word == '<<' else 'right'} by {right}")
                continue
            left = self.constant_integer(expr.args[0])
            if left is None:
                continue
            value = ARITHMETIC[word](left, right)
            lower, upper = INTEGER_BOUNDS[annotation]
            if not lower <= value < upper:
                raise expr.token.error(f"this arithmetic operation will overflow: "
                                       f"attempt to compute {left} {word} {right} as {annotation}")

    def check_opaque_witnesses(self):
        witnesses = self.program.opaque_witnesses
        for name, annotation in witnesses.items():
            witnesses[name] = self.resolve(annotation)
        visiting, finished = set(), set()

        def visit(name):
            if name in visiting:
                raise self.functions[name].name.error("cannot resolve recursive opaque return type")
            if name in finished:
                return
            visiting.add(name)
            parts = list(self.type_parts(witnesses[name]))
            while parts:
                part = parts.pop()
                if isinstance(part, OpaqueType):
                    visit(part.function)
                    for argument in part.arguments:
                        parts.extend(self.type_parts(argument))
                elif isinstance(part, FunctionType):
                    for argument in (*part.parameters, part.result):
                        parts.extend(self.type_parts(argument))
            visiting.remove(name)
            finished.add(name)

        for name in witnesses:
            visit(name)
            function = self.functions[name]
            witness = self.program.concrete_type(function.result)
            for binding, expected in function.result.bindings:
                if binding.item == ARGS_ITEM:
                    continue
                if binding.item == CALL_ITEM:
                    self.require_callable(witnesses[name], expected, function.name)
                    continue
                trait = self.program.associated_trait(binding.bound, binding.item)
                projection = binding.project(witnesses[name], trait)
                self.require_associated_equality(projection, expected, function.name)

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
        if isinstance(annotation, OpaqueType):
            self.check_bounds((annotation,), token)
            for binding, expected in annotation.bindings:
                self.check_associated_binding(binding, expected, token)
            return
        if isinstance(annotation, FunctionType):
            for kind in annotation.parameters:
                self.check_type(kind, token)
            self.check_type(annotation.result, token, allow_never=True)
            if isinstance(annotation.result, ReferenceType):
                params = [(token, False, kind) for kind in annotation.parameters]
                Parser.return_inputs(token, params, annotation.result, {})
            return
        if isinstance(annotation, IteratorType):
            self.check_type(annotation.item, token)
            return
        if isinstance(annotation, TupleType):
            for kind in annotation.elements:
                self.check_type(kind, token)
            return
        if isinstance(annotation, ArrayType):
            if borrowing_iterator(annotation.element):
                raise token.error("borrowed iterator elements in arrays are unsupported")
            self.check_type(annotation.element, token)
            self.check_const(annotation.length, "usize", token)
            return
        if isinstance(annotation, AssociatedType):
            self.check_type(annotation.owner, token)
            trait = self.program.projection_trait(annotation)
            if not trait:
                raise token.error(f"unknown or ambiguous associated type {annotation}")
            self.obligation(annotation.owner, trait.name.text, token)
            self.check_associated_arguments(trait, annotation.item, annotation.arguments, token)
            return
        if isinstance(annotation, ReferenceType):
            if annotation.inner == "str":
                return
            if isinstance(annotation.inner, DynType):
                self.check_dyn(annotation.inner, token)
            elif isinstance(annotation.inner, SliceType):
                element = annotation.inner.element
                self.check_type(element, token)
            else:
                self.check_type(annotation.inner, token)
            return
        if isinstance(annotation, DynType):
            raise token.error("trait objects require a reference")
        if isinstance(annotation, SliceType):
            raise token.error("slice type requires a reference")
        if annotation == "str":
            raise token.error("str requires a reference")
        if isinstance(annotation, AppliedType):
            parameters = self.program.type_parameters.get(annotation.name)
            if not parameters or len(parameters) != len(annotation.arguments):
                raise token.error(f"{annotation.name} expects {len(parameters or ())} type arguments")
            for parameter, argument in zip(parameters, annotation.arguments):
                if parameter.const_type:
                    self.check_const(argument, parameter.const_type, token)
                    continue
                if annotation.name in POINTER_TYPES and isinstance(argument, (DynType, SliceType)) or (
                        annotation.name in POINTER_TYPES and argument == "str"):
                    if isinstance(argument, DynType):
                        self.check_dyn(argument, token)
                    continue
                self.check_type(argument, token)
                self.generic_arguments.append((argument, token))
                for trait in sorted(parameter.bounds):
                    self.obligation(argument, trait, token)
            arguments = dict(zip(parameters, annotation.arguments))
            self.associated_equalities.extend(self.associated_requirements(arguments, token))
            return
        if self.program.type_parameters.get(annotation):
            count = len(self.program.type_parameters[annotation])
            raise token.error(f"{annotation} expects {count} type arguments")
        if annotation in PRIMITIVES or annotation in {STRING, EXIT_CODE} or annotation in self.program.types:
            return
        if annotation != NEVER or not allow_never:
            raise token.error(f"unsupported type {annotation!r}")

    def check_dyn(self, annotation, token):
        if annotation.trait in CALLABLE_TRAITS:
            signature = callable_signature(annotation)
            if signature is None or len(annotation.bindings) != 1:
                raise token.error(f"dyn {annotation.trait} needs a call signature")
            self.check_type(signature, token)
            return
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
                for part in self.type_parts(kind, inline_only=True):
                    if part in self.program.types:
                        self.check_layout(part, token, visiting, finished)
                used.update(part for part in self.type_parts(kind) if isinstance(part, TypeParameter))
        for param in self.program.type_parameters[name]:
            if param not in used and not param.const_type and self.program.types[name][0].module[:1] != STD_ROOT:
                raise token.error(f"unused type parameter {param}")
        visiting.remove(name)
        finished.add(name)

    @staticmethod
    def type_parts(annotation, *, inline_only=False):
        if isinstance(annotation, AppliedType):
            yield annotation.name
            if not (inline_only and annotation.name in INDIRECT_TYPES):
                for argument in annotation.arguments:
                    yield from Checker.type_parts(argument, inline_only=inline_only)
        elif isinstance(annotation, ArrayType):
            yield from Checker.type_parts(annotation.element, inline_only=inline_only)
            yield annotation.length
        elif isinstance(annotation, TupleType):
            for kind in annotation.elements:
                yield from Checker.type_parts(kind, inline_only=inline_only)
        elif isinstance(annotation, FunctionType):
            if not inline_only:
                for kind in (*annotation.parameters, annotation.result):
                    yield from Checker.type_parts(kind)
        elif isinstance(annotation, ReferenceType):
            if not inline_only:
                yield from Checker.type_parts(annotation.inner)
        elif isinstance(annotation, (SliceType,)):
            yield from Checker.type_parts(annotation.element, inline_only=inline_only)
        elif isinstance(annotation, AssociatedType):
            yield from Checker.type_parts(annotation.owner, inline_only=inline_only)
            yield annotation
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

    def prepare_pattern(self, expr, annotation, *, borrowed=None):
        kind, token, args = expr.kind, expr.token, expr.args
        annotation = self.resolve(annotation)
        unit = self.program.constructors.get(token.text) if kind == "bind" else None
        structural = kind not in {"bind", "wild", "or"} or (unit and unit.style == "unit")
        if kind == "pattern_binding":
            name, subpattern = args
            alias, own = self.prepare_pattern(name, annotation, borrowed=borrowed)
            if not alias.binding:
                raise token.error("the left side of '@' must bind a name")
            pattern, nested = self.prepare_pattern(subpattern, annotation, borrowed=borrowed)
            if alias.binding in nested:
                raise token.error(f"duplicate pattern binding {token.text!r}")
            if alias.by_reference or not self.implements(self.resolve(annotation), "Copy"):
                for kind, _ in nested.values():
                    if not isinstance(kind, ReferenceType):
                        self.obligation(kind, "Copy", token)
                    elif not alias.by_reference:
                        self.obligation(annotation, "Copy", token)
                    elif own[alias.binding][0].mutable or kind.mutable:
                        raise token.error("overlapping mutable borrows in an '@' pattern")
            return replace(pattern, aliases=(alias, *pattern.aliases)), own | nested
        if kind == "pattern_reference":
            mutable, operand = args
            if isinstance(annotation, TypeVariable):
                inner = TypeVariable("referent", token)
                self.variables.append(inner)
                reference = ReferenceType(inner, mutable)
                self.require(annotation, reference, token)
                annotation = reference
            if not isinstance(annotation, ReferenceType) or annotation.mutable != mutable:
                raise token.error("reference pattern must match the reference's mutability")
            inner, bindings = self.prepare_pattern(operand, annotation.inner)
            ctor = Constructor("&", annotation, "reference", (("0", annotation.inner),))
            return Pattern(tag=ctor, children=(inner,)), bindings
        if isinstance(annotation, ReferenceType) and structural:
            inner, bindings = self.prepare_pattern(expr, annotation.inner, borrowed=annotation.mutable)
            ctor = Constructor("&", annotation, "reference", (("0", annotation.inner),))
            return Pattern(tag=ctor, children=(inner,)), bindings
        if kind == "wild":
            return Pattern(), {}
        if kind == "rest":
            raise token.error("'..' is only allowed within a tuple, array, or record pattern")
        if kind == "literal":
            actual, value = args
            if is_str_reference(actual) and annotation == "str":
                actual = "str"
            hint = self.resolve(annotation)
            if expr.unsuffixed and hint in INTEGER_TYPES:
                actual = hint
            if actual in INTEGER_TYPES:
                checked(value, token, actual)
            if actual in FLOAT_TYPES:
                if expr.unsuffixed and hint in FLOAT_TYPES:
                    actual = hint
                if math.isnan(value):
                    raise token.error("NaN cannot appear in a pattern")
                value = float_literal_value(value, actual, token)
            self.require(actual, annotation, token)
            return Pattern(tag=(actual, value)), {}
        if kind == "pattern_range":
            first, last, inclusive = args
            scalar = self.resolve(annotation)
            if scalar in FLOAT_TYPES or (isinstance(scalar, TypeVariable) and any(
                endpoint and endpoint.args[0] in FLOAT_TYPES for endpoint in (first, last)
            )):
                if scalar not in FLOAT_TYPES:
                    scalar = next(endpoint.args[0] for endpoint in (first, last)
                                  if endpoint and endpoint.args[0] in FLOAT_TYPES)
                self.require(scalar, annotation, token)
                endpoints = []
                for endpoint in (first, last):
                    if endpoint is None:
                        endpoints.append(None)
                        continue
                    actual, value = endpoint.args
                    actual = scalar if endpoint.unsuffixed and actual in FLOAT_TYPES else actual
                    self.require(actual, scalar, endpoint.token)
                    if math.isnan(value):
                        raise endpoint.token.error("NaN cannot appear in a range pattern")
                    endpoints.append(float_literal_value(value, scalar, endpoint.token))
                start, stop = endpoints
                if start is not None and stop is not None and (start > stop or start == stop and not inclusive):
                    raise token.error("empty floating-point range pattern")
                return Pattern(tag=FloatRange(start, stop, inclusive)), {}
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
                reference_mutability = args[1] if len(args) > 1 else borrowed if not args[0] else None
                if reference_mutability is not None:
                    reference = ReferenceType(annotation, reference_mutability)
                    self.check_type(reference, token)
                    return Pattern(binding=token.key, by_reference=True), {token.key: (reference, False)}
                if isinstance(self.resolve(annotation), SliceType):
                    raise token.error("cannot bind an unsized slice value")
                if isinstance(annotation, DynType):
                    raise token.error("cannot bind an unsized trait object value")
                if annotation == "str":
                    raise token.error("cannot bind an unsized str value")
                return Pattern(binding=token.key), {token.key: (annotation, args[0])}
            if args[0] or len(args) > 1:
                raise token.error(f"cannot shadow unit struct {token.text!r}")
            kind = "pattern_unit"
            args = ()
        style = kind.removeprefix("pattern_")
        if style == "array":
            if isinstance(annotation, TypeVariable) and all(child.kind != "rest" for child in args):
                inferred = ArrayType(self.fill_holes(None, token), ConstValue(len(args), "usize"))
                self.require(inferred, annotation, token)
                annotation = inferred
            def is_rest(child):
                return child.kind == "rest" or (child.kind == "pattern_binding" and child.args[1].kind == "rest")
            if (isinstance(annotation, ArrayType) and isinstance(annotation.length, ConstValue)
                    and any(child.kind == "pattern_binding" and child.args[1].kind == "rest" for child in args)):
                # `[head, tail @ ..]` on an array binds the middle as a shorter array.
                rest = [index for index, child in enumerate(args) if is_rest(child)]
                prefix, suffix = rest[0], len(args) - rest[0] - 1
                name = args[rest[0]].args[0].token
                middle = ArrayType(annotation.element, ConstValue(annotation.length.value - prefix - suffix, "usize"))
                view = ReferenceType(middle, bool(borrowed)) if borrowed is not None else middle
                rest_bindings = {name.key: (view, False)}
                ctor = SlicePattern(SliceType(annotation.element), prefix, suffix, name.key, borrowed is None)
                assignments = enumerate(child for child in args if not is_rest(child))
            elif isinstance(annotation, SliceType):
                def is_rest(child):
                    return child.kind == "rest" or (child.kind == "pattern_binding" and child.args[1].kind == "rest")
                rest = [index for index, child in enumerate(args) if is_rest(child)]
                if len(rest) > 1:
                    raise args[rest[1]].token.error("a slice pattern can contain only one '..'")
                prefix = rest[0] if rest else len(args)
                suffix = len(args) - prefix - 1 if rest else None
                rest_binding = None
                rest_bindings = {}
                if rest and args[rest[0]].kind == "pattern_binding":
                    name = args[rest[0]].args[0].token
                    rest_binding = name.key
                    view = ReferenceType(annotation, bool(borrowed))
                    rest_bindings = {name.key: (view, False)}
                ctor = SlicePattern(annotation, prefix, suffix, rest_binding)
                assignments = enumerate(child for child in args if not is_rest(child))
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
                if not isinstance(annotation, ReferenceType):
                    self.obligation(annotation, "Copy", token)
            bindings.update(rest_bindings)
        return Pattern(tag=ctor, children=tuple(children)), bindings

    def match_type(self, expr, scope, expected=None):
        scrutinee, arms = expr.args
        annotation = self.type_of(scrutinee, scope)
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
            source, iterable, _ = args
            if iterable.kind == "range":
                start, stop, _ = iterable.args
                first, last = self.operand_types(start, stop, scope, same_type=True)
                literal_variables = [kind for kind in (self.resolve(first), self.resolve(last))
                                     if isinstance(kind, TypeVariable) and kind.integer_literal]
                if "char" in {self.resolve(first), self.resolve(last)}:
                    item = "char"
                else:
                    item = literal_variables[0] if len(literal_variables) == 2 else self.integer_pair(first, last)
                self.require(first, item, token)
                self.require(last, item, token)
                if NEVER in {first, last}:
                    test = NEVER
            else:
                annotation = self.type_of(iterable, scope)
                item = self.program.iteration_item(self.resolve(annotation))
                if item is None:
                    raise iterable.token.error(f"cannot iterate over {annotation}")
            pattern, bindings = self.prepare_pattern(source, item)
            missing = uncovered([(pattern,)], (self.resolve(item),), self.program)
            if missing is not None:
                raise source.token.error(f"refutable for pattern, missing {show_missing(missing[0])}")
            self.program.patterns[id(source)] = pattern, bindings
            scope = Scope(scope, {name: Binding(kind, mutable) for name, (kind, mutable) in bindings.items()})
        self.loops.append(target)
        try:
            if kind == "while_chain":
                target.in_condition = True
                test, scope = self.chain_type(args[0], scope)
                target.in_condition = False
            elif kind in {"while", "while_let"}:
                condition = args[0]
                target.in_condition = True
                test = self.type_of(condition, scope)
                target.in_condition = False
                if kind == "while":
                    self.require(test, "bool", condition.token)
                else:
                    source = args[1]
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

    def chain_type(self, conditions, scope):
        inner = Scope(scope)
        first = "bool"
        for index, (source, expr) in enumerate(conditions):
            actual = self.type_of(expr, inner, "bool" if source is None else None)
            if index == 0 and actual == NEVER:
                first = NEVER
            if source is None:
                self.require(actual, "bool", expr.token)
            else:
                pattern, bindings = self.prepare_pattern(source, actual)
                self.program.patterns[id(source)] = pattern, bindings
                inner.bindings.update({name: Binding(kind, mutable) for name, (kind, mutable) in bindings.items()})
        return first, inner

    def type_of(self, expr, scope, expected=None):
        if id(expr) in self.pretyped:
            return self.pretyped.pop(id(expr))
        if id(expr) in self.program.call_functions or expr.kind in {
            "length", "is_empty", "float_method", "iterator", "reverse_iterator", "map_iterator",
            "string_iterator", "iterator_terminal", "lazy_adapter", "enum_method", "string_method",
            "vector_method", "to_string", "scalar_cast", "range_value", "chars_as_str", "enum_cast"
        } and id(expr) in self.program.expr_types:
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
            try:
                result = self.check_expr(expr, scope, expected)
            except AssertionError:
                # Expressions already lowered to an internal kind keep the type found the first time.
                if id(expr) not in self.program.expr_types:
                    raise
                return self.resolve(self.program.expr_types[id(expr)])
        result = self.resolve(result)
        expected = self.resolve(expected)
        if (isinstance(result, AppliedType) and isinstance(expected, AppliedType) and result.name == expected.name
                and result.name in POINTER_TYPES and result != expected and isinstance(expected.arguments[0], DynType)
                and (is_box(result) or not isinstance(self.resolve(result.arguments[0]), DynType))):
            self.check_dyn_conversion(result.arguments[0], expected.arguments[0], expr.token)
            result = expected
        result = self.coerce_reference(expr, result, expected)
        self.program.expr_types[id(expr)] = result
        return result

    def coerce_reference(self, expr, actual, expected):
        if (not isinstance(expected, ReferenceType) or not isinstance(actual, ReferenceType)
                or actual.inner == expected.inner
                or (expected.mutable and not actual.mutable)):
            return actual
        target = expected.inner
        inner = self.resolve(actual.inner)
        if isinstance(inner, ReferenceType) and not isinstance(self.resolve(target), (ReferenceType, TypeVariable)):
            # Deref coercion: &&T becomes &T by reading through the outer reference.
            original = replace(expr)
            for table in (self.program.call_types, self.program.call_functions, self.program.dispatches):
                if id(expr) in table:
                    table[id(original)] = table.pop(id(expr))
            self.program.expr_types[id(original)] = actual
            expr.kind, expr.args, expr.label = "deref", (original, False), None
            self.program.expr_types[id(expr)] = inner
            return self.coerce_reference(expr, inner, expected)
        if isinstance(inner, AppliedType) and inner.name == BOX and self.resolve(target) != inner and not (
                isinstance(self.resolve(target), (TypeVariable, DynType))):
            # &Box<T> and &Rc<T> coerce to &T.
            original = replace(expr)
            for table in (self.program.call_types, self.program.call_functions, self.program.dispatches):
                if id(expr) in table:
                    table[id(original)] = table.pop(id(expr))
            self.program.expr_types[id(original)] = actual
            pointee = Expr("box_deref", expr.token, (Expr("deref", expr.token, (original, False)),))
            self.program.expr_types[id(pointee.args[0])] = inner
            self.program.expr_types[id(pointee)] = inner.arguments[0]
            expr.kind, expr.args, expr.label = "borrow", (pointee, actual.mutable), None
            coerced = ReferenceType(inner.arguments[0], actual.mutable)
            self.program.expr_types[id(expr)] = coerced
            return self.coerce_reference(expr, coerced, expected)
        if target == "str" and actual.inner == STRING and not expected.mutable:
            conversion = "string_reference"
        elif isinstance(target, SliceType) and (isinstance(actual.inner, ArrayType) or is_vector(actual.inner)):
            element = actual.inner.arguments[0] if is_vector(actual.inner) else actual.inner.element
            self.require(element, target.element, expr.token)
            conversion = "slice_reference"
        elif isinstance(target, DynType):
            self.check_dyn_conversion(actual.inner, target, expr.token)
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

    def check_dyn_conversion(self, actual, target, token):
        self.check_dyn(target, token)
        self.obligation(actual, target.trait, token)
        if target.trait in CALLABLE_TRAITS:
            self.require_callable(actual, callable_signature(target), token)
            return
        for name, kind in target.bindings:
            trait = self.program.associated_trait(target.trait, name)
            if trait is None:
                raise token.error(f"ambiguous associated type {name!r}")
            self.require(AssociatedType(actual, name, trait.name.text), kind, token)

    def require_place(self, expr, *, write=False, allow_temporary=False):
        place = place_root(expr)
        if place.kind == "static_item" or place.kind == "lazy_static":
            if write and not (place.kind == "lazy_static" and place.args[0].mutable):
                raise expr.token.error("cannot mutate an immutable static item")
            return
        if place.kind == "lazy_const":
            return
        if place.kind == "box_deref":
            self.require_place(place.args[0], write=write, allow_temporary=allow_temporary)
            return
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

    def assignment_pattern(self, expr, scope, writes, operator, value=None):
        original = expr
        expr = ungroup(expr)
        if expr is not original and expr.kind == "rest":
            expr = Expr("tuple", original.token, (expr,))
        kind, token, args = expr.kind, expr.token, expr.args
        if kind in {"wild", "rest"}:
            return expr, None
        if kind == "literal" and args[0] == "()":
            return expr, "()"
        if kind in {"tuple", "array"}:
            literal = ungroup(value) if value is not None else None
            inputs = {}
            if kind == "tuple" and literal is not None and literal.kind == "tuple":
                positions = self.positional_patterns(args, len(literal.args), token, "tuple assignment")
                inputs = {id(target): literal.args[index] for index, target in positions}
            entries = [self.assignment_pattern(target, scope, writes, operator, inputs.get(id(target)))
                       for target in args]
            patterns = tuple(pattern for pattern, _ in entries)
            rest = any(pattern.kind == "rest" for pattern in patterns)
            if kind == "tuple":
                pattern = Expr("pattern_anonymous_tuple", token, patterns)
                if rest and not inputs:
                    return pattern, None
                length = len(literal.args) if rest else len(entries)
                fields = [None] * length
                hints = {id(child): hint for child, hint in entries}
                for index, child in self.positional_patterns(patterns, length, token, "tuple assignment"):
                    fields[index] = hints[id(child)]
                return pattern, TupleType(tuple(self.fill_holes(field, token) for field in fields))
            element = next((hint for _, hint in entries if hint is not None), TypeVariable("array element", token))
            for _, hint in entries:
                if hint is not None:
                    self.require(hint, element, token)
            length = TypeVariable("array length", token, "usize") if rest else ConstValue(len(entries), "usize")
            return Expr("pattern_array", token, patterns), ArrayType(element, length)
        if kind in {"record", "call"}:
            style = "record" if kind == "record" else "tuple"
            ctor = self.constructor(expr, style)
            if style == "record":
                fields, rest = args
                if isinstance(rest, Expr):
                    raise rest.token.error("a struct update value is only allowed on the right side")
                indices = self.field_indices(ctor, fields, rest, token)
                patterns = []
                for index, (name, value) in zip(indices, fields):
                    pattern, hint = self.assignment_pattern(value, scope, writes, operator)
                    if hint is not None:
                        self.require(hint, ctor.fields[index][1], value.token)
                    patterns.append((name, pattern))
                result = Expr("pattern_record", token, (tuple(patterns), rest), type_arguments=expr.type_arguments)
            else:
                patterns = [Expr("wild", token) for _ in ctor.fields]
                for index, value in self.positional_patterns(args, len(ctor.fields), token, ctor.path):
                    pattern, hint = self.assignment_pattern(value, scope, writes, operator)
                    if hint is not None:
                        self.require(hint, ctor.fields[index][1], value.token)
                    patterns[index] = pattern
                result = Expr("pattern_tuple", token, tuple(patterns), type_arguments=expr.type_arguments)
            return result, self.resolve(ctor.annotation)
        ctor = self.program.constructors.get(token.text) if kind == "name" else None
        if ctor and ctor.style == "unit":
            return Expr("pattern_unit", token, type_arguments=expr.type_arguments), ctor.annotation
        annotation = self.type_of(expr, scope)
        self.require_place(expr, write=True)
        name = replace(token, text=f"@assignment:{len(writes)}")
        writes.append(Expr("assign", operator, (expr, Expr("name", name))))
        return Expr("bind", name, (False,)), annotation

    def check_expr(self, expr, scope, expected=None):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "closure":
            return self.closure_type(expr, scope, expected)
        if kind == "indirect_call":
            callee = self.type_of(args[0], scope)
            return self.indirect_type(expr, scope, expected, callee)
        if kind in {"function_pointer", "closure_value", "box"}:
            return self.program.expr_types[id(expr)]
        if kind in {"lazy_const", "lazy_static"}:
            return args[0].annotation
        if kind == "rest":
            expr.kind, expr.args = "range_value", (None, None, False)
            return self.check_expr(expr, scope, expected)
        if kind == "range_value":
            start, end, inclusive = args
            names = {(True, True): "RangeInclusive" if inclusive else "Range", (True, False): "RangeFrom",
                     (False, True): "RangeToInclusive" if inclusive else "RangeTo", (False, False): "RangeFull"}
            name = MODULE_SEPARATOR.join(("std", "ops", names[start is not None, end is not None]))
            bounds = [item for item in (start, end) if item is not None]
            if not bounds:
                return name
            hint = self.resolve(expected).arguments[0] if isinstance(self.resolve(expected), AppliedType) else None
            if hint is None or isinstance(hint, TypeVariable):
                hint = next(filter(None, map(self.literal_variable, bounds)), hint)
            first = self.type_of(bounds[0], scope, hint)
            if len(bounds) == 2:
                second = self.type_of(bounds[1], scope, first)
                self.require(second, first, token)
            if NEVER in {self.resolve(self.program.expr_types[id(item)]) for item in bounds}:
                return NEVER
            return AppliedType(name, (first,))
        if kind == "wild":
            raise token.error(f"{token.text!r} is only allowed in an assignment target here")
        if kind == "box_deref":
            return self.type_of(args[0], scope).arguments[0]
        if kind == "try":
            value = args[0]
            enclosing = self.resolve(self.result)
            hint = None
            if expected is not None and isinstance(enclosing, AppliedType) and enclosing.name in {OPTION, RESULT}:
                error = ()
                if enclosing.name == RESULT:
                    error = (TypeVariable("error", token),)
                    self.variables.append(error[0])
                hint = AppliedType(enclosing.name, (expected, *error))
            actual = self.type_of(value, scope, hint)
            if actual == NEVER:
                return NEVER
            if not isinstance(actual, AppliedType) or actual.name not in {OPTION, RESULT}:
                raise token.error("? requires Option or Result")
            if hint is not None and hint.name == RESULT and actual.name == RESULT:
                if isinstance(self.resolve(hint.arguments[1]), TypeVariable):
                    self.require(hint.arguments[1], actual.arguments[1], token)
            if not isinstance(enclosing, AppliedType) or actual.name != enclosing.name:
                raise token.error("? requires a function returning the same enum family")
            if actual.name == RESULT:
                source, target = self.resolve(actual.arguments[1]), self.resolve(enclosing.arguments[1])
                if isinstance(source, TypeVariable) or isinstance(target, TypeVariable) or source == target:
                    self.require(source, target, token)
                else:
                    self.error_conversions.append((source, target, token, expr))
            expr.args = (value, enclosing)
            return self.resolve(actual.arguments[0])
        if kind == "macro_definition":
            scope.remember(token)
            self.macro_labels[token.definition] = tuple(self.loops)
            return "()"
        if kind == "cast":
            value, target = args
            if isinstance(target, ReferenceType) and (isinstance(target.inner, (DynType, SliceType)) or is_str_reference(target)):
                result = self.type_of(value, scope, target)
                self.require_argument(result, target, token)
                expr.kind, expr.args = "group", (value,)
                return result
            if target in INTEGER_TYPES and not self.unsuffixed_number(value):
                actual = self.resolve(self.type_of(value, scope))
                layout = self.program.layout(actual) if not isinstance(actual, (TypeVariable, ReferenceType)) else None
                if (layout and actual not in PRIMITIVES and isinstance(actual, str) and len(layout[1]) >= 1
                        and all(ctor.style == "unit" for ctor in layout[1]) and "::" in layout[1][0].path):
                    expr.kind = "enum_cast"
                    return target
                self.pretyped[id(value)] = actual
            if isinstance(target, AppliedType) and target.name in POINTER_TYPES and isinstance(target.arguments[0], DynType):
                result = self.type_of(value, scope, target)
                self.require_argument(result, target, token)
                expr.kind, expr.args = "group", (value,)
                return result
            if target in SCALAR_BOUNDS or target in FLOAT_TYPES:
                hint = ("u8" if target == "char" else target) if self.unsuffixed_number(value) else None
                actual = self.type_of(value, scope, hint)
                if actual == NEVER:
                    expr.kind, expr.args = "group", (value,)
                    return NEVER
                if isinstance(actual, TypeVariable):
                    self.scalar_casts.append((actual, target, token))
                elif not scalar_cast_supported(actual, target):
                    raise token.error(f"cannot cast {actual} to {target}")
                expr.kind = "scalar_cast"
                return target
            raise token.error("supported casts convert numbers, characters, or borrowed references")
        if kind in {"dyn_reference", "slice_reference", "string_reference"}:
            return self.program.expr_types[id(expr)]
        if kind == "literal":
            annotation, value = args
            hint = self.resolve(expected)
            if annotation in FLOAT_TYPES:
                if expr.unsuffixed and hint in FLOAT_TYPES:
                    annotation = hint
                elif (expr.unsuffixed and isinstance(hint, TypeVariable)
                      and (hint.float_literal or hint in self.deferred_binding_types)):
                    self.float_literals.append((hint, value, token))
                    return hint
                elif expr.unsuffixed and (hint is None or isinstance(hint, TypeVariable)):
                    variable = TypeVariable("float literal", token, float_literal=True)
                    self.variables.append(variable)
                    if isinstance(hint, TypeVariable):
                        self.require(variable, hint, token)
                    self.float_literals.append((variable, value, token))
                    return variable
                float_literal_value(value, annotation, token)
                return annotation
            if expr.unsuffixed and hint in INTEGER_TYPES:
                annotation = hint
            elif (expr.unsuffixed and annotation in INTEGER_TYPES and isinstance(hint, TypeVariable)
                  and (hint.integer_literal or hint in self.deferred_binding_types)):
                self.integer_literals.append((hint, value, token))
                return hint
            elif expr.unsuffixed and annotation in INTEGER_TYPES and (hint is None or isinstance(hint, TypeVariable)):
                # An unsuffixed literal is {integer} until inference settles it, defaulting to i32.
                variable = TypeVariable("integer literal", token, integer_literal=True)
                self.variables.append(variable)
                if isinstance(hint, TypeVariable):
                    self.require(variable, hint, token)
                self.integer_literals.append((variable, value, token))
                return variable
            if annotation in INTEGER_TYPES and not OVERFLOWING_LITERALS[0]:
                checked(value, token, annotation)
            return annotation
        if kind == "byte_string":
            return ReferenceType(ArrayType("u8", ConstValue(len(args), "usize")), False, "'static")
        if kind == "inline_const":
            hint = self.resolve(expected)
            hint = hint if hint in CONST_VALUE_TYPES else None
            parser = Parser("", modules=self.modules, unit=self.modules.units[token.module])
            value = parser.const_expression(args[0], hint)
            annotation = const_kind(value) or hint
            if annotation is None:
                annotation = "f64" if isinstance(value.value, float) else "i32"
            check_constant_kind(value, annotation, token)
            if isinstance(value, ConstValue):
                value = replace(value, kind=annotation)
            expr.kind, expr.args = "const_value", (value,)
            return annotation
        if kind in {"const_value", "static_item"}:
            return const_kind(args[0])
        if kind in {"array", "array_repeat", "vector", "vector_repeat"}:
            return self.sequence_type(expr, scope, expected)
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
            if kind == "index" and isinstance(annotation, ArrayType) and token.source_file != "<std>":
                self.constant_lints.append((expr, annotation.length))
            while isinstance(annotation, ReferenceType):
                base = Expr("deref", base.token, (base, False))
                expr.args = (base, *bounds)
                annotation = self.type_of(base, scope)
            string_range = kind == "range_index" and annotation in {"str", STRING}
            while self.derefs(annotation) and not (
                    self.implements(self.resolve(annotation), MODULE_SEPARATOR.join(("std", "ops", "Index")))):
                base, annotation = self.deref_call(base, annotation, scope, token)
                expr.args = (base, *bounds)
            if kind == "index" and annotation != NEVER and not (
                    isinstance(annotation, (ArrayType, SliceType)) or is_vector(annotation)):
                return self.index_call(expr, scope, base, bounds[0], annotation)
            if annotation != NEVER and not (isinstance(annotation, (ArrayType, SliceType)) or is_vector(annotation) or string_range):
                raise token.error(f"cannot index {annotation}")
            diverges = annotation == NEVER
            for index in (bounds if kind == "index" else bounds[:2]):
                if index is not None:
                    actual = self.type_of(index, scope, "usize")
                    self.require(actual, "usize", index.token)
                    diverges |= actual == NEVER
            if diverges:
                return NEVER
            if string_range:
                return "str"
            element = annotation.arguments[0] if is_vector(annotation) else annotation.element
            return element if kind == "index" else SliceType(element)
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
            arity = self.path_arity(expr, scope)
            if arity is not None:
                # A path to a constructor or associated function used as a value becomes a closure.
                names = [replace(token, text=f"@argument{index}", context=(*token.context, "@path"))
                         for index in range(arity)]
                body = Expr("call", token, tuple(Expr("name", name) for name in names),
                            type_arguments=expr.type_arguments, qualified=expr.qualified)
                expr.kind, expr.args = "closure", ([(name, False, None) for name in names], None, body, False)
                expr.type_arguments = expr.qualified = None
                return self.closure_type(expr, scope, expected)
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
            while isinstance(annotation, ReferenceType):
                value = Expr("deref", value.token, (value, False))
                annotation = self.type_of(value, scope)
            while is_box(annotation) or (not self.has_field(annotation, name) and self.derefs(annotation)):
                if is_box(annotation):
                    value = Expr("box_deref", value.token, (value,))
                    annotation = self.type_of(value, scope)
                else:
                    value, annotation = self.deref_call(value, annotation, scope, value.token)
            expr.args = (value, name)
            return self.field_type(annotation, name)
        if kind == "borrow":
            value, mutable = args
            hint = expected.inner if isinstance(expected, ReferenceType) else None
            if hint == "str" or isinstance(self.resolve(hint), (SliceType, DynType)):
                hint = None
            if mutable:
                self.mark_mutable_place(value)
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
            if is_box(annotation):
                expr.kind, expr.args = "box_deref", (value,)
                return annotation.arguments[0]
            if not isinstance(annotation, ReferenceType) and self.derefs(annotation):
                target, inner = self.deref_call(value, annotation, scope, token)
                expr.args = target.args
                return inner
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
            fields, update = args
            if update is True:
                raise token.error("'..' requires an assignment target or a struct update value")
            ctor = self.constructor(expr, "record", expected)
            indices = self.field_indices(ctor, fields, bool(update), token)
            actual = [self.type_of(value, scope, ctor.fields[index][1])
                      for index, (_, value) in zip(indices, fields)]
            for index, kind in zip(indices, actual):
                self.require(kind, ctor.fields[index][1], token)
            if update:
                if self.program.constructors[token.text].annotation != token.text:
                    raise token.error("struct update requires a struct constructor")
                for name, _ in ctor.fields:
                    self.visible_field(ctor, replace(token, text=name))
                base = self.type_of(update, scope, ctor.annotation)
                self.require(base, ctor.annotation, update.token)
                actual.append(base)
            return NEVER if NEVER in actual else self.resolve(ctor.annotation)
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
        if kind in {"let", "parameter_let", "assignment_let"}:
            source, annotation, value, otherwise = args
            parameter = kind == "parameter_let"
            expr.kind = "let"
            if annotation is None and value is None:
                annotation = TypeVariable(source.token.text, source.token)
                self.variables.append(annotation)
            if annotation is not None:
                annotation = self.fill_holes(annotation, token)
                self.check_type(annotation, token)
            hint = annotation
            if annotation is None and value is not None:
                literal = ungroup(value)
                signed = literal.kind == "unary" and literal.token.text == "-"
                number = ungroup(literal.args[0]) if signed else literal
                if number.kind == "literal" and number.unsuffixed and number.args[0] in NUMBER_TYPES:
                    floating = number.args[0] in FLOAT_TYPES
                    hint = TypeVariable("float literal" if floating else "integer literal", number.token,
                                        integer_literal=not floating, float_literal=floating)
                    self.variables.append(hint)
                    if signed:
                        literal.kind, literal.args = "literal", (number.args[0], -number.args[1])
                        literal.unsuffixed = True
            actual = self.type_of(value, scope, hint) if value else annotation
            if kind == "assignment_let" and isinstance(actual, ReferenceType) and source.kind != "wild":
                raise token.error("destructuring assignment does not implicitly dereference its source")
            if annotation:
                self.require_argument(actual, annotation, token)
                expr.args = (source, annotation, value, otherwise)
                self.ascriptions.append(expr)
            annotation = self.resolve(annotation or actual)
            if isinstance(annotation, ReferenceType):
                if value is not None and otherwise and source.kind in {"bind", "wild"}:
                    raise token.error("reference let requires a simple initialized binding")
            pattern, bindings = self.prepare_pattern(source, annotation)
            if value is None:
                for kind, _ in bindings.values():
                    if isinstance(kind, TypeVariable):
                        self.deferred_binding_types.add(kind)
            self.program.patterns[id(source)] = (pattern, bindings)
            if otherwise:
                self.require(self.type_of(otherwise, scope), NEVER, otherwise.token)
            else:
                missing = uncovered([(pattern,)], (self.resolve(annotation),), self.program)
                if missing is not None:
                    position = "parameter" if parameter else "let"
                    raise token.error(f"refutable {position} pattern, missing {show_missing(missing[0])}")
            for name, (kind, mutable) in bindings.items():
                if parameter and (name in scope.bindings or name in scope.parent.bindings):
                    raise token.error(f"duplicate parameter {token.binding(name).text!r}")
                scope.bindings[name] = Binding(kind, mutable)
            return NEVER if actual == NEVER else "()"
        if kind == "discard":
            value, requires_unit = args
            actual = self.type_of(value, scope)
            if requires_unit:
                self.require(actual, "()", token)
            return NEVER if actual == NEVER else "()"
        if kind in {"if", "if_chain"}:
            condition, yes, no = args
            if kind == "if_chain":
                test, inner = self.chain_type(condition, scope)
            else:
                test = self.type_of(condition, scope, "bool")
                self.require(test, "bool", condition.token)
                inner = scope
            left = self.type_of(yes, inner, expected)
            right_hint = expected if expected is not None else (left if left != NEVER else None)
            right = self.type_of(no, scope, right_hint) if no else "()"
            if left != NEVER and not (shared_coercion(left, right) or shared_coercion(right, left)):
                self.require(right, left, token)
            if test == NEVER:
                return NEVER
            return right if left == NEVER or shared_coercion(left, right) else left
        if kind in {"while", "while_let", "while_chain", "loop", "for"}:
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
            hint = hint if hint in CONST_VALUE_TYPES else None
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
            if self.overloadable(actual):
                trait, method = UNARY_TRAITS[token.text]
                return self.operator_call(expr, scope, expected, trait, method, (operand,), actual)
            if actual in FLOAT_TYPES:
                if token.text != "-":
                    raise token.error(f"cannot negate {actual} with '!'")
                return actual
            if token.text == "!" and actual == "bool":
                return "bool"
            if actual in INTEGER_TYPES:
                if token.text == "-" and INTEGER_BOUNDS[actual][0] == 0:
                    raise token.error(f"cannot negate {actual} with '-'")
                return actual
            if isinstance(actual, TypeVariable):
                self.unary_operands.append((actual, token))
                return actual
            self.numeric_operands.append((actual, "i32", token))
            return "i32"
        if kind == "assign":
            left, right = args
            target = ungroup(left)
            ctor = self.program.constructors.get(target.token.text) if target.kind == "name" else None
            parenthesized_rest = left.kind == "group" and target.kind == "rest"
            pattern_target = (parenthesized_rest or target.kind in {"wild", "tuple", "array", "record", "call"}
                              or target.kind == "literal" and target.args[0] == "()"
                              or ctor and ctor.style == "unit")
            if token.text == "=" and pattern_target:
                writes = []
                pattern, hint = self.assignment_pattern(left, scope, writes, token, right)
                # Match the source once, then write destinations in source order.
                bind = Expr("assignment_let", token, (pattern, hint, right, None))
                expr.kind, expr.args = "block", ([bind, *writes], None)
                return self.check_expr(expr, scope, expected)
            self.mark_mutable_place(left)
            if token.text != "=" and token.text[:-1] in OPERATOR_TRAITS:
                left_type = self.resolve(self.program.expr_types.get(id(left)) or self.type_of(left, scope))
                if self.overloadable(left_type):
                    trait, method = OPERATOR_TRAITS[token.text[:-1]]
                    self.require_place(left, write=True)
                    CaptureContext.mark_mutable(scope, left)
                    return self.operator_call(expr, scope, expected, trait + "Assign", method + "_assign",
                                              args, left_type)
                self.program.expr_types[id(left)] = left_type
            annotation = self.program.expr_types.get(id(left))
            if annotation is None:
                annotation = self.type_of(left, scope)
            annotation = self.resolve(annotation)
            self.require_place(left, write=True)
            CaptureContext.mark_mutable(scope, left)
            if isinstance(annotation, SliceType):
                raise token.error("cannot assign an unsized slice value")
            if annotation == "str":
                raise token.error("cannot assign an unsized str value")
            operator = token.text.removesuffix("=")
            hint = STR_REFERENCE if annotation == STRING and token.text == "+=" else annotation if token.text == "=" or (
                operator not in SHIFTS and self.unsuffixed_number(right)
            ) else None
            actual = self.type_of(right, scope, hint)
            if token.text == "=":
                self.require(actual, annotation, token)
            else:
                if operator != "+" or annotation != STRING:
                    while isinstance(actual, ReferenceType) and self.numeric_like(actual.inner):
                        right = Expr("deref", right.token, (right, False))
                        self.program.expr_types[id(right)] = self.resolve(actual.inner)
                        expr.args = (expr.args[0], right)
                        actual = self.resolve(actual.inner)
                self.binary_type(replace(token, text=operator), annotation, actual)
            return NEVER if NEVER in {annotation, actual} else "()"
        if kind == "binary":
            if token.text in {"+", "-", "*", "/", "%", *SHIFTS} and token.source_file != "<std>":
                self.constant_lints.append((expr, None))
            hint = self.resolve(expected)
            if token.text in SHIFTS:
                left, right = args
                left_hint = hint if hint in INTEGER_TYPES and self.unsuffixed_number(left) else None
                operands = [left, right]
                types = [self.type_of(left, scope, left_hint), self.type_of(right, scope)]
                for index in range(2):
                    while isinstance(types[index], ReferenceType) and self.numeric_like(types[index].inner):
                        operands[index] = Expr("deref", operands[index].token, (operands[index], False))
                        self.program.expr_types[id(operands[index])] = types[index].inner
                        types[index] = self.resolve(types[index].inner)
                expr.args = tuple(operands)
                return self.binary_type(token, *types)
            if token.text in OPERATOR_TRAITS and not self.unsuffixed_number(args[0]):
                left_type = self.type_of(args[0], scope)
                if self.overloadable(left_type):
                    trait, method = OPERATOR_TRAITS[token.text]
                    return self.operator_call(expr, scope, expected, trait, method, args, left_type)
                self.pretyped[id(args[0])] = left_type
            if token.text in {"&&", "||"}:
                hint = "bool"
            elif (hint not in NUMBER_TYPES and not (hint == "bool" and token.text in BITWISE)) or token.text in COMPARE:
                hint = None
            left, right = self.operand_types(*args, scope, hint, same_type=token.text in COMPARE)
            if token.text not in COMPARE and token.text not in {"&&", "||"}:
                operands = list(args)
                types = [left, right]
                for index in range(2):
                    while isinstance(types[index], ReferenceType) and self.numeric_like(types[index].inner):
                        operands[index] = Expr("deref", operands[index].token, (operands[index], False))
                        self.program.expr_types[id(operands[index])] = types[index].inner
                        types[index] = self.resolve(types[index].inner)
                if operands != list(args):
                    expr.args = tuple(operands)
                    args = expr.args
                    left, right = types
                    if isinstance(left, TypeVariable) or isinstance(right, TypeVariable):
                        self.require(left, right, token)
            if left == STRING and token.text == "+":
                right = self.coerce_reference(args[1], right, STR_REFERENCE)
                self.program.expr_types[id(args[1])] = right
            if (token.text in OPERATOR_TRAITS and not self.overloadable(left) and self.overloadable(right)
                    and not isinstance(self.resolve(left), TypeVariable) or (
                        token.text in OPERATOR_TRAITS and isinstance(self.resolve(left), TypeVariable)
                        and self.resolve(left).float_literal | self.resolve(left).integer_literal
                        and self.overloadable(right))):
                # `2.0 * v` uses an impl like `Mul<Vec2> for f64`.
                trait_name, method = OPERATOR_TRAITS[token.text]
                trait = MODULE_SEPARATOR.join(("std", "ops", trait_name))
                left_type = self.resolve(left)
                if isinstance(left_type, TypeVariable):
                    owners = [impl.target for impl in self.program.impls if impl.trait == trait
                              and impl.target in PRIMITIVES and any(
                                  self.resolve(right) == argument for argument in impl.trait_arguments)]
                    if len(owners) == 1:
                        self.require(left_type, owners[0], token)
                        left_type = owners[0]
                if self.program.implementation(left_type, trait, arguments_wanted=(self.resolve(right),)):
                    self.pretyped[id(args[1])] = right
                    return self.operator_call(expr, scope, expected, trait_name, method, args, left_type)
            if token.text in COMPARE and NEVER not in {left, right}:
                peeled_left, peeled_right = self.resolve(left), self.resolve(right)
                while isinstance(peeled_left, ReferenceType) and isinstance(peeled_right, ReferenceType):
                    peeled_left, peeled_right = self.resolve(peeled_left.inner), self.resolve(peeled_right.inner)
                trait = "PartialEq" if token.text in {"==", "!="} else "PartialOrd"
                if (peeled_left != peeled_right and not isinstance(peeled_left, TypeVariable)
                        and not isinstance(peeled_right, TypeVariable)
                        and self.program.implementation(peeled_left, trait, arguments_wanted=(peeled_right,))):
                    self.program.cross_compares[id(expr)] = (peeled_left, peeled_right, trait)
            return self.binary_type(token, left, right)
        if kind == "call":
            return self.call_type(expr, scope, expected)
        if kind in {"print", "panic", "format"}:
            return self.format_type(expr, scope)
        raise AssertionError(kind)

    def format_type(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        if not args:
            if kind == "panic":
                return NEVER
            if token.text in {"println", "eprintln"}:
                return "()"
            raise token.error("expected a format string literal")
        template = ungroup(args[0])
        if template.kind != "literal" or not is_str_reference(template.args[0]):
            raise template.token.error(f"{token.text}! requires a format string literal")
        values, named = [], {}
        for value in args[1:]:
            if value.kind == "assign" and value.token.text == "=" and value.args[0].kind == "name":
                name, value = value.args
                if name.token.text in named:
                    raise name.token.error(f"duplicate format argument {name.token.text!r}")
                named[name.token.text] = len(values)
            else:
                if named:
                    raise value.token.error("positional format arguments must precede named arguments")
            values.append(value)
        expr.args = (template, *values)
        used = set()
        next_position = 0

        def argument_index(argument):
            nonlocal next_position
            if argument is None:
                argument = next_position
                next_position += 1
            if isinstance(argument, str) and argument in named:
                argument = named[argument]
            if isinstance(argument, int):
                if argument >= len(values):
                    raise token.error("format argument count does not match format string")
                used.add(argument)
            return argument

        counts = {}
        parts = []
        for field in format_parts(template.args[1], template.token):
            if isinstance(field, str):
                parts.append(field)
                continue
            changes = {}
            for dimension in ("width", "precision"):
                count = getattr(field.spec, dimension)
                if isinstance(count, FormatCount):
                    argument = argument_index(count.argument)
                    counts[argument] = None
                    changes[dimension] = FormatCount(argument)
            parts.append(replace(field, argument=argument_index(field.argument), spec=replace(field.spec, **changes)))
        if len(used) != len(values):
            raise token.error("unused format argument")
        types = [self.type_of(value, scope, "usize" if index in counts else None) for index, value in enumerate(values)]
        resolved_counts = {}
        for argument in counts:
            resolved, actual = self.format_argument_type(argument, types, template.token, scope)
            self.require(actual, "usize", token)
            resolved_counts[argument] = FormatCount(resolved)
        typed_parts = []
        for field in parts:
            if isinstance(field, str):
                typed_parts.append(field)
                continue
            argument, actual = self.format_argument_type(field.argument, types, template.token, scope)
            if isinstance(actual, SliceType) or actual == "str":
                raise token.error("formatting an unsized slice requires a reference")
            while isinstance(actual, ReferenceType):
                actual = actual.inner
            if field.spec.kind == "" and actual == "()":
                raise token.error("() requires debug formatting")
            self.obligation(actual, "Debug" if field.spec.kind == "?" else "Display", token)
            width, precision = field.spec.width, field.spec.precision
            spec = replace(field.spec, width=resolved_counts[width.argument] if isinstance(width, FormatCount) else width,
                           precision=resolved_counts[precision.argument] if isinstance(precision, FormatCount) else precision)
            typed_parts.append(replace(field, argument=argument, spec=spec, annotation=actual))
        self.program.formats[id(expr)] = (token, typed_parts)
        if kind == "panic" or NEVER in types:
            return NEVER
        return STRING if kind == "format" else "()"

    def format_argument_type(self, argument, types, token, scope):
        if isinstance(argument, int):
            return argument, types[argument]
        name_token = replace(token, text=argument)
        binding = self.local_binding(scope, name_token)
        if binding is not None:
            return argument, binding.value
        parameter = self.parameters.get(argument)
        constant = parameter if parameter and parameter.const_type else (
            self.program.constants_in_scope.get(token.module, {}).get(argument))
        if constant is None:
            raise name_token.error(f"unknown binding {argument!r}")
        return constant, const_kind(constant)

    @staticmethod
    def unsuffixed_number(expr):
        expr = ungroup(expr)
        if expr.kind == "unary" and expr.token.text == "-":
            return Checker.unsuffixed_number(expr.args[0])
        if expr.kind == "binary" and expr.token.text in SHIFTS:
            return Checker.unsuffixed_number(expr.args[0])
        if expr.kind == "binary" and expr.token.text in {*ARITHMETIC, "/", "%"}:
            return all(map(Checker.unsuffixed_number, expr.args))
        return expr.unsuffixed

    def operand_types(self, left, right, scope, hint=None, *, same_type=False):
        left_literal, right_literal = self.unsuffixed_number(left), self.unsuffixed_number(right)
        if hint is None and left_literal and not right_literal:
            right_type = self.type_of(right, scope)
            peeled = self.peel_numeric_reference(right_type)
            literal_variable = isinstance(peeled, TypeVariable) and (peeled.integer_literal or peeled.float_literal)
            left_type = self.type_of(left, scope, peeled if peeled in NUMBER_TYPES or literal_variable else None)
        else:
            left_type = self.type_of(left, scope, hint if hint == "bool" or left_literal else None)
            right_hint = left_type if left_type != NEVER else hint
            if right_literal and not same_type:
                right_hint = self.peel_numeric_reference(right_hint)
            if same_type and ungroup(right).kind in {"array", "array_repeat"} and not isinstance(
                    self.resolve(right_hint), (ArrayType, TypeVariable)):
                right_hint = None
            right_type = self.type_of(right, scope, right_hint if hint == "bool" or same_type or right_literal else None)
        return left_type, right_type

    def peel_numeric_reference(self, annotation):
        annotation = self.resolve(annotation)
        while isinstance(annotation, ReferenceType) and self.numeric_like(annotation.inner):
            annotation = self.resolve(annotation.inner)
        return annotation

    @staticmethod
    def integer_pair(left, right):
        if left in INTEGER_TYPES:
            return left
        return right if right in INTEGER_TYPES else "i32"

    def sequence_type(self, expr, scope, expected):
        kind, token, args = expr.kind, expr.token, expr.args
        expected = self.resolve(expected)
        vector = kind in {"vector", "vector_repeat"}
        if isinstance(expected, SliceType) or (not vector and isinstance(expected, ArrayType)):
            element = expected.element
        elif vector and is_vector(expected):
            element = expected.arguments[0]
        else:
            element = None
        if args and args[0].unsuffixed and (element is None or isinstance(element, TypeVariable)):
            floating = args[0].kind == "literal" and args[0].args[0] in FLOAT_TYPES
            literal = TypeVariable("float literal" if floating else "integer literal", token,
                                   integer_literal=not floating, float_literal=floating)
            self.variables.append(literal)
            if element is not None:
                self.require(literal, element, token)
            element = literal
        diverges = False
        if kind == "vector_repeat":
            value, count = args
            element = self.type_of(value, scope, element)
            count_type = self.type_of(count, scope, "usize")
            self.require(count_type, "usize", count.token)
            self.obligation(element, "Clone", token)
            diverges = NEVER in {element, count_type}
            length = None
        elif kind == "array_repeat":
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
                element = TypeVariable("vector element" if vector else "array element", token)
                self.variables.append(element)
        if diverges:
            return NEVER
        result = AppliedType(VEC, (element,)) if vector else ArrayType(element, length)
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
            if owner == "str" or isinstance(owner, (SliceType, DynType)):
                self.check_type(ReferenceType(owner, False), token)
            else:
                self.check_type(owner, token)
            if trait:
                return self.trait_call(expr, scope, expected, owner, trait, token.text)
            return self.call_on_type(expr, scope, expected, owner, token.text)
        if token.text == f"{VEC}::new":
            owner = None
            if expr.type_arguments is not None:
                arguments = self.specialize(self.program.type_parameters[VEC], expr.type_arguments, token)
                owner = AppliedType(VEC, tuple(arguments.values()))
            return self.vector_constructor(expr, scope, owner or expected)
        if token.text == f"{BOX}::new":
            return self.box_constructor(expr, scope, expected)
        if token.text == f"{EXIT_CODE}::from":
            if len(args) != 1 or expr.type_arguments is not None:
                raise token.error("ExitCode::from takes one u8")
            value = self.type_of(args[0], scope, "u8")
            self.require(value, "u8", token)
            expr.kind = "exit_code"
            return NEVER if value == NEVER else EXIT_CODE
        if token.text == PROCESS_EXIT:
            if len(args) != 1 or expr.type_arguments is not None:
                raise token.error("process::exit takes one i32")
            value = self.type_of(args[0], scope, "i32")
            self.require(value, "i32", token)
            expr.kind = "process_exit"
            return NEVER
        if token.text in {f"{STRING}::new", f"{STRING}::from"}:
            return self.string_constructor(expr, scope, token.text.rsplit("::", 1)[-1])
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
            arguments = self.specialize(function.type_parameters, expr.type_arguments, token, callables=False)
            return self.function_call(expr, scope, expected, function, arguments)
        return self.check_call(expr, scope, expected, params, result, arguments)

    def function_call(self, expr, scope, expected, function, arguments, first=None):
        if function.associated_const != (expr.kind == "name"):
            raise expr.token.error("associated constants and methods use different call syntax")
        expr.kind = "call"
        if function.method_parameters:
            local = self.specialize(function.method_parameters, expr.type_arguments, expr.token, callables=False,
                                    requirements=False)
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
        defer_projection = any(isinstance(part, AssociatedType) for part in type_leaves(self.resolve(result)))
        if expected is not None and any(isinstance(part, DynType) for part in type_leaves(self.resolve(expected))):
            # Unsized coercions to trait objects happen after the call, in type_of.
            expected = None
        if expected is not None and arguments and not defer_projection:
            self.require_argument(result, expected, token)
        for parameter, owner in arguments.items():
            if isinstance(parameter, TypeParameter):
                for binding, value in parameter.associated_bindings.items():
                    if binding.item == ARGS_ITEM and isinstance(value, TupleType) and binding.bound in self.program.traits:
                        # A bound like `B: Extend<Self::Item>` lets a unique impl settle B.
                        self.pending_selections.append((owner, binding.bound,
                                                        tuple(substitute(kind, arguments) for kind in value.elements),
                                                        token))
                signature = callable_signature(parameter)
                if signature is not None:
                    self.require_callable(owner, substitute(signature, arguments), token)
                elif PATTERN in parameter.bounds and isinstance(self.resolve(owner), TypeVariable):
                    self.callable_hints[self.resolve(owner)] = FunctionType(("char",), "bool")
        actual_types = []
        for index, (expr_arg, parameter) in enumerate(zip(args, params)):
            hint = self.resolve(parameter)
            actual = first if index == 0 and first is not None else self.type_of(expr_arg, scope, hint)
            self.require_argument(actual, parameter, expr_arg.token)
            if isinstance(hint, ReferenceType) and hint.mutable:
                CaptureContext.mark_mutable(scope, expr_arg)
            actual_types.append(actual)
        self.associated_equalities.extend(self.solve_associated_equalities(self.associated_requirements(arguments, token)))
        self.solve_callables()
        self.select_trait_impls()
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
        while isinstance(callee, ReferenceType) or is_box(callee):
            callee = self.resolve(callee.inner if isinstance(callee, ReferenceType) else callee.arguments[0])
        if not isinstance(callee, FunctionType) and callable_signature(callee) is not None:
            callee = self.resolve(callable_signature(callee))
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
        if isinstance(hint, TypeVariable) and hint in self.callable_hints:
            hint = self.resolve(self.callable_hints[hint])
        elif is_box(hint) or isinstance(hint, (TypeParameter, OpaqueType, DynType)):
            inner = hint.arguments[0] if is_box(hint) else hint
            hint = callable_signature(self.resolve(inner)) or hint
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
        while isinstance(actual, ReferenceType) and isinstance(self.resolve(actual.inner), ReferenceType):
            original = Expr("deref", original.token, (original, False))
            actual = self.type_of(original, scope)
            expr.args = (original, *rest)
        guard = actual.inner if isinstance(actual, ReferenceType) else actual
        if isinstance(self.resolve(guard), AppliedType) and self.resolve(guard).name in GUARDS and name not in {"fmt"}:
            # Ref and RefMut have no methods of their own; calls go to the borrowed value.
            target, _ = self.deref_call(original, actual, scope, expr.token)
            expr.args = (target, *rest)
            return self.method_type(expr, scope, expected)
        if actual == NEVER:
            expr.kind, expr.args = "group", (original,)
            return NEVER
        base = actual.inner if isinstance(actual, ReferenceType) else actual
        base = self.resolve(base)
        if isinstance(base, TypeVariable) and (base.integer_literal or base.float_literal):
            representative = "f64" if base.float_literal else "i32"
            if not any(name in trait.methods and self.implements(representative, trait_name)
                       for trait_name, trait in self.program.traits.items()):
                self.require(base, representative, expr.token)
                base = representative
        if base in FLOAT_TYPES and name in {*FLOAT_PREDICATES, "abs", "sqrt"} and not rest and expr.type_arguments is None:
            if isinstance(actual, ReferenceType):
                original = Expr("deref", original.token, (original, False))
                self.type_of(original, scope)
            expr.kind, expr.args = "float_method", (original,)
            return "bool" if name in FLOAT_PREDICATES else base
        if isinstance(actual, IteratorType) and name in {"rev", "into_iter"} and not rest and expr.type_arguments is None:
            expr.kind, expr.args = ("reverse_iterator" if name == "rev" else "group"), (original,)
            return replace(actual, source=("rev", actual)) if name == "rev" else actual
        if isinstance(actual, IteratorType) and name == "map":
            mapped = self.iterator_map(expr, scope, actual)
            if mapped is not None:
                return mapped
        if isinstance(actual, IteratorType) and name in {"collect", "count", "sum", "product"}:
            return self.iterator_terminal(expr, expected, actual)
        if (name in {"cloned", "copied", "flatten"} and not rest and expr.type_arguments is None
                and not isinstance(actual, ReferenceType) and ITERATOR in self.program.traits
                and self.implements(self.resolve(base), ITERATOR)):
            item = self.resolve(AssociatedType(base, "Item", ITERATOR))
            if name == "flatten":
                inner = self.program.iteration_item(item)
                if inner is None:
                    raise expr.token.error(f"cannot flatten items of type {item}")
            else:
                if not isinstance(item, ReferenceType):
                    raise expr.token.error(f"{name} requires an iterator over references, found {item}")
                inner = item.inner
                self.obligation(inner, "Copy" if name == "copied" else "Clone", expr.token)
            expr.kind, expr.args = "lazy_adapter", (original,)
            return AppliedType(LAZY, (self.resolve(inner),))
        if (name in {"collect", "sum", "product"} and not isinstance(base, IteratorType)
                and not isinstance(actual, ReferenceType) and ITERATOR in self.program.traits
                and self.implements(self.resolve(base), ITERATOR)):
            item = self.resolve(AssociatedType(base, "Item", ITERATOR))
            return self.iterator_terminal(expr, expected, IteratorType(item, ("trait",)))
        if (isinstance(actual, IteratorType) or isinstance(base, IteratorType)) and name == "as_str" and not rest:
            iterator = actual if isinstance(actual, IteratorType) else base
            if iterator.source[:1] == ("chars",):
                if isinstance(actual, ReferenceType):
                    original = Expr("deref", original.token, (original, False))
                    self.type_of(original, scope)
                expr.kind, expr.args = "chars_as_str", (original,)
                return STR_REFERENCE
        if base in {STRING, "str"} and name in {"chars", "bytes"} and not rest and expr.type_arguments is None:
            if isinstance(actual, ReferenceType):
                original = Expr("deref", original.token, (original, False))
            original = Expr("borrow", expr.token, (original, False))
            self.type_of(original, scope)
            expr.kind, expr.args = "string_iterator", (original,)
            return IteratorType("char" if name == "chars" else "u8", (name,), True)
        if (name in {"iter", "iter_mut", "into_iter"} and not isinstance(base, IteratorType)
                and iteration_item(ReferenceType(base, False)) is not None):
            return self.iterator_constructor(expr, scope, original, actual)
        if isinstance(base, AppliedType) and base.name in {OPTION, RESULT}:
            result = self.prelude_method(expr, scope, original, actual, base, rest)
            if result is not None:
                return result
        has_length = isinstance(base, (ArrayType, SliceType)) or is_vector(base) or base in {STRING, "str"}
        if has_length and name in {"len", "is_empty"} and not rest and expr.type_arguments is None:
            if isinstance(actual, ReferenceType):
                original = Expr("deref", expr.token, (original, False))
                self.type_of(original, scope)
            expr.kind, expr.args = ("length" if name == "len" else name), (original,)
            return "usize" if name == "len" else "bool"
        if is_vector(base) and name in {"push", "pop", "clear", "truncate", "as_slice", "as_mut_slice"}:
            return self.vector_method(expr, scope, expected, actual, base)
        if base == STRING and name in {"push", "push_str", "pop", "clear", "truncate", "as_str", "as_mut_str"}:
            arguments = {"push": ("char",), "push_str": (STR_REFERENCE,), "truncate": ("usize",)}.get(name, ())
            result = AppliedType(OPTION, ("char",)) if name == "pop" else "()"
            mutable = name != "as_str"
            if name in {"as_str", "as_mut_str"}:
                result = ReferenceType("str", mutable)
            return self.builtin_receiver_call(expr, scope, expected, actual, STRING, arguments,
                                              result, kind="string_method", mutable=mutable)
        if base in {STRING, "str"} and name in {"make_ascii_uppercase", "make_ascii_lowercase"}:
            return self.builtin_receiver_call(expr, scope, expected, actual, base, (),
                                              "()", kind="string_method", mutable=True)
        if not self.std_string_search and (base in {STRING, "str"} and name in {"contains", "starts_with", "ends_with", "find", "rfind"}
                and len(rest) == 1 and rest[0].kind not in {"closure", "name"} or (
                    base in {STRING, "str"} and name in {"contains", "starts_with", "ends_with", "find", "rfind"}
                    and len(rest) == 1 and rest[0].kind == "name" and "::" not in rest[0].token.text)):
            pattern = self.type_of(rest[0], scope)
            if pattern not in {"char", NEVER} and not (
                isinstance(pattern, ReferenceType) and not pattern.mutable and pattern.inner in {"str", STRING}
            ):
                self.pretyped[id(rest[0])] = pattern
                return self.method_type_after_builtins(expr, scope, expected)
            argument = "char" if pattern == "char" else STR_REFERENCE
            result = AppliedType(OPTION, ("usize",)) if name in {"find", "rfind"} else "bool"
            return self.builtin_receiver_call(expr, scope, expected, actual, base, (argument,),
                                              result, kind="string_method", mutable=False)
        # There can be multiple inherent impl blocks on one target.
        for impl in self.program.impls:
            if impl.trait or name not in impl.methods:
                continue
            arguments = {}
            if match_types(impl.target, base, impl.parameters, arguments) and all(
                isinstance(self.resolve(arguments[p]), TypeVariable) or self.implements(self.resolve(arguments[p]), bound)
                for p in impl.parameters if p in arguments for bound in p.bounds
            ):
                for p in impl.parameters:
                    if p in arguments and isinstance(self.resolve(arguments[p]), TypeVariable):
                        for bound in p.bounds:
                            self.obligation(arguments[p], bound, expr.token)
                method = impl.methods[name]
                if self.has_receiver(method):
                    self.visible_method(method, expr.token)
                    return self.receiver_call(expr, scope, expected, method, arguments, actual)
        probe = base
        if isinstance(base, TypeVariable) and (base.integer_literal or base.float_literal):
            probe = "f64" if base.float_literal else "i32"
        candidates = [trait for trait in self.program.traits.values()
                      if name in trait.methods and self.has_receiver(trait.methods[name])
                      and self.implements(probe, trait.name.text)
                      and self.trait_available(probe, trait.name.text, expr.token)]
        receiver = base
        if not candidates and is_box(base):
            pointee = base.arguments[0]
            candidates = [trait for trait in self.program.traits.values()
                          if name in trait.methods and self.has_receiver(trait.methods[name])
                          and is_box(trait.methods[name].params[0][2])
                          and self.implements(pointee, trait.name.text)
                          and self.trait_available(pointee, trait.name.text, expr.token)]
            if candidates:
                receiver = pointee
        if not candidates and isinstance(actual, ReferenceType):
            # A by-value method can take the reference itself, as `"text".into()` does.
            candidates = [trait for trait in self.program.traits.values()
                          if name in trait.methods and self.has_receiver(trait.methods[name])
                          and self.implements(actual, trait.name.text)
                          and self.trait_available(actual, trait.name.text, expr.token)]
            if candidates:
                receiver = actual
        if (candidates and isinstance(actual, ReferenceType) and actual.mutable and receiver is base
                and not isinstance(candidates[0].methods[name].params[0][2], ReferenceType)
                and self.implements(actual, candidates[0].name.text)
                and not isinstance(base, (TypeParameter, OpaqueType))):
            # Method lookup probes the receiver's own type first, so `iter.by_ref().take(n)` uses &mut I.
            receiver = actual
        if len(candidates) > 1:
            raise expr.token.error(f"multiple traits provide method {name!r}")
        if candidates:
            trait = candidates[0]
            if isinstance(receiver, DynType):
                method = trait.methods[name]
                if method.self_sized:
                    raise expr.token.error(f"method {name!r} requires Self: Sized")
                if not isinstance(method.params[0][2], ReferenceType) and not is_box(method.params[0][2]):
                    raise expr.token.error("cannot pass an unsized trait object by value")
            bindings = self.trait_bindings(trait, receiver, expr)
            if trait.name.text == ITERATOR and name == "unzip":
                # Iterator::unzip requires Item = (A, B), which its std signature cannot state here.
                method = trait.methods[name]
                local = self.specialize(method.method_parameters, expr.type_arguments, expr.token,
                                        callables=False, requirements=False)
                first, second = (local[parameter] for parameter in method.method_parameters[:2])
                self.require(self.resolve(AssociatedType(receiver, "Item", ITERATOR)), TupleType((first, second)),
                             expr.token)
                expr.type_arguments = tuple(local[parameter] for parameter in method.method_parameters)
            return self.receiver_call(expr, scope, expected, trait.methods[name], bindings, actual)
        if is_box(base) and not (
            name == "clone" and self.implements(base, "Clone")
            or name == "to_string" and self.implements(base, "Display")
        ):
            if isinstance(actual, ReferenceType):
                original = Expr("deref", original.token, (original, False))
            item = Expr("box_deref", original.token, (original,))
            if name in {"as_ref", "as_mut"}:
                if rest or expr.type_arguments is not None:
                    raise expr.token.error(f"{name} takes no arguments")
                borrowed = Expr("borrow", expr.token, (item, name == "as_mut"))
                expr.kind, expr.args = "group", (borrowed,)
                return self.type_of(borrowed, scope, expected)
            expr.args = (item, *rest)
            return self.method_type(expr, scope, expected)
        unsized = SliceType(base.arguments[0]) if is_vector(base) else SliceType(base.element) if isinstance(
            base, ArrayType) else "str" if base == STRING else None
        if unsized is not None:
            method = self.inherent_method(unsized, name)
            if method is not None and self.has_receiver(method):
                wanted = method.params[0][2]
                mutable = isinstance(wanted, ReferenceType) and wanted.mutable
                receiver = Expr("deref", original.token, (original, False)) if isinstance(actual, ReferenceType) else original
                view = Expr("range_index", expr.token, (receiver, None, None, False))
                expr.args = (Expr("borrow", expr.token, (view, mutable), auto_borrow=True), *rest)
                return self.method_type(expr, scope, expected)
        if name == "clone" and not rest:
            expr.kind = "clone"
            return self.check_expr(expr, scope, expected)
        if ((name == "to_string" or (name == "to_owned" and base in {STRING, "str"}))
                and not rest and expr.type_arguments is None):
            self.obligation(base, "Display", expr.token)
            expr.kind, expr.args = "to_string", (original,)
            return STRING
        if self.derefs(base):
            target, _ = self.deref_call(original, actual, scope, expr.token)
            expr.args = (target, *rest)
            return self.method_type(expr, scope, expected)
        raise expr.token.error(f"no method {name!r} for {base}")

    def method_type_after_builtins(self, expr, scope, expected):
        """Resolve a str search through std's Pattern-taking methods instead of the built-in fast path."""
        self.std_string_search = True
        try:
            return self.method_type(expr, scope, expected)
        finally:
            self.std_string_search = False

    def inherent_method(self, target, name):
        for impl in self.program.impls:
            if impl.trait is None and name in impl.methods and match_types(impl.target, target, impl.parameters, {}):
                return impl.methods[name]
        return None

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
            hint = STR_REFERENCE if name == "expect" else result
            actual = self.type_of(rest[0], scope, hint)
            self.require(actual, hint, rest[0].token)
            if actual == NEVER:
                return NEVER
        return self.resolve(result)

    def vector_constructor(self, expr, scope, expected):
        if expr.args:
            raise expr.token.error("Vec::new takes no arguments")
        expr.kind = "vector"
        return self.sequence_type(expr, scope, expected)

    def box_constructor(self, expr, scope, expected):
        token = expr.token
        if len(expr.args) != 1:
            raise token.error("Box::new takes one value argument")
        hinted = self.resolve(expected)
        inner = hinted.arguments[0] if is_box(hinted) else None
        if expr.type_arguments is not None:
            parameters = self.program.type_parameters[BOX]
            if any(isinstance(argument, DynType) for argument in expr.type_arguments):
                raise token.error("Box::new requires a sized value")
            inner = self.specialize(parameters, expr.type_arguments, token)[parameters[0]]
        if isinstance(inner, DynType):
            hint = inner if inner.trait in CALLABLE_TRAITS else None
            inner = None
        else:
            hint = inner
        value = self.type_of(expr.args[0], scope, hint)
        if inner is not None:
            self.require(value, inner, token)
        if value == NEVER:
            expr.kind = "group"
            return NEVER
        if isinstance(value, DynType):
            raise token.error("Box::new requires a sized value")
        result = AppliedType(BOX, (self.resolve(value),))
        self.check_type(result, token)
        expr.kind = "box"
        return result

    def string_constructor(self, expr, scope, name):
        count = int(name == "from")
        if len(expr.args) != count or expr.type_arguments is not None:
            raise expr.token.error(f"String::{name} takes {count} arguments")
        if count:
            actual = self.type_of(expr.args[0], scope)
            if actual == NEVER:
                expr.kind, expr.args = "group", (expr.args[0],)
                return NEVER
            if not is_str_reference(actual) and actual not in {"char", STRING}:
                raise expr.token.error(f"cannot construct String from {actual}")
        expr.kind = "string"
        return STRING

    def iterator_constructor(self, expr, scope, original, actual):
        name = expr.token.text
        if len(expr.args) != 1 or expr.type_arguments is not None:
            raise expr.token.error(f"{name} takes no arguments")
        if name != "into_iter":
            if isinstance(actual, ReferenceType):
                original = Expr("deref", original.token, (original, False))
            original = Expr("borrow", expr.token, (original, name == "iter_mut"))
            actual = self.type_of(original, scope)
        item = iteration_item(actual)
        if item is None:
            raise expr.token.error(f"cannot iterate over {actual}")
        expr.kind, expr.args = "iterator", (original,)
        borrowed = isinstance(actual, ReferenceType)
        base = actual.inner if borrowed else actual
        if borrowed and (isinstance(base, (ArrayType, SliceType)) or is_vector(base)):
            origin = ("slice", actual.mutable)
        elif isinstance(base, ArrayType):
            origin = ("array", base.length)
        else:
            origin = (base.name, actual.mutable if borrowed else None)
        return IteratorType(item, origin, borrowed)

    def iterator_map(self, expr, scope, iterator):
        if len(expr.args) != 2 or expr.type_arguments is not None:
            raise expr.token.error("map expects one callable argument")
        mapper = expr.args[1]
        output = TypeVariable("mapped item", expr.token)
        self.variables.append(output)
        signature = FunctionType((iterator.item,), output)
        actual = self.type_of(mapper, scope, signature)
        if not isinstance(actual, FunctionType) or len(actual.parameters) != 1:
            raise mapper.token.error("map expects a callable with one parameter")
        self.require(actual.parameters[0], iterator.item, mapper.token)
        self.require(actual.result, output, mapper.token)
        item = self.resolve(output)
        if isinstance(item, ReferenceType) or borrowing_iterator(item):
            # Borrowed outputs go through the std Map adapter, which the borrow checker leaves untracked.
            return None
        identity = mapper.args[0] if mapper.kind == "function_pointer" else None
        expr.kind = "map_iterator"
        return IteratorType(item, ("map", iterator, identity, actual), iterator.borrowed or bool(actual.closure))

    def iterator_terminal(self, expr, expected, iterator):
        name, token = expr.token.text, expr.token
        if len(expr.args) != 1:
            raise token.error(f"{name} takes no value arguments")
        arguments = expr.type_arguments
        if name == "count":
            if arguments is not None:
                raise token.error("count takes no type arguments")
            result = "usize"
        else:
            if arguments is not None and len(arguments) != 1:
                raise token.error(f"{name} expects one type argument")
            target = arguments[0] if arguments is not None else expected
            result = self.fill_holes(target, token)
            if expected is not None:
                self.require(result, expected, token)
            resolved = self.resolve(result)
            if name == "collect" and is_vector(resolved):
                self.require(iterator.item, resolved.arguments[0], token)
            elif name in {"sum", "product"} and resolved in NUMBER_TYPES:
                item = iterator.item.inner if isinstance(iterator.item, ReferenceType) else iterator.item
                self.require(item, resolved, token)
            elif name in {"sum", "product"} and isinstance(resolved, AppliedType) and resolved.name in {OPTION, RESULT}:
                self.require(iterator.item, resolved, token)
            self.iterator_results.append((name, iterator.item, result, token))
        expr.kind = "iterator_terminal"
        return result

    def check_iterator_result(self, method, item, target, token):
        if isinstance(target, TypeVariable):
            raise token.error(f"type annotations needed for {method}")
        if method == "collect":
            self.check_collect(item, target, token)
        else:
            element = item.inner if isinstance(item, ReferenceType) else item
            if isinstance(target, AppliedType) and target.name in {OPTION, RESULT}:
                # Summing Options or Results stops at the first None or Err.
                self.require(element, target, token)
                if target.arguments[0] not in NUMBER_TYPES:
                    raise token.error(f"{method} requires a numeric result, found {target.arguments[0]}")
                return
            if target not in NUMBER_TYPES:
                raise token.error(f"{method} requires a numeric result, found {target}")
            self.require(element, target, token)

    def check_collect(self, item, target, token):
        item, target = self.resolve(item), self.resolve(target)
        name = target.name if isinstance(target, AppliedType) else target
        if is_vector(target) or name in {DEQUE, HEAP} or MAP_TYPES.get(name, "").endswith("Set"):
            self.check_type(target, token)
            self.require(item, target.arguments[0], token)
        elif name in MAP_TYPES:
            self.check_type(target, token)
            self.require(item, TupleType(target.arguments), token)
        elif target == STRING and (item in {"char", STRING} or is_str_reference(item)
                                   or isinstance(item, ReferenceType) and item.inner in {"char", STRING}):
            return
        elif name in {OPTION, RESULT} and isinstance(item, TypeVariable) and not (
                item.integer_literal or item.float_literal):
            # Collecting into Result<C, E> or Option<C> means the items are Results or Options too.
            element = TypeVariable("collected element", token)
            self.variables.append(element)
            self.require(item, AppliedType(name, (element, *target.arguments[1:])), token)
            self.check_collect(element, target.arguments[0], token)
        elif name in {OPTION, RESULT} and isinstance(item, AppliedType) and item.name == name:
            if name == RESULT:
                self.require(item.arguments[1], target.arguments[1], token)
            self.check_collect(item.arguments[0], target.arguments[0], token)
        else:
            raise token.error(f"cannot collect items of type {item} into {target}")

    def vector_method(self, expr, scope, expected, actual, base):
        name = expr.token.text
        element = base.arguments[0]
        mutable = name != "as_slice"
        arguments = (element,) if name == "push" else ("usize",) if name == "truncate" else ()
        if name == "pop":
            result = AppliedType(OPTION, (element,))
        elif name in {"as_slice", "as_mut_slice"}:
            result = ReferenceType(SliceType(element), mutable)
        else:
            result = "()"
        return self.builtin_receiver_call(expr, scope, expected, actual, base, arguments,
                                          result, kind="vector_method", mutable=mutable)

    def builtin_receiver_call(self, expr, scope, expected, actual, base, arguments, result, *, kind, mutable):
        original, *rest = expr.args
        receiver = ReferenceType(base, mutable)
        if not isinstance(actual, ReferenceType):
            original = Expr("borrow", expr.token, (original, mutable), auto_borrow=True)
            actual = self.type_of(original, scope)
        if expr.type_arguments is not None:
            raise expr.token.error(f"{expr.token.text} takes no type arguments")
        expr.kind, expr.args = kind, (original, *rest)
        params = (receiver, *arguments)
        checked_result = self.check_call(expr, scope, expected, params, result, {}, actual)
        if isinstance(result, ReferenceType):
            self.program.call_functions[id(expr)] = Function(
                expr.token, [(expr.token, False, receiver)], result, None, (0,))
        return checked_result

    def trait_bindings(self, trait, receiver, expr):
        """Bind Self and fresh variables for a generic trait's parameters, and record the dispatch."""
        bindings = {trait.self_type: receiver}
        arguments = []
        for parameter in trait.parameters:
            variable = TypeVariable(parameter.name, expr.token)
            self.variables.append(variable)
            bindings[parameter] = variable
            arguments.append(variable)
        if arguments:
            self.trait_obligations.append((receiver, trait.name.text, tuple(arguments), expr.token))
            self.pending_selections.append((receiver, trait.name.text, tuple(arguments), expr.token))
            resolved = self.resolve(receiver)
            if not isinstance(resolved, (TypeVariable, TypeParameter, AssociatedType, OpaqueType)):
                matches = []
                for impl in self.program.impls:
                    found = {}
                    if impl.trait == trait.name.text and match_types(impl.target, resolved, impl.parameters, found):
                        matches.append((impl, found))
                if len(matches) == 1:
                    # With a single candidate impl, its trait arguments are known, as in rustc.
                    impl, found = matches[0]
                    declared = self.program.trait_arguments(impl.trait, impl.trait_arguments, impl.target)
                    for variable, template in zip(arguments, declared):
                        if template is not None and all(part in found or part not in impl.parameters
                                                        for part in type_leaves(template)):
                            self.require(variable, substitute(template, found), expr.token)
        self.program.dispatches[id(expr)] = (trait.name.text, receiver, expr.token.text.rsplit("::", 1)[-1],
                                             tuple(arguments))
        return bindings

    def select_trait_impls(self):
        """When exactly one impl fits a generic trait call's known types, let it settle the unknown ones."""
        for _ in range(4):
            before = len(self.substitutions)
            self.select_trait_impls_once()
            if len(self.substitutions) == before:
                return

    def select_trait_impls_once(self):
        pending, self.pending_selections = self.pending_selections, []
        for entry in pending:
            receiver, trait, arguments, token = entry
            receiver = self.resolve(receiver)
            arguments = tuple(map(self.resolve, arguments))
            if not any(isinstance(part, TypeVariable) for kind in (receiver, *arguments) for part in type_leaves(kind)):
                continue
            literal = isinstance(receiver, TypeVariable) and (receiver.integer_literal or receiver.float_literal)
            if isinstance(receiver, (TypeVariable, TypeParameter, OpaqueType, DynType, AssociatedType)) and not literal:
                self.pending_selections.append(entry)
                continue
            matches = []
            for impl in self.program.impls:
                if impl.trait != trait:
                    continue
                found = {}
                declared = self.program.trait_arguments(trait, impl.trait_arguments, impl.target)
                if loose_match(impl.target, receiver, impl.parameters, found) and all(
                        template is None or loose_match(template, argument, impl.parameters, found)
                        for template, argument in zip(declared, arguments)):
                    matches.append((impl, declared))
            if len(matches) != 1:
                self.pending_selections.append(entry)
                continue
            impl, declared = matches[0]
            fresh = {}
            for parameter in impl.parameters:
                variable = TypeVariable(parameter.name, token, parameter.const_type)
                self.variables.append(variable)
                fresh[parameter] = variable
            try:
                self.require(substitute(impl.target, fresh), receiver, token)
                for template, argument in zip(declared, arguments):
                    if template is not None:
                        self.require(substitute(template, fresh), argument, token)
            except RustError:
                continue
            for parameter in impl.parameters:
                # A bound like `U: From<T>` on the chosen impl is itself a selection to make.
                for binding, value in getattr(parameter, "associated_bindings", {}).items():
                    if binding.item == ARGS_ITEM and isinstance(value, TupleType):
                        self.pending_selections.append((fresh[parameter], binding.bound,
                                                        tuple(substitute(kind, fresh) for kind in value.elements),
                                                        token))

    def builtin_trait_arguments(self, receiver, trait, arguments):
        """Built-in impls, such as f64: PartialOrd or i32: Add, that exist without an impl block."""
        if trait in TRAIT_MODULES or trait in BUILTIN_OPERATORS:
            return self.implements(receiver, trait)
        return trait in {INTO_ITERATOR, ITERATOR} and self.implements(receiver, trait)

    def trait_available(self, base, name, token):
        if isinstance(base, DynType):
            return name in self.program.trait_closure({base.trait})
        if isinstance(base, (TypeParameter, OpaqueType)) and name in self.program.trait_closure(base.bounds):
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
        if isinstance(wanted, ReferenceType) and (not isinstance(actual, ReferenceType) or (
                isinstance(self.resolve(wanted.inner), ReferenceType)
                and self.resolve(wanted.inner) == self.resolve(actual))):
            original = Expr("borrow", expr.token, (original, wanted.mutable), auto_borrow=True)
            actual = self.type_of(original, scope)
        elif isinstance(actual, ReferenceType) and not isinstance(wanted, ReferenceType):
            original = Expr("deref", expr.token, (original, False))
            actual = self.type_of(original, scope)
        expr.kind = "call"
        expr.args = (original, *rest)
        return self.function_call(expr, scope, expected, method, arguments, actual)

    def associated_type(self, expr, scope, expected):
        owner, separator, name = expr.token.text.rpartition("::")
        if not separator:
            raise expr.token.error(f"unknown function {name!r}")
        if owner in self.parameters and isinstance(self.parameters[owner], TypeParameter):
            return self.call_on_type(expr, scope, expected, self.parameters[owner], name)
        if owner in PRIMITIVES or owner == "str":
            if expr.args and not any(name in impl.methods and impl.target == owner and impl.trait is None
                                     for impl in self.program.impls):
                traits = [trait for trait in self.program.traits.values() if name in trait.methods
                          and self.implements(owner, trait.name.text)]
                if not traits:
                    # Built-in methods like `str::len(s)` or `f64::sqrt(x)` take their receiver first.
                    method = Expr("method", replace(expr.token, text=name), expr.args)
                    result = self.method_type(method, scope, expected)
                    expr.kind, expr.token, expr.args = method.kind, method.token, method.args
                    for table in (self.program.call_types, self.program.call_functions, self.program.dispatches):
                        if id(method) in table:
                            table[id(expr)] = table.pop(id(method))
                    return result
            return self.call_on_type(expr, scope, expected, owner, name)
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
            if not candidates and expr.args and owner in PRELUDE_OWNERS:
                # `Option::is_some(&x)` and `Vec::len(&v)`: call the built-in method on the first argument.
                try:
                    method = Expr("method", replace(expr.token, text=name), expr.args)
                    result = self.method_type(method, scope, expected)
                    expr.kind, expr.token, expr.args = method.kind, method.token, method.args
                    expr.type_arguments = method.type_arguments
                    for table in (self.program.call_types, self.program.call_functions, self.program.dispatches):
                        if id(method) in table:
                            table[id(expr)] = table.pop(id(method))
                    return result
                except RustError:
                    pass
            if not candidates and owner in self.program.type_parameters:
                arguments = []
                for parameter in self.program.type_parameters[owner]:
                    variable = TypeVariable(parameter.name, expr.token, parameter.const_type)
                    self.variables.append(variable)
                    arguments.append(variable)
                return self.call_on_type(expr, scope, expected, AppliedType(owner, tuple(arguments)), name)
            raise expr.token.error(f"unknown or ambiguous associated function {expr.token.text!r}")
        impl = candidates[0]
        method = impl.methods[name]
        self.visible_method(method, expr.token)
        explicit = None if method.method_parameters else expr.type_arguments
        arguments = self.specialize(impl.parameters, explicit, expr.token, callables=False)
        return self.function_call(expr, scope, expected, method, arguments)

    def path_arity(self, expr, scope):
        """The parameter count of a constructor or associated function named by a path, or None."""
        text = expr.token.text
        if self.local_binding(scope, expr.token) is not None:
            return None
        ctor = self.program.constructors.get(text)
        if ctor is not None:
            return len(ctor.fields) if ctor.style == "tuple" else None
        if "::" not in text:
            return None
        builtin = {f"{VEC}::new": 0, f"{STRING}::new": 0, f"{STRING}::from": 1, f"{BOX}::new": 1}
        if text in builtin:
            return builtin[text]
        owner, _, name = text.rpartition("::")
        if owner in PRELUDE_OWNERS | PRIMITIVES | {"str"} and name in BUILTIN_METHOD_ARITIES:
            if not any(name in impl.methods for impl in self.program.impls if impl.trait is None
                       and (impl.target.name if isinstance(impl.target, AppliedType) else impl.target) == owner):
                return BUILTIN_METHOD_ARITIES[name]
        if owner in PRIMITIVES or owner == "str":
            for trait in self.program.traits.values():
                if name in trait.methods and self.implements(owner, trait.name.text):
                    return len(trait.methods[name].params)
        if expr.qualified:
            owner_type, trait_name = expr.qualified
            trait = self.program.traits.get(trait_name) if trait_name else None
            if trait and name in trait.methods:
                method = trait.methods[name]
                return None if method.associated_const else len(method.params)
            owner = owner_type.name if isinstance(owner_type, AppliedType) else owner_type
        trait = self.program.traits.get(owner)
        if trait and name in trait.methods:
            method = trait.methods[name]
            return None if method.associated_const else len(method.params)
        for impl in self.program.impls:
            target = impl.target.name if isinstance(impl.target, AppliedType) else impl.target
            if impl.trait is None and name in impl.methods and target == owner:
                method = impl.methods[name]
                return None if method.associated_const else len(method.params)
        if owner in self.parameters:
            for bound in self.program.trait_closure(self.parameters[owner].bounds):
                trait = self.program.traits.get(bound)
                if trait and name in trait.methods and not trait.methods[name].associated_const:
                    return len(trait.methods[name].params)
        return None

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
        bindings = self.trait_bindings(trait, owner, expr)
        return self.function_call(expr, scope, expected, trait.methods[name], bindings, first)

    def call_on_type(self, expr, scope, expected, owner, name):
        if is_vector(owner) and name == "new":
            return self.vector_constructor(expr, scope, owner)
        if owner == STRING and name in {"new", "from"}:
            return self.string_constructor(expr, scope, name)
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

    def literal_variable(self, expr):
        """A fresh inference variable for an unsuffixed numeric literal, or None."""
        literal = ungroup(expr)
        if literal.kind == "unary" and literal.token.text == "-":
            literal = ungroup(literal.args[0])
        if literal.kind != "literal" or not literal.unsuffixed or literal.args[0] not in NUMBER_TYPES:
            return None
        floating = literal.args[0] in FLOAT_TYPES
        variable = TypeVariable("float literal" if floating else "integer literal", literal.token,
                                integer_literal=not floating, float_literal=floating)
        self.variables.append(variable)
        (self.float_literals if floating else self.integer_literals).append(
            (variable, 0.0 if floating else 0, literal.token))
        return variable

    def has_field(self, annotation, name):
        layout = self.program.layout(self.resolve(annotation))
        if not layout or len(layout[1]) != 1:
            return False
        return any(field == name.text for field, _ in layout[1][0].fields) and (
            name.module[:1] == STD_ROOT or self.resolve(annotation) not in self.program.types
            and not (isinstance(self.resolve(annotation), AppliedType)
                     and self.resolve(annotation).name.startswith("std" + MODULE_SEPARATOR)))

    def derefs(self, annotation):
        annotation = self.resolve(annotation)
        return (DEREF in self.program.traits and not isinstance(annotation, (ReferenceType, TypeVariable))
                and not is_box(annotation) and self.implements(annotation, DEREF))

    def deref_call(self, value, actual, scope, token):
        """`*Deref::deref(&value)` with its target type, for auto-deref through smart pointers."""
        base = self.resolve(actual)
        base = base.inner if isinstance(base, ReferenceType) else base
        trait = self.program.traits[DEREF]
        call = Expr("method", replace(token, text="deref"), (value,))
        bindings = self.trait_bindings(trait, base, call)
        reference = self.resolve(self.receiver_call(call, scope, None, trait.methods["deref"], bindings, actual))
        if isinstance(base, AppliedType) and base.name in GUARD_TYPES:
            # RefMut holds the cell's exclusive borrow, so it hands out &mut as DerefMut would.
            reference = replace(reference, mutable=True)
            function = self.program.call_functions.get(id(call))
            if function is not None:
                self.program.call_functions[id(call)] = replace(function, result=reference, return_inputs=())
        self.program.expr_types[id(call)] = reference
        target = Expr("deref", token, (call, False))
        self.program.expr_types[id(target)] = reference.inner
        return target, self.resolve(reference.inner)

    def index_call(self, expr, scope, base, index, annotation):
        """Index a non-built-in collection through std::ops::Index or IndexMut."""
        mutable = id(expr) in self.mutable_places
        name = "IndexMut" if mutable else "Index"
        trait = self.program.traits[MODULE_SEPARATOR.join(("std", "ops", name))]
        receiver = self.resolve(annotation)
        if not self.implements(receiver, trait.name.text) and not isinstance(receiver, (TypeParameter, AssociatedType)):
            raise expr.token.error(f"cannot index {receiver}")
        call = Expr("method", replace(expr.token, text="index_mut" if mutable else "index"), (base, index))
        bindings = self.trait_bindings(trait, receiver, call)
        reference = self.resolve(self.receiver_call(call, scope, None, trait.methods[call.token.text], bindings, receiver))
        self.program.expr_types[id(call)] = reference
        expr.kind, expr.args = "deref", (call, False)
        return reference.inner

    def mark_mutable_place(self, target):
        target = ungroup(target)
        while target.kind in {"field", "index"}:
            if target.kind == "index":
                self.mutable_places.add(id(target))
            target = ungroup(target.args[0])

    def overloadable(self, annotation):
        """Whether an operator on this type goes through a std::ops trait rather than a built-in."""
        annotation = self.resolve(annotation)
        while isinstance(annotation, ReferenceType):
            annotation = self.resolve(annotation.inner)
        if annotation in PRIMITIVES or annotation in {STRING, NEVER, "str"} or isinstance(annotation, TypeVariable):
            return False
        return True

    def operator_call(self, expr, scope, expected, trait_name, method, operands, actual):
        trait = self.program.traits.get(MODULE_SEPARATOR.join(("std", "ops", trait_name)))
        receiver = self.resolve(actual)
        if trait is None or not (self.implements(receiver, trait.name.text) or isinstance(
                receiver, (TypeParameter, AssociatedType, OpaqueType))):
            raise expr.token.error(f"cannot apply {expr.token.text!r} to {receiver}")
        expr.kind, expr.token, expr.args = "method", replace(expr.token, text=method), tuple(operands)
        bindings = self.trait_bindings(trait, receiver, expr)
        return self.receiver_call(expr, scope, expected, trait.methods[method], bindings, actual)

    def numeric_like(self, annotation):
        annotation = self.resolve(annotation)
        return (annotation in NUMBER_TYPES or annotation == "bool" or isinstance(annotation, ReferenceType)
                and self.numeric_like(annotation.inner)
                or isinstance(annotation, TypeVariable) and (annotation.integer_literal or annotation.float_literal))

    def binary_type(self, token, left, right):
        word = token.text
        if word == "+" and left == STRING:
            self.require(right, STR_REFERENCE, token)
            return NEVER if right == NEVER else STRING
        if word in {"&&", "||"}:
            self.require(left, "bool", token)
            self.require(right, "bool", token)
            return NEVER if left == NEVER else "bool"
        if word in COMPARE:
            if left != NEVER and right != NEVER:
                peeled_left, peeled_right = self.resolve(left), self.resolve(right)
                while isinstance(peeled_left, ReferenceType) and isinstance(peeled_right, ReferenceType):
                    peeled_left, peeled_right = self.resolve(peeled_left.inner), self.resolve(peeled_right.inner)
                text_pair = {peeled_left, peeled_right} in ({STRING, STR_REFERENCE}, {STRING, "str"})
                trait = "PartialEq" if word in {"==", "!="} else "PartialOrd"
                cross = (peeled_left != peeled_right and not isinstance(peeled_left, TypeVariable)
                         and not isinstance(peeled_right, TypeVariable)
                         and self.program.implementation(peeled_left, trait, arguments_wanted=(peeled_right,)) is not None)
                sequences = [kind.arguments[0] if is_vector(kind) else kind.element
                             for kind in (peeled_left, peeled_right)
                             if is_vector(kind) or isinstance(kind, (ArrayType, SliceType))]
                if len(sequences) == 2 and peeled_left != peeled_right and word in {"==", "!="}:
                    # Vec, arrays, and slices compare with each other element by element.
                    self.require(sequences[1], sequences[0], token)
                elif not cross and (word not in {"==", "!="} or not text_pair):
                    self.require(right, left, token)
                self.obligation(left, "PartialEq" if word in {"==", "!="} else "PartialOrd", token)
            return NEVER if NEVER in {left, right} else "bool"
        if word in SHIFTS:
            self.numeric_operands.extend(((left, None, token), (right, None, token)))
            return NEVER if NEVER in {left, right} else left
        if word in FLOAT_OPERATORS and (left in FLOAT_TYPES or right in FLOAT_TYPES):
            kind = left if left in FLOAT_TYPES else right
            self.require(left, kind, token)
            self.require(right, kind, token)
            return NEVER if NEVER in {left, right} else kind
        if word in BITWISE and "bool" in {left, right}:
            self.require(left, "bool", token)
            self.require(right, "bool", token)
            return NEVER if NEVER in {left, right} else "bool"
        variable = next((kind for kind in (left, right)
                         if isinstance(kind, TypeVariable) and (kind.integer_literal or kind.float_literal)), None)
        if variable is not None:
            if variable.float_literal and word not in FLOAT_OPERATORS:
                raise token.error(f"cannot apply {word!r} to a float")
            self.require(left, variable, token)
            self.require(right, variable, token)
            return NEVER if NEVER in {left, right} else self.resolve(variable)
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
    temporary: bool = False
    # For the untraced referent of a reference local, that reference local.
    untraced_holder: Local | None = None


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
            loan = self.holder.loan
            projection = self.fields[len(loan.place.fields):]
            if projection:
                return tuple(replace(origin, fields=(*origin.fields, *projection))
                             if not origin.indexed else origin for origin in loan.possible)
            return loan.possible
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
        if self.kind in {"declare", "uninitialized", "end_temporaries", "storage_dead"}:
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


@dataclass
class PatternAccesses:
    reads: dict = field(default_factory=dict)
    moves: dict = field(default_factory=dict)
    tests: dict = field(default_factory=dict)
    borrows: dict = field(default_factory=dict)


@dataclass(frozen=True)
class MoveState:
    missing: frozenset = frozenset()
    written: frozenset = frozenset()
    # (reference local, borrowed local) pairs recorded when a borrow is assigned to a local.
    pointing: frozenset = frozenset()

    def merge(self, other):
        return MoveState(self.missing | other.missing, self.written | other.written,
                         self.pointing | other.pointing)

    def transfer(self, effect):
        if effect is None:
            return self
        kind, place = effect.kind, effect.place
        if kind == "write" and place.holder:
            return self
        if kind == "move":
            return MoveState(self.missing | {(place, "moved")}, self.written, self.pointing)
        if kind == "points_to":
            pointing = frozenset(item for item in self.pointing if item[0] is not place.local)
            return MoveState(self.missing, self.written,
                             pointing | {(place.local, origin) for origin in effect.roots})
        if kind == "storage_dead":
            # References assigned a borrow of a local that goes out of scope dangle from here on.
            dangling = {(Place(local), f"dangling:{origin.token.text}")
                        for local, origin in self.pointing if origin is place.local}
            pointing = frozenset(item for item in self.pointing if place.local not in item)
            return MoveState(self.missing | dangling, self.written, pointing)
        if kind in {"declare", "uninitialized", "write"}:
            missing = frozenset(item for item in self.missing if not place.contains(item[0]))
            pointing = self.pointing if place.fields else frozenset(
                item for item in self.pointing if item[0] is not place.local)
            if kind == "uninitialized":
                return MoveState(missing | {(place, "uninitialized")}, self.written - {place.local}, pointing)
            return MoveState(missing, self.written | {place.local}, pointing)
        return self

    def validate(self, effect):
        if effect is None:
            return
        kind, place, token = effect.kind, effect.place, effect.token
        if kind not in {"declare", "uninitialized", "storage_dead", "points_to"} and not (
                kind == "write" and not place.holder):
            for item, reason in self.missing:
                if reason.startswith("dangling:") and item.local in {place.local, place.holder,
                                                                     place.local.untraced_holder}:
                    raise token.error(f"borrowed value {reason.removeprefix('dangling:')!r} does not live "
                                      f"long enough (still used through {item.name!r})")
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
            if function.body is None:
                continue
            added = []
            for parameter, bounds, _ in function.where_bounds:
                added.append((parameter, bounds - parameter.bounds))
                parameter.bounds.update(bounds)
            try:
                self.check_function(function)
            finally:
                for parameter, bounds in added:
                    parameter.bounds.difference_update(bounds)

    def check_function(self, function):
        if True:
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
            if function.name.module[:1] == STD_ROOT:
                # std bodies are trusted: lower them to record moves for drop tracking, but skip the checks.
                try:
                    self.lower_return(function.body, scope, cursor)
                except RustError:
                    pass
                return
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
        if kind == "storage_dead":
            # A block's local goes out of scope while a reference to it is still used later.
            if any(origin.local is place.local for loan in live for origin in loan.regions()):
                raise token.error(f"borrowed value {place.name!r} does not live long enough")
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
        if expr.kind == "static_item":
            place = Place(Local(expr.token, False, static=True))
            return self.emit(cursor, "declare", place, expr.token), place
        if expr.kind == "box_deref":
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
        if holder.loan is None:
            # The checker did not trace where this reference points, so its referent is untracked.
            token = replace(holder.token, text=f"*{holder.token.text}")
            return Place(Local(token, True, static=True, untraced_holder=holder))
        owner = holder.loan.place
        return Place(owner.local, owner.fields, holder)

    def start_loan(self, cursor, place, mutable, token, *, two_phase=False, expiring=frozenset()):
        parent = place.holder.loan if place.holder else None
        parents = (parent,) if parent else ()
        possible = place.regions() if parent and parent.possible else ()
        if place.local.temporary:
            expiring = expiring | {place.local}
        if parent:
            expiring = expiring | parent.expiring
        loan = Loan(replace(place, holder=None), mutable, parents, two_phase, possible, expiring)
        protected = self.guard_borrows + self.guard_tests
        if mutable and (place.local in self.guarded or any(place.overlaps(item) for item in protected)):
            return self.emit(cursor, "guard_write", place, token), loan
        return self.emit(cursor, "borrow", place, token, loan=loan), loan

    def lower_reference(self, expr, scope, cursor, *, reborrow=None, two_phase=False, extend_to=None):
        expr = ungroup(expr)
        annotation = self.annotation(expr)
        if annotation == NEVER:
            return self.lower(expr, scope, cursor), None
        if expr.kind in {"dyn_reference", "slice_reference", "string_reference"}:
            original = expr.args[0]
            if reborrow is None and self.annotation(original).mutable and not annotation.mutable:
                reborrow = annotation
            return self.lower_reference(original, scope, cursor, reborrow=reborrow,
                                        two_phase=two_phase, extend_to=extend_to)
        if expr.kind in {"call", "indirect_call", "vector_method", "string_method"}:
            cursor, loan = self.lower_call(expr, scope, cursor)
            if loan is None and cursor is not None:
                # A reference returned without a traceable input borrow is left untracked.
                return self.static_loan(cursor, expr.token, annotation.mutable)
            return cursor, loan
        if expr.kind == "byte_string" or (is_str_reference(annotation) and expr.kind in {"literal", "default"}):
            return self.static_loan(cursor, expr.token)
        if expr.kind == "clone":
            return self.lower_reference(expr.args[0], scope, cursor, reborrow=reborrow,
                                        two_phase=two_phase, extend_to=extend_to)
        if expr.kind == "block":
            statements, tail = expr.args
            inner = Scope(scope)
            for statement in statements:
                cursor = self.lower(statement, inner, cursor)
            cursor, loan = self.lower_reference(tail, inner, cursor, extend_to=extend_to)
            return self.leave_reference_scope(cursor, loan, inner, tail.token), loan
        if expr.kind in {"if", "if_chain"}:
            condition, yes, no = expr.args
            if expr.kind == "if_chain":
                satisfied, failed, inner = self.lower_chain(condition, scope, cursor)
            else:
                satisfied = failed = self.lower(condition, scope, cursor)
                inner = scope
            yes_end, yes_loan = self.lower_reference(yes, inner, satisfied, extend_to=extend_to)
            if inner is not scope:
                yes_end = self.leave_reference_scope(yes_end, yes_loan, inner, yes.token)
            no_end, no_loan = self.lower_reference(no, scope, failed, extend_to=extend_to)
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
        if expr.kind != "name":
            # References copied out of places the checker does not model are not tracked.
            return self.static_loan(self.lower(expr, scope, cursor, consume=False), expr.token,
                                    isinstance(annotation, ReferenceType) and annotation.mutable)
        holder = scope.find(expr.token)
        if reborrow:
            return self.start_loan(cursor, self.referent(holder), reborrow.mutable, expr.token,
                                   two_phase=two_phase)
        return self.lower(expr, scope, cursor), holder.loan

    def static_loan(self, cursor, token, mutable=False):
        place = Place(Local(token, mutable, static=True))
        cursor = self.emit(cursor, "declare", place, token)
        return cursor, Loan(place, mutable)

    def promotable(self, expr):
        expr = ungroup(expr)
        if expr.kind == "literal":
            return True
        if expr.kind in {"name", "call", "record"}:
            ctor = self.program.constructors.get(expr.token.text)
            style = {"name": "unit", "call": "tuple", "record": "record"}[expr.kind]
            if not ctor or ctor.style != style:
                return False
            if expr.kind == "record":
                fields, update = expr.args
                values = [value for _, value in fields]
                if update:
                    values.append(update)
            else:
                values = expr.args
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
        if not isinstance(self.annotation(expr), ReferenceType) or self.function.name.text.startswith("@closure:"):
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
            elif isinstance(annotation, FunctionType):
                cursor = self.lower(pointer, scope, cursor)
            else:
                once = not self.program.implements(annotation, "FnMut")
                cursor = self.lower(pointer, scope, cursor, consume=once)
        function = self.program.call_functions.get(id(expr))
        expected, _ = self.program.call_types[id(expr)]
        passed = {}
        for index, (expr, annotation) in enumerate(zip(args, expected)):
            if isinstance(annotation, ReferenceType) and self.annotation(expr) != NEVER:
                cursor, loan = self.lower_reference(
                    expr, scope, cursor, reborrow=annotation, two_phase=annotation.mutable
                )
                cursor, holder = self.keep_loan(cursor, loan, expr.token)
                declared = function.params[index][2] if function and index < len(function.params) else None
                if (annotation.lifetime == "'static" and isinstance(declared, ReferenceType)
                        and declared.lifetime == "'static"):
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

    def lower_print(self, expr, scope, cursor):
        token, args = expr.token, expr.args
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
        for field in self.program.formats[id(expr)][1]:
            if not isinstance(field, FormatField):
                continue
            arguments = [field.argument]
            arguments.extend(count.argument for count in (field.spec.width, field.spec.precision)
                             if isinstance(count, FormatCount))
            for name in arguments:
                if isinstance(name, str):
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
            # Moving a captured variable makes a non-move closure capture it by value, as Rust infers.
            key = place.local.token.key
            self.function.captures = tuple(replace(capture, by_move=True) if capture.name.key == key else capture
                                           for capture in self.function.captures)
            self.function.consumes_captures = True
            return "move"
        if place.local in self.owned_captures:
            self.function.consumes_captures = True
        if place.local in self.guarded or any(place.overlaps(item) for item in self.guard_borrows):
            return "guard_move"
        return "move"

    def lower(self, expr, scope, cursor, *, consume=True):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind in {"iterator_terminal", "lazy_adapter"}:
            source = args[0]
            if borrowing_iterator(self.annotation(source)):
                cursor, loan = self.lower_iterator(source, scope, cursor)
                if loan:
                    cursor, holder = self.keep_loan(cursor, loan, token)
                    cursor = self.emit(cursor, "read", holder, token)
                return cursor
            return self.lower(source, scope, cursor)
        if kind in {"iterator", "reverse_iterator", "map_iterator", "string_iterator"}:
            if borrowing_iterator(self.annotation(expr)):
                cursor, _ = self.lower_iterator(expr, scope, cursor)
                return cursor
            for argument in args:
                cursor = self.lower(argument, scope, cursor)
            return cursor
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
        if kind in {"dyn_reference", "slice_reference", "string_reference"}:
            return self.lower(args[0], scope, cursor, consume=consume)
        cursor, place = self.lower_address(expr, scope, cursor)
        if place is not None:
            if consume and place.holder is None and not self.program.is_copy(self.annotation(expr)):
                self.program.moves.add(id(expr))
            return self.access(cursor, place, self.annotation(expr), token, consume=consume)
        if kind in {"literal", "name", "default", "const_value", "function_pointer", "byte_string",
                    "lazy_const", "lazy_static"}:
            return cursor
        if kind == "borrow":
            cursor, _ = self.lower_reference(expr, scope, cursor)
            return cursor
        if kind in {"group", "field", "unary", "deref", "scalar_cast", "box", "box_deref", "exit_code"}:
            return self.lower(args[0], scope, cursor)
        if kind == "process_exit":
            self.lower(args[0], scope, cursor)
            return None
        if kind == "clone":
            return self.lower(args[0], scope, cursor, consume=False)
        if kind in {"length", "is_empty", "to_string", "float_method", "chars_as_str", "enum_cast"}:
            return self.lower(args[0], scope, cursor, consume=False)
        if kind == "range_value":
            for item in args[:2]:
                if item is not None:
                    cursor = self.lower(item, scope, cursor)
            return cursor
        if kind == "range_index":
            for item in args[:3]:
                if item is not None:
                    cursor = self.lower(item, scope, cursor)
            return cursor
        if kind in {"array", "tuple", "index", "array_repeat", "vector", "vector_repeat", "string"}:
            annotation = self.annotation(expr)
            for index, item in enumerate(args[:1] if kind == "array_repeat" else args):
                if isinstance(annotation, TupleType) and kind == "tuple":
                    expected = annotation.elements[index]
                elif isinstance(annotation, ArrayType) and kind in {"array", "array_repeat"}:
                    expected = annotation.element
                elif is_vector(annotation) and (kind == "vector" or kind == "vector_repeat" and index == 0):
                    expected = annotation.arguments[0]
                else:
                    expected = None
                # Element types here come from inference, so their 'static lifetimes are not checked.
                cursor = self.lower_stored_value(item, None, scope, cursor)
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
            if expr is not self.function.body:
                for local in inner.owned:
                    cursor = self.emit(cursor, "storage_dead", Place(local), local.token)
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
            if borrowing_iterator(value_type):
                cursor, loan = self.lower_iterator(value, scope, cursor)
                cursor = self.declare(source, types, scope, cursor, loan=loan)
                if loan and loan.expiring and types:
                    holder = Place(scope.find(source.token))
                    cursor = self.emit(cursor, "end_temporaries", holder, source.token)
                return cursor
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
            cursor, origin, loan = self.lower_pattern_input(value, scope, cursor, (source,), extend_to=scope)
            access = self.pattern_accesses(pattern, origin, value_type)
            cursor = self.read_pattern(cursor, access.tests | access.reads, source.token)
            if otherwise:
                self.lower(otherwise, Scope(scope), cursor)
            cursor = self.move_pattern(cursor, access.moves, source.token)
            cursor = self.declare(source, types, scope, cursor, loan=loan, borrows=access.borrows)
            if loan and isinstance(annotation, ReferenceType) and annotation.lifetime == "'static":
                cursor = self.emit(cursor, "static_reference", origin, source.token)
            for name, (kind, _) in types.items():
                if isinstance(kind, ReferenceType):
                    holder = Place(scope.find(source.token.binding(name)))
                    if holder.local.loan and holder.local.loan.expiring:
                        cursor = self.emit(cursor, "end_temporaries", holder, source.token)
            return cursor
        if kind == "match":
            end, _ = self.lower_match(expr, scope, cursor)
            return end
        if kind == "discard":
            return self.lower(args[0], scope, cursor)
        if kind in {"if", "if_chain"}:
            condition, yes, no = args
            if kind == "if_chain":
                satisfied, failed, inner = self.lower_chain(condition, scope, cursor)
            else:
                satisfied = failed = self.lower(condition, scope, cursor)
                inner = scope
            yes_end = self.lower(yes, inner, satisfied)
            no_end = self.lower(no, scope, failed) if no else failed
            return self.join(yes_end, no_end)
        if kind in {"while", "while_let", "while_chain", "loop", "for"}:
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
            target_type = self.annotation(left)
            argument = None
            origins = None
            if target_type == STRING and token.text == "+=" and self.annotation(right) != NEVER:
                cursor, loan = self.lower_reference(right, scope, cursor)
                cursor, argument = self.keep_loan(cursor, loan, right.token)
            elif (token.text == "=" and isinstance(target_type, ReferenceType) and not is_static_str(target_type)
                  and ungroup(left).kind == "name" and self.annotation(right) != NEVER):
                cursor, origins = self.assigned_reference(ungroup(right), scope, cursor)
            else:
                cursor = self.lower_stored_value(right, target_type, scope, cursor)
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
            cursor = self.emit(cursor, action, target_place, token)
            if origins and not target_place.fields and not target_place.holder:
                cursor = self.emit(cursor, "points_to", target_place, token, roots=frozenset(origins))
            return self.emit(cursor, "read", argument, right.token) if argument else cursor
        if kind == "binary":
            if token.text in COMPARE and self.annotation(args[0]) not in CONST_TYPES:
                return self.lower_comparison(args, scope, cursor)
            left, right = args
            cursor = self.lower(left, scope, cursor)
            right_end = self.lower(right, scope, cursor)
            return self.join(cursor, right_end) if token.text in {"&&", "||"} else right_end
        if kind == "record":
            explicit, update = args
            layout = self.program.layout(self.annotation(expr))
            ctor = next((ctor for ctor in layout[1] if ctor.path == token.text), None) if layout else None
            fields = dict(ctor.fields) if ctor else {}
            template = self.program.constructors.get(token.text)
            declared = dict(template.fields) if template else {}
            for name, value in explicit:
                cursor = self.lower_stored_value(value, declared.get(name.text), scope, cursor)
            if update:
                cursor, place = self.lower_address(update, scope, cursor)
                if place is None:
                    return self.lower(update, scope, cursor)
                supplied = {name.text for name, _ in explicit}
                for name, annotation in fields.items():
                    if name not in supplied:
                        cursor = self.access(cursor, place.child(name), annotation, update.token, consume=True)
            return cursor
        if kind in {"call", "indirect_call", "vector_method", "string_method"}:
            cursor, _ = self.lower_call(expr, scope, cursor)
            return None if self.annotation(expr) == NEVER else cursor
        if kind in {"print", "panic", "format"}:
            if kind == "panic":
                self.lower_print(expr, scope, cursor)
                return None
            return self.lower_print(expr, scope, cursor)
        raise AssertionError(kind)

    def assigned_reference(self, right, scope, cursor):
        """Lower a reference assigned to a local, with the locals it borrows when they are traceable."""
        if right.kind in {"call", "indirect_call", "vector_method", "string_method"}:
            cursor, loan = self.lower_call(right, scope, cursor)
            return cursor, loan and [origin.local for origin in loan.regions()]
        cursor = self.lower(right, scope, cursor)
        if right.kind == "name":
            try:
                local = scope.find(right.token)
            except RustError:
                return cursor, None
            return cursor, local.loan and [origin.local for origin in local.loan.regions()]
        if right.kind == "borrow":
            root = place_root(right.args[0])
            if root.kind == "name" and root.token.text not in self.program.constructors:
                try:
                    local = scope.find(root.token)
                except RustError:
                    return cursor, None
                if local.loan is None and not local.static:
                    return cursor, [local]
        return cursor, None

    def lower_stored_value(self, expr, expected, scope, cursor):
        # Only a field declared as &'static str is checked; generic fields substituted with 'static are not.
        if is_static_str(expected) and self.annotation(expr) != NEVER:
            cursor, loan = self.lower_reference(expr, scope, cursor)
            cursor, holder = self.keep_loan(cursor, loan, expr.token)
            return self.emit(cursor, "static_reference", holder, expr.token)
        return self.lower(expr, scope, cursor)

    def lower_closure(self, expr, scope, cursor, *, consume=True):
        expr = ungroup(expr)
        if expr.kind == "name":
            return self.lower(expr, scope, cursor, consume=consume), scope.find(expr.token).loan
        if expr.kind == "clone":
            return self.lower_closure(expr.args[0], scope, cursor, consume=False)
        if expr.kind != "closure_value":
            # Closures produced by calls carry captures the checker does not trace.
            return self.lower(expr, scope, cursor, consume=consume), None
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

    def lower_iterator(self, expr, scope, cursor, *, consume=True):
        expr = ungroup(expr)
        if expr.kind == "name":
            return self.lower(expr, scope, cursor, consume=consume), scope.find(expr.token).loan
        if expr.kind == "clone":
            return self.lower_iterator(expr.args[0], scope, cursor, consume=False)
        if expr.kind == "reverse_iterator":
            return self.lower_iterator(expr.args[0], scope, cursor)
        if expr.kind == "map_iterator":
            source, mapper = expr.args
            iterator_loan = None
            if borrowing_iterator(self.annotation(source)):
                cursor, iterator_loan = self.lower_iterator(source, scope, cursor)
            else:
                cursor = self.lower(source, scope, cursor)
            if iterator_loan:
                cursor, holder = self.keep_loan(cursor, iterator_loan, source.token)
            mapper_type = self.annotation(mapper)
            mapper_loan = None
            if mapper_type.closure:
                if self.program.functions[mapper_type.closure].consumes_captures:
                    raise mapper.token.error("iterator map cannot use a closure that consumes its captures")
                cursor, mapper_loan = self.lower_closure(mapper, scope, cursor)
            else:
                cursor = self.lower(mapper, scope, cursor)
            if iterator_loan:
                cursor = self.emit(cursor, "read", holder, source.token)
            return cursor, self.merge_loans(tuple(loan for loan in (iterator_loan, mapper_loan) if loan), False)
        if expr.kind in {"iterator", "string_iterator"}:
            return self.lower_reference(expr.args[0], scope, cursor)
        return self.lower(expr, scope, cursor, consume=consume), None

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
        loan = None
        iterator = None
        if kind == "for":
            source, iterable, body = args
            if iterable.kind == "range":
                start, stop, _ = iterable.args
                first, last = self.annotation(start), self.annotation(stop)
                item_type = first if first in INTEGER_TYPES else last if last in INTEGER_TYPES else "i32"
                cursor = self.lower(stop, scope, self.lower(start, scope, cursor))
            else:
                iterable_type = self.annotation(iterable)
                item_type = self.program.iteration_item(iterable_type)
                if isinstance(iterable_type, ReferenceType):
                    cursor, loan = self.lower_reference(iterable, scope, cursor)
                elif borrowing_iterator(iterable_type):
                    cursor, loan = self.lower_iterator(iterable, scope, cursor)
                else:
                    cursor = self.lower(iterable, scope, cursor)
                if loan:
                    cursor, iterator = self.keep_loan(cursor, loan, iterable.token)
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
            elif kind == "while_chain":
                target.in_condition = True
                body_entry, failed, scope = self.lower_chain(args[0], scope, restart)
                target.in_condition = False
                self.connect(failed, finish)
            elif kind == "while_let":
                condition, source, _ = args
                target.in_condition = True
                body_entry, origin, loan = self.lower_pattern_input(condition, scope, restart, (source,))
                target.in_condition = False
                pattern, types = self.program.patterns[id(source)]
                annotation = self.annotation(condition)
                access = self.pattern_accesses(pattern, origin, annotation)
                body_entry = self.read_pattern(body_entry, access.tests | access.reads, source.token)
                if uncovered([(pattern,)], (annotation,), self.program) is not None:
                    self.connect(body_entry, finish)
                body_entry = self.move_pattern(body_entry, access.moves, source.token)
                scope = Scope(scope)
                body_entry = self.declare(source, types, scope, body_entry, loan=loan, borrows=access.borrows)
            if kind == "for":
                step = self.emit(restart, "read", iterator, token) if iterator else restart
                self.connect(step, finish)
                item_loan = loan if isinstance(item_type, ReferenceType) else None
                item = Place(Local(replace(source.token, text="<iteration item>"), True, item_loan))
                body_entry = self.emit(step, "declare", item, source.token)
                pattern, types = self.program.patterns[id(source)]
                access = self.pattern_accesses(pattern, item, item_type)
                body_entry = self.read_pattern(body_entry, access.tests | access.reads, source.token)
                body_entry = self.move_pattern(body_entry, access.moves, source.token)
                scope = Scope(scope)
                scope.owned.add(item.local)
                body_entry = self.declare(source, types, scope, body_entry, loan=loan, borrows=access.borrows)
            self.connect(self.lower(body, scope, body_entry), restart)
        finally:
            self.loops.pop()
        return finish

    def lower_chain(self, conditions, scope, cursor):
        inner = Scope(scope)
        failures = []
        for source, condition in conditions:
            if source is None:
                cursor = self.lower(condition, inner, cursor)
                failures.append(cursor)
                continue
            cursor, origin, loan = self.lower_pattern_input(condition, inner, cursor, (source,), extend_to=inner)
            pattern, types = self.program.patterns[id(source)]
            annotation = self.annotation(condition)
            access = self.pattern_accesses(pattern, origin, annotation)
            cursor = self.read_pattern(cursor, access.tests | access.reads, source.token)
            if uncovered([(pattern,)], (annotation,), self.program) is not None:
                failures.append(cursor)
            cursor = self.move_pattern(cursor, access.moves, source.token)
            cursor = self.declare(source, types, inner, cursor, loan=loan, borrows=access.borrows)
        return cursor, self.join(*failures), inner

    def lower_match(self, expr, scope, cursor, *, result_type=None, extend_to=None):
        scrutinee, arms = expr.args
        cursor, origin, scrutinee_loan = self.lower_pattern_input(
            scrutinee, scope, cursor, (source for source, _, _ in arms))
        finishes = []
        results = []
        prepared = []
        protected = {}
        for source, guard, body in arms:
            pattern, types = self.program.patterns[id(source)]
            access = self.pattern_accesses(pattern, origin, self.annotation(scrutinee))
            protected.update(access.tests)
            prepared.append((source, guard, body, types, access))
        for source, guard, body, types, access in prepared:
            tested = self.read_pattern(cursor, access.tests | access.reads, source.token)
            inner = Scope(scope)
            ready = self.declare(source, types, inner, tested, loan=scrutinee_loan, borrows=access.borrows)
            if guard:
                earlier = self.guarded, self.guard_borrows, self.guard_tests
                self.guarded = self.guarded | set(inner.bindings.values())
                self.guard_borrows = (*self.guard_borrows, *access.reads, *access.moves)
                self.guard_tests = (*self.guard_tests, *protected)
                try:
                    ready = self.lower(guard, inner, ready)
                finally:
                    self.guarded, self.guard_borrows, self.guard_tests = earlier
                cursor = self.join(tested, ready)
            else:
                cursor = tested
            ready = self.move_pattern(ready, access.moves, source.token)
            if result_type:
                end, loan = self.lower_reference(body, inner, ready, extend_to=extend_to)
                finishes.append(self.leave_reference_scope(end, loan, inner, body.token))
                if loan:
                    results.append(loan)
            else:
                finishes.append(self.lower(body, inner, ready))
        loan = self.merge_loans(tuple(results), result_type.mutable) if result_type else None
        return self.join(*finishes), loan

    def lower_pattern_input(self, expr, scope, cursor, sources, *, extend_to=None):
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
            references = [kind for source in sources for kind, _ in self.program.patterns[id(source)][1].values()
                          if isinstance(kind, ReferenceType)]
            if references:
                promoted = not any(kind.mutable for kind in references) and self.promotable(expr)
                local = Local(replace(expr.token, text="<pattern temporary>"), not promoted, static=promoted,
                              temporary=extend_to is None and not promoted)
                origin = Place(local)
                cursor = self.emit(cursor, "declare", origin, expr.token)
                if extend_to is not None and not promoted:
                    extend_to.owned.add(local)
        return cursor, origin, None

    def declare(self, source, types, scope, cursor, *, initialized=True, loan=None, borrows=None):
        action = "declare" if initialized else "uninitialized"
        borrows = borrows or {}
        for name, (annotation, mutable) in types.items():
            carries_loan = (isinstance(annotation, ReferenceType) or borrowing_iterator(annotation)
                            or (isinstance(annotation, FunctionType) and annotation.closure))
            binding_loan = loan if carries_loan else None
            locations = borrows.get(name, ())
            if locations and isinstance(annotation, ReferenceType):
                paths = [self.start_loan(cursor, place, annotation.mutable, source.token) for place in locations]
                branches, loans = zip(*paths)
                cursor = branches[0] if len(branches) == 1 else self.join(*branches)
                binding_loan = self.merge_loans(loans, annotation.mutable)
            if binding_loan is None and is_static_str(annotation):
                cursor, binding_loan = self.static_loan(cursor, source.token)
            local = Local(source.token.binding(name), mutable, binding_loan)
            scope.bindings[name] = local
            scope.owned.add(local)
            cursor = self.emit(cursor, action, Place(local), source.token)
        return cursor

    def pattern_accesses(self, pattern, origin, annotation):
        access = PatternAccesses()
        if origin is not None:
            self.collect_pattern(pattern, origin, access, annotation)
        return access

    def collect_pattern(self, pattern, place, access, annotation, *, opaque=False, projection=None):
        projection = projection if projection is not None else place
        for alias in pattern.aliases:
            self.collect_pattern(alias, place, access, annotation, opaque=opaque, projection=projection)
        if pattern.alternatives:
            for option in pattern.alternatives:
                self.collect_pattern(option, place, access, annotation, opaque=opaque, projection=projection)
        elif pattern.binding:
            if pattern.by_reference:
                access.reads[place] = None
                access.borrows.setdefault(pattern.binding, {})[projection] = None
            elif self.program.is_copy(annotation):
                access.reads[place] = None
            else:
                access.moves[place] = annotation
        elif isinstance(pattern.tag, Constructor):
            ctor = pattern.tag
            if ctor.style == "reference":
                self.collect_pattern(pattern.children[0], self.referent(place.local), access, annotation.inner)
                return
            variant = "::" in ctor.path
            if variant and len(self.program.layout(ctor.annotation)[1]) > 1:
                access.tests[place] = None
            opaque = opaque or (variant and place.holder is None)
            for child, (field, kind) in zip(pattern.children, ctor.fields):
                projected = place if opaque else place.child(field)
                self.collect_pattern(child, projected, access, kind, opaque=opaque, projection=projection.child(field))
        elif isinstance(pattern.tag, SlicePattern):
            access.tests[place] = None
            for index, child in enumerate(pattern.children):
                if place.holder:
                    offset = index if index < pattern.tag.prefix else index - len(pattern.children)
                    projected = place.child(str(offset))
                else:
                    projected = replace(place, indexed=True)
                self.collect_pattern(child, projected, access, annotation.element, opaque=not place.holder)
        elif pattern.tag is not None:
            access.tests[place] = None

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


def rounded_float(value, annotation):
    value = float(value)
    if annotation == "f64":
        return value
    try:
        return Float32(struct.unpack("!f", struct.pack("!f", value))[0])
    except OverflowError:
        return Float32(math.copysign(math.inf, value))


def float_literal_value(value, annotation, token):
    rounded = rounded_float(value, annotation)
    if token.text[:1].isdigit() and not math.isfinite(rounded):
        raise token.error(f"floating-point literal is out of range for {annotation}")
    return rounded


def float_binary(word, left, right, annotation):
    if word == "/":
        if right == 0.0:
            sign = math.copysign(1.0, left) * math.copysign(1.0, right)
            value = math.nan if left == 0.0 or math.isnan(left) else math.copysign(math.inf, sign)
        else:
            value = left / right
    elif word == "%":
        value = math.nan if right == 0.0 or math.isinf(left) else math.fmod(left, right)
    else:
        value = ARITHMETIC[word](left, right)
    return rounded_float(value, annotation)


def scalar_cast_supported(actual, target):
    if target == "char":
        return actual == "u8"
    if target in FLOAT_TYPES:
        return actual in NUMBER_TYPES
    return actual in CONST_VALUE_TYPES


def convert_scalar(value, target):
    number = ordinal(value)
    if target in FLOAT_TYPES:
        return rounded_float(number, target)
    if target == "char":
        return Character(chr(number))
    if isinstance(number, float):
        lower, upper = INTEGER_BOUNDS[target]
        if math.isnan(number):
            return 0
        if number >= upper:
            return upper - 1
        if number <= lower:
            return lower
        return math.trunc(number)
    return truncate_integer(int(number), target)


def complement(value, annotation):
    return not value if annotation == "bool" else truncate_integer(~value, annotation)


def binary(word, left, right, token, integer="i32", *, runtime=False):
    if integer in FLOAT_TYPES:
        return float_binary(word, left, right, integer)
    if word == "+" and integer == STRING:
        return left + display(right, False)
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


def decimal_float(value):
    text = repr(float(value))
    if isinstance(value, Float32):
        for digits in range(1, 10):
            candidate = format(value, f".{digits}g")
            if rounded_float(float(candidate), "f32") == value:
                text = candidate
                break
    return Decimal(text)


def format_exponent(value, kind, precision):
    if isinstance(value, float) and not math.isfinite(value):
        return format_float(value, False)
    decimal = decimal_float(value) if isinstance(value, float) else Decimal(value)
    if precision is None:
        text = format(decimal, "e")
    elif isinstance(value, float):
        text = format(value, f".{precision}e")
    else:
        text = format(decimal, f".{precision}e")
    coefficient, exponent = text.split("e")
    if precision is None and "." in coefficient:
        coefficient = coefficient.rstrip("0").rstrip(".")
    return f"{coefficient}{kind}{int(exponent)}"


def format_float(value, debug):
    if math.isnan(value):
        return "NaN"
    if math.isinf(value):
        return "-inf" if value < 0 else "inf"
    decimal = decimal_float(value)
    if debug and value != 0.0 and not -4 <= decimal.adjusted() < 16:
        return format_exponent(value, "e", None)
    text = format(decimal, "f")
    text = text.rstrip("0").rstrip(".") if "." in text else text
    return text + ".0" if debug and "." not in text else text


def pretty_debug(value, indent=0):
    while isinstance(value, (Address, DynAddress, SliceAddress, StringAddress, BoxValue, RcValue)):
        value = value.read()
    if USER_FORMAT is not None and isinstance(value, Value):
        text = USER_FORMAT(value, True, alternate=True)
        if text is not None:
            return text.replace("\n", "\n" + " " * indent)
    if isinstance(value, MapValue) and value.entries:
        entries = MAP_ORDER(value)
        if all(item[1] is MAP_MEMBER for item in entries):
            lines = [pretty_debug(key, indent + 4) for key, _ in entries]
        else:
            lines = [f"{pretty_debug(key, indent + 4)}: {pretty_debug(item, indent + 4)}" for key, item in entries]
        return "{\n" + "".join(" " * (indent + 4) + line + ",\n" for line in lines) + " " * indent + "}"
    if isinstance(value, ArrayValue) and value.items:
        opening, closing = "[", "]"
        entries = [pretty_debug(item, indent + 4) for item in value.items]
    elif isinstance(value, Value) and value.fields:
        ctor = value.constructor
        name = ctor.path.rsplit("::", 1)[-1].rsplit(MODULE_SEPARATOR, 1)[-1]
        entries = [pretty_debug(item, indent + 4) for item in value.fields]
        if ctor.style == "record":
            opening, closing = f"{name} {{", "}"
            entries = [f"{field}: {entry}" for (field, _), entry in zip(ctor.fields, entries)]
        else:
            opening, closing = f"{name}(", ")"
    else:
        return display(value, True)
    lines = "".join(" " * (indent + 4) + entry + ",\n" for entry in entries)
    return opening + "\n" + lines + " " * indent + closing


def format_value(value, annotation, spec):
    while isinstance(value, (Address, DynAddress, SliceAddress, StringAddress, BoxValue, RcValue)):
        value = value.read()
    if USER_FORMAT is not None and isinstance(value, Value) and spec.kind in {"", "?"}:
        # A Display or Debug impl decides its own padding; Rust ignores width unless the impl uses it.
        text = USER_FORMAT(value, spec.kind == "?", alternate=spec.alternate, spec=spec)
        if text is not None:
            return text
    kind = spec.kind
    numeric = annotation in NUMBER_TYPES
    prefix = ""
    if kind in {"b", "o", "x", "X"}:
        if value < 0:
            value %= 1 << INTEGER_WIDTHS[annotation]
        text = format(value, kind)
        if spec.alternate:
            prefix = {"b": "0b", "o": "0o", "x": "0x", "X": "0x"}[kind]
    elif kind in {"e", "E"}:
        text = format_exponent(value, kind, spec.precision)
    elif isinstance(value, float) and spec.precision is not None and math.isfinite(value):
        text = format(value, f".{spec.precision}f")
    elif kind == "?" and spec.alternate:
        text = pretty_debug(value)
    else:
        text = display(value, kind == "?")
        if spec.precision is not None and not numeric and kind == "":
            text = text[:spec.precision]
    sign = ""
    if numeric and text.startswith("-"):
        sign, text = "-", text[1:]
    elif numeric and spec.sign == "+" and text != "NaN":
        sign = "+"
    width = max(0, spec.width - len(sign + prefix + text))
    if numeric and spec.zero:
        return sign + prefix + "0" * width + text
    align = spec.align or (">" if numeric else "<")
    left = width if align == ">" else width // 2 if align == "^" else 0
    return spec.fill * left + sign + prefix + text + spec.fill * (width - left)


USER_FORMAT = None
MAP_ORDER = None
MAP_MEMBER = object()
FORMATTER = MODULE_SEPARATOR.join(("std", "fmt", "Formatter"))


def display(value, debug):
    if isinstance(value, (Address, DynAddress, SliceAddress, StringAddress, BoxValue, RcValue)):
        return display(value.read(), debug)
    if isinstance(value, WeakValue):
        return "(Weak)"
    if USER_FORMAT is not None and isinstance(value, Value):
        text = USER_FORMAT(value, debug)
        if text is not None:
            return text
    if isinstance(value, MapValue):
        entries = MAP_ORDER(value) if MAP_ORDER else value.entries.values()
        if all(item[1] is MAP_MEMBER for item in entries):
            return "{" + ", ".join(display(key, True) for key, _ in entries) + "}"
        return "{" + ", ".join(f"{display(key, True)}: {display(item, True)}" for key, item in entries) + "}"
    if isinstance(value, ArrayValue):
        return "[" + ", ".join(display(item, True) for item in value.items) + "]"
    if isinstance(value, ExitCodeValue):
        code = f"unix_exit_status({value.status})" if os.name == "posix" else str(value.status)
        return f"ExitCode({code})"
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
    if isinstance(value, float):
        return format_float(value, debug)
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
        global USER_FORMAT, MAP_ORDER
        USER_FORMAT = self.user_format
        MAP_ORDER = self.map_order
        self.track_drops = any(impl.trait == DROP for impl in program.impls)
        self.implementations = {}
        self.temporaries = []
        self.program = program
        self.functions = program.functions
        self.stdout = stdout
        self.type_arguments = {}
        self.statics = {name: Binding(value.value, False) for name, value in program.statics.items()}

    def match_bindings(self, source, address):
        pattern, types = self.program.patterns[id(source)]
        for bindings in pattern.bindings(address.read(), address):
            scope = {}
            for name, item in bindings.items():
                annotation, mutable = types[name]
                if isinstance(item, FunctionValue) and item.environment and self.program.is_copy(annotation):
                    item = self.clone_value(item)
                if (self.track_drops and isinstance(address, Address) and isinstance(address.binding, Binding)
                        and not isinstance(item, (Address, SliceAddress, StringAddress, DynAddress))
                        and not self.program.is_copy(self.program.normalize(substitute(annotation, self.type_arguments)))):
                    address.binding.moved = True
                scope[name] = Binding(item, mutable)
            yield scope

    def eval_chain(self, conditions, scope):
        inner = Scope(scope)
        for source, condition in conditions:
            if source is None:
                if not self.eval(condition, inner):
                    return None
                continue
            bindings = next(self.match_bindings(source, self.address(condition, inner)), None)
            if bindings is None:
                return None
            inner.bindings.update(bindings)
        return inner

    def clone_value(self, value):
        if isinstance(value, Value) and value.constructor.path and "Clone" in self.program.traits:
            implementation = self.cached_implementation(value.constructor.annotation, "Clone")
            if implementation:
                impl, arguments = implementation
                if "clone" in impl.methods:
                    return self.call(impl.methods["clone"].name.text, (shared(value),), arguments)
        if isinstance(value, FunctionValue) and value.environment:
            bindings = dict(value.environment)
            for capture in self.program.functions[value.name].captures:
                if capture.by_move:
                    binding = bindings[capture.name.key]
                    bindings[capture.name.key] = Binding(self.clone_value(binding.value), binding.mutable)
            return replace(value, environment=bindings)
        if isinstance(value, IteratorValue):
            if value.cursor is not None:
                first, second = itertools.tee(value.cursor[0])
                value.cursor[0] = first
                return replace(value, cursor=[second])
            collection = value.collection if value.borrowed else self.clone_value(value.collection)
            return replace(value, collection=collection, mappers=tuple(map(self.clone_value, value.mappers)))
        if isinstance(value, BoxValue):
            return BoxValue(Address(Binding(self.clone_value(value.read()), True)), value.concrete)
        if isinstance(value, MapValue):
            return MapValue({key: [self.clone_value(item[0]), self.clone_value(item[1])]
                             for key, item in value.entries.items()}, value.ordered)
        if isinstance(value, RcValue):
            value.counts[0] += 1
            return RcValue(value.cell, value.counts, value.concrete)
        if isinstance(value, WeakValue):
            value.counts[1] += 1
            return WeakValue(value.cell, value.counts, value.concrete)
        if isinstance(value, ArrayValue):
            return ArrayValue(tuple(map(self.clone_value, value.items)))
        if isinstance(value, Value):
            return Value(value.constructor, tuple(map(self.clone_value, value.fields)))
        return value

    def mapped_items(self, values, mappers):
        for value in values:
            for mapper in mappers:
                value = self.call(mapper.name, (value,), mapper.arguments, mapper.environment)
            yield value

    def iteration_values(self, collection, annotation, token):
        iterator = collection if isinstance(collection, IteratorValue) else None
        if iterator and iterator.cursor is not None:
            return iterator.cursor[0]
        if iterator is None:
            concrete = self.program.concrete_type(
                self.program.normalize(substitute(annotation, self.type_arguments)))
            while is_box(concrete) and isinstance(collection, BoxValue):
                # Box<I> iterates as I, and Box<dyn Iterator> as the boxed concrete iterator.
                inner = concrete.arguments[0]
                concrete = self.program.concrete_type(collection.concrete) if isinstance(inner, DynType) else inner
                collection = collection.read()
            if isinstance(concrete, DynType) and isinstance(collection, DynAddress):
                concrete = self.program.concrete_type(collection.concrete)
                collection = collection.read()
            if isinstance(collection, IteratorValue):
                return self.iteration_values(collection, concrete, token)
            if self.program.builtin_iteration_item(concrete) is None:
                return self.trait_iteration(collection, concrete)
        if iterator:
            collection, borrowed = iterator.collection, iterator.borrowed
        else:
            borrowed = isinstance(annotation, ReferenceType)
        stored = collection.read() if borrowed else collection
        if iterator and iterator.string_mode:
            values = map(Character, stored) if iterator.string_mode == "chars" else stored.encode("utf-8")
        elif isinstance(stored, Value):
            present = stored.constructor.path.rsplit("::", 1)[-1] in {"Some", "Ok"}
            if not present:
                values = ()
            elif borrowed:
                values = (collection.child(replace(token, text="0")),)
            else:
                values = stored.fields
        elif borrowed:
            values = (collection.child(replace(token, text=str(index))) for index in range(len(stored.items)))
        else:
            values = stored.items
        if iterator and iterator.reverse:
            values = reversed(tuple(values))
        if iterator and iterator.mappers:
            values = self.mapped_items(values, iterator.mappers)
        return values

    def receiver_width(self):
        receiver = self.intrinsic_receiver
        if isinstance(receiver, ReferenceType):
            receiver = receiver.inner
        return INTEGER_WIDTHS[receiver]

    def builtin_operator(self, symbol, values, receiver, result, token):
        """Apply a std::ops trait method on primitive operands."""
        def plain(value):
            while isinstance(value, (Address, StringAddress)):
                value = value.read()
            return value
        kind = receiver.inner if isinstance(receiver, ReferenceType) else receiver
        if symbol.endswith("=") and len(symbol) > 1 and symbol not in {"==", "!=", "<=", ">="}:
            address, right = values
            current = address.read()
            operator = symbol[:-1]
            if kind == STRING:
                address.write(current + display(plain(right), False))
            elif kind in FLOAT_TYPES:
                address.write(rounded_float(float_binary(operator, current, plain(right), kind), kind))
            else:
                address.write(binary(operator, current, plain(right), token, kind, runtime=True))
            return None
        if len(values) == 1:
            value = plain(values[0])
            if symbol == "-":
                return rounded_float(-value, kind) if kind in FLOAT_TYPES else checked(-value, token, kind, runtime=True)
            return (not value) if kind == "bool" else complement(value, kind)
        left, right = map(plain, values)
        if kind == STRING:
            return left + display(right, False)
        if kind in FLOAT_TYPES:
            return rounded_float(float_binary(symbol, left, right, kind), kind)
        if kind == "bool":
            return BITWISE[symbol](left, right)
        return binary(symbol, left, right, token, kind, runtime=True)

    def builtin_compare(self, name, left, right, result):
        if name == "eq":
            return self.equal(left, right)
        order = self.compare(left, right)
        if order is None:
            return self.variant(result, "None")
        variant = "Less" if order < 0 else "Greater" if order > 0 else "Equal"
        ctor = next(ctor for ctor in self.program.layout(ORDERING)[1] if ctor.path.endswith(f"::{variant}"))
        ordering = Value(ctor, ())
        return ordering if name == "cmp" else self.variant(result, "Some", ordering)

    def cached_implementation(self, annotation, trait, other=None):
        key = (annotation, trait, other)
        if key not in self.implementations:
            wanted = (other,) if trait in {"PartialEq", "PartialOrd"} else None
            self.implementations[key] = (self.program.implementation(annotation, trait, arguments_wanted=wanted)
                                         if trait in self.program.traits else None)
        return self.implementations[key]

    def user_compare(self, left, right, equality):
        """Compare through PartialEq, PartialOrd, or Ord impls written in Rust; None means no impl applies."""
        annotation = left.constructor.annotation
        other = right.constructor.annotation if isinstance(right, Value) and right.constructor.path else (
            "bool" if isinstance(right, bool) else "char" if isinstance(right, Character) else
            STRING if isinstance(right, str) else None)
        if equality:
            implementation = self.cached_implementation(annotation, "PartialEq", other)
            if implementation:
                impl, arguments = implementation
                same = self.call(impl.methods["eq"].name.text, (shared(left), shared(right)), arguments)
                return 0 if same else 1
        for trait, method in (("PartialOrd", "partial_cmp"), ("Ord", "cmp")):
            implementation = self.cached_implementation(annotation, trait, other if trait == "PartialOrd" else None)
            if implementation:
                impl, arguments = implementation
                order = self.call(impl.methods[method].name.text, (shared(left), shared(right)), arguments)
                if method == "partial_cmp":
                    if order.constructor.path.endswith("::None"):
                        return "unordered"
                    order = order.fields[0]
                return ordering_number(order)
        return None

    def map_key(self, value):
        """A hashable stand-in for a key value, reading through references and boxes."""
        while isinstance(value, (Address, StringAddress, SliceAddress, DynAddress, BoxValue, RcValue)):
            value = value.read()
        if isinstance(value, Value):
            return (value.constructor.path, tuple(map(self.map_key, value.fields)))
        if isinstance(value, ArrayValue):
            return ("[", tuple(map(self.map_key, value.items)))
        if isinstance(value, MapValue):
            return ("{", tuple(sorted(map(repr, value.entries))))
        if isinstance(value, bool):
            return ("bool", value)
        if isinstance(value, str):
            return str(value)
        return value

    def map_order(self, value):
        entries = list(value.entries.values())
        if value.ordered:
            entries.sort(key=functools.cmp_to_key(lambda left, right: self.compare(left[0], right[0])))
        return entries

    def collect(self, values, target):
        """Build a collection of the requested type from iterator items."""
        name = target.name if isinstance(target, AppliedType) else target
        if target == STRING:
            return "".join(display(value, False) for value in values)
        if name in {OPTION, RESULT}:
            items = []
            for value in values:
                value = map_of(value)
                variant = value.constructor.path.rsplit("::", 1)[-1]
                if variant in {"None", "Err"}:
                    return Value(next(ctor for ctor in self.program.layout(target)[1]
                                      if ctor.path.endswith(f"::{variant}")), value.fields)
                items.append(value.fields[0])
            return self.variant(target, "Some" if name == OPTION else "Ok", self.collect(items, target.arguments[0]))
        if name in MAP_TYPES:
            collected = MapValue(ordered=MAP_TYPES[name].startswith("BTree"))
            for value in values:
                if MAP_TYPES[name].endswith("Set"):
                    collected.entries.setdefault(self.map_key(value), [value, MAP_MEMBER])
                else:
                    key, item = map_of(value).fields
                    collected.entries[self.map_key(key)] = [key, item]
            return collected
        element = collection_element(target)
        items = ArrayValue(tuple(plain_item(self, value, element) if element is not None else value
                                 for value in values))
        if name == DEQUE:
            return Value(self.program.layout(target)[1][0], (items,))
        if name == HEAP:
            ordered = sorted(items.items, key=functools.cmp_to_key(self.compare), reverse=True)
            return Value(self.program.layout(target)[1][0], (ArrayValue(tuple(ordered)),))
        return items

    def drop_value(self, value):
        """Run Rust drop glue: Drop impls first, then fields; Rc values release their counts."""
        if isinstance(value, RcValue):
            value.counts[0] -= 1
            if value.counts[0] == 0:
                self.drop_value(value.cell.value)
        elif isinstance(value, WeakValue):
            value.counts[1] -= 1
        elif isinstance(value, Value):
            if value.constructor.path:
                implementation = self.cached_implementation(value.constructor.annotation, DROP)
                if implementation:
                    impl, arguments = implementation
                    self.call(impl.methods["drop"].name.text, (Address(Binding(value, True)),), arguments)
            for item in value.fields:
                self.drop_value(item)
        elif isinstance(value, ArrayValue):
            for item in value.items:
                self.drop_value(item)
        elif isinstance(value, MapValue):
            for key, item in value.entries.values():
                self.drop_value(key)
                if item is not MAP_MEMBER:
                    self.drop_value(item)
        elif isinstance(value, BoxValue):
            self.drop_value(value.read())

    def drop_temporaries(self, mark, statement):
        """Drop temporaries a statement created, except a borrowed initializer that a let extends."""
        created = self.temporaries[mark:]
        del self.temporaries[mark:]
        if statement.kind == "let" and statement.args[2] is not None and ungroup(statement.args[2]).kind == "borrow":
            return
        for binding in reversed(created):
            if not binding.moved:
                binding.moved = True
                self.drop_value(binding.value)

    def drop_scope(self, bindings):
        for binding in reversed(list(bindings.values())):
            if not binding.moved:
                binding.moved = True
                self.drop_value(binding.value)

    def call_value(self, callee, *arguments):
        """Call a closure, function pointer, or reference to one from Python."""
        while isinstance(callee, (Address, BoxValue, DynAddress)):
            callee = callee.read()
        return self.call(callee.name, arguments, callee.arguments, callee.environment)

    def equal(self, left, right):
        return self.compare(left, right, equality=True) == 0

    @staticmethod
    def slice_parts(items):
        if isinstance(items, SliceAddress):
            return items.address, items.start
        return items, 0

    def range_bounds(self, bounds, size):
        """Resolve a std range value against a length, or (None, None) when out of range."""
        name = bounds.constructor.path.rsplit(MODULE_SEPARATOR, 1)[-1]
        fields = dict(zip((field for field, _ in bounds.constructor.fields), bounds.fields))
        start = fields.get("start", 0)
        stop = fields.get("end", size)
        if name in {"RangeInclusive", "RangeToInclusive"}:
            stop += 1
        if not 0 <= start <= stop <= size:
            return None, None
        return start, stop

    def pattern_matcher(self, pattern):
        """Return a function finding the first match of a str pattern at or after a byte offset."""
        if isinstance(pattern, Character):
            needle = pattern
        elif isinstance(pattern, FunctionValue) or isinstance(pattern, (Address, BoxValue)) and isinstance(
                text_of(pattern) if not isinstance(pattern, BoxValue) else None, FunctionValue):
            def predicate(char):
                return self.call_value(pattern, Character(char))
            return lambda text, start: next(((index, index + 1) for index in range(start, len(text))
                                             if predicate(text[index])), None)
        else:
            value = text_of(pattern)
            if isinstance(value, ArrayValue):
                characters = set(value.items)
                return lambda text, start: next(((index, index + 1) for index in range(start, len(text))
                                                 if text[index] in characters), None)
            needle = value

        def find(text, start):
            index = text.find(needle, start)
            return None if index < 0 else (index, index + len(needle))
        return find

    def find_pattern(self, text, pattern, *, anchored=None, reverse=False):
        """Find a pattern in text, returning character offsets (start, stop) or None."""
        find = self.pattern_matcher(pattern)
        matches = []
        start = 0
        while start <= len(text):
            found = find(text, start)
            if found is None:
                break
            if anchored == "start":
                return found if found[0] == 0 else None
            matches.append(found)
            if not reverse and anchored is None:
                return found
            start = found[1] if found[1] > found[0] else found[0] + 1
        if anchored == "end":
            return next((found for found in reversed(matches) if found[1] == len(text)), None)
        return matches[-1] if matches else None

    def split_text(self, text, pattern, limit=None):
        find = self.pattern_matcher(pattern)
        parts = []
        start = search = 0
        while limit is None or len(parts) < limit - 1:
            found = find(text, search)
            if found is None:
                break
            if found[1] == found[0]:
                if found[0] >= len(text):
                    break
                if found[0] == 0 and not parts and start == 0:
                    parts.append("")
                    search = 1
                    continue
                parts.append(text[start:found[0]])
                start = found[0]
                search = found[0] + 1
                continue
            parts.append(text[start:found[0]])
            start = search = found[1]
        parts.append(text[start:])
        return parts

    def formatter_value(self, alternate=False, spec=None):
        ctor = self.program.layout(FORMATTER)[1][0]
        count = AppliedType(OPTION, ("usize",))

        def optional(number):
            return self.variant(count, "None") if number is None else self.variant(count, "Some", number)
        width = spec.width if spec is not None and isinstance(spec.width, int) and spec.width else None
        precision = spec.precision if spec is not None and isinstance(spec.precision, int) else None
        align = {"<": 1, ">": 2, "^": 3}.get(spec.align if spec is not None else None, 0)
        return Value(ctor, ("", alternate, optional(width), optional(precision),
                            Character(spec.fill if spec is not None else " "), align,
                            bool(spec is not None and spec.sign == "+"), bool(spec is not None and spec.zero)))

    def user_format(self, value, debug, alternate=False, spec=None):
        """Format a value through a Display or Debug impl written in Rust, if it has one."""
        trait = "Debug" if debug else "Display"
        if trait not in self.program.traits:
            return None
        implementation = self.program.implementation(value.constructor.annotation, trait)
        if implementation is None:
            return None
        impl, arguments = implementation
        formatter = Address(Binding(self.formatter_value(alternate or bool(spec and spec.alternate), spec), True))
        result = self.call(impl.methods["fmt"].name.text, (Address(Binding(value, False)), formatter), arguments)
        if result.constructor.path.endswith("::Err"):
            raise RustPanic("a formatting trait implementation returned an error when the underlying stream did not")
        return formatter.read().fields[0]

    def builtin_fmt(self, value, formatter, debug, result):
        """`x.fmt(f)` for a built-in x, honouring the formatter's width, fill, and precision as Rust does."""
        stored = formatter.read()
        buffer, alternate, width, precision, fill, align, plus, zero = stored.fields
        spec = FormatSpec("?" if debug else "", str(fill), {1: "<", 2: ">", 3: "^"}.get(align),
                          "+" if plus else None, alternate, zero,
                          width.fields[0] if width.fields else 0, precision.fields[0] if precision.fields else None)
        while isinstance(value, (Address, DynAddress, BoxValue, RcValue)):
            value = value.read()
        annotation = ("f64" if isinstance(value, float) else "i64" if isinstance(value, int)
                      and not isinstance(value, bool) else None)
        text = format_value(value, annotation, spec)
        formatter.write(Value(stored.constructor, (buffer + text, *stored.fields[1:])))
        return self.variant(result, "Ok", None)

    def trait_iteration(self, value, annotation):
        """Drive any Iterator (or IntoIterator) implementation by calling its next method."""
        if isinstance(annotation, ReferenceType) and annotation.mutable:
            address, annotation = value, self.program.concrete_type(annotation.inner)
            if isinstance(address.read(), IteratorValue):
                yield from self.iteration_values(self.start_iterator(address), annotation, None)
                return
        else:
            implementation = self.program.implementation(annotation, ITERATOR)
            if implementation is None:
                impl, arguments = self.program.implementation(annotation, INTO_ITERATOR)
                value = self.call(impl.methods["into_iter"].name.text, (value,), arguments)
                target = self.program.concrete_type(self.program.normalize(
                    substitute(impl.associated["IntoIter"].body, arguments)))
                yield from self.iteration_values(value, target, None)
                return
            address = Address(Binding(value, True))
        impl, arguments = self.program.implementation(annotation, ITERATOR)
        name = impl.methods["next"].name.text
        while True:
            result = self.call(name, (address,), arguments)
            if result.constructor.path.endswith("::None"):
                return
            yield result.fields[0]

    def iterate(self, value, type_index):
        """Iterate a value whose type is the intrinsic's type argument at type_index."""
        annotation = self.program.concrete_type(self.intrinsic_types[type_index])
        if isinstance(value, Address) and not isinstance(annotation, ReferenceType) and isinstance(
                value.read(), (IteratorValue, Value)) and self.program.implements(annotation, ITERATOR):
            return self.trait_iteration(value, ReferenceType(annotation, True))
        return self.iteration_values(value, annotation, Token("0", 0, 0))

    def start_iterator(self, address):
        iterator = address.read()
        if iterator.cursor is None:
            values = self.iteration_values(iterator, None, Token("0", 0, 0))
            iterator = replace(iterator, cursor=[iter(values)])
            address.write(iterator)
        return iterator

    def iterator_next(self, address, result):
        iterator = self.start_iterator(address)
        sentinel = object()
        item = next(iterator.cursor[0], sentinel)
        return self.variant(result, "None") if item is sentinel else self.variant(result, "Some", item)

    def placeholder(self, annotation):
        # Wildcard patterns can inspect an aggregate's shape without reading
        # its fields. Ownership prevents an uninitialized field from escaping.
        declaration = self.program.layout(substitute(annotation, self.type_arguments))
        if declaration and declaration[1]:
            ctor = declaration[1][0]
            return Value(ctor, tuple(self.placeholder(kind) for _, kind in ctor.fields))
        return None

    def call(self, name, values, arguments=None, environment=None):
        if self.track_drops:
            parameters = {}
            try:
                return self.call_body(name, values, arguments, environment, parameters)
            finally:
                self.drop_scope(parameters)
        return self.call_body(name, values, arguments, environment, None)

    def call_body(self, name, values, arguments, environment, parameters):
        function = self.functions[name]
        if function.body is None:
            previous = self.type_arguments
            self.type_arguments = {p: self.program.normalize(substitute(t, previous))
                                   for p, t in (arguments or {}).items()}
            try:
                result = self.program.normalize(substitute(function.result, self.type_arguments))
                self.intrinsic_types = [self.type_arguments.get(parameter) for parameter in function.type_parameters]
                if name.startswith("@impl:"):
                    target = self.program.impls[int(name.split(":")[1])].target
                    owner = target.name if isinstance(target, AppliedType) else str(target)
                    method = name.rsplit(":", 1)[-1]
                    key = f"{owner.rsplit(MODULE_SEPARATOR, 1)[-1]}.{method}"
                    if key not in INTRINSICS:
                        key = "." + method
                else:
                    key = name.rsplit(MODULE_SEPARATOR, 1)[-1]
                self.intrinsic_receiver = function.params[0][2] if function.params else None
                return INTRINSICS[key](self, result, *values)
            finally:
                self.type_arguments = previous
        own = {
            param.key: Binding(value, mutable)
            for (param, mutable, _), value in zip(function.params, values)
            if param.text != "_"
        }
        if parameters is not None:
            parameters.update(own)
            for index, ((param, _, _), value) in enumerate(zip(function.params, values)):
                if param.text == "_":
                    parameters[("_", index)] = Binding(value, False)
        bindings = dict(environment or {}) | own
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
        if annotation in FLOAT_TYPES:
            return rounded_float(0.0, annotation)
        if annotation in PRIMITIVES:
            return {"bool": False, "char": Character("\0"), "()": None}[annotation]
        if is_str_reference(annotation):
            return Address(Binding("", False))
        if annotation == STRING:
            return ""
        if isinstance(annotation, ArrayType):
            return ArrayValue(tuple(self.default_value(annotation.element)
                                    for _ in range(annotation.length.value)))
        if is_vector(annotation):
            return ArrayValue(())
        name = annotation.name if isinstance(annotation, AppliedType) else annotation
        if name in MAP_TYPES:
            return MapValue(ordered=MAP_TYPES[name].startswith("BTree"))
        implementation = self.cached_implementation(annotation, "Default") if "Default" in self.program.traits else None
        if implementation:
            impl, arguments = implementation
            return self.call(impl.methods["default"].name.text, (), arguments)
        constructors = self.program.layout(annotation)[1]
        name = annotation.name if isinstance(annotation, AppliedType) else annotation
        path = self.program.default_variants.get(name)
        ctor = next(ctor for ctor in constructors if not path or ctor.path == path)
        return Value(ctor, tuple(self.default_value(kind) for _, kind in ctor.fields))

    def compare(self, left, right, equality=False):
        while isinstance(left, (Address, DynAddress, SliceAddress, StringAddress, BoxValue, RcValue)):
            left = left.read()
        while isinstance(right, (Address, DynAddress, SliceAddress, StringAddress, BoxValue, RcValue)):
            right = right.read()
        if isinstance(left, Value) and left.constructor.path:
            order = self.user_compare(left, right, equality)
            if order == "unordered":
                return None
            if order is not None:
                return order
        if isinstance(left, ExitCodeValue):
            return left.status - right.status
        if isinstance(left, MapValue):
            if equality:
                if left.entries.keys() != right.entries.keys():
                    return 1
                return 0 if all(self.compare(left.entries[key][1], right.entries[key][1], True) == 0
                                for key in left.entries) else 1
            mine, theirs = self.map_order(left), self.map_order(right)
            for (a_key, a_item), (b_key, b_item) in zip(mine, theirs):
                order = self.compare(a_key, b_key) or self.compare(a_item, b_item)
                if order:
                    return order
            return len(mine) - len(theirs)
        if isinstance(left, ArrayValue):
            for a, b in zip(left.items, right.items):
                result = self.compare(a, b, equality)
                if result != 0:
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
                result = self.compare(a, b, equality)
                if result != 0:
                    return result
            return 0
        if left is None:
            return 0
        if isinstance(left, float) and (math.isnan(left) or math.isnan(right)):
            return None
        return (left > right) - (left < right)

    def runtime_constructor(self, expr, template):
        annotation = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
        layout = self.program.layout(annotation)
        if layout:
            return next(ctor for ctor in layout[1] if ctor.path == template.path)
        return template

    def variant(self, annotation, name, *fields):
        ctor = next(ctor for ctor in self.program.layout(annotation)[1] if ctor.path.endswith(f"::{name}"))
        return Value(ctor, fields)

    def option_value(self, expr, fields):
        annotation = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
        variant = "Some" if fields else "None"
        ctor = next(ctor for ctor in self.program.layout(annotation)[1] if ctor.path.endswith(f"::{variant}"))
        return Value(ctor, fields)

    def format_argument(self, argument, evaluated, token, scope):
        if isinstance(argument, int):
            return evaluated[argument]
        if isinstance(argument, str):
            return scope.find(replace(token, text=argument)).value
        return substitute(argument, self.type_arguments).value

    def lazy_static(self, name):
        if name not in self.statics:
            item = self.program.lazy_items[name]
            self.statics[name] = Binding(self.call(f"@lazy:{name}", ()), item.mutable)
        return self.statics[name]

    def address(self, expr, scope):
        expr = ungroup(expr)
        if expr.kind == "static_item":
            return Address(self.statics[expr.token.text])
        if expr.kind == "lazy_static":
            return Address(self.lazy_static(expr.token.text))
        if expr.kind == "box_deref":
            value = self.eval(expr.args[0], scope)
            annotation = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
            if isinstance(annotation, DynType):
                concrete = self.program.concrete_type(value.concrete)
                return DynAddress(value.address, concrete)
            return value.address
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
            stored = root.read()
            utf8 = stored.encode("utf-8") if isinstance(stored, str) else None
            length = len(utf8) if utf8 is not None else len(stored.items)
            start = self.eval(first, scope) if first else 0
            stop = self.eval(last, scope) + int(inclusive) if last else length
            if not 0 <= start <= stop <= length:
                raise RustPanic(f"slice range {start}..{stop} is out of bounds for length {length}")
            if utf8 is not None:
                if any(offset < length and utf8[offset] & 0xC0 == 0x80 for offset in (start, stop)):
                    raise RustPanic("string index is not at a UTF-8 character boundary")
                if isinstance(root, StringAddress):
                    return StringAddress(root.address, root.start + start, root.start + stop)
                return StringAddress(root, start, stop)
            if isinstance(root, SliceAddress):
                return SliceAddress(root.address, root.start + start, root.start + stop)
            return SliceAddress(root, start, stop)
        binding = Binding(self.eval(expr, scope), True)
        if self.track_drops:
            self.temporaries.append(binding)
        return Address(binding)

    def eval(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "exit_code":
            return ExitCodeValue(self.eval(args[0], scope))
        if kind == "process_exit":
            raise ProcessExit(self.eval(args[0], scope))
        if kind == "box":
            value = self.eval(args[0], scope)
            concrete = self.program.normalize(substitute(self.program.expr_types[id(args[0])], self.type_arguments))
            return BoxValue(Address(Binding(value, True)), concrete)
        if kind == "box_deref":
            return self.address(expr, scope).read()
        if kind == "iterator_terminal":
            source = args[0]
            annotation = self.program.expr_types[id(source)]
            collection = self.address(source, scope) if ungroup(source).kind == "box_deref" else self.eval(source, scope)
            values = self.iteration_values(collection, annotation, source.token)
            result = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
            if token.text == "collect":
                return self.collect(values, result)
            if token.text == "count":
                return checked(sum(1 for _ in values), token, "usize", runtime=True)
            if isinstance(result, AppliedType) and result.name in {OPTION, RESULT}:
                inner = result.arguments[0]
                product = token.text == "product"
                total = rounded_float(float(int(product)), inner) if inner in FLOAT_TYPES else int(product)
                for value in values:
                    value = map_of(value)
                    variant = value.constructor.path.rsplit("::", 1)[-1]
                    if variant in {"None", "Err"}:
                        return Value(next(ctor for ctor in self.program.layout(result)[1]
                                          if ctor.path.endswith(f"::{variant}")), value.fields)
                    element = value.fields[0]
                    if inner in FLOAT_TYPES:
                        total = rounded_float(total * element if product else total + element, inner)
                    else:
                        total = binary("*" if product else "+", total, element, token, inner, runtime=True)
                return self.variant(result, "Some" if result.name == OPTION else "Ok", total)
            product = token.text == "product"
            total = int(product)
            if result in FLOAT_TYPES:
                total = rounded_float(float(total), result)
            for value in values:
                element = value.read() if isinstance(value, Address) else value
                if result in FLOAT_TYPES:
                    total = rounded_float(total * element if product else total + element, result)
                else:
                    total = binary("*" if product else "+", total, element, token, result, runtime=True)
            return total
        if kind == "iterator":
            return IteratorValue(self.eval(args[0], scope), borrowing_iterator(self.program.expr_types[id(expr)]))
        if kind == "enum_cast":
            value = map_of(self.eval(args[0], scope))
            return truncate_integer(self.program.discriminant(value.constructor), args[1])
        if kind == "chars_as_str":
            iterator = self.eval(args[0], scope)
            if iterator.cursor is None:
                return shared("".join(map(str, self.iteration_values(iterator, None, token))))
            first, second = itertools.tee(iterator.cursor[0])
            iterator.cursor[0] = first
            return shared("".join(map(str, second)))
        if kind == "range_value":
            start, end, inclusive = args
            annotation = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
            ctor = self.program.layout(annotation)[1][0]
            fields = tuple(self.eval(item, scope) for item in (start, end) if item is not None)
            return Value(ctor, (*fields, False) if inclusive and start is not None else fields)
        if kind == "lazy_adapter":
            source = args[0]
            annotation = self.program.normalize(substitute(self.program.expr_types[id(source)], self.type_arguments))
            collection = self.address(source, scope) if ungroup(source).kind == "box_deref" else self.eval(source, scope)
            values = self.iteration_values(collection, annotation, source.token)
            if token.text == "flatten":
                item = self.program.normalize(AssociatedType(self.program.concrete_type(annotation), "Item", ITERATOR))
                values = (inner for value in values
                          for inner in self.iteration_values(value, self.program.concrete_type(item), token))
            else:
                values = (self.clone_value(value.read()) for value in values)
            return IteratorValue(None, False, cursor=[iter(values)])
        if kind == "string_iterator":
            return IteratorValue(self.eval(args[0], scope), True, string_mode=token.text)
        if kind == "reverse_iterator":
            iterator = self.eval(args[0], scope)
            return replace(iterator, reverse=not iterator.reverse)
        if kind == "map_iterator":
            iterator = self.eval(args[0], scope)
            mapper = self.eval(args[1], scope)
            if mapper.environment and self.program.is_copy(self.program.expr_types[id(args[1])]):
                mapper = self.clone_value(mapper)
            return replace(iterator, mappers=(*iterator.mappers, mapper))
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
            while isinstance(callee, (Address, BoxValue, DynAddress)):
                callee = callee.read()
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
            conversion = self.program.try_conversions.get(id(expr))
            if conversion is not None:
                source, wanted = (self.program.normalize(substitute(kind, self.type_arguments)) for kind in conversion)
                impl, arguments = self.program.implementation(wanted, CONVERT_FROM, arguments_wanted=(source,))
                raise Return(Value(ctor, (self.call(impl.methods["from"].name.text, value.fields, arguments),)))
            raise Return(Value(ctor, value.fields))
        if kind == "enum_method":
            receiver, arguments, wanted, consuming = args
            value = self.eval(receiver, scope)
            actual = value.constructor.path.rsplit("::", 1)[-1]
            supplied = [self.eval(item, scope) for item in arguments]
            if not consuming:
                if self.track_drops and ungroup(receiver).kind not in {"name", "field", "index", "deref"}:
                    self.drop_value(value)
                return actual == wanted
            if actual == wanted:
                return value.fields[0]
            if token.text == "unwrap_or":
                return supplied[0]
            message = display(supplied[0], False) if token.text == "expect" else f"called {token.text} on {actual}"
            if actual == "Err":
                message += f": {display(value.fields[0], True)}"
            raise RustPanic(message)
        if kind == "vector_method":
            address = self.eval(args[0], scope)
            if token.text in {"as_slice", "as_mut_slice"}:
                return address
            arguments = tuple(self.eval(arg, scope) for arg in args[1:])
            items = address.read().items
            if token.text == "push":
                address.write(ArrayValue((*items, arguments[0])))
            elif token.text == "clear":
                address.write(ArrayValue(()))
            elif token.text == "truncate":
                address.write(ArrayValue(items[:arguments[0]]))
            elif token.text == "pop":
                if items:
                    address.write(ArrayValue(items[:-1]))
                return self.option_value(expr, items[-1:])
            return None
        if kind == "string_method":
            address = self.eval(args[0], scope)
            if token.text in {"as_str", "as_mut_str"}:
                return address
            arguments = tuple(self.eval(arg, scope) for arg in args[1:])
            value = address.read()
            if token.text in {"contains", "starts_with", "ends_with", "find", "rfind"}:
                pattern = display(arguments[0], False)
                if token.text == "contains":
                    return pattern in value
                if token.text == "starts_with":
                    return value.startswith(pattern)
                if token.text == "ends_with":
                    return value.endswith(pattern)
                offset = value.find(pattern) if token.text == "find" else value.rfind(pattern)
                found = (len(value[:offset].encode("utf-8")),) if offset >= 0 else ()
                return self.option_value(expr, found)
            if token.text in {"push", "push_str"}:
                address.write(value + display(arguments[0], False))
            elif token.text == "make_ascii_uppercase":
                address.write(value.translate(str.maketrans(string.ascii_lowercase, string.ascii_uppercase)))
            elif token.text == "make_ascii_lowercase":
                address.write(value.translate(str.maketrans(string.ascii_uppercase, string.ascii_lowercase)))
            elif token.text == "clear":
                address.write("")
            elif token.text == "truncate":
                try:
                    address.write(value.encode("utf-8")[:arguments[0]].decode("utf-8"))
                except UnicodeDecodeError:
                    raise RustPanic("String::truncate requires a UTF-8 character boundary") from None
            elif token.text == "pop":
                if value:
                    address.write(value[:-1])
                return self.option_value(expr, (Character(value[-1]),) if value else ())
            return None
        if kind == "macro_definition":
            scope.remember(token)
            return None
        if kind == "dyn_reference":
            value = self.eval(args[0], scope)
            if isinstance(value, DynAddress):
                return value
            concrete = self.program.normalize(substitute(args[1], self.type_arguments))
            return DynAddress(value, self.program.concrete_type(concrete))
        if kind in {"slice_reference", "string_reference"}:
            return self.eval(args[0], scope)
        if kind == "literal":
            if is_str_reference(args[0]):
                return Address(Binding(args[1], False))
            if args[0] in FLOAT_TYPES:
                return rounded_float(args[1], self.program.expr_types[id(expr)])
            value = args[1]
            if type(value) is int:
                kind = self.program.expr_types.get(id(expr))
                if kind in INTEGER_BOUNDS and not INTEGER_BOUNDS[kind][0] <= value < INTEGER_BOUNDS[kind][1]:
                    # Only reachable under #![allow(overflowing_literals)]: the literal wraps.
                    value = truncate_integer(value, kind)
            return value
        if kind == "byte_string":
            return Address(Binding(ArrayValue(args), False))
        if kind == "const_value":
            return substitute(args[0], self.type_arguments).value
        if kind == "static_item":
            return self.statics[token.text].value
        if kind == "lazy_const":
            return self.call(f"@lazy:{token.text}", ())
        if kind == "lazy_static":
            return self.lazy_static(token.text).value
        if kind == "string":
            return display(self.eval(args[0], scope), False) if args else ""
        if kind == "to_string":
            return display(self.eval(args[0], scope), False)
        if kind == "float_method":
            value = self.eval(args[0], scope)
            if token.text in FLOAT_PREDICATES:
                return FLOAT_PREDICATES[token.text](value)
            result = abs(value) if token.text == "abs" else math.nan if value < 0 else math.sqrt(value)
            return rounded_float(result, self.program.expr_types[id(expr)])
        if kind in {"array", "vector"}:
            return ArrayValue(tuple(self.eval(item, scope) for item in args))
        if kind == "tuple":
            fields = tuple(self.eval(item, scope) for item in args)
            kind = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
            return Value(self.program.layout(kind)[1][0], fields)
        if kind in {"array_repeat", "vector_repeat"}:
            value, length = args
            item = self.eval(value, scope)
            count = self.eval(length, scope) if kind == "vector_repeat" else substitute(length, self.type_arguments).value
            if count > 1_000_000:
                raise token.error("array exceeds interpreter allocation limit")
            return ArrayValue((item,) * count)
        if kind in {"index", "range_index"}:
            return self.address(expr, scope).read()
        if kind in {"length", "is_empty"}:
            root = self.address(args[0], scope)
            if isinstance(root, SliceAddress):
                length = root.length
            else:
                value = root.read()
                length = len(value.encode("utf-8")) if isinstance(value, str) else len(value.items)
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
            return self.clone_value(self.eval(args[0], scope))
        if kind == "name":
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "unit":
                return Value(self.runtime_constructor(expr, ctor), ())
            binding = scope.find(token)
            if id(expr) in self.program.moves:
                binding.moved = True
            return binding.value
        if kind == "field":
            value, name = args
            if id(expr) in self.program.moves:
                root = place_root(expr)
                if root.kind == "name":
                    scope.find(root.token).moved = True
            aggregate = self.eval(value, scope)
            return aggregate.fields[aggregate.constructor.index(name)]
        if kind == "record":
            explicit, update = args
            ctor = self.runtime_constructor(expr, self.program.constructors[token.text])
            supplied = [(ctor.index(name), self.eval(value, scope)) for name, value in explicit]
            fields = list(self.address(update, scope).read().fields) if update else [None] * len(ctor.fields)
            for index, value in supplied:
                fields[index] = value
            return Value(ctor, tuple(fields))
        if kind == "match":
            scrutinee, arms = args
            address = self.address(scrutinee, scope)
            for source, guard, body in arms:
                for bindings in self.match_bindings(source, address):
                    inner = Scope(scope, bindings)
                    if guard is None or self.eval(guard, inner):
                        try:
                            return self.eval(body, inner)
                        finally:
                            if self.track_drops:
                                self.drop_scope(bindings)
            raise AssertionError("type checker accepted a non-exhaustive match")
        if kind == "block":
            statements, tail = args
            inner = Scope(scope)
            try:
                for statement in statements:
                    mark = len(self.temporaries)
                    self.eval(statement, inner)
                    if self.track_drops and len(self.temporaries) > mark:
                        self.drop_temporaries(mark, statement)
                return self.eval(tail, inner) if tail else None
            except Break as signal:
                if not expr.label or not handles_jump(expr, signal):
                    raise
                return signal.args[1]
            finally:
                if self.track_drops:
                    self.drop_scope(inner.bindings)
        if kind == "let":
            source, annotation, value, otherwise = args
            if value is None:
                _, types = self.program.patterns[id(source)]
                for name, (kind, mutable) in types.items():
                    scope.bindings[name] = Binding(self.placeholder(kind), mutable)
                return None
            address = self.address(value, scope)
            bindings = next(self.match_bindings(source, address), None)
            if bindings is None:
                if otherwise:
                    self.eval(otherwise, scope)
                raise AssertionError("type checker accepted a fall-through let pattern")
            scope.bindings.update(bindings)
            return None
        if kind == "discard":
            value = self.eval(args[0], scope)
            if self.track_drops and not isinstance(value, (Address, SliceAddress, StringAddress, DynAddress)):
                self.drop_value(value)
            return None
        if kind == "if":
            condition, yes, no = args
            branch = yes if self.eval(condition, scope) else no
            return self.eval(branch, scope) if branch else None
        if kind == "if_chain":
            conditions, yes, no = args
            inner = self.eval_chain(conditions, scope)
            if inner is None:
                return self.eval(no, scope) if no else None
            try:
                return self.eval(yes, inner)
            finally:
                if self.track_drops:
                    self.drop_scope(inner.bindings)
        if kind in {"while", "while_let", "while_chain", "loop"}:
            body = args[-1]
            while True:
                try:
                    inner = scope
                    if kind == "while" and not self.eval(args[0], scope):
                        return None
                    if kind == "while_let":
                        condition, source, _ = args
                        bindings = next(self.match_bindings(source, self.address(condition, scope)), None)
                        if bindings is None:
                            return None
                        inner = Scope(scope, bindings)
                    if kind == "while_chain":
                        inner = self.eval_chain(args[0], scope)
                        if inner is None:
                            return None
                    try:
                        self.eval(body, inner)
                    finally:
                        if self.track_drops and inner is not scope:
                            self.drop_scope(inner.bindings)
                except Continue as signal:
                    if not handles_jump(expr, signal):
                        raise
                except Break as signal:
                    if not handles_jump(expr, signal):
                        raise
                    return signal.args[1]
        if kind == "for":
            source, iterable, body = args
            if iterable.kind == "range":
                start, stop, inclusive = iterable.args
                first, last = self.eval(start, scope), self.eval(stop, scope)
                if isinstance(first, Character):
                    values = (Character(chr(code)) for code in range(ord(first), ord(last) + int(inclusive))
                              if code not in SURROGATES)
                else:
                    values = range(first, last + int(inclusive))
            else:
                values = self.iteration_values(self.eval(iterable, scope), self.program.expr_types[id(iterable)],
                                               iterable.token)
            for value in values:
                address = Address(Binding(value, True))
                inner = Scope(scope, next(self.match_bindings(source, address)))
                try:
                    self.eval(body, inner)
                    if self.track_drops:
                        self.drop_scope(inner.bindings)
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
                if annotation in FLOAT_TYPES:
                    return rounded_float(-value, annotation)
                return checked(-value, token, annotation, runtime=True)
            return complement(value, annotation)
        if kind == "scalar_cast":
            value, target = args
            return convert_scalar(self.eval(value, scope), target)
        if kind == "assign":
            left, right = args
            value = self.eval(right, scope)
            address = self.address(left, scope)
            if self.track_drops and token.text == "=" and isinstance(address, Address):
                binding = address.binding
                if isinstance(binding, Binding):
                    if not binding.moved:
                        self.drop_value(address.read())
                    if not address.fields:
                        binding.moved = False
                else:
                    self.drop_value(address.read())
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
            if token.text in COMPARE and id(expr) in self.program.cross_compares:
                other = self.eval(right, scope)
                left_type, right_type, trait = (self.program.normalize(substitute(kind, self.type_arguments))
                                                if not isinstance(kind, str) or kind not in {"PartialEq", "PartialOrd"}
                                                else kind for kind in self.program.cross_compares[id(expr)])
                impl, arguments = self.program.implementation(left_type, trait, arguments_wanted=(right_type,))
                pair = (shared(map_of(value)), shared(map_of(other)))
                if trait == "PartialEq":
                    same = self.call(impl.methods["eq"].name.text, pair, arguments)
                    return same if token.text == "==" else not same
                order = self.call(impl.methods["partial_cmp"].name.text, pair, arguments)
                if order.constructor.path.endswith("::None"):
                    return False
                return COMPARE[token.text](ordering_number(order.fields[0]), 0)
            if token.text in COMPARE:
                other = self.eval(right, scope)
                compared = self.compare(value, other, equality=token.text in {"==", "!="})
                return token.text == "!=" if compared is None else COMPARE[token.text](compared, 0)
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
                trait, receiver, name, wanted = dispatch
                receiver = self.program.normalize(substitute(receiver, self.type_arguments))
                wanted = tuple(self.program.normalize(substitute(kind, self.type_arguments)) for kind in wanted)
                if isinstance(receiver, DynType):
                    receiver = values[0].concrete
                    if isinstance(values[0], DynAddress):
                        values = (values[0].address, *values[1:])
                receiver = self.program.concrete_type(receiver)
                cache_key = (receiver, trait, wanted)
                try:
                    implementation = self.program.dispatch_cache[cache_key]
                except KeyError:
                    implementation = self.program.dispatch_cache[cache_key] = self.program.implementation(
                        receiver, trait, arguments_wanted=wanted or None)
                except TypeError:
                    implementation = self.program.implementation(receiver, trait, arguments_wanted=wanted or None)
                if implementation is None and trait == TERMINATION:
                    return ExitCodeValue(self.exit_status(values[0], receiver))
                if implementation is None and trait in BUILTIN_OPERATORS:
                    symbol = BUILTIN_OPERATORS[trait]
                    result = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
                    return self.builtin_operator(symbol, values, receiver, result, token)
                if implementation is None and trait == "Clone" and name == "clone":
                    return self.clone_value(map_of(values[0]))
                if implementation is None and trait == "Default" and name == "default":
                    return self.default_value(receiver)
                if implementation is None and trait in {"PartialEq", "PartialOrd", "Ord"} and name in {
                        "eq", "partial_cmp", "cmp"}:
                    result = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
                    return self.builtin_compare(name, values[0], values[1], result)
                if implementation is None and trait in {"Display", "Debug"} and name == "fmt":
                    result = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
                    return self.builtin_fmt(values[0], values[1], trait == "Debug", result)
                if implementation is None and trait == INTO_ITERATOR and name == "into_iter":
                    if isinstance(values[0], IteratorValue) or iteration_item(receiver) is None:
                        return values[0]
                    return IteratorValue(values[0], isinstance(receiver, ReferenceType))
                if implementation is None and trait == ITERATOR and name == "next":
                    result = self.program.normalize(substitute(self.program.expr_types[id(expr)], self.type_arguments))
                    return self.iterator_next(values[0], result)
                impl, specialized = implementation or (Implementation(token, receiver, trait, (), {}, {}), {})
                if name in impl.methods:
                    prototype = function
                    function = impl.methods[name]
                    arguments = specialized | {actual: arguments[declared] for declared, actual in
                                               zip(prototype.method_parameters, function.method_parameters)}
                else:
                    arguments = arguments | {self.program.traits[trait].self_type: receiver}
            return self.call(function.name.text, values, arguments)
        if kind in {"print", "panic", "format"}:
            if not args:
                if kind == "panic":
                    raise RustPanic(token.text)
                print(file=sys.stderr if token.text == "eprintln" else self.stdout)
                return None
            template, *values = args
            evaluated = tuple(self.eval(value, scope) for value in values)
            rendered = []
            for field in self.program.formats[id(expr)][1]:
                if isinstance(field, str):
                    rendered.append(field)
                    continue
                item = self.format_argument(field.argument, evaluated, template.token, scope)
                spec = field.spec
                adjustments = {dimension: self.format_argument(count.argument, evaluated, template.token, scope)
                               for dimension, count in (("width", spec.width), ("precision", spec.precision))
                               if isinstance(count, FormatCount)}
                annotation = self.program.normalize(substitute(field.annotation, self.type_arguments))
                rendered.append(format_value(item, annotation, replace(spec, **adjustments)))
            rendered = "".join(rendered)
            if self.track_drops:
                # Format arguments are borrowed; temporaries among them, like a RefCell guard, drop here.
                for value_expr, value in zip(values, evaluated):
                    if ungroup(value_expr).kind not in {"name", "field", "index", "deref", "literal", "borrow"}:
                        self.drop_value(value)
            if kind == "panic":
                raise RustPanic(rendered)
            if kind == "format":
                return rendered
            destination = sys.stderr if token.text in {"eprint", "eprintln"} else self.stdout
            print(rendered, end="\n" if token.text in {"println", "eprintln"} else "", file=destination)
            return None
        raise AssertionError(kind)

    def exit_status(self, value, annotation):
        annotation = self.program.concrete_type(self.program.normalize(annotation))
        if annotation == "()":
            return 0
        if annotation == EXIT_CODE:
            return value.status
        if isinstance(annotation, AppliedType) and annotation.name == RESULT:
            if value.constructor.path.endswith("::Err"):
                print(f"Error: {display(value.fields[0], True)}", file=sys.stderr)
                return 1
            return self.exit_status(value.fields[0], annotation.arguments[0])
        impl, arguments = self.program.implementation(annotation, TERMINATION)
        result = self.call(impl.methods["report"].name.text, (value,), arguments)
        return result.status


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


def rustc_source(source, libraries, source_path=None, *, edition="2021"):
    # Keep the interpreter's library entry points through rustc expansion.
    prefix = "__rust_in_python_host"
    while prefix in source:
        prefix += "_"
    marker = prefix + ":derive:"
    formats = ("println", "print", "eprintln", "eprint", "format", "panic")
    shims = {f"{prefix}_{name}": name for name in (*formats, "vec")}
    repeat_shim = f"{prefix}_vec_repeat"
    shims[repeat_shim] = "vec"
    trees, _ = token_trees(lex(source, edition=edition))
    line_offsets = [0, *(match.end() for match in re.finditer("\n", source))]
    edits = []

    def offset(position):
        line, column = position
        return line_offsets[line - 1] + column - 1

    def keep_derives(items):
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
                builtin = [part[0] for part in pieces if len(part) == 1 and tree_text(part[0]) in DERIVABLE]
                if builtin:
                    closing = offset(attribute.closing.end)
                    annotation = "#[doc=" + json.dumps(marker + ",".join(part.text for part in builtin)) + "]"
                    edits.append((closing, closing, annotation))
                    if len(builtin) < len(pieces):
                        removed = {id(part) for part in builtin}
                        for position, part in enumerate(derives.items):
                            if id(part) not in removed:
                                continue
                            first = offset((part.line, part.column))
                            end = offset(part.end)
                            edits.append((first, end, " " * (end - first)))
                            if position + 1 < len(derives.items):
                                comma = derives.items[position + 1]
                                if tree_text(comma) == ",":
                                    at = offset((comma.line, comma.column))
                                    edits.append((at, at + 1, " "))
                    else:
                        first = offset((item.line, item.column))
                        blank = re.sub(r"[^\n]", " ", source[first:closing])
                        edits.append((first, closing, blank))
                index += 2
                continue
            if isinstance(item, TokenGroup):
                keep_derives(item.items)
            index += 1

    keep_derives(trees)
    declarations = [f"fn {shim}() {{}}" for shim in shims]
    macros = [f"macro_rules! {name} {{ ($($args:tt)*) => {{ {prefix}_{name}($($args)*) }}; }}"
              for name in formats]
    macros.append("macro_rules! vec {"
                  f"($value:expr; $count:expr) => {{ {repeat_shim}($value, $count) }};"
                  f"($($value:expr),* $(,)?) => {{ {prefix}_vec($($value),*) }};"
                  "}")
    macro = " ".join((*declarations, *macros))
    # Inner attributes belong first in a crate.
    start = 0
    while (start + 2 < len(trees) and tree_text(trees[start]) == "#" and tree_text(trees[start + 1]) == "!"
           and isinstance(trees[start + 2], TokenGroup)):
        start += 3
    include_root = False
    if source_path is not None and not edits and not start:
        try:
            include_root = Path(source_path).read_text(encoding="utf-8") == source
        except (OSError, UnicodeError):
            pass
    insertion = offset(trees[start - 1].closing.end) if start else 0
    edits.append((insertion, insertion, f" {macro} "))
    for first, end, replacement in sorted(edits, reverse=True):
        source = source[:first] + replacement + source[end:]
    with TemporaryDirectory(prefix="rust-in-python-") as folder:
        directory = Path(source_path).resolve().parent if source_path is not None else folder
        crate_name = Path(source_path).stem.replace("-", "_") if source_path else "rust_out"
        args = ["rustc", f"--edition={edition}", "--crate-name", crate_name,
                "--crate-type", "lib", "-Zunpretty=expanded,hygiene", "-"]
        for name, library in libraries.items():
            args.extend(("--extern", f"{name}={library}"))
        expand = functools.partial(subprocess.run, args, text=True, capture_output=True, timeout=30,
                                   env={**os.environ, "RUSTC_BOOTSTRAP": "1"}, check=False)
        try:
            if include_root:
                # Read an unmodified crate root by name to preserve rustc spans.
                wrapper = f"{macro} include!({display(os.fspath(source_path), True)});"
                compiler = expand(cwd=Path.cwd(), input=wrapper)
                # include! changes rustc's treatment of local exported macros.
                if compiler.returncode and "macro_expanded_macro_exports_accessed_by_absolute_paths" in compiler.stderr:
                    compiler = expand(cwd=directory, input=source)
            else:
                compiler = expand(cwd=directory, input=source)
        except subprocess.TimeoutExpired as error:
            raise RustError("procedural macro expansion exceeded 30 seconds") from error
        if compiler.returncode:
            raise RustError("rustc macro expansion failed\n" + compiler.stderr.strip())
    output, end = token_trees(lex(compiler.stdout, edition=edition,
                                  host_contexts=expanded_contexts(compiler.stdout)))
    return strip_host_items(output, marker, shims, repeat_shim, libraries), end


def expanded_contexts(source):
    _, separator, table = source.rpartition("\nSyntaxContexts:\n")
    if not separator:
        raise RustError("rustc did not emit expanded syntax contexts")
    contexts = {0: ()}
    for current, parent, kind in re.findall(
            r"(?m)^#(\d+): parent: #(\d+), outer_mark: \([^\n]*, (\w+)\)$", table):
        index, ancestor = int(current), int(parent)
        if index:
            base = contexts[ancestor]
            contexts[index] = base if kind == "Transparent" else (*base, (0, index))
    return contexts


def strip_host_items(items, marker, shims, repeat_shim, libraries, *, top=True):
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
        import_head = words[2] if len(words) > 2 and words[1] == "::" else words[1] if len(words) > 1 else None
        external_crate = len(words) > 2 and words[:2] == ("extern", "crate") and words[2] in {*libraries, "std"}
        host_import = words[0] == "use" and (import_head in libraries or (
            top and words[1:4] == ("std", "::", "prelude")
        ))
        if external_crate or host_import:
            while index < len(items) and tree_text(items[index]) != ";":
                index += 1
            index += 1
            continue
        if top and (len(words) >= 2 and words[0] == "fn" and words[1] in shims):
            index += 4
            continue
        if top and words[:2] == ("macro_rules", "!") and len(words) == 4 and isinstance(items[index + 3], TokenGroup):
            if any(token.text in shims for token in flatten_trees(items[index + 3].items)):
                index += 4
                continue
        if isinstance(current, Token) and current.text in shims:
            result.extend((replace(current, text="@host:" + shims[current.text]), replace(current, text="!")))
            if current.text == repeat_shim and index + 1 < len(items) and isinstance(items[index + 1], TokenGroup):
                group = items[index + 1]
                parts = list(group.items)
                separator = next((at for at, part in enumerate(parts) if tree_text(part) == ","), None)
                if separator is None:
                    raise current.error("invalid expanded vec repeat")
                parts[separator] = replace(parts[separator], text=";")
                parts = strip_host_items(tuple(parts), marker, shims, repeat_shim, libraries, top=False)
                result.append(replace(group, items=parts))
                index += 2
                continue
        elif isinstance(current, TokenGroup):
            inner = strip_host_items(current.items, marker, shims, repeat_shim, libraries, top=False)
            result.append(replace(current, items=inner))
        else:
            result.append(current)
        index += 1
    return tuple(result)


def macro_libraries(values):
    libraries = {}
    for item in values:
        name, separator, file = item.partition("=")
        if not separator or not name.isascii() or not name.isidentifier() or not file:
            raise RustError("expected a compiled Rust macro crate as NAME=PATH")
        path = Path(file).resolve()
        if not path.is_file():
            raise RustError(f"compiled Rust macro crate does not exist: {path}")
        if name in libraries:
            raise RustError(f"duplicate macro crate {name!r}")
        libraries[name] = path
    return libraries


INTRINSICS = {}


def intrinsic(function):
    """Register a Python body for a std function (rust_name), method (method_name), or Type__method."""
    name = function.__name__
    if name.startswith("rust_"):
        key = name.removeprefix("rust_")
    elif name.startswith("method_"):
        key = "." + name.removeprefix("method_")
    else:
        key = name.replace("__", ".", 1)
    INTRINSICS[key] = function
    return function


@intrinsic
def rust_vec_last(interpreter, result, vector):
    items = vector.read().items
    if not items:
        return interpreter.variant(result, "None")
    return interpreter.variant(result, "Some", vector.child(Token(str(len(items) - 1), 0, 0)))


ORDERING = MODULE_SEPARATOR.join(("std", "cmp", "Ordering"))


def items_of(value):
    while isinstance(value, (Address, SliceAddress, DynAddress, BoxValue)):
        value = value.read()
    return value.items


def text_of(value):
    while isinstance(value, (Address, StringAddress, SliceAddress, DynAddress, BoxValue)):
        value = value.read()
    return value


def child(address, index):
    return address.child(Token(str(index), 0, 0))


def shared(value):
    """A shared reference to a value that nothing else can reach."""
    return Address(Binding(value, False))


def option(interpreter, result, *value):
    return interpreter.variant(result, "Some", *value) if value else interpreter.variant(result, "None")


def element_type(result):
    """The T in Option<T>, Result<T, _>, Vec<T>, or Lazy<T>."""
    return result.arguments[0]


def sort_key(interpreter, compare):
    return functools.cmp_to_key(compare)


def lazy(values):
    return IteratorValue(None, False, cursor=[iter(values)])


@intrinsic
def method_first(interpreter, result, items):
    return option(interpreter, result, child(items, 0)) if items_of(items) else option(interpreter, result)


@intrinsic
def method_last(interpreter, result, items):
    count = len(items_of(items))
    return option(interpreter, result, child(items, count - 1)) if count else option(interpreter, result)


rust_first_mut = INTRINSICS[".first_mut"] = method_first
INTRINSICS[".last_mut"] = method_last


@intrinsic
def method_get(interpreter, result, items, index):
    if isinstance(index, Value):
        start, stop = interpreter.range_bounds(index, len(items_of(items)))
        if start is None:
            return option(interpreter, result)
        return option(interpreter, result, SliceAddress(*interpreter.slice_parts(items), start, stop)
                      if not isinstance(text_of(items), str) else shared(text_of(items).encode()[start:stop].decode()))
    return option(interpreter, result, child(items, index)) if index < len(items_of(items)) else option(interpreter, result)


INTRINSICS[".get_mut"] = method_get


@intrinsic
def method_split_first(interpreter, result, items):
    values = items_of(items)
    if not values:
        return option(interpreter, result)
    pair = element_type(result)
    return option(interpreter, result, Value(interpreter.program.layout(pair)[1][0],
                                             (child(items, 0), shared(ArrayValue(values[1:])))))


@intrinsic
def method_split_last(interpreter, result, items):
    values = items_of(items)
    if not values:
        return option(interpreter, result)
    pair = element_type(result)
    return option(interpreter, result, Value(interpreter.program.layout(pair)[1][0],
                                             (child(items, len(values) - 1), shared(ArrayValue(values[:-1])))))


@intrinsic
def method_split_at(interpreter, result, items, middle):
    value = text_of(items)
    if isinstance(value, str):
        data = value.encode()
        if middle > len(data):
            raise RustPanic("byte index out of bounds")
        parts = (shared(data[:middle].decode()), shared(data[middle:].decode()))
    else:
        values = items_of(items)
        if middle > len(values):
            raise RustPanic("mid > len")
        parts = (shared(ArrayValue(values[:middle])), shared(ArrayValue(values[middle:])))
    return Value(interpreter.program.layout(result)[1][0], parts)


@intrinsic
def method_contains(interpreter, result, items, needle):
    value = text_of(items)
    if isinstance(value, str):
        return interpreter.find_pattern(value, needle) is not None
    return any(interpreter.equal(item, needle) for item in value.items)


@intrinsic
def method_starts_with(interpreter, result, items, needle):
    value = text_of(items)
    if isinstance(value, str):
        found = interpreter.find_pattern(value, needle, anchored="start")
        return found is not None
    prefix = items_of(needle)
    return len(prefix) <= len(value.items) and all(map(interpreter.equal, value.items, prefix))


@intrinsic
def method_ends_with(interpreter, result, items, needle):
    value = text_of(items)
    if isinstance(value, str):
        return interpreter.find_pattern(value, needle, anchored="end") is not None
    suffix = items_of(needle)
    return len(suffix) <= len(value.items) and all(
        map(interpreter.equal, value.items[len(value.items) - len(suffix):], suffix))


@intrinsic
def method_swap(interpreter, result, items, a, b):
    values = list(items_of(items))
    for index in (a, b):
        if index >= len(values):
            raise RustPanic(f"index out of bounds: the len is {len(values)} but the index is {index}")
    values[a], values[b] = values[b], values[a]
    items.write(ArrayValue(tuple(values)))


@intrinsic
def method_reverse(interpreter, result, items):
    items.write(ArrayValue(tuple(reversed(items_of(items)))))


def ordering_number(value):
    name = value.constructor.path.rsplit("::", 1)[-1]
    return {"Less": -1, "Equal": 0, "Greater": 1}[name]


@intrinsic
def method_sort(interpreter, result, items):
    items.write(ArrayValue(tuple(sorted(items_of(items), key=functools.cmp_to_key(interpreter.compare)))))


INTRINSICS[".sort_unstable"] = method_sort


@intrinsic
def method_sort_by(interpreter, result, items, compare):
    def order(left, right):
        return ordering_number(interpreter.call_value(compare, shared(left), shared(right)))
    items.write(ArrayValue(tuple(sorted(items_of(items), key=functools.cmp_to_key(order)))))


INTRINSICS[".sort_unstable_by"] = method_sort_by


@intrinsic
def method_sort_by_key(interpreter, result, items, key):
    values = items_of(items)
    keys = [interpreter.call_value(key, shared(item)) for item in values]
    order = sorted(range(len(values)), key=functools.cmp_to_key(lambda a, b: interpreter.compare(keys[a], keys[b])))
    items.write(ArrayValue(tuple(values[index] for index in order)))


INTRINSICS[".sort_unstable_by_key"] = INTRINSICS[".sort_by_cached_key"] = method_sort_by_key


def binary_search(interpreter, result, values, probe):
    low, high = 0, len(values)
    while low < high:
        middle = (low + high) // 2
        order = probe(values[middle])
        if order == 0:
            return interpreter.variant(result, "Ok", middle)
        if order < 0:
            low = middle + 1
        else:
            high = middle
    return interpreter.variant(result, "Err", low)


@intrinsic
def method_binary_search(interpreter, result, items, target):
    return binary_search(interpreter, result, items_of(items), lambda item: interpreter.compare(item, target))


@intrinsic
def method_binary_search_by(interpreter, result, items, compare):
    return binary_search(interpreter, result, items_of(items),
                         lambda item: ordering_number(interpreter.call_value(compare, shared(item))))


@intrinsic
def method_binary_search_by_key(interpreter, result, items, target, key):
    return binary_search(interpreter, result, items_of(items),
                         lambda item: interpreter.compare(interpreter.call_value(key, shared(item)), target))


@intrinsic
def method_partition_point(interpreter, result, items, predicate):
    values = items_of(items)
    low, high = 0, len(values)
    while low < high:
        middle = (low + high) // 2
        if interpreter.call_value(predicate, shared(values[middle])):
            low = middle + 1
        else:
            high = middle
    return low


@intrinsic
def method_to_vec(interpreter, result, items):
    return ArrayValue(tuple(map(interpreter.clone_value, items_of(items))))


INTRINSICS[".into_vec"] = method_to_vec


@intrinsic
def method_repeat(interpreter, result, items, count):
    value = text_of(items)
    if isinstance(value, str):
        return value * count
    return ArrayValue(tuple(interpreter.clone_value(item) for _ in range(count) for item in value.items))


@intrinsic
def method_fill(interpreter, result, items, value):
    items.write(ArrayValue(tuple(interpreter.clone_value(value) for _ in items_of(items))))


@intrinsic
def method_rotate_left(interpreter, result, items, middle):
    values = items_of(items)
    items.write(ArrayValue(values[middle:] + values[:middle]))


@intrinsic
def method_rotate_right(interpreter, result, items, count):
    values = items_of(items)
    split = len(values) - count
    items.write(ArrayValue(values[split:] + values[:split]))


@intrinsic
def method_copy_from_slice(interpreter, result, items, source):
    values = items_of(source)
    if len(values) != len(items_of(items)):
        raise RustPanic("source slice length does not match destination slice length")
    items.write(ArrayValue(tuple(map(interpreter.clone_value, values))))


INTRINSICS[".clone_from_slice"] = method_copy_from_slice


def views(items, starts):
    root, offset = (items.address, items.start) if isinstance(items, SliceAddress) else (items, 0)
    return lazy(SliceAddress(root, offset + start, offset + stop) for start, stop in starts)


@intrinsic
def method_windows(interpreter, result, items, size):
    if size == 0:
        raise RustPanic("window size must be non-zero")
    count = len(items_of(items))
    return views(items, ((start, start + size) for start in range(count - size + 1)))


@intrinsic
def method_chunks(interpreter, result, items, size):
    if size == 0:
        raise RustPanic("chunk size must be non-zero")
    count = len(items_of(items))
    return views(items, ((start, min(start + size, count)) for start in range(0, count, size)))


@intrinsic
def method_chunks_exact(interpreter, result, items, size):
    if size == 0:
        raise RustPanic("chunk size must be non-zero")
    count = len(items_of(items)) // size * size
    return views(items, ((start, start + size) for start in range(0, count, size)))


@intrinsic
def method_rchunks(interpreter, result, items, size):
    if size == 0:
        raise RustPanic("chunk size must be non-zero")
    count = len(items_of(items))
    return views(items, ((max(stop - size, 0), stop) for stop in range(count, 0, -size)))


@intrinsic
def method_split(interpreter, result, items, separator):
    value = text_of(items)
    if isinstance(value, str):
        return lazy(shared(part) for part in interpreter.split_text(value, separator))
    values = value.items
    starts = []
    start = 0
    for index, item in enumerate(values):
        if interpreter.call_value(separator, shared(item)):
            starts.append((start, index))
            start = index + 1
    starts.append((start, len(values)))
    return views(items, starts)


@intrinsic
def method_concat(interpreter, result, items):
    values = [text_of(item) for item in items_of(items)]
    if result == STRING:
        return "".join(values)
    return ArrayValue(tuple(element for value in values for element in items_of(value)))


@intrinsic
def method_join(interpreter, result, items, separator):
    values = [text_of(item) for item in items_of(items)]
    if result == STRING:
        return text_of(separator).join(values) if not isinstance(separator, Character) else separator.join(values)
    joined = []
    gap = items_of(separator)
    for index, value in enumerate(values):
        if index:
            joined.extend(map(interpreter.clone_value, gap))
        joined.extend(map(interpreter.clone_value, items_of(value)))
    return ArrayValue(tuple(joined))


@intrinsic
def method_insert(interpreter, result, items, index, element):
    value = items.read()
    if isinstance(value, str):
        data = value.encode()
        if index > len(data) or (index < len(data) and data[index] & 0xC0 == 0x80):
            raise RustPanic("assertion failed: self.is_char_boundary(idx)")
        items.write(data[:index].decode() + display(element, False) + data[index:].decode())
        return None
    values = value.items
    if index > len(values):
        raise RustPanic(f"insertion index (is {index}) should be <= len (is {len(values)})")
    items.write(ArrayValue((*values[:index], element, *values[index:])))


INTRINSICS[".insert_str"] = method_insert


@intrinsic
def method_remove(interpreter, result, items, index):
    value = items.read()
    if isinstance(value, str):
        data = value.encode()
        if index >= len(data):
            raise RustPanic("cannot remove a char from the end of a string")
        removed = data[index:].decode()[0]
        items.write(data[:index].decode() + data[index + len(removed.encode()):].decode())
        return Character(removed)
    values = value.items
    if index >= len(values):
        raise RustPanic(f"removal index (is {index}) should be < len (is {len(values)})")
    items.write(ArrayValue((*values[:index], *values[index + 1:])))
    return values[index]


@intrinsic
def method_swap_remove(interpreter, result, items, index):
    values = list(items_of(items))
    if index >= len(values):
        raise RustPanic(f"swap_remove index (is {index}) should be < len (is {len(values)})")
    removed = values[index]
    values[index] = values[-1]
    values.pop()
    items.write(ArrayValue(tuple(values)))
    return removed


@intrinsic
def method_append(interpreter, result, items, other):
    items.write(ArrayValue(items_of(items) + items_of(other)))
    other.write(ArrayValue(()))


@intrinsic
def method_extend_from_slice(interpreter, result, items, other):
    items.write(ArrayValue(items_of(items) + tuple(map(interpreter.clone_value, items_of(other)))))


@intrinsic
def method_extend(interpreter, result, items, source):
    added = tuple(interpreter.iterate(source, -1))
    value = items.read()
    if isinstance(value, str):
        items.write(value + "".join(display(item, False) for item in added))
    else:
        copies = not isinstance(interpreter.intrinsic_types[0], ReferenceType)
        items.write(ArrayValue(value.items + tuple(interpreter.clone_value(item.read())
                                                   if copies and isinstance(item, (Address, SliceAddress))
                                                   else item for item in added)))


@intrinsic
def method_retain(interpreter, result, items, keep):
    value = items.read()
    if isinstance(value, str):
        items.write("".join(char for char in value if interpreter.call_value(keep, Character(char))))
        return None
    items.write(ArrayValue(tuple(item for index, item in enumerate(value.items)
                                 if interpreter.call_value(keep, child(items, index)))))


INTRINSICS[".retain_mut"] = method_retain


@intrinsic
def method_dedup(interpreter, result, items):
    kept = []
    for item in items_of(items):
        if not kept or not interpreter.equal(kept[-1], item):
            kept.append(item)
    items.write(ArrayValue(tuple(kept)))


@intrinsic
def method_dedup_by_key(interpreter, result, items, key):
    kept, keys = [], []
    for item in items_of(items):
        current = interpreter.call_value(key, Address(Binding(item, True)))
        if not keys or not interpreter.equal(keys[-1], current):
            kept.append(item)
            keys.append(current)
    items.write(ArrayValue(tuple(kept)))


@intrinsic
def method_dedup_by(interpreter, result, items, same):
    kept = []
    for item in items_of(items):
        if not kept or not interpreter.call_value(same, Address(Binding(item, True)), Address(Binding(kept[-1], True))):
            kept.append(item)
    items.write(ArrayValue(tuple(kept)))


@intrinsic
def method_split_off(interpreter, result, items, at):
    value = items.read()
    if isinstance(value, str):
        data = value.encode()
        items.write(data[:at].decode())
        return data[at:].decode()
    if at > len(value.items):
        raise RustPanic(f"`at` split index (is {at}) should be <= len (is {len(value.items)})")
    items.write(ArrayValue(value.items[:at]))
    return ArrayValue(value.items[at:])


@intrinsic
def method_resize(interpreter, result, items, length, value):
    values = items_of(items)
    extra = tuple(interpreter.clone_value(value) for _ in range(length - len(values)))
    items.write(ArrayValue(values[:length] + extra))


@intrinsic
def method_resize_with(interpreter, result, items, length, make):
    values = items_of(items)
    extra = tuple(interpreter.call_value(make) for _ in range(length - len(values)))
    items.write(ArrayValue(values[:length] + extra))


@intrinsic
def method_drain(interpreter, result, items, bounds):
    value = items.read()
    size = len(value.encode()) if isinstance(value, str) else len(value.items)
    start, stop = interpreter.range_bounds(bounds, size)
    if start is None:
        raise RustPanic(f"range end index out of range for slice of length {size}")
    if isinstance(value, str):
        data = value.encode()
        items.write((data[:start] + data[stop:]).decode())
        return lazy(map(Character, data[start:stop].decode()))
    items.write(ArrayValue(value.items[:start] + value.items[stop:]))
    return lazy(value.items[start:stop])


def range_step(value, annotation, step):
    if isinstance(value, Character):
        code = ord(value) + step
        if code in SURROGATES:
            code += step * len(SURROGATES)
        return Character(chr(code))
    return value + step


@intrinsic
def rust_range_next(interpreter, result, address):
    value = address.read()
    start, end = value.fields
    if interpreter.compare(start, end) >= 0:
        return option(interpreter, result)
    address.write(Value(value.constructor, (range_step(start, None, 1), end)))
    return option(interpreter, result, start)


@intrinsic
def rust_range_inclusive_next(interpreter, result, address):
    value = address.read()
    start, end, exhausted = value.fields
    if exhausted or interpreter.compare(start, end) > 0:
        return option(interpreter, result)
    done = interpreter.compare(start, end) == 0
    address.write(Value(value.constructor, (start if done else range_step(start, None, 1), end, done)))
    return option(interpreter, result, start)


@intrinsic
def rust_range_from_next(interpreter, result, address):
    value = address.read()
    start, = value.fields
    address.write(Value(value.constructor, (range_step(start, None, 1),)))
    return option(interpreter, result, start)


@intrinsic
def rust_replace(interpreter, result, destination, value):
    previous = destination.read()
    destination.write(value)
    return previous


@intrinsic
def rust_swap(interpreter, result, first, second):
    left, right = first.read(), second.read()
    first.write(right)
    second.write(left)


@intrinsic
def rust_compare(interpreter, result, left, right):
    return interpreter.compare(left, right)


@intrinsic
def rust_iter_max(interpreter, result, iterator, last):
    best = None
    found = False
    for item in interpreter.iterate(iterator, 0):
        if not found or (interpreter.compare(item, best) >= 0 if last else interpreter.compare(item, best) < 0):
            best, found = item, True
    return interpreter.variant(result, "Some", best) if found else interpreter.variant(result, "None")


@intrinsic
def rust_iter_collect(interpreter, result, iterator):
    return ArrayValue(tuple(interpreter.iterate(iterator, 0)))


@intrinsic
def rust_lazy_next(interpreter, result, address):
    return interpreter.iterator_next(address, result)


def make_tuple(interpreter, annotation, *values):
    return Value(interpreter.program.layout(annotation)[1][0], values)


def byte_offset(text, index):
    return len(text[:index].encode("utf-8"))


def lines_of(text):
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    return [line[:-1] if line.endswith("\r") else line for line in lines]


@intrinsic
def method_char_indices(interpreter, result, text):
    value = text_of(text)
    pair = element_type(result)
    return lazy(make_tuple(interpreter, pair, byte_offset(value, index), Character(char))
                for index, char in enumerate(value))


@intrinsic
def method_rsplit(interpreter, result, text, pattern):
    return lazy(map(shared, reversed(interpreter.split_text(text_of(text), pattern))))


@intrinsic
def method_splitn(interpreter, result, text, count, pattern):
    if count == 0:
        return lazy(())
    return lazy(map(shared, interpreter.split_text(text_of(text), pattern, count)))


@intrinsic
def method_rsplitn(interpreter, result, text, count, pattern):
    remaining = text_of(text)
    if count == 0:
        return lazy(())
    pieces = []
    while len(pieces) < count - 1:
        found = interpreter.find_pattern(remaining, pattern, reverse=True)
        if found is None:
            break
        pieces.append(remaining[found[1]:])
        remaining = remaining[:found[0]]
    pieces.append(remaining)
    return lazy(map(shared, pieces))


@intrinsic
def method_split_terminator(interpreter, result, text, pattern):
    parts = interpreter.split_text(text_of(text), pattern)
    if parts and parts[-1] == "":
        parts.pop()
    return lazy(map(shared, parts))


@intrinsic
def method_split_inclusive(interpreter, result, text, pattern):
    value = text_of(text)
    parts, start = [], 0
    while start < len(value):
        found = interpreter.pattern_matcher(pattern)(value, start)
        if found is None:
            break
        stop = max(found[1], found[0] + 1)
        parts.append(value[start:stop])
        start = stop
    if start < len(value):
        parts.append(value[start:])
    return lazy(map(shared, parts))


@intrinsic
def method_split_whitespace(interpreter, result, text):
    return lazy(map(shared, text_of(text).split()))


INTRINSICS[".split_ascii_whitespace"] = method_split_whitespace


@intrinsic
def method_lines(interpreter, result, text):
    return lazy(map(shared, lines_of(text_of(text))))


@intrinsic
def method_split_once(interpreter, result, text, pattern):
    value = text_of(text)
    found = interpreter.find_pattern(value, pattern)
    if found is None:
        return option(interpreter, result)
    return option(interpreter, result, make_tuple(interpreter, element_type(result),
                                                  shared(value[:found[0]]), shared(value[found[1]:])))


@intrinsic
def method_rsplit_once(interpreter, result, text, pattern):
    value = text_of(text)
    found = interpreter.find_pattern(value, pattern, reverse=True)
    if found is None:
        return option(interpreter, result)
    return option(interpreter, result, make_tuple(interpreter, element_type(result),
                                                  shared(value[:found[0]]), shared(value[found[1]:])))


@intrinsic
def method_trim(interpreter, result, text):
    return shared(text_of(text).strip())


@intrinsic
def method_trim_start(interpreter, result, text):
    return shared(text_of(text).lstrip())


@intrinsic
def method_trim_end(interpreter, result, text):
    return shared(text_of(text).rstrip())


INTRINSICS[".trim_left"] = method_trim_start
INTRINSICS[".trim_right"] = method_trim_end


def trim_pattern(interpreter, value, pattern, start, end):
    if start:
        while value:
            found = interpreter.find_pattern(value, pattern, anchored="start")
            if found is None or found[1] == 0:
                break
            value = value[found[1]:]
    if end:
        while value:
            found = interpreter.find_pattern(value, pattern, anchored="end")
            if found is None or found[0] == len(value):
                break
            value = value[:found[0]]
    return shared(value)


@intrinsic
def method_trim_matches(interpreter, result, text, pattern):
    return trim_pattern(interpreter, text_of(text), pattern, True, True)


@intrinsic
def method_trim_start_matches(interpreter, result, text, pattern):
    return trim_pattern(interpreter, text_of(text), pattern, True, False)


@intrinsic
def method_trim_end_matches(interpreter, result, text, pattern):
    return trim_pattern(interpreter, text_of(text), pattern, False, True)


@intrinsic
def method_strip_prefix(interpreter, result, text, pattern):
    value = text_of(text)
    found = interpreter.find_pattern(value, pattern, anchored="start")
    return option(interpreter, result, shared(value[found[1]:])) if found else option(interpreter, result)


@intrinsic
def method_strip_suffix(interpreter, result, text, pattern):
    value = text_of(text)
    found = interpreter.find_pattern(value, pattern, anchored="end")
    return option(interpreter, result, shared(value[:found[0]])) if found else option(interpreter, result)


@intrinsic
def method_find(interpreter, result, text, pattern):
    value = text_of(text)
    found = interpreter.find_pattern(value, pattern)
    return option(interpreter, result, byte_offset(value, found[0])) if found else option(interpreter, result)


@intrinsic
def method_rfind(interpreter, result, text, pattern):
    value = text_of(text)
    found = interpreter.find_pattern(value, pattern, reverse=True)
    return option(interpreter, result, byte_offset(value, found[0])) if found else option(interpreter, result)


def all_matches(interpreter, value, pattern):
    find = interpreter.pattern_matcher(pattern)
    start = 0
    while start <= len(value):
        found = find(value, start)
        if found is None:
            return
        yield found
        start = found[1] if found[1] > found[0] else found[0] + 1


@intrinsic
def method_matches(interpreter, result, text, pattern):
    value = text_of(text)
    return lazy(shared(value[a:b]) for a, b in all_matches(interpreter, value, pattern))


@intrinsic
def method_rmatches(interpreter, result, text, pattern):
    value = text_of(text)
    return lazy(shared(value[a:b]) for a, b in reversed(list(all_matches(interpreter, value, pattern))))


@intrinsic
def method_match_indices(interpreter, result, text, pattern):
    value = text_of(text)
    pair = element_type(result)
    return lazy(make_tuple(interpreter, pair, byte_offset(value, a), shared(value[a:b]))
                for a, b in all_matches(interpreter, value, pattern))


@intrinsic
def method_replace(interpreter, result, text, pattern, replacement, count=None):
    value = text_of(text)
    output, start, done = [], 0, 0
    for a, b in all_matches(interpreter, value, pattern):
        if count is not None and done >= count:
            break
        output.append(value[start:a])
        output.append(text_of(replacement))
        start = b
        done += 1
    output.append(value[start:])
    return "".join(output)


@intrinsic
def method_replacen(interpreter, result, text, pattern, replacement, count):
    return method_replace(interpreter, result, text, pattern, replacement, count)


ASCII_UPPER = str.maketrans(string.ascii_lowercase, string.ascii_uppercase)
ASCII_LOWER = str.maketrans(string.ascii_uppercase, string.ascii_lowercase)


@intrinsic
def method_to_uppercase(interpreter, result, text):
    value = text_of(text)
    if isinstance(value, Character):
        return lazy(map(Character, value.upper()))
    return value.upper()


@intrinsic
def method_to_lowercase(interpreter, result, text):
    value = text_of(text)
    if isinstance(value, Character):
        return lazy(map(Character, value.lower()))
    return value.lower()


@intrinsic
def method_to_ascii_uppercase(interpreter, result, text):
    value = text_of(text)
    converted = value.translate(ASCII_UPPER)
    return Character(converted) if isinstance(value, Character) else converted


@intrinsic
def method_to_ascii_lowercase(interpreter, result, text):
    value = text_of(text)
    converted = value.translate(ASCII_LOWER)
    return Character(converted) if isinstance(value, Character) else converted


@intrinsic
def method_make_ascii_uppercase(interpreter, result, address):
    address.write(Character(address.read().translate(ASCII_UPPER)))


@intrinsic
def method_make_ascii_lowercase(interpreter, result, address):
    address.write(Character(address.read().translate(ASCII_LOWER)))


@intrinsic
def method_is_char_boundary(interpreter, result, text, index):
    data = text_of(text).encode()
    return index == len(data) or (index < len(data) and data[index] & 0xC0 != 0x80)


@intrinsic
def method_as_bytes(interpreter, result, text):
    return shared(ArrayValue(tuple(text_of(text).encode())))


@intrinsic
def method_into_bytes(interpreter, result, text):
    return ArrayValue(tuple(text_of(text).encode()))


@intrinsic
def method_eq_ignore_ascii_case(interpreter, result, text, other):
    return text_of(text).translate(ASCII_LOWER) == text_of(other).translate(ASCII_LOWER)


@intrinsic
def method_is_ascii(interpreter, result, text):
    return text_of(text).isascii()


@intrinsic
def method_escape_default(interpreter, result, text):
    return display(text_of(text), True)[1:-1]


INTRINSICS[".escape_debug"] = method_escape_default


@intrinsic
def method_from_utf8(interpreter, result, data):
    raw = bytes(items_of(data))
    try:
        return interpreter.variant(result, "Ok", raw.decode("utf-8"))
    except UnicodeDecodeError:
        error = result.arguments[1]
        return interpreter.variant(result, "Err", Value(interpreter.program.layout(error)[1][0], (data,)))


@intrinsic
def method_from_utf8_lossy(interpreter, result, data):
    return bytes(items_of(data)).decode("utf-8", errors="replace")


@intrinsic
def method_replace_range(interpreter, result, address, bounds, replacement):
    data = address.read().encode()
    start, stop = interpreter.range_bounds(bounds, len(data))
    if start is None:
        raise RustPanic("range out of bounds")
    address.write((data[:start] + text_of(replacement).encode() + data[stop:]).decode())


def char_predicate(test):
    def body(interpreter, result, value):
        return test(text_of(value))
    return body


RUST_WHITESPACE = set("\t\n\x0b\x0c\r \x85\xa0          "
                      "      　")
for _name, _test in {
    "is_alphabetic": str.isalpha, "is_numeric": lambda c: unicodedata.category(c) in {"Nd", "Nl", "No"},
    "is_alphanumeric": lambda c: c.isalpha() or unicodedata.category(c) in {"Nd", "Nl", "No"},
    "is_whitespace": lambda c: c in RUST_WHITESPACE, "is_uppercase": str.isupper, "is_lowercase": str.islower,
    "is_control": lambda c: unicodedata.category(c) == "Cc",
    "is_ascii_digit": lambda c: "0" <= c <= "9", "is_ascii_hexdigit": lambda c: c in string.hexdigits,
    "is_ascii_alphabetic": lambda c: c.isascii() and c.isalpha(),
    "is_ascii_alphanumeric": lambda c: c.isascii() and c.isalnum(),
    "is_ascii_uppercase": lambda c: "A" <= c <= "Z", "is_ascii_lowercase": lambda c: "a" <= c <= "z",
    "is_ascii_punctuation": lambda c: c in string.punctuation,
    "is_ascii_whitespace": lambda c: c in " \t\n\x0c\r", "is_ascii_graphic": lambda c: "!" <= c <= "~",
    "is_ascii_control": lambda c: c.isascii() and unicodedata.category(c) == "Cc",
}.items():
    INTRINSICS["." + _name] = char_predicate(_test)


@intrinsic
def method_to_digit(interpreter, result, value, radix):
    char = text_of(value)
    digit = int(char, 36) if char.isascii() and char.isalnum() else None
    if digit is None or digit >= radix:
        return option(interpreter, result)
    return option(interpreter, result, digit)


@intrinsic
def method_is_digit(interpreter, result, value, radix):
    char = text_of(value)
    return char.isascii() and char.isalnum() and int(char, 36) < radix


@intrinsic
def method_len_utf8(interpreter, result, value):
    return len(text_of(value).encode())


@intrinsic
def method_from_u32(interpreter, result, code):
    if code >= CHAR_LIMIT or code in SURROGATES:
        return option(interpreter, result)
    return option(interpreter, result, Character(chr(code)))


@intrinsic
def method_from_digit(interpreter, result, number, radix):
    if radix > 36:
        raise RustPanic("from_digit: radix is too high (maximum 36)")
    if number >= radix:
        return option(interpreter, result)
    return option(interpreter, result, Character("0123456789abcdefghijklmnopqrstuvwxyz"[number]))


def integer_result(value, annotation, operation="multiply"):
    low, high = INTEGER_BOUNDS[annotation]
    if not low <= value < high:
        raise RustPanic(f"attempt to {operation} with overflow")
    return value


def wrap(value, annotation):
    return truncate_integer(value, annotation)


@intrinsic
def method_pow(interpreter, result, base, exponent):
    return integer_result(base ** exponent, result)


@intrinsic
def method_abs(interpreter, result, value):
    if isinstance(value, float):
        return rounded_float(abs(value), result)
    return integer_result(abs(value), result, "negate")


@intrinsic
def method_signum(interpreter, result, value):
    if isinstance(value, float):
        return value if math.isnan(value) else rounded_float(math.copysign(1.0, value), result)
    return (value > 0) - (value < 0)


def checked_operation(operation):
    def body(interpreter, result, left, right=None):
        kind = element_type(result)
        try:
            value = operation(left, right)
        except ZeroDivisionError:
            return option(interpreter, result)
        low, high = INTEGER_BOUNDS[kind]
        return option(interpreter, result, value) if value is not None and low <= value < high else option(interpreter, result)
    return body


def rust_div(left, right):
    if right == 0:
        raise ZeroDivisionError
    quotient = abs(left) // abs(right)
    return quotient if (left >= 0) == (right >= 0) else -quotient


def rust_rem(left, right):
    return left - right * rust_div(left, right)


for _name, _operation in {
    "add": lambda a, b: a + b, "sub": lambda a, b: a - b, "mul": lambda a, b: a * b,
    "div": rust_div, "rem": rust_rem, "pow": lambda a, b: a ** b, "neg": lambda a, b: -a, "abs": lambda a, b: abs(a),
}.items():
    INTRINSICS[f".checked_{_name}"] = checked_operation(_operation)
    INTRINSICS[f".wrapping_{_name}"] = (lambda operation: lambda interpreter, result, left, right=None:
                                        wrap(operation(left, right), result))(_operation)
    INTRINSICS[f".saturating_{_name}"] = (lambda operation: lambda interpreter, result, left, right=None:
                                          min(max(operation(left, right), INTEGER_BOUNDS[result][0]),
                                              INTEGER_BOUNDS[result][1] - 1))(_operation)
    INTRINSICS[f".overflowing_{_name}"] = (lambda operation: lambda interpreter, result, left, right=None: make_tuple(
        interpreter, result, wrap(operation(left, right), result.elements[0]),
        not INTEGER_BOUNDS[result.elements[0]][0] <= operation(left, right) < INTEGER_BOUNDS[result.elements[0]][1]
    ))(_operation)


@intrinsic
def method_checked_shl(interpreter, result, value, amount):
    kind = element_type(result)
    if amount >= INTEGER_WIDTHS[kind]:
        return option(interpreter, result)
    return option(interpreter, result, wrap(value << amount, kind))


@intrinsic
def method_checked_shr(interpreter, result, value, amount):
    kind = element_type(result)
    if amount >= INTEGER_WIDTHS[kind]:
        return option(interpreter, result)
    return option(interpreter, result, value >> amount)


@intrinsic
def method_wrapping_shl(interpreter, result, value, amount):
    return wrap(value << (amount % INTEGER_WIDTHS[result]), result)


@intrinsic
def method_wrapping_shr(interpreter, result, value, amount):
    return value >> (amount % INTEGER_WIDTHS[result])


@intrinsic
def method_rem_euclid(interpreter, result, left, right):
    if isinstance(left, float):
        value = math.fmod(left, right)
        return rounded_float(value + abs(right) if value < 0 else value, result)
    if right == 0:
        raise RustPanic("attempt to calculate the remainder with a divisor of zero")
    return left % abs(right)


@intrinsic
def method_div_euclid(interpreter, result, left, right):
    if isinstance(left, float):
        quotient = math.trunc(left / right)
        if math.fmod(left, right) < 0:
            quotient = quotient - 1 if right > 0 else quotient + 1
        return rounded_float(float(quotient), result)
    if right == 0:
        raise RustPanic("attempt to divide by zero")
    remainder = left % abs(right)
    return integer_result((left - remainder) // right, result, "divide")


@intrinsic
def method_abs_diff(interpreter, result, left, right):
    return abs(left - right)


@intrinsic
def method_unsigned_abs(interpreter, result, value):
    return abs(value)


def unsigned_bits(value, width):
    return value % (1 << width)


@intrinsic
def method_count_ones(interpreter, result, value):
    return bin(unsigned_bits(value, interpreter.receiver_width())).count("1")


@intrinsic
def method_count_zeros(interpreter, result, value):
    width = interpreter.receiver_width()
    return width - bin(unsigned_bits(value, width)).count("1")


@intrinsic
def method_leading_zeros(interpreter, result, value):
    width = interpreter.receiver_width()
    return width - unsigned_bits(value, width).bit_length()


@intrinsic
def method_trailing_zeros(interpreter, result, value):
    width = interpreter.receiver_width()
    bits = unsigned_bits(value, width)
    return width if bits == 0 else (bits & -bits).bit_length() - 1


@intrinsic
def method_leading_ones(interpreter, result, value):
    width = interpreter.receiver_width()
    bits = unsigned_bits(value, width)
    return width - unsigned_bits(~bits, width).bit_length()


@intrinsic
def method_trailing_ones(interpreter, result, value):
    width = interpreter.receiver_width()
    bits = unsigned_bits(value, width)
    return ((bits ^ (bits + 1)).bit_length() - 1) if bits != (1 << width) - 1 else width


@intrinsic
def method_rotate_left(interpreter, result, value, amount):
    width = INTEGER_WIDTHS[result]
    bits, amount = unsigned_bits(value, width), amount % width
    return wrap(((bits << amount) | (bits >> (width - amount))) & ((1 << width) - 1), result)


@intrinsic
def method_rotate_right(interpreter, result, value, amount):
    width = INTEGER_WIDTHS[result]
    bits, amount = unsigned_bits(value, width), amount % width
    return wrap(((bits >> amount) | (bits << (width - amount))) & ((1 << width) - 1), result)


@intrinsic
def method_swap_bytes(interpreter, result, value):
    width = INTEGER_WIDTHS[result]
    return wrap(int.from_bytes(unsigned_bits(value, width).to_bytes(width // 8, "big"), "little"), result)


@intrinsic
def method_reverse_bits(interpreter, result, value):
    width = INTEGER_WIDTHS[result]
    return wrap(int(format(unsigned_bits(value, width), f"0{width}b")[::-1], 2), result)


@intrinsic
def method_is_power_of_two(interpreter, result, value):
    return value > 0 and value & (value - 1) == 0


@intrinsic
def method_next_power_of_two(interpreter, result, value):
    return integer_result(1 if value <= 1 else 1 << (value - 1).bit_length(), result)


@intrinsic
def method_isqrt(interpreter, result, value):
    if value < 0:
        raise RustPanic("argument of integer square root cannot be negative")
    return math.isqrt(value)


@intrinsic
def method_ilog2(interpreter, result, value):
    if value <= 0:
        raise RustPanic("argument of integer logarithm must be positive")
    return value.bit_length() - 1


@intrinsic
def method_ilog10(interpreter, result, value):
    if value <= 0:
        raise RustPanic("argument of integer logarithm must be positive")
    return len(str(value)) - 1


@intrinsic
def method_ilog(interpreter, result, value, base):
    if value <= 0 or base < 2:
        raise RustPanic("argument of integer logarithm must be positive")
    count = 0
    while value >= base:
        value //= base
        count += 1
    return count


@intrinsic
def method_midpoint(interpreter, result, left, right):
    if isinstance(left, float):
        return rounded_float((left + right) / 2, result)
    return (left + right) // 2


@intrinsic
def method_to_be_bytes(interpreter, result, value):
    width = interpreter.receiver_width()
    return ArrayValue(tuple(unsigned_bits(value, width).to_bytes(width // 8, "big")))


@intrinsic
def method_to_le_bytes(interpreter, result, value):
    width = interpreter.receiver_width()
    return ArrayValue(tuple(unsigned_bits(value, width).to_bytes(width // 8, "little")))


def parse_error(interpreter, result, kind):
    error = result.arguments[1]
    kinds = MODULE_SEPARATOR.join(("std", "num", "IntErrorKind"))
    variant = next(ctor for ctor in interpreter.program.layout(kinds)[1] if ctor.path.endswith(f"::{kind}"))
    return interpreter.variant(result, "Err", Value(interpreter.program.layout(error)[1][0], (Value(variant, ()),)))


def parse_integer(interpreter, result, text, radix=10):
    kind = result.arguments[0]
    value = text_of(text)
    if not value:
        return parse_error(interpreter, result, "Empty")
    sign = 1
    digits = value
    if value[0] in "+-":
        if value[0] == "-":
            if INTEGER_BOUNDS[kind][0] == 0:
                return parse_error(interpreter, result, "InvalidDigit")
            sign = -1
        digits = value[1:]
        if not digits:
            return parse_error(interpreter, result, "InvalidDigit")
    number = 0
    for char in digits:
        if not (char.isascii() and char.isalnum()) or int(char, 36) >= radix:
            return parse_error(interpreter, result, "InvalidDigit")
        number = number * radix + int(char, 36)
    number *= sign
    low, high = INTEGER_BOUNDS[kind]
    if number >= high:
        return parse_error(interpreter, result, "PosOverflow")
    if number < low:
        return parse_error(interpreter, result, "NegOverflow")
    return interpreter.variant(result, "Ok", number)


@intrinsic
def rust_try_integer(interpreter, result, value):
    low, high = INTEGER_BOUNDS[result.arguments[0]]
    if low <= value < high:
        return interpreter.variant(result, "Ok", value)
    error = result.arguments[1]
    return interpreter.variant(result, "Err", Value(interpreter.program.layout(error)[1][0], ()))


@intrinsic
def rust_parse_int(interpreter, result, text):
    return parse_integer(interpreter, result, text)


@intrinsic
def method_from_str_radix(interpreter, result, text, radix):
    return parse_integer(interpreter, result, text, radix)


FLOAT_SYNTAX = re.compile(r"[+-]?(?:inf|infinity|nan|(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?)",
                          re.IGNORECASE)


@intrinsic
def rust_parse_float(interpreter, result, text):
    kind = result.arguments[0]
    value = text_of(text)
    if not FLOAT_SYNTAX.fullmatch(value):
        error = result.arguments[1]
        return interpreter.variant(result, "Err", Value(interpreter.program.layout(error)[1][0], (not value,)))
    return interpreter.variant(result, "Ok", rounded_float(float(value), kind))


def float_function(function):
    def body(interpreter, result, value, *rest):
        output = function(value, *rest)
        if isinstance(output, tuple):
            return make_tuple(interpreter, result, *(rounded_float(item, result.elements[0]) for item in output))
        return rounded_float(output, result) if isinstance(output, float) else output
    return body


def rust_round(value):
    if not math.isfinite(value):
        return value
    whole = math.floor(abs(value))
    if abs(value) - whole >= 0.5:
        whole += 1
    return math.copysign(float(whole), value)


def safe(function):
    def body(*arguments):
        try:
            return function(*arguments)
        except (ValueError, OverflowError):
            return math.nan
    return body


def float_extreme(pick):
    def body(left, right):
        if math.isnan(left):
            return right
        if math.isnan(right):
            return left
        return pick(left, right)
    return body


for _name, _function in {
    "floor": lambda x: float(math.floor(x)) if math.isfinite(x) else x,
    "ceil": lambda x: float(math.ceil(x)) if math.isfinite(x) else x,
    "round": rust_round, "round_ties_even": lambda x: float(round(x)) if math.isfinite(x) else x,
    "trunc": lambda x: float(math.trunc(x)) if math.isfinite(x) else x,
    "fract": lambda x: x - math.trunc(x) if math.isfinite(x) else math.nan,
    "cbrt": lambda x: math.copysign(abs(x) ** (1 / 3), x) if math.isfinite(x) else x,
    "powi": safe(lambda x, n: math.pow(x, n)), "powf": safe(math.pow), "exp": safe(math.exp),
    "exp2": safe(lambda x: math.pow(2.0, x)), "exp_m1": safe(math.expm1),
    "ln": lambda x: -math.inf if x == 0 else math.nan if x < 0 else math.log(x),
    "ln_1p": safe(math.log1p),
    "log": lambda x, b: math.nan if x < 0 else -math.inf if x == 0 else math.log(x) / math.log(b),
    "log10": lambda x: -math.inf if x == 0 else math.nan if x < 0 else math.log10(x),
    "log2": lambda x: -math.inf if x == 0 else math.nan if x < 0 else math.log2(x),
    "sin": safe(math.sin), "cos": safe(math.cos), "tan": safe(math.tan), "asin": safe(math.asin),
    "acos": safe(math.acos), "atan": math.atan, "atan2": math.atan2, "sinh": safe(math.sinh),
    "cosh": safe(math.cosh), "tanh": math.tanh, "sin_cos": lambda x: (math.sin(x), math.cos(x)),
    "hypot": math.hypot, "to_degrees": math.degrees, "to_radians": math.radians,
    "max": float_extreme(max), "min": float_extreme(min),
    "clamp": lambda x, low, high: min(max(x, low), high) if not math.isnan(x) else x,
    "mul_add": lambda x, a, b: x * a + b, "recip": lambda x: 1 / x if x else math.copysign(math.inf, x),
    "copysign": math.copysign, "abs_sub": lambda x, y: max(x - y, 0.0),
    "is_sign_positive": lambda x: math.copysign(1, x) > 0, "is_sign_negative": lambda x: math.copysign(1, x) < 0,
    "is_normal": lambda x: math.isfinite(x) and x != 0 and abs(x) >= 2.2250738585072014e-308,
}.items():
    INTRINSICS["." + _name] = float_function(_function)


@intrinsic
def method_total_cmp(interpreter, result, left, right):
    def key(value):
        bits = struct.unpack("<q", struct.pack("<d", value))[0]
        return bits ^ ((1 << 63) - 1) if bits < 0 else bits
    a, b = key(text_of(left)), key(text_of(right))
    order = (a > b) - (a < b)
    name = "Less" if order < 0 else "Greater" if order > 0 else "Equal"
    return Value(next(ctor for ctor in interpreter.program.layout(ORDERING)[1] if ctor.path.endswith(f"::{name}")), ())


@intrinsic
def method_to_bits(interpreter, result, value):
    if result == "u32":
        return struct.unpack("<I", struct.pack("<f", value))[0]
    return struct.unpack("<Q", struct.pack("<d", value))[0]


@intrinsic
def method_from_bits(interpreter, result, bits):
    if result == "f32":
        return Float32(struct.unpack("<f", struct.pack("<I", bits))[0])
    return struct.unpack("<d", struct.pack("<Q", bits))[0]


def map_of(value):
    while isinstance(value, (Address, DynAddress, BoxValue)):
        value = value.read()
    return value


def map_tuple(interpreter, annotation, *values):
    return make_tuple(interpreter, annotation, *values)


def map_entries(interpreter, value):
    return interpreter.map_order(map_of(value))


@intrinsic
def rust_map_new(interpreter, result):
    return MapValue(ordered=MAP_TYPES[result.name].startswith("BTree"))


@intrinsic
def rust_map_len(interpreter, result, value):
    return len(map_of(value).entries)


@intrinsic
def rust_map_clear(interpreter, result, value):
    map_of(value).entries.clear()


@intrinsic
def rust_map_insert(interpreter, result, value, key, item):
    entries = map_of(value).entries
    normalized = interpreter.map_key(key)
    previous = entries.get(normalized)
    if previous is not None:
        old = previous[1]
        previous[1] = item
        return option(interpreter, result, old)
    entries[normalized] = [key, item]
    return option(interpreter, result)


@intrinsic
def rust_map_get(interpreter, result, value, key):
    target = map_of(value)
    normalized = interpreter.map_key(key)
    if normalized not in target.entries:
        return option(interpreter, result)
    return option(interpreter, result, Address(MapSlot(target, normalized)))


@intrinsic
def rust_map_get_key_value(interpreter, result, value, key):
    target = map_of(value)
    normalized = interpreter.map_key(key)
    if normalized not in target.entries:
        return option(interpreter, result)
    stored, item = target.entries[normalized]
    return option(interpreter, result, map_tuple(interpreter, element_type(result), shared(stored),
                                                 Address(MapSlot(target, normalized))))


@intrinsic
def rust_map_has(interpreter, result, value, key):
    return interpreter.map_key(key) in map_of(value).entries


@intrinsic
def rust_map_remove(interpreter, result, value, key):
    removed = map_of(value).entries.pop(interpreter.map_key(key), None)
    return option(interpreter, result) if removed is None else option(interpreter, result, removed[1])


@intrinsic
def rust_map_remove_entry(interpreter, result, value, key):
    removed = map_of(value).entries.pop(interpreter.map_key(key), None)
    if removed is None:
        return option(interpreter, result)
    return option(interpreter, result, map_tuple(interpreter, element_type(result), *removed))


@intrinsic
def rust_map_or_insert(interpreter, result, value, key, item):
    target = map_of(value)
    normalized = interpreter.map_key(key)
    if normalized not in target.entries:
        target.entries[normalized] = [key, item]
    return Address(MapSlot(target, normalized))


@intrinsic
def rust_map_slot(interpreter, result, value, key):
    return Address(MapSlot(map_of(value), interpreter.map_key(key)))


def borrowed(value):
    return isinstance(value, (Address, DynAddress))


@intrinsic
def rust_map_keys(interpreter, result, value):
    entries = map_entries(interpreter, value)
    if borrowed(value):
        return lazy(shared(key) for key, _ in entries)
    return lazy(key for key, _ in entries)


@intrinsic
def rust_map_values(interpreter, result, value):
    target = map_of(value)
    keys = [normalized for normalized, _ in sorted(target.entries.items(), key=functools.cmp_to_key(
        lambda left, right: interpreter.compare(left[1][0], right[1][0])))] if target.ordered else list(target.entries)
    if borrowed(value):
        return lazy(Address(MapSlot(target, key)) for key in keys)
    return lazy(target.entries[key][1] for key in keys)


@intrinsic
def rust_map_items(interpreter, result, value):
    target = map_of(value)
    pair = element_type(result)
    ordered = interpreter.map_order(target)
    if borrowed(value):
        return lazy(map_tuple(interpreter, pair, shared(key), Address(MapSlot(target, interpreter.map_key(key))))
                    for key, _ in ordered)
    return lazy(map_tuple(interpreter, pair, key, item) for key, item in ordered)


@intrinsic
def rust_map_drain(interpreter, result, value):
    target = map_of(value)
    ordered = interpreter.map_order(target)
    target.entries.clear()
    pair = element_type(result)
    return lazy(map_tuple(interpreter, pair, key, item) for key, item in ordered)


@intrinsic
def rust_map_retain(interpreter, result, value, keep):
    target = map_of(value)
    for normalized in list(target.entries):
        key = target.entries[normalized][0]
        if not interpreter.call_value(keep, shared(key), Address(MapSlot(target, normalized))):
            del target.entries[normalized]


@intrinsic
def rust_map_extend(interpreter, result, value, items):
    target = map_of(value)
    for pair in interpreter.iterate(items, 1):
        pair = map_of(pair)
        key, item = (field.read() if isinstance(field, Address) else field for field in pair.fields)
        target.entries[interpreter.map_key(key)] = [key, item]


@intrinsic
def rust_map_end(interpreter, result, value, last):
    target = map_of(value)
    ordered = interpreter.map_order(target)
    if not ordered:
        return option(interpreter, result)
    key, _ = ordered[-1 if last else 0]
    return option(interpreter, result, map_tuple(interpreter, element_type(result), shared(key),
                                                 Address(MapSlot(target, interpreter.map_key(key)))))


@intrinsic
def rust_map_pop(interpreter, result, value, last):
    target = map_of(value)
    ordered = interpreter.map_order(target)
    if not ordered:
        return option(interpreter, result)
    key, item = ordered[-1 if last else 0]
    del target.entries[interpreter.map_key(key)]
    return option(interpreter, result, map_tuple(interpreter, element_type(result), key, item))


def in_range(interpreter, key, bounds):
    name = bounds.constructor.path.rsplit(MODULE_SEPARATOR, 1)[-1]
    fields = dict(zip((field for field, _ in bounds.constructor.fields), bounds.fields))
    if "start" in fields and interpreter.compare(key, fields["start"]) < 0:
        return False
    if "end" in fields:
        order = interpreter.compare(key, fields["end"])
        return order <= 0 if name in {"RangeInclusive", "RangeToInclusive"} else order < 0
    return True


@intrinsic
def rust_map_range(interpreter, result, value, bounds):
    target = map_of(value)
    pair = element_type(result)
    return lazy(map_tuple(interpreter, pair, shared(key), Address(MapSlot(target, interpreter.map_key(key))))
                for key, _ in interpreter.map_order(target) if in_range(interpreter, key, bounds))


@intrinsic
def rust_map_debug(interpreter, result, value, formatter):
    return interpreter.builtin_fmt(map_of(value), formatter, True, result)


@intrinsic
def rust_set_insert(interpreter, result, value, item):
    entries = map_of(value).entries
    normalized = interpreter.map_key(item)
    if normalized in entries:
        return False
    entries[normalized] = [item, MAP_MEMBER]
    return True


@intrinsic
def rust_set_remove(interpreter, result, value, item):
    return map_of(value).entries.pop(interpreter.map_key(item), None) is not None


@intrinsic
def rust_set_get(interpreter, result, value, item):
    stored = map_of(value).entries.get(interpreter.map_key(item))
    return option(interpreter, result) if stored is None else option(interpreter, result, shared(stored[0]))


@intrinsic
def rust_set_take(interpreter, result, value, item):
    stored = map_of(value).entries.pop(interpreter.map_key(item), None)
    return option(interpreter, result) if stored is None else option(interpreter, result, stored[0])


@intrinsic
def rust_set_drain(interpreter, result, value):
    target = map_of(value)
    ordered = interpreter.map_order(target)
    target.entries.clear()
    return lazy(key for key, _ in ordered)


@intrinsic
def rust_set_retain(interpreter, result, value, keep):
    target = map_of(value)
    for normalized in list(target.entries):
        if not interpreter.call_value(keep, shared(target.entries[normalized][0])):
            del target.entries[normalized]


@intrinsic
def rust_set_extend(interpreter, result, value, items):
    target = map_of(value)
    for item in interpreter.iterate(items, 1):
        if isinstance(item, Address) and not isinstance(interpreter.intrinsic_types[0].arguments[0], ReferenceType):
            item = interpreter.clone_value(item.read())
        target.entries.setdefault(interpreter.map_key(item), [item, MAP_MEMBER])


@intrinsic
def rust_set_combine(interpreter, result, value, other, mode):
    first, second = map_of(value), map_of(other)
    if mode == 0:
        keys = [*first.entries.values(), *(item for key, item in second.entries.items() if key not in first.entries)]
    elif mode == 1:
        keys = [item for key, item in first.entries.items() if key in second.entries]
    elif mode == 2:
        keys = [item for key, item in first.entries.items() if key not in second.entries]
    else:
        keys = [*(item for key, item in first.entries.items() if key not in second.entries),
                *(item for key, item in second.entries.items() if key not in first.entries)]
    if first.ordered:
        keys.sort(key=functools.cmp_to_key(lambda left, right: interpreter.compare(left[0], right[0])))
    return lazy(shared(item[0]) for item in keys)


@intrinsic
def rust_set_relation(interpreter, result, value, other, mode):
    first, second = map_of(value), map_of(other)
    if mode == 0:
        return first.entries.keys() <= second.entries.keys()
    return not (first.entries.keys() & second.entries.keys())


@intrinsic
def rust_set_end(interpreter, result, value, last):
    ordered = interpreter.map_order(map_of(value))
    return option(interpreter, result, shared(ordered[-1 if last else 0][0])) if ordered else option(interpreter, result)


@intrinsic
def rust_set_pop(interpreter, result, value, last):
    target = map_of(value)
    ordered = interpreter.map_order(target)
    if not ordered:
        return option(interpreter, result)
    key = ordered[-1 if last else 0][0]
    del target.entries[interpreter.map_key(key)]
    return option(interpreter, result, key)


@intrinsic
def rust_set_range(interpreter, result, value, bounds):
    return lazy(shared(key) for key, _ in interpreter.map_order(map_of(value)) if in_range(interpreter, key, bounds))


@intrinsic
def rust_lazy_elements(interpreter, result, items):
    if isinstance(items, (Address, SliceAddress)):
        return lazy(child(items, index) for index in range(len(items_of(items))))
    return lazy(items.items)


@intrinsic
def rust_lazy_range(interpreter, result, items, bounds):
    start, stop = interpreter.range_bounds(bounds, len(items_of(items)))
    if start is None:
        raise RustPanic("range out of bounds")
    return lazy(child(items, index) for index in range(start, stop))


def Rc__new(interpreter, result, value):
    interpreter.track_drops = True
    return RcValue(Binding(value, True), [1, 0], interpreter.program.concrete_type(result.arguments[0]))


def Rc__clone(interpreter, result, this):
    rc = map_of(this)
    rc.counts[0] += 1
    return RcValue(rc.cell, rc.counts, rc.concrete)


def Rc__strong_count(interpreter, result, this):
    return map_of(this).counts[0]


def Rc__weak_count(interpreter, result, this):
    return map_of(this).counts[1]


def Rc__ptr_eq(interpreter, result, this, other):
    return map_of(this).cell is map_of(other).cell


def Rc__as_ptr(interpreter, result, this):
    return id(map_of(this).cell)


def Rc__downgrade(interpreter, result, this):
    rc = map_of(this)
    rc.counts[1] += 1
    return WeakValue(rc.cell, rc.counts, rc.concrete)


def Rc__try_unwrap(interpreter, result, this):
    if this.counts[0] == 1:
        this.counts[0] = 0
        return interpreter.variant(result, "Ok", this.cell.value)
    return interpreter.variant(result, "Err", this)


def Rc__into_inner(interpreter, result, this):
    this.counts[0] -= 1
    return option(interpreter, result, this.cell.value) if this.counts[0] == 0 else option(interpreter, result)


def Rc__unwrap_or_clone(interpreter, result, this):
    if this.counts[0] == 1:
        return this.cell.value
    this.counts[0] -= 1
    return interpreter.clone_value(this.cell.value)


def Rc__get_mut(interpreter, result, this):
    rc = this.read()
    if rc.counts[0] == 1 and rc.counts[1] == 0:
        return option(interpreter, result, Address(rc.cell))
    return option(interpreter, result)


def Rc__make_mut(interpreter, result, this):
    rc = this.read()
    if rc.counts[0] != 1:
        rc.counts[0] -= 1
        rc = RcValue(Binding(interpreter.clone_value(rc.cell.value), True), [1, 0])
        this.write(rc)
    return Address(rc.cell)


def Weak__new(interpreter, result):
    return WeakValue(None, [0, 1])


def Weak__upgrade(interpreter, result, this):
    weak = map_of(this)
    if weak.cell is None or weak.counts[0] == 0:
        return option(interpreter, result)
    weak.counts[0] += 1
    return option(interpreter, result, RcValue(weak.cell, weak.counts, weak.concrete))


def Weak__strong_count(interpreter, result, this):
    weak = map_of(this)
    return 0 if weak.cell is None else weak.counts[0]


def Weak__weak_count(interpreter, result, this):
    weak = map_of(this)
    return 0 if weak.cell is None or weak.counts[0] == 0 else weak.counts[1]


for _function in (Rc__new, Rc__clone, Rc__strong_count, Rc__weak_count, Rc__ptr_eq, Rc__as_ptr, Rc__downgrade,
                  Rc__try_unwrap, Rc__into_inner, Rc__unwrap_or_clone, Rc__get_mut, Rc__make_mut, Weak__new,
                  Weak__upgrade, Weak__strong_count, Weak__weak_count):
    intrinsic(_function)


@intrinsic
def rust_rc_deref(interpreter, result, this):
    rc = map_of(this)
    if isinstance(result.inner, DynType):
        return DynAddress(Address(rc.cell), rc.concrete)
    return Address(rc.cell)


@intrinsic
def rust_cell_value(interpreter, result, cell):
    return cell.child(Token("value", 0, 0))


INTRINSICS["refcell_value"] = rust_cell_value


@intrinsic
def rust_duration_debug(interpreter, result, nanos, formatter):
    if nanos >= 1_000_000_000:
        text = f"{format_float(nanos / 1e9, False)}s"
    elif nanos >= 1_000_000:
        text = f"{format_float(nanos / 1e6, False)}ms"
    elif nanos >= 1_000:
        text = f"{format_float(nanos / 1e3, False)}µs"
    else:
        text = f"{nanos}ns"
    stored = formatter.read()
    formatter.write(Value(stored.constructor, (stored.fields[0] + text, stored.fields[1])))
    return interpreter.variant(result, "Ok", None)


@intrinsic
def rust_clock_nanos(interpreter, result):
    import time
    return time.perf_counter_ns()


@intrinsic
def rust_wall_nanos(interpreter, result):
    import time
    return time.time_ns()


@intrinsic
def rust_emit(interpreter, result, text, error):
    print(text_of(text), end="", file=sys.stderr if error else interpreter.stdout)


@intrinsic
def rust_stdin_read_line(interpreter, result, buffer):
    line = sys.stdin.readline()
    buffer.write(buffer.read() + line)
    return len(line.encode("utf-8"))


@intrinsic
def rust_stdin_read_all(interpreter, result, buffer):
    text = sys.stdin.read()
    buffer.write(buffer.read() + text)
    return len(text.encode("utf-8"))


@intrinsic
def rust_stdin_lines(interpreter, result):
    item = element_type(result)
    return lazy(interpreter.variant(item, "Ok", line.rstrip("\n").removesuffix("\r")) for line in sys.stdin)


@intrinsic
def rust_env_args(interpreter, result):
    return lazy(iter(PROGRAM_ARGUMENTS))


@intrinsic
def rust_env_var(interpreter, result, key):
    value = os.environ.get(text_of(key))
    return option(interpreter, result) if value is None else option(interpreter, result, value)


PROGRAM_ARGUMENTS = ["main"]


@intrinsic
def rust_hash_value(interpreter, result, value):
    return hash(repr(interpreter.map_key(value))) % (1 << 64)


def plain_item(interpreter, item, annotation):
    """Read through a reference when a collection of values receives borrowed items."""
    if isinstance(item, (Address, SliceAddress)) and not isinstance(annotation, ReferenceType):
        return interpreter.clone_value(item.read())
    return item


def collection_element(annotation):
    if isinstance(annotation, AppliedType) and annotation.arguments:
        return annotation.arguments[0]
    return None


@intrinsic
def rust_iter_partition(interpreter, result, iterator, predicate):
    target = result.elements[0]
    left, right = [], []
    for item in interpreter.iterate(iterator, 0):
        (left if interpreter.call_value(predicate, shared(item)) else right).append(
            plain_item(interpreter, item, collection_element(target)))
    return make_tuple(interpreter, result, interpreter.collect(left, target), interpreter.collect(right, result.elements[1]))


@intrinsic
def rust_iter_unzip(interpreter, result, iterator):
    first, second = [], []
    for item in interpreter.iterate(iterator, 0):
        pair = map_of(item)
        a, b = pair.fields
        first.append(plain_item(interpreter, a, collection_element(result.elements[0])))
        second.append(plain_item(interpreter, b, collection_element(result.elements[1])))
    return make_tuple(interpreter, result, interpreter.collect(first, result.elements[0]),
                      interpreter.collect(second, result.elements[1]))


def short_circuit(value):
    name = value.constructor.path.rsplit("::", 1)[-1]
    return name in {"None", "Err", "Break"}


@intrinsic
def rust_iter_try_fold(interpreter, result, iterator, accumulator, step):
    for item in interpreter.iterate(iterator, 0):
        outcome = interpreter.call_value(step, accumulator, item)
        if short_circuit(outcome):
            return outcome
        accumulator = outcome.fields[0]
    return interpreter.variant(result, "Some" if result.name == OPTION else "Ok", accumulator)


@intrinsic
def rust_iter_try_for_each(interpreter, result, iterator, step):
    for item in interpreter.iterate(iterator, 0):
        outcome = interpreter.call_value(step, item)
        if short_circuit(outcome):
            return outcome
    return interpreter.variant(result, "Some" if result.name == OPTION else "Ok", None)


@intrinsic
def rust_iter_compare(interpreter, result, iterator, other, ordering):
    left = list(interpreter.iterate(iterator, 0))
    right = list(interpreter.iterate(other, 1))
    for a, b in zip(left, right):
        order = interpreter.compare(a, b, equality=not ordering)
        if order:
            return order
    return len(left) - len(right)


@intrinsic
def rust_iter_is_sorted(interpreter, result, iterator):
    items = list(interpreter.iterate(iterator, 0))
    return all(interpreter.compare(a, b) <= 0 for a, b in zip(items, items[1:]))


@intrinsic
def rust_iter_cycle(interpreter, result, iterator):
    items = list(interpreter.iterate(iterator, 0))
    return lazy(interpreter.clone_value(item) for _ in itertools.count() if items for item in items)


@intrinsic
def rust_iter_chunks(interpreter, result, iterator, size):
    items = list(interpreter.iterate(iterator, 0))
    return lazy(ArrayValue(tuple(items[start:start + size])) for start in range(0, len(items), size))


@intrinsic
def method_split_at_mut(interpreter, result, items, middle):
    count = len(items_of(items))
    if middle > count:
        raise RustPanic("mid > len")
    root, offset = interpreter.slice_parts(items)
    return make_tuple(interpreter, result, SliceAddress(root, offset, offset + middle),
                      SliceAddress(root, offset + middle, offset + count))


def rust_layout(interpreter, annotation):
    """Approximate Rust's (size, alignment) for a type, reordering struct fields as rustc does."""
    program = interpreter.program
    annotation = program.concrete_type(program.normalize(annotation))
    if annotation in INTEGER_WIDTHS:
        size = INTEGER_WIDTHS[annotation] // 8
        return size, min(size, 8 if annotation not in {"i128", "u128"} else 16)
    scalars = {"bool": (1, 1), "char": (4, 4), "f32": (4, 4), "f64": (8, 8), "()": (0, 1), NEVER: (0, 1),
               STRING: (24, 8)}
    if annotation in scalars:
        return scalars[annotation]
    if isinstance(annotation, (ReferenceType, AppliedType)) and (
            isinstance(annotation, ReferenceType) or annotation.name in {BOX, RC, WEAK}):
        pointee = annotation.inner if isinstance(annotation, ReferenceType) else annotation.arguments[0]
        fat = isinstance(pointee, (SliceType, DynType)) or pointee == "str"
        return (16 if fat else 8), 8
    if is_vector(annotation) or (isinstance(annotation, AppliedType) and annotation.name == DEQUE):
        return 24 if is_vector(annotation) else 32, 8
    if isinstance(annotation, AppliedType) and annotation.name in MAP_TYPES:
        return (48 if annotation.name.endswith("HashMap") or annotation.name.endswith("HashSet") else 24), 8
    if isinstance(annotation, ArrayType):
        size, align = rust_layout(interpreter, annotation.element)
        return size * annotation.length.value, align
    if isinstance(annotation, FunctionType):
        return (0, 1) if annotation.closure else (8, 8)
    layout = program.layout(annotation)
    if layout is None:
        return 8, 8
    constructors = layout[1]

    def record(fields):
        parts = [rust_layout(interpreter, kind) for _, kind in fields]
        parts.sort(key=lambda part: -part[1])
        size = 0
        align = max((part[1] for part in parts), default=1)
        for part_size, part_align in parts:
            size = (size + part_align - 1) // part_align * part_align + part_size
        return (size + align - 1) // align * align, align

    if len(constructors) == 1 and not (isinstance(annotation, AppliedType) and annotation.name in {OPTION, RESULT}):
        return record(constructors[0].fields)
    payloads = [record(ctor.fields) for ctor in constructors]
    largest = max(payload[0] for payload in payloads)
    align = max(payload[1] for payload in payloads)
    if isinstance(annotation, AppliedType) and annotation.name == OPTION:
        inner = program.concrete_type(program.normalize(annotation.arguments[0]))
        if (isinstance(inner, ReferenceType) or (isinstance(inner, AppliedType) and inner.name in {BOX, RC, VEC})
                or inner in {"bool", "char", STRING}):
            return rust_layout(interpreter, inner)
    if all(not ctor.fields for ctor in constructors):
        return (1 if len(constructors) > 1 else 0), 1
    tag = 1 if len(constructors) <= 256 else 2
    size = (largest + tag + align - 1) // align * align
    return size, align


@intrinsic
def rust_size_of(interpreter, result):
    return rust_layout(interpreter, interpreter.intrinsic_types[0])[0]


@intrinsic
def rust_align_of(interpreter, result):
    return rust_layout(interpreter, interpreter.intrinsic_types[0])[1]


@intrinsic
def rust_size_of_val(interpreter, result, value):
    target = map_of(value)
    if isinstance(target, FunctionValue):
        if not target.name.startswith("@closure:"):
            return 8
        # A closure's size is its captures: values it owns, or one pointer per borrowed capture.
        size = 0
        for capture in interpreter.program.functions[target.name].captures:
            size += rust_layout(interpreter, capture.annotation)[0] if capture.by_move else 8
        return size
    if isinstance(target, str):
        return len(target.encode())
    if isinstance(target, ArrayValue):
        element = interpreter.intrinsic_types[0]
        element = element.element if isinstance(element, (SliceType, ArrayType)) else element
        return len(target.items) * rust_layout(interpreter, element)[0] if isinstance(
            interpreter.intrinsic_types[0], SliceType) else rust_layout(interpreter, interpreter.intrinsic_types[0])[0]
    return rust_layout(interpreter, interpreter.intrinsic_types[0])[0]


@intrinsic
def rust_discriminant(interpreter, result, value):
    target = map_of(value)
    variants = interpreter.program.layout(target.constructor.annotation)[1]
    index = next(position for position, ctor in enumerate(variants) if ctor.path == target.constructor.path)
    return Value(interpreter.program.layout(result)[1][0], (index,))


@intrinsic
def rust_pad_text(interpreter, result, text, width, precision, fill, align):
    value = text_of(text)
    if precision.fields:
        value = value[:precision.fields[0]]
    if not width.fields or len(value) >= width.fields[0]:
        return value
    gap = width.fields[0] - len(value)
    left = gap if align == 2 else gap // 2 if align == 3 else 0
    return str(fill) * left + value + str(fill) * (gap - left)


def rust_type_text(interpreter, annotation):
    program = interpreter.program
    annotation = program.concrete_type(program.normalize(annotation))
    if isinstance(annotation, ReferenceType):
        return ("&mut " if annotation.mutable else "&") + rust_type_text(interpreter, annotation.inner)
    if isinstance(annotation, TupleType):
        return "(" + ", ".join(rust_type_text(interpreter, kind) for kind in annotation.elements) + ")"
    if isinstance(annotation, ArrayType):
        return f"[{rust_type_text(interpreter, annotation.element)}; {annotation.length}]"
    if isinstance(annotation, SliceType):
        return f"[{rust_type_text(interpreter, annotation.element)}]"
    if isinstance(annotation, DynType):
        return "dyn " + annotation.trait.replace(MODULE_SEPARATOR, "::")
    if isinstance(annotation, FunctionType):
        arguments = ", ".join(rust_type_text(interpreter, kind) for kind in annotation.parameters)
        result = "" if annotation.result == "()" else " -> " + rust_type_text(interpreter, annotation.result)
        return f"fn({arguments}){result}"
    name = annotation.name if isinstance(annotation, AppliedType) else str(annotation)
    if annotation not in PRIMITIVES and annotation != "str" and name.split(MODULE_SEPARATOR)[0] not in STD_CRATES:
        name = MODULE_SEPARATOR.join((program.crate_name, name))
    text = name.replace(MODULE_SEPARATOR, "::")
    if isinstance(annotation, AppliedType):
        text += "<" + ", ".join(rust_type_text(interpreter, kind) for kind in annotation.arguments) + ">"
    return text


@intrinsic
def rust_type_name(interpreter, result):
    return shared(rust_type_text(interpreter, interpreter.intrinsic_types[0]))


@intrinsic
def rust_range_length(interpreter, result, bounds):
    start, end = map_of(bounds).fields
    return max(0, end - start)


@intrinsic
def rust_wrapping_op(interpreter, result, left, right, operator):
    value = (left + right, left - right, left * right)[operator]
    return wrap(value, result)


@intrinsic
def rust_from_utf8(interpreter, result, data):
    raw = bytes(items_of(data))
    try:
        return interpreter.variant(result, "Ok", shared(raw.decode("utf-8")))
    except UnicodeDecodeError as error:
        return interpreter.variant(result, "Err", Value(interpreter.program.layout(result.arguments[1])[1][0],
                                                        (error.start,)))


@intrinsic
def rust_box_unsized(interpreter, result, value):
    return BoxValue(Address(Binding(value, True)), result.arguments[0])


@intrinsic
def rust_track_drops(interpreter, result):
    interpreter.track_drops = True


def compile_program(source, *, proc_extern=(), source_path=None, edition="2021"):
    PROJECTION_BOUNDS.clear()
    TYPE_BOUNDS.clear()
    OVERFLOWING_LITERALS[0] = False
    ALLOWED_LINTS.clear()
    UNSAFE_FUNCTIONS.clear()
    ALLOWED_LINTS.update(re.findall(r"\b(?:arithmetic_overflow|unconditional_panic)\b", source))
    if edition not in RUST_EDITIONS:
        raise RustError(f"unsupported Rust edition {edition!r}")
    if proc_extern:
        trees, end = rustc_source(source, macro_libraries(proc_extern), source_path, edition=edition)
    else:
        source_file = os.fspath(source_path) if source_path is not None else None
        trees, end = token_trees(lex(source, edition=edition, source_file=source_file))
    modules = Modules(trees, source_path, edition)
    modules.expand()
    program = Program(type_aliases=modules.type_aliases, statics=modules.statics, lazy_items=modules.lazy_items)
    program.crate_name = Path(source_path).stem.replace("-", "_") if source_path else "rust_out"
    program.add_prelude()
    scalars = modules.constants | modules.statics
    prelude = modules.units[STD_PRELUDE]
    prelude_traits = {symbol.name for symbol in (prelude.aliases | prelude.symbols).values()
                      if symbol and symbol.kind == "trait"}
    for unit in modules.units.values():
        Parser("", tokens=(*flatten_trees(unit.items), end), modules=modules, unit=unit).parse(program)
        symbols = unit.glob_aliases | unit.aliases | unit.symbols
        program.traits_in_scope[unit.path] = unit.hidden_traits | prelude_traits | {
            symbol.name for symbol in symbols.values() if symbol and symbol.kind == "trait"
        }
        program.values_in_scope[unit.path] = {name: symbol.name for name, symbol in symbols.items()
                                              if symbol and symbol.kind == "fn"}
        program.constants_in_scope[unit.path] = {name: scalars[symbol.name] for name, symbol in symbols.items()
                                                 if symbol and symbol.kind in {"const", "static"}
                                                 and symbol.name in scalars}
    for name, item in modules.lazy_items.items():
        program.functions[f"@lazy:{name}"] = Function(replace(item.name, text=f"@lazy:{name}"), [],
                                                     item.annotation, item.expression)
    Checker(program, modules).check()
    Ownership(program).check()
    program.normalized = {}
    return program


def run(source, stdout=None, *, proc_extern=(), source_path=None, edition="2021"):
    program = compile_program(source, proc_extern=proc_extern, source_path=source_path, edition=edition)
    interpreter = Interpreter(program, sys.stdout if stdout is None else stdout)
    try:
        value = interpreter.call("main", ())
        status = interpreter.exit_status(value, program.functions["main"].result)
    except ProcessExit as stop:
        status = stop.status & 255 if os.name == "posix" else stop.status
    return status or None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", help="Rust source file, or - for stdin")
    parser.add_argument("--demo", action="store_true", help="run a built-in Rust program")
    parser.add_argument("--check", action="store_true", help="check the source without running it")
    parser.add_argument("--edition", choices=RUST_EDITIONS, default="2021", help="Rust edition used for syntax")
    parser.add_argument("--proc-extern", "--macro-extern", action="append", default=[], metavar="NAME=PATH",
                        help="load a compiled declarative or procedural macro crate through rustc")
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
        result = operation(source, proc_extern=args.proc_extern, source_path=source_path, edition=args.edition)
        if not args.check and result is not None:
            return result
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
