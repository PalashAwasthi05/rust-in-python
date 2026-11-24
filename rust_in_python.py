"""A standard-library-only interpreter for a scalar subset of Rust.

Run: python3 rust_in_python.py program.rs
Demo: python3 rust_in_python.py --demo

Supported types: i32, bool, &str (string literals), (), structs, and enums.
Supported syntax: functions, initialized let bindings, mut, blocks, if/else,
while, loop, for over integer ranges, return, break, continue, scalar operators,
exhaustive match, and println!. Integers use checked i32 arithmetic.
This is a subset interpreter. Ownership, borrowing, general lifetime checking,
and the Rust standard library are outside its scope.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
import operator
from pathlib import Path
import re
import sys


class RustError(Exception):
    pass


@dataclass(frozen=True)
class Token:
    text: str
    line: int
    column: int

    def error(self, message):
        return RustError(f"{self.line}:{self.column}: {message}")


TOKEN = re.compile(
    r'\s+|//[^\n]*|/\*|"(?:(?:\\[\s\S])|[^"\\])*"|'
    r"'static(?![A-Za-z_0-9])|"
    r'[0-9][0-9_]*(?:i32)?|[A-Za-z_][A-Za-z_0-9]*|'
    r'\.\.=|\.\.|::|->|=>|==|!=|<=|>=|&&|\|\||[+*/%\-]=|[{}().:,;=!+*/%&<>|\-]'
)
COMMENT_EDGE = re.compile(r'/\*|\*/')
IDENT = re.compile(r'[A-Za-z_][A-Za-z_0-9]*\Z')
RESERVED = set("""
    as async await break const continue crate dyn else enum extern false fn for
    gen if impl in let loop match mod move mut pub ref return self Self static
    struct super trait true type unsafe use where while abstract become box do
    final macro override priv try typeof unsized virtual yield _
""".split())
PRECEDENCE = {
    "=": 1, "+=": 1, "-=": 1, "*=": 1, "/=": 1, "%=": 1,
    "||": 2, "&&": 3, "==": 4, "!=": 4, "<": 4, "<=": 4, ">": 4, ">=": 4,
    "+": 5, "-": 5, "*": 6, "/": 6, "%": 6,
}
BLOCK_EXPRESSIONS = {"block", "if", "while", "loop", "for", "match"}
PRIMITIVES = {"i32", "bool", "&str", "()"}
LIMIT = 2**31
NEVER = "!"
INTEGER_CONSTANTS = {"i32::MIN": -LIMIT, "i32::MAX": LIMIT - 1}


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


@dataclass
class Expr:
    kind: str
    token: Token
    args: tuple = ()


@dataclass
class Function:
    name: Token
    params: list
    result: str
    body: Expr


@dataclass(frozen=True)
class Constructor:
    path: str
    annotation: str
    style: str
    fields: tuple

    def index(self, token):
        for index, (name, _) in enumerate(self.fields):
            if name == token.text:
                return index
        raise token.error(f"{self.path} has no field {token.text!r}")


@dataclass
class Program:
    functions: dict = field(default_factory=dict)
    types: dict = field(default_factory=dict)
    constructors: dict = field(default_factory=dict)
    patterns: dict = field(default_factory=dict)


class Parser:
    def __init__(self, source):
        self.tokens = iter(lex(source))
        self.current = next(self.tokens)

    def advance(self):
        token = self.current
        if not token.text:
            raise token.error("unexpected end of input")
        self.current = next(self.tokens)
        return token

    def accept(self, text):
        if self.current.text == text:
            return self.advance()
        return None

    def expect(self, text):
        token = self.accept(text)
        if token is None:
            raise self.current.error(f"expected {text!r}, found {self.current.text!r}")
        return token

    def identifier(self):
        token = self.current
        if IDENT.fullmatch(token.text) and token.text not in RESERVED:
            return self.advance()
        raise token.error("expected an identifier")

    def type_name(self, *, static_string=False):
        token = self.advance()
        if token.text in {"i32", "bool"}:
            return token.text
        if token.text == "&":
            lifetime = self.accept("'static")
            self.expect("str")
            if static_string and not lifetime:
                raise token.error("this position requires &'static str")
            return "&str"
        if token.text == "(":
            self.expect(")")
            return "()"
        if token.text == "!":
            return NEVER
        if IDENT.fullmatch(token.text) and token.text not in RESERVED:
            return token.text
        raise token.error(f"unsupported type {token.text!r}")

    def parse(self):
        program = Program()
        while self.current.text:
            self.accept("pub")
            item = self.accept("struct") or self.accept("enum")
            if item:
                self.type_item(program, item)
                continue
            self.expect("fn")
            name = self.identifier()
            if name.text in program.functions:
                raise name.error(f"duplicate function {name.text!r}")
            self.expect("(")
            params = []
            while self.current.text != ")":
                mutable = bool(self.accept("mut"))
                param = self.identifier()
                self.expect(":")
                params.append((param, mutable, self.type_name()))
                if not self.accept(","):
                    break
            self.expect(")")
            result = self.type_name(static_string=True) if self.accept("->") else "()"
            program.functions[name.text] = Function(name, params, result, self.block())
        return program

    def type_item(self, program, item):
        name = self.identifier()
        if name.text in program.types or name.text in PRIMITIVES:
            raise name.error(f"duplicate type {name.text!r}")
        constructors = []
        if item.text == "struct":
            constructor = self.constructor(name.text, name.text)
            if constructor.style != "record":
                self.expect(";")
            constructors.append(constructor)
        else:
            self.expect("{")
            variants = set()
            while self.current.text != "}":
                variant = self.identifier()
                if variant.text in variants:
                    raise variant.error(f"duplicate variant {variant.text!r}")
                variants.add(variant.text)
                constructors.append(self.constructor(name.text, f"{name.text}::{variant.text}"))
                if not self.accept(","):
                    break
            self.expect("}")
        program.types[name.text] = (name, tuple(constructors))
        for constructor in constructors:
            program.constructors[constructor.path] = constructor

    def constructor(self, annotation, path):
        fields = []
        if self.accept("{"):
            names = set()
            while self.current.text != "}":
                self.accept("pub")
                name = self.identifier()
                if name.text in names:
                    raise name.error(f"duplicate field {name.text!r}")
                names.add(name.text)
                self.expect(":")
                fields.append((name.text, self.type_name(static_string=True)))
                if not self.accept(","):
                    break
            self.expect("}")
            return Constructor(path, annotation, "record", tuple(fields))
        if self.accept("("):
            while self.current.text != ")":
                self.accept("pub")
                fields.append((str(len(fields)), self.type_name(static_string=True)))
                if not self.accept(","):
                    break
            self.expect(")")
            return Constructor(path, annotation, "tuple", tuple(fields))
        return Constructor(path, annotation, "unit", ())

    def block(self):
        token = self.expect("{")
        statements = []
        tail = None
        while self.current.text != "}":
            if self.accept(";"):
                continue
            let = self.accept("let")
            if let:
                pattern = self.pattern()
                annotation = self.type_name() if self.accept(":") else None
                self.expect("=")
                value = self.expression()
                otherwise = self.block() if self.accept("else") else None
                if otherwise and value.kind in BLOCK_EXPRESSIONS:
                    raise value.token.error("parenthesize this expression before let-else")
                if otherwise and value.kind == "binary" and value.token.text in {"&&", "||"}:
                    raise value.token.error("parenthesize the boolean expression before let-else")
                self.expect(";")
                statements.append(Expr("let", pattern.token, (pattern, annotation, value, otherwise)))
                continue
            value = self.expression()
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

    def expression(self, minimum=1, *, allow_record=True):
        left = self.prefix(allow_record)
        while dot := self.accept("."):
            name = self.advance()
            if not (IDENT.fullmatch(name.text) or re.fullmatch(r'0|[1-9][0-9]*', name.text)):
                raise name.error("expected a field name or tuple field index")
            left = Expr("field", dot, (left, name))
        compared = False
        while PRECEDENCE.get(self.current.text, 0) >= minimum:
            token = self.advance()
            precedence = PRECEDENCE[token.text]
            if precedence == 4:
                if compared:
                    raise token.error("comparison operators cannot be chained")
                compared = True
            right = self.expression(
                precedence if precedence == 1 else precedence + 1, allow_record=allow_record
            )
            left = Expr("assign" if precedence == 1 else "binary", token, (left, right))
        return left

    def arguments(self):
        self.expect("(")
        args = []
        while self.current.text != ")":
            args.append(self.expression())
            if not self.accept(","):
                break
        self.expect(")")
        return tuple(args)

    def prefix(self, allow_record=True):
        if self.current.text == "{":
            return self.block()
        token = self.advance()
        word = token.text
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
                branch = self.conditional_match(token, condition, pattern, body, Expr("break", token))
                return Expr("loop", token, (branch,))
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
            if word == "return" and self.current.text not in {";", "}", ")", ","}:
                return Expr(word, token, (self.expression(),))
            return Expr(word, token)
        if word in {"-", "!"}:
            return Expr("unary", token, (self.expression(7, allow_record=allow_record),))
        if word == "(":
            if self.accept(")"):
                return Expr("literal", token, ("()", None))
            value = self.expression()
            self.expect(")")
            return Expr("group", token, (value,))
        literal = self.literal(token)
        if literal:
            return literal
        if IDENT.fullmatch(word) and word not in RESERVED:
            if self.accept("!"):
                if word != "println":
                    raise token.error(f"unsupported macro {word!r}")
                return Expr("print", token, self.arguments())
            path = self.path(token)
            if path.text in INTEGER_CONSTANTS:
                return Expr("literal", path, ("i32", INTEGER_CONSTANTS[path.text]))
            if allow_record and self.current.text == "{":
                return Expr("record", path, (self.record_fields(self.expression, "name")[0],))
            if self.current.text == "(":
                return Expr("call", path, self.arguments())
            return Expr("name", path)
        raise token.error(f"unexpected token {word!r}")

    @staticmethod
    def literal(token, negative=False):
        word = token.text
        if word[0].isdigit():
            digits = word.removesuffix("i32").replace("_", "").lstrip("0") or "0"
            if len(digits) > 10 or int(digits) > LIMIT:
                raise token.error("integer literal out of range for i32")
            value = int(digits)
            return Expr("literal", token, ("i32", -value if negative else value))
        if negative:
            raise token.error("expected an integer after '-'")
        if word in {"true", "false"}:
            return Expr("literal", token, ("bool", word == "true"))
        if word.startswith('"'):
            return Expr("literal", token, ("&str", decode_string(token)))
        return None

    def path(self, token):
        if self.accept("::"):
            member = self.identifier()
            return Token(f"{token.text}::{member.text}", token.line, token.column)
        return token

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

    def pattern(self):
        first = self.range_pattern()
        alternatives = [first]
        while self.accept("|"):
            alternatives.append(self.range_pattern())
        if len(alternatives) > 1:
            return Expr("or", first.token, tuple(alternatives))
        return first

    def range_pattern(self):
        first = None if self.current.text in {"..", "..="} else self.simple_pattern()
        separator = self.accept("..") or self.accept("..=")
        if separator is None:
            return first
        if first is not None and first.kind != "literal":
            raise first.token.error("expected an integer range bound")
        last = None
        if self.current.text not in {",", ")", "}", "=>", "if", "|", "=", ":"}:
            last = self.simple_pattern()
        if last is not None and last.kind != "literal":
            raise last.token.error("expected an integer range bound")
        if last is None and (first is None or separator.text == "..="):
            raise separator.error("expected an integer range bound")
        return Expr("pattern_range", separator, (first, last, separator.text == "..="))

    def simple_pattern(self):
        token = self.advance()
        if token.text == "_":
            return Expr("wild", token)
        if token.text == "(":
            if self.accept(")"):
                return Expr("literal", token, ("()", None))
            inner = self.pattern()
            self.expect(")")
            return inner
        literal = self.literal(self.advance(), negative=True) if token.text == "-" else self.literal(token)
        if literal:
            return literal
        if token.text == "mut":
            return Expr("bind", self.identifier(), (True,))
        if not IDENT.fullmatch(token.text) or token.text in RESERVED:
            raise token.error("expected a pattern")
        path = self.path(token)
        if path.text in INTEGER_CONSTANTS:
            return Expr("literal", path, ("i32", INTEGER_CONSTANTS[path.text]))
        if self.current.text == "{":
            return Expr("pattern_record", path, self.record_fields(self.pattern, "bind"))
        if self.accept("("):
            patterns = []
            while self.current.text != ")":
                patterns.append(self.pattern())
                if not self.accept(","):
                    break
            self.expect(")")
            return Expr("pattern_tuple", path, tuple(patterns))
        if path != token:
            return Expr("pattern_unit", path)
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


def decode_string(token):
    source = token.text[1:-1]
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
        elif len(escape) == 3 and escape[0] == "x" and int(escape[1:], 16) < 128:
            parts.append(chr(int(escape[1:], 16)))
        elif escape.startswith("u{"):
            digits = escape[2:-1].rstrip("_").replace("_", "")
            codepoint = int(digits, 16) if digits else -1
            if not (1 <= len(digits) <= 6 and 0 <= codepoint <= 0x10FFFF
                    and not 0xD800 <= codepoint <= 0xDFFF
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

    def find(self, token):
        scope = self
        while scope is not None:
            if token.text in scope.bindings:
                return scope.bindings[token.text]
            scope = scope.parent
        raise token.error(f"unknown binding {token.text!r}")


def require(actual, expected, token):
    if actual not in {expected, NEVER}:
        raise token.error(f"expected {expected}, found {actual}")


def target(expr, scope):
    fields = []
    while expr.kind in {"group", "field"}:
        if expr.kind == "field":
            fields.append(expr.args[1])
        expr = expr.args[0]
    if expr.kind != "name":
        raise expr.token.error("invalid assignment target")
    binding = scope.find(expr.token)
    if not binding.mutable:
        raise expr.token.error(f"cannot assign to immutable binding {expr.token.text!r}")
    return binding, tuple(reversed(fields))


@dataclass(frozen=True)
class Value:
    constructor: Constructor
    fields: tuple

    def replace(self, path, value):
        if not path:
            return value
        index = self.constructor.index(path[0])
        fields = list(self.fields)
        fields[index] = self.fields[index].replace(path[1:], value) if path[1:] else value
        return Value(self.constructor, tuple(fields))


@dataclass(frozen=True)
class IntegerRange:
    start: int
    stop: int

    def contains(self, value):
        return self.start <= value < self.stop


@dataclass(frozen=True)
class Pattern:
    tag: object = None
    children: tuple = ()
    binding: str | None = None
    alternatives: tuple = ()

    def bindings(self, value):
        if self.alternatives:
            for alternative in self.alternatives:
                yield from alternative.bindings(value)
            return
        if self.tag is None:
            yield {self.binding: value} if self.binding else {}
        elif isinstance(self.tag, Constructor):
            if isinstance(value, Value) and value.constructor == self.tag:
                yield from self.field_bindings(value.fields)
        elif isinstance(self.tag, IntegerRange):
            if self.tag.contains(value):
                yield {}
        elif value == self.tag[1]:
            yield {}

    def field_bindings(self, values, index=0):
        if index == len(self.children):
            yield {}
            return
        for first in self.children[index].bindings(values[index]):
            for rest in self.field_bindings(values, index + 1):
                yield first | rest


def uncovered(matrix, types, program):
    """Find a missing case by specializing one constructor column at a time."""
    if not types:
        return None if matrix else ()
    annotation, *remaining_types = types
    if annotation == NEVER:
        return None
    if annotation in program.types:
        constructors = program.types[annotation][1]
        signatures = [(ctor, tuple(kind for _, kind in ctor.fields)) for ctor in constructors]
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
    if annotation == "i32" and explicit:
        return uncovered_integer(explicit, default, remaining_types, program)
    if signatures is None or (not explicit and signatures):
        missing = uncovered(default, remaining_types, program)
        return None if missing is None else (None, *missing)
    for signature, fields in signatures:
        wildcards = (Pattern(),) * len(fields)
        rows = [wildcards + tail for tail in default]
        rows.extend(first.children + tail for first, tail in explicit if first.tag == signature)
        missing = uncovered(rows, (*fields, *remaining_types), program)
        if missing is not None:
            nested, tail = missing[:len(fields)], missing[len(fields):]
            return ((signature, nested), *tail)
    return None


def uncovered_integer(explicit, default, types, program):
    boundaries = {-LIMIT, LIMIT}
    ranges = []
    for first, tail in explicit:
        tag = first.tag
        interval = tag if isinstance(tag, IntegerRange) else IntegerRange(tag[1], tag[1] + 1)
        boundaries.update((interval.start, interval.stop))
        ranges.append((interval, tail))
    # A pattern's membership is constant between adjacent boundaries.
    for value in sorted(boundaries)[:-1]:
        rows = default + [tail for interval, tail in ranges if interval.contains(value)]
        missing = uncovered(rows, types, program)
        if missing is not None:
            return ((("i32", value), ()), *missing)
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
    if tag.style == "tuple":
        return f"{tag.path}({', '.join(fields)})"
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


ARITHMETIC = {"+": operator.add, "-": operator.sub, "*": operator.mul}
COMPARE = {
    "==": operator.eq, "!=": operator.ne, "<": operator.lt,
    "<=": operator.le, ">": operator.gt, ">=": operator.ge,
}


class Checker:
    def __init__(self, program):
        self.program = program
        self.functions = program.functions
        self.result = "()"
        self.loops = []

    def check(self):
        finished = set()
        for name, (token, _) in self.program.types.items():
            self.check_layout(name, token, set(), finished)
        main = self.functions.get("main")
        if main is None:
            raise RustError("missing fn main()")
        if main.params or main.result != "()":
            raise main.name.error("main must take no arguments and return ()")
        for function in self.functions.values():
            constructor = self.program.constructors.get(function.name.text)
            if constructor and constructor.style != "record":
                raise function.name.error(f"duplicate value {function.name.text!r}")
            scope = Scope()
            for name, mutable, annotation in function.params:
                self.check_type(annotation, name)
                if name.text in scope.bindings:
                    raise name.error(f"duplicate parameter {name.text!r}")
                scope.bindings[name.text] = Binding(annotation, mutable)
            self.result = function.result
            self.check_type(self.result, function.name, allow_never=True)
            result = self.type_of(function.body, scope)
            require(result, function.result, function.body.token)

    def check_type(self, annotation, token, *, allow_never=False):
        if annotation in PRIMITIVES or annotation in self.program.types:
            return
        if annotation != NEVER or not allow_never:
            raise token.error(f"unsupported type {annotation!r}")

    def check_layout(self, name, token, visiting, finished):
        if name in finished:
            return
        if name in visiting:
            raise token.error(f"recursive type {name!r} has infinite size")
        visiting.add(name)
        for ctor in self.program.types[name][1]:
            for _, kind in ctor.fields:
                self.check_type(kind, token)
                if kind in self.program.types:
                    self.check_layout(kind, token, visiting, finished)
        visiting.remove(name)
        finished.add(name)

    def constructor(self, token, style):
        ctor = self.program.constructors.get(token.text)
        if not ctor or ctor.style != style:
            raise token.error(f"unknown {style} constructor {token.text!r}")
        return ctor

    def field_type(self, annotation, token):
        if annotation == NEVER:
            return NEVER
        ctor = self.program.constructors.get(annotation)
        if not ctor:
            raise token.error(f"{annotation} has no accessible fields")
        return ctor.fields[ctor.index(token)][1]

    @staticmethod
    def field_indices(ctor, fields, rest, token):
        indices = []
        for name, _ in fields:
            index = ctor.index(name)
            if index in indices:
                raise name.error(f"duplicate field {name.text!r}")
            indices.append(index)
        if not rest and len(indices) != len(ctor.fields):
            missing = [name for i, (name, _) in enumerate(ctor.fields) if i not in indices]
            raise token.error(f"missing fields: {', '.join(missing)}")
        return indices

    def prepare_pattern(self, expr, annotation):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "wild":
            return Pattern(), {}
        if kind == "literal":
            actual, value = args
            if actual == "i32":
                checked(value, token)
            require(actual, annotation, token)
            return Pattern(tag=(actual, value)), {}
        if kind == "pattern_range":
            require("i32", annotation, token)
            first, last, inclusive = args
            for endpoint in (first, last):
                if endpoint:
                    require(endpoint.args[0], "i32", endpoint.token)
                    checked(endpoint.args[1], endpoint.token)
            start = first.args[1] if first else -LIMIT
            stop = last.args[1] + int(inclusive) if last else LIMIT
            if start >= stop:
                raise token.error("empty integer range pattern")
            return Pattern(tag=IntegerRange(start, stop)), {}
        if kind == "or":
            pattern, bindings = self.prepare_pattern(args[0], annotation)
            alternatives = list(pattern.alternatives or (pattern,))
            for alternative in args[1:]:
                option, other = self.prepare_pattern(alternative, annotation)
                if bindings != other:
                    raise alternative.token.error("all alternatives must bind the same names and types")
                alternatives.extend(option.alternatives or (option,))
            return Pattern(alternatives=tuple(alternatives)), bindings
        if kind == "bind":
            ctor = self.program.constructors.get(token.text)
            if not ctor or ctor.style != "unit":
                return Pattern(binding=token.text), {token.text: (annotation, args[0])}
            if args[0]:
                raise token.error(f"cannot shadow unit struct {token.text!r}")
            kind = "pattern_unit"
            args = ()
        style = kind.removeprefix("pattern_")
        ctor = self.constructor(token, style)
        require(ctor.annotation, annotation, token)
        if style == "record":
            fields, rest = args
            indices = self.field_indices(ctor, fields, rest, token)
            assignments = zip(indices, (value for _, value in fields))
        else:
            if len(args) != len(ctor.fields):
                raise token.error(f"{ctor.path} takes {len(ctor.fields)} fields")
            assignments = enumerate(args)
        children = [Pattern()] * len(ctor.fields)
        bindings = {}
        for index, child in assignments:
            child_pattern, added = self.prepare_pattern(child, ctor.fields[index][1])
            duplicate = bindings.keys() & added.keys()
            if duplicate:
                raise child.token.error(f"duplicate pattern binding {sorted(duplicate)[0]!r}")
            bindings.update(added)
            children[index] = child_pattern
        return Pattern(tag=ctor, children=tuple(children)), bindings

    def match_type(self, expr, scope):
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
                require(self.type_of(guard, inner), "bool", guard.token)
                for name, (_, mutable) in bindings.items():
                    inner.bindings[name].mutable = mutable
            else:
                coverage.append((pattern,))
            actual = self.type_of(body, inner)
            if result == NEVER:
                result = actual
            else:
                require(actual, result, body.token)
        missing = uncovered(coverage, (annotation,), self.program)
        if missing is not None:
            raise expr.token.error(f"non-exhaustive match, missing {show_missing(missing[0])}")
        return NEVER if annotation == NEVER else result

    def loop_body(self, body, scope):
        self.loops.append(False)
        try:
            require(self.type_of(body, scope), "()", body.token)
            return self.loops[-1]
        finally:
            self.loops.pop()

    def type_of(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "literal":
            annotation, value = args
            if annotation == "i32" and value >= LIMIT:
                raise token.error("integer literal out of range for i32")
            return annotation
        if kind == "group":
            return self.type_of(args[0], scope)
        if kind == "name":
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "unit":
                return ctor.annotation
            return scope.find(token).value
        if kind == "field":
            value, name = args
            return self.field_type(self.type_of(value, scope), name)
        if kind == "record":
            ctor = self.constructor(token, "record")
            fields = args[0]
            indices = self.field_indices(ctor, fields, False, token)
            actual = [self.type_of(value, scope) for _, value in fields]
            for index, kind in zip(indices, actual):
                require(kind, ctor.fields[index][1], token)
            return NEVER if NEVER in actual else ctor.annotation
        if kind == "match":
            return self.match_type(expr, scope)
        if kind == "block":
            statements, tail = args
            inner = Scope(scope)
            diverges = False
            for statement in statements:
                diverges |= self.type_of(statement, inner) == NEVER
            result = self.type_of(tail, inner) if tail else "()"
            return NEVER if diverges else result
        if kind == "let":
            source, annotation, value, otherwise = args
            actual = self.type_of(value, scope)
            if annotation:
                self.check_type(annotation, token)
                require(actual, annotation, token)
            annotation = annotation or actual
            pattern, bindings = self.prepare_pattern(source, annotation)
            self.program.patterns[id(source)] = (pattern, bindings)
            if otherwise:
                require(self.type_of(otherwise, scope), NEVER, otherwise.token)
            else:
                missing = uncovered([(pattern,)], (annotation,), self.program)
                if missing is not None:
                    raise token.error(f"refutable let pattern, missing {show_missing(missing[0])}")
            for name, (kind, mutable) in bindings.items():
                scope.bindings[name] = Binding(kind, mutable)
            return NEVER if actual == NEVER else "()"
        if kind == "discard":
            value, requires_unit = args
            actual = self.type_of(value, scope)
            if requires_unit:
                require(actual, "()", token)
            return NEVER if actual == NEVER else "()"
        if kind == "if":
            condition, yes, no = args
            test = self.type_of(condition, scope)
            require(test, "bool", condition.token)
            left = self.type_of(yes, scope)
            right = self.type_of(no, scope) if no else "()"
            if left != NEVER:
                require(right, left, token)
            if test == NEVER:
                return NEVER
            return right if left == NEVER else left
        if kind == "while":
            condition, body = args
            test = self.type_of(condition, scope)
            require(test, "bool", condition.token)
            self.loop_body(body, scope)
            return NEVER if test == NEVER else "()"
        if kind == "loop":
            return "()" if self.loop_body(args[0], scope) else NEVER
        if kind == "for":
            name, mutable, start, end, _, body = args
            bounds = [self.type_of(bound, scope) for bound in (start, end)]
            for bound in bounds:
                require(bound, "i32", token)
            self.loop_body(body, Scope(scope, {name.text: Binding("i32", mutable)}))
            return NEVER if NEVER in bounds else "()"
        if kind == "return":
            actual = self.type_of(args[0], scope) if args else "()"
            require(actual, self.result, token)
            return NEVER
        if kind in {"break", "continue"}:
            if not self.loops:
                raise token.error(f"{kind} outside a loop")
            if kind == "break":
                self.loops[-1] = True
            return NEVER
        if kind == "unary":
            operand = args[0]
            if min_literal(expr):
                return "i32"
            actual = self.type_of(operand, scope)
            if token.text == "!" and actual == "bool":
                return "bool"
            require(actual, "i32", token)
            return actual
        if kind == "assign":
            left, right = args
            binding, fields = target(left, scope)
            annotation = binding.value
            for name in fields:
                annotation = self.field_type(annotation, name)
            if token.text != "=":
                require(annotation, "i32", token)
            actual = self.type_of(right, scope)
            require(actual, annotation, token)
            return NEVER if actual == NEVER else "()"
        if kind == "binary":
            return self.binary_type(token, *(self.type_of(arg, scope) for arg in args))
        if kind == "call":
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "tuple":
                if len(args) != len(ctor.fields):
                    raise token.error(f"{ctor.path} takes {len(ctor.fields)} arguments")
                types = [self.type_of(arg, scope) for arg in args]
                for actual, (_, expected) in zip(types, ctor.fields):
                    require(actual, expected, token)
                return NEVER if NEVER in types else ctor.annotation
            function = self.functions.get(token.text)
            if function is None:
                raise token.error(f"unknown function {token.text!r}")
            if len(args) != len(function.params):
                raise token.error(f"{token.text} takes {len(function.params)} arguments")
            types = [self.type_of(arg, scope) for arg in args]
            for actual, (_, _, expected) in zip(types, function.params):
                require(actual, expected, token)
            return NEVER if NEVER in types else function.result
        if kind == "print":
            if not args:
                return "()"
            template, *values = args
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
                    actual = scope.find(Token(name, token.line, token.column)).value
                else:
                    actual = next(positional)
                if mode == "display" and actual == "()":
                    raise token.error("() requires debug formatting")
                if actual in self.program.types:
                    raise token.error(f"{actual} does not implement the requested formatting trait")
            return NEVER if NEVER in types else "()"
        raise AssertionError(kind)

    @staticmethod
    def binary_type(token, left, right):
        word = token.text
        if word in {"&&", "||"}:
            require(left, "bool", token)
            require(right, "bool", token)
            return NEVER if left == NEVER else "bool"
        if word in COMPARE:
            if left != NEVER and right != NEVER:
                require(right, left, token)
                if left not in PRIMITIVES:
                    raise token.error(f"{left} does not implement the requested comparison trait")
            return NEVER if NEVER in {left, right} else "bool"
        require(left, "i32", token)
        require(right, "i32", token)
        return NEVER if NEVER in {left, right} else "i32"


def min_literal(expr):
    operand = expr.args[0]
    while operand.kind == "group":
        operand = operand.args[0]
    return expr.token.text == "-" and operand.kind == "literal" and operand.args == ("i32", LIMIT)


def checked(value, token):
    if not -LIMIT <= value < LIMIT:
        raise token.error("i32 arithmetic overflow")
    return value


def binary(word, left, right, token):
    if word in COMPARE:
        # The unit type has exactly one value, so any two units compare equal.
        if left is None:
            left = right = ()
        return COMPARE[word](left, right)
    if word in ARITHMETIC:
        return checked(ARITHMETIC[word](left, right), token)
    if right == 0:
        raise token.error("division by zero")
    quotient = abs(left) // abs(right)
    if (left < 0) != (right < 0):
        quotient = -quotient
    checked(quotient, token)
    return quotient if word == "/" else left - quotient * right


def display(value, debug):
    if value is None:
        return "()"
    if isinstance(value, bool):
        return "true" if value else "false"
    if debug and isinstance(value, str):
        parts = ['"']
        for char in value:
            if char in {'"', '\\', '\n', '\r', '\t', '\0'}:
                escape = next(key for key, item in ESCAPES.items() if item == char)
                parts.append("\\" + escape)
            elif not char.isprintable():
                parts.append(f"\\u{{{ord(char):x}}}")
            else:
                parts.append(char)
        return "".join(parts) + '"'
    return str(value)


class Return(Exception):
    pass


class Break(Exception):
    pass


class Continue(Exception):
    pass


class Interpreter:
    def __init__(self, program, stdout):
        self.program = program
        self.functions = program.functions
        self.stdout = stdout

    def match_bindings(self, source, value):
        pattern, types = self.program.patterns[id(source)]
        for bindings in pattern.bindings(value):
            yield {name: Binding(item, types[name][1]) for name, item in bindings.items()}

    def call(self, name, values):
        function = self.functions[name]
        bindings = {
            param.text: Binding(value, mutable)
            for (param, mutable, _), value in zip(function.params, values)
        }
        try:
            return self.eval(function.body, Scope(bindings=bindings))
        except Return as signal:
            return signal.args[0]

    def eval(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "literal":
            return args[1]
        if kind == "group":
            return self.eval(args[0], scope)
        if kind == "name":
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "unit":
                return Value(ctor, ())
            return scope.find(token).value
        if kind == "field":
            value, name = args
            aggregate = self.eval(value, scope)
            return aggregate.fields[aggregate.constructor.index(name)]
        if kind == "record":
            ctor = self.program.constructors[token.text]
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
            for statement in statements:
                self.eval(statement, inner)
            return self.eval(tail, inner) if tail else None
        if kind == "let":
            source, _, value, otherwise = args
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
        if kind in {"while", "loop"}:
            condition, body = args if kind == "while" else (None, args[0])
            while condition is None or self.eval(condition, scope):
                try:
                    self.eval(body, scope)
                except Continue:
                    continue
                except Break:
                    break
            return None
        if kind == "for":
            name, mutable, start, end, inclusive, body = args
            first = self.eval(start, scope)
            stop = self.eval(end, scope) + int(inclusive)
            for value in range(first, stop):
                inner = Scope(scope, {name.text: Binding(value, mutable)})
                try:
                    self.eval(body, inner)
                except Continue:
                    continue
                except Break:
                    break
            return None
        if kind == "return":
            raise Return(self.eval(args[0], scope) if args else None)
        if kind == "break":
            raise Break()
        if kind == "continue":
            raise Continue()
        if kind == "unary":
            if min_literal(expr):
                return -LIMIT
            value = self.eval(args[0], scope)
            if token.text == "-":
                return checked(-value, token)
            return not value if isinstance(value, bool) else ~value
        if kind == "assign":
            left, right = args
            value = self.eval(right, scope)
            binding, fields = target(left, scope)
            if token.text != "=":
                old = binding.value
                for name in fields:
                    old = old.fields[old.constructor.index(name)]
                value = binary(token.text[0], old, value, token)
            binding.value = binding.value.replace(fields, value) if fields else value
            return None
        if kind == "binary":
            left, right = args
            value = self.eval(left, scope)
            if token.text == "&&":
                return value and self.eval(right, scope)
            if token.text == "||":
                return value or self.eval(right, scope)
            return binary(token.text, value, self.eval(right, scope), token)
        if kind == "call":
            values = tuple(self.eval(arg, scope) for arg in args)
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "tuple":
                return Value(ctor, values)
            return self.call(token.text, values)
        if kind == "print":
            if not args:
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
                    item = scope.find(Token(value, token.line, token.column)).value
                else:
                    item = next(evaluated)
                rendered.append(display(item, mode == "debug"))
            print("".join(rendered), file=self.stdout)
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


def run(source, stdout=None):
    program = Parser(source).parse()
    Checker(program).check()
    Interpreter(program, sys.stdout if stdout is None else stdout).call("main", ())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", help="Rust source file, or - for stdin")
    parser.add_argument("--demo", action="store_true", help="run a built-in Rust program")
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
        run(source)
    except (RustError, OSError, UnicodeError) as error:
        print(f"{label}: {error}", file=sys.stderr)
        return 1
    except RecursionError:
        print(f"{label}: Python recursion limit reached", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
