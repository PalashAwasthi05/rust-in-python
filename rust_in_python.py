"""A standard-library-only interpreter for a scalar subset of Rust.

Run: python3 rust_in_python.py program.rs
Demo: python3 rust_in_python.py --demo

Supported types: i32, bool, &str (string literals), (), structs, and enums.
Supported syntax: functions, let bindings, mut, blocks, if/else,
while, loop, for over integer ranges, return, break, continue, scalar operators,
exhaustive match, and println!. Integers use checked i32 arithmetic.
This subset checks moves, initialization, and local shared and mutable loans.
Reference returns support elision and named lifetimes on function signatures.
References in aggregates and the standard library are outside its scope.
"""

from __future__ import annotations

import argparse
from collections import deque
from dataclasses import dataclass, field, replace
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
    r"'[A-Za-z_][A-Za-z_0-9]*|"
    r'[0-9][0-9_]*(?:i32)?|[A-Za-z_][A-Za-z_0-9]*|'
    r'\.\.=|\.\.|::|->|=>|==|!=|<=|>=|&&|\|\||[+*/%\-]=|[{}()[\]#.:,;=!+*/%&<>|\-]'
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
BLOCK_EXPRESSIONS = {"block", "if", "while", "while_let", "loop", "for", "match"}
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
    label: Token | None = None
    type_arguments: tuple | None = None


@dataclass
class Function:
    name: Token
    params: list
    result: object
    body: Expr
    return_inputs: tuple = ()
    type_parameters: tuple = ()


@dataclass(frozen=True)
class Constructor:
    path: str
    annotation: object
    style: str
    fields: tuple

    def index(self, token):
        for index, (name, _) in enumerate(self.fields):
            if name == token.text:
                return index
        raise token.error(f"{self.path} has no field {token.text!r}")


@dataclass(frozen=True)
class ReferenceType:
    inner: object
    mutable: bool
    lifetime: str | None = field(default=None, compare=False)

    def __str__(self):
        prefix = "&mut " if self.mutable else "&"
        return f"{prefix}{self.inner}"


@dataclass(frozen=True, eq=False)
class TypeParameter:
    name: str
    bounds: set = field(default_factory=set)

    def __str__(self):
        return self.name


@dataclass(frozen=True, eq=False)
class TypeVariable:
    name: str
    token: Token

    def __str__(self):
        return self.name


@dataclass(frozen=True)
class AppliedType:
    name: str
    arguments: tuple

    def __str__(self):
        return f"{self.name}<{', '.join(map(str, self.arguments))}>"


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


def implies(bounds, trait):
    return trait in bounds or any(trait in SUPERTRAITS.get(bound, ()) for bound in bounds)


def substitute(annotation, arguments):
    if isinstance(annotation, ReferenceType):
        return replace(annotation, inner=substitute(annotation.inner, arguments))
    if isinstance(annotation, AppliedType):
        return AppliedType(annotation.name, tuple(substitute(arg, arguments) for arg in annotation.arguments))
    return arguments.get(annotation, annotation)


@dataclass
class Program:
    functions: dict = field(default_factory=dict)
    types: dict = field(default_factory=dict)
    constructors: dict = field(default_factory=dict)
    patterns: dict = field(default_factory=dict)
    expr_types: dict = field(default_factory=dict)
    derives: dict = field(default_factory=dict)
    call_types: dict = field(default_factory=dict)
    type_parameters: dict = field(default_factory=dict)
    default_variants: dict = field(default_factory=dict)

    def layout(self, annotation):
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

    def implements(self, annotation, trait, assumed=()):
        if trait == "Sized":
            return True
        if isinstance(annotation, TypeParameter):
            return annotation in assumed or implies(annotation.bounds, trait)
        if isinstance(annotation, ReferenceType):
            if trait in {"Copy", "Clone"}:
                return not annotation.mutable
            return trait != "Default" and self.implements(annotation.inner, trait, assumed)
        if isinstance(annotation, AppliedType):
            if trait == "Default" and annotation.name in self.default_variants:
                return True
            return (trait in self.derives.get(annotation.name, {})
                    and all(self.implements(arg, trait, assumed) for arg in annotation.arguments))
        if annotation in PRIMITIVES:
            return trait != "Display" or annotation != "()"
        return trait in self.derives.get(annotation, {})

    def is_copy(self, annotation):
        return self.implements(annotation, "Copy")


class Parser:
    def __init__(self, source):
        self.tokens = iter(lex(source))
        self.current = next(self.tokens)
        self.type_parameters = {}
        self.trait_names = {name: name for name in TRAIT_MODULES if TRAIT_MODULES[name] != "fmt"}

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

    def type_name(self, *, static_string=False, signature=None, output=False):
        token = self.advance()
        if token.text in {"i32", "bool"}:
            return token.text
        if token.text == "&":
            lifetime = self.advance() if self.current.text.startswith("'") else None
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
            self.expect(")")
            return "()"
        if token.text == "!":
            return NEVER
        if IDENT.fullmatch(token.text) and token.text not in RESERVED:
            if self.accept("<"):
                return AppliedType(token.text, self.type_arguments())
            return self.type_parameters.get(token.text, token.text)
        raise token.error(f"unsupported type {token.text!r}")

    def parse(self):
        program = Program()
        while self.current.text:
            if self.accept("use"):
                self.trait_import()
                continue
            derives = self.attributes()
            self.accept("pub")
            item = self.accept("struct") or self.accept("enum")
            if item:
                self.type_item(program, item, derives)
                continue
            if derives:
                raise self.current.error("derive requires a struct or enum")
            self.expect("fn")
            name = self.identifier()
            if name.text in program.functions:
                raise name.error(f"duplicate function {name.text!r}")
            lifetimes, self.type_parameters = self.generic_parameters()
            self.expect("(")
            params = []
            while self.current.text != ")":
                mutable = bool(self.accept("mut"))
                param = self.accept("_") or self.identifier()
                if mutable and param.text == "_":
                    raise param.error("wildcard parameters cannot be mutable")
                self.expect(":")
                params.append((param, mutable, self.type_name(signature=lifetimes)))
                if not self.accept(","):
                    break
            self.expect(")")
            result = self.type_name(signature=lifetimes, output=True) if self.accept("->") else "()"
            self.where_clause(lifetimes)
            return_inputs = self.return_inputs(name, params, result, lifetimes)
            parameters = tuple(self.type_parameters.values())
            program.functions[name.text] = Function(name, params, result, self.block(),
                                                    return_inputs, parameters)
            self.type_parameters = {}
        return program

    def generic_parameters(self):
        lifetimes = {}
        types = {}
        if not self.accept("<"):
            return lifetimes, types
        while self.current.text != ">":
            token = self.advance()
            if token.text.startswith("'") and token.text not in {"'static", "'_"}:
                if types:
                    raise token.error("lifetime parameters must precede type parameters")
                if token.text in lifetimes:
                    raise token.error(f"duplicate lifetime {token.text}")
                lifetimes[token.text] = self.lifetime_bounds() if self.accept(":") else set()
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
        token = self.identifier()
        parts = [token.text]
        while self.accept("::"):
            parts.append(self.identifier().text)
        if len(parts) == 1 and parts[0] in self.trait_names:
            return self.trait_names[parts[0]]
        if (len(parts) == 3 and parts[0] in {"std", "core"}
                and TRAIT_MODULES.get(parts[2]) == parts[1]):
            return parts[2]
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
            if not token.text.startswith("'"):
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
            lhs = self.advance()
            self.expect(":")
            if lhs.text.startswith("'"):
                self.require_lifetimes((lhs,), lifetimes)
                bounds = self.lifetime_bounds()
                self.require_lifetimes(bounds, lifetimes)
                lifetimes[lhs.text].update(bounds)
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
        name = self.identifier()
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
            if name.text == "clone" and self.current.text == "(":
                if self.arguments():
                    raise name.error("clone takes no arguments")
                left = Expr("clone", name, (left.args[0],))
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
        if word.startswith("'"):
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
            if word != "return" and self.current.text.startswith("'"):
                label = self.advance()
            value = None
            if word != "continue" and self.current.text not in {";", "}", ")", ",", "=>", "else"}:
                value = self.expression(allow_record=allow_record)
            return Expr(word, token, (value,), label)
        if word in {"-", "!"}:
            return Expr("unary", token, (self.expression(7, allow_record=allow_record),))
        if word in {"&", "*"}:
            mutable = bool(self.accept("mut")) if word == "&" else False
            operand = self.expression(7, allow_record=allow_record)
            return Expr("borrow" if word == "&" else "deref", token, (operand, mutable))
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
            path, type_arguments = self.expression_path(token)
            if path.text in INTEGER_CONSTANTS:
                return Expr("literal", path, ("i32", INTEGER_CONSTANTS[path.text]))
            if allow_record and self.current.text == "{":
                return Expr("record", path, (self.record_fields(self.expression, "name")[0],),
                            type_arguments=type_arguments)
            if self.current.text == "(":
                return Expr("call", path, self.arguments(), type_arguments=type_arguments)
            return Expr("name", path, type_arguments=type_arguments)
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

    def expression_path(self, token):
        if not self.accept("::"):
            return token, None
        arguments = None
        if self.accept("<"):
            arguments = self.type_arguments()
            if not self.accept("::"):
                return token, arguments
        member = self.identifier()
        path = Token(f"{token.text}::{member.text}", token.line, token.column)
        if self.accept("::"):
            if arguments is not None:
                raise path.error("type arguments may appear only once in the path")
            self.expect("<")
            arguments = self.type_arguments()
        return path, arguments

    def type_arguments(self):
        arguments = []
        while self.current.text not in {">", ">="}:
            arguments.append(None if self.accept("_") else self.type_name())
            if not self.accept(","):
                break
        self.expect(">")
        return tuple(arguments)

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
        if last is None and first is None and separator.text == "..":
            return Expr("rest", separator)
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
        path, type_arguments = self.expression_path(token)
        if path.text in INTEGER_CONSTANTS:
            return Expr("literal", path, ("i32", INTEGER_CONSTANTS[path.text]))
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
    owned: set = field(default_factory=set)

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


def ungroup(expr):
    while expr.kind == "group":
        expr = expr.args[0]
    return expr


def place_root(expr):
    root = ungroup(expr)
    while root.kind == "field":
        root = ungroup(root.args[0])
    return root


def reference_source(expr):
    expr = ungroup(expr)
    return expr.kind in {"name", "borrow", "call", "if", "match"} or (
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
            value = value.fields[value.constructor.index(name)]
        return value

    def write(self, value):
        if self.fields:
            value = self.binding.value.replace(self.fields, value)
        self.binding.value = value


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
            if isinstance(value, Value) and value.constructor.path == self.tag.path:
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
    declaration = program.layout(annotation)
    if declaration is not None:
        constructors = declaration[1]
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
        tag = signature.path if isinstance(signature, Constructor) else signature
        rows.extend(first.children + tail for first, tail in explicit
                    if (first.tag.path if isinstance(first.tag, Constructor) else first.tag) == tag)
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


@dataclass
class ControlTarget:
    expr: Expr
    exit_types: list = field(default_factory=list)
    in_condition: bool = False
    restart: FlowNode | None = None
    finish: FlowNode | None = None


def find_control(targets, jump):
    for target in reversed(targets):
        if jump.label:
            label = target.expr.label
            if label is None or label.text != jump.label.text:
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
        self.substitutions = {}
        self.variables = []
        self.obligations = []
        self.generic_arguments = []
        self.numeric_operands = []
        self.unary_results = []
        self.parameters = {}
        self.ascriptions = []

    def check(self):
        finished = set()
        for name, (token, _) in self.program.types.items():
            self.check_layout(name, token, set(), finished)
            self.check_derives(name)
        main = self.functions.get("main")
        if main is None:
            raise RustError("missing fn main()")
        if main.params or main.result != "()" or main.type_parameters:
            raise main.name.error("main must take no arguments and return ()")
        for function in self.functions.values():
            self.parameters = {parameter.name: parameter for parameter in function.type_parameters}
            constructor = self.program.constructors.get(function.name.text)
            if constructor and constructor.style != "record":
                raise function.name.error(f"duplicate value {function.name.text!r}")
            scope = Scope()
            for name, mutable, annotation in function.params:
                self.check_type(annotation, name)
                if name.text == "_":
                    continue
                if name.text in scope.bindings:
                    raise name.error(f"duplicate parameter {name.text!r}")
                scope.bindings[name.text] = Binding(annotation, mutable)
            self.result = function.result
            self.check_type(self.result, function.name, allow_never=True)
            result = self.type_of(function.body, scope, function.result)
            self.require_argument(result, function.result, function.body.token)
        self.finish_inference()

    def resolve(self, annotation):
        if isinstance(annotation, TypeVariable) and annotation in self.substitutions:
            result = self.resolve(self.substitutions[annotation])
            self.substitutions[annotation] = result
            return result
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.resolve(annotation.inner))
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.resolve(arg) for arg in annotation.arguments))
        return annotation

    def occurs(self, variable, annotation):
        annotation = self.resolve(annotation)
        if isinstance(annotation, ReferenceType):
            return self.occurs(variable, annotation.inner)
        if isinstance(annotation, AppliedType):
            return any(self.occurs(variable, arg) for arg in annotation.arguments)
        return annotation is variable

    def require(self, actual, expected, token):
        actual, expected = self.resolve(actual), self.resolve(expected)
        if actual in {NEVER, expected}:
            return
        if isinstance(expected, TypeVariable) and not self.occurs(expected, actual):
            self.substitutions[expected] = actual
            return
        if isinstance(actual, TypeVariable) and expected != NEVER and not self.occurs(actual, expected):
            self.substitutions[actual] = expected
            return
        if isinstance(actual, ReferenceType) and isinstance(expected, ReferenceType):
            if actual.mutable == expected.mutable:
                self.require(actual.inner, expected.inner, token)
                return
        if isinstance(actual, AppliedType) and isinstance(expected, AppliedType):
            if actual.name == expected.name and len(actual.arguments) == len(expected.arguments):
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

    def specialize(self, parameters, explicit, token):
        if explicit is not None and (not parameters or len(explicit) != len(parameters)):
            raise token.error(f"{token.text} expects {len(parameters)} type arguments")
        arguments = {}
        for index, parameter in enumerate(parameters):
            argument = explicit[index] if explicit is not None else None
            if argument is None:
                argument = TypeVariable(parameter.name, token)
                self.variables.append(argument)
            else:
                argument = self.fill_holes(argument, token)
                self.check_type(argument, token)
            arguments[parameter] = argument
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
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.fill_holes(arg, token)
                                                       for arg in annotation.arguments))
        return annotation

    def finish_inference(self):
        for operand, result, token in self.unary_results:
            operand = self.resolve(operand)
            if not isinstance(operand, TypeVariable):
                if operand not in {"i32", "bool"}:
                    raise token.error(f"cannot negate {operand} with '!'")
                self.require(result, operand, token)
        for variable in self.variables:
            if isinstance(self.resolve(variable), TypeVariable):
                raise variable.token.error(f"type annotations needed for {variable.name}")
        for annotation, token in self.generic_arguments:
            annotation = self.resolve(annotation)
            if isinstance(annotation, ReferenceType):
                raise token.error("references as generic type arguments are unsupported")
        for annotation, trait, token in self.obligations:
            annotation = self.resolve(annotation)
            if annotation != NEVER and not self.implements(annotation, trait):
                raise token.error(f"{annotation} does not implement {trait}")
        for annotation, token in self.numeric_operands:
            self.require(annotation, "i32", token)
        for key, value in self.program.expr_types.items():
            self.program.expr_types[key] = self.resolve(value)
        for key, (types, arguments) in self.program.call_types.items():
            self.program.call_types[key] = (tuple(self.resolve(kind) for kind in types),
                                            {p: self.resolve(arg) for p, arg in arguments.items()})
        for key, (pattern, bindings) in self.program.patterns.items():
            self.program.patterns[key] = (
                self.resolve_pattern(pattern),
                {name: (self.resolve(kind), mutable) for name, (kind, mutable) in bindings.items()},
            )
        for expr in self.ascriptions:
            source, annotation, value, otherwise = expr.args
            expr.args = (source, self.resolve(annotation), value, otherwise)

    def resolve_pattern(self, pattern):
        tag = pattern.tag
        if isinstance(tag, Constructor):
            tag = replace(tag, annotation=self.resolve(tag.annotation),
                          fields=tuple((name, self.resolve(kind)) for name, kind in tag.fields))
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
            return
        if isinstance(annotation, ReferenceType):
            if isinstance(annotation.inner, ReferenceType):
                raise token.error("nested reference types are unsupported")
            self.check_type(annotation.inner, token)
            return
        if isinstance(annotation, AppliedType):
            parameters = self.program.type_parameters.get(annotation.name)
            if not parameters or len(parameters) != len(annotation.arguments):
                raise token.error(f"{annotation.name} expects {len(parameters or ())} type arguments")
            for parameter, argument in zip(parameters, annotation.arguments):
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
            if param not in used:
                raise token.error(f"unused type parameter {param}")
        visiting.remove(name)
        finished.add(name)

    @staticmethod
    def type_parts(annotation):
        if isinstance(annotation, AppliedType):
            yield annotation.name
            for argument in annotation.arguments:
                yield from Checker.type_parts(argument)
        else:
            yield annotation

    def constructor(self, expr, style, expected=None):
        token = expr.token
        ctor = self.program.constructors.get(token.text)
        if not ctor or ctor.style != style:
            raise token.error(f"unknown {style} constructor {token.text!r}")
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
        if kind == "rest":
            raise token.error("'..' is only allowed within a tuple or record pattern")
        if kind == "literal":
            actual, value = args
            if actual == "i32":
                checked(value, token)
            self.require(actual, annotation, token)
            return Pattern(tag=(actual, value)), {}
        if kind == "pattern_range":
            self.require("i32", annotation, token)
            first, last, inclusive = args
            for endpoint in (first, last):
                if endpoint:
                    self.require(endpoint.args[0], "i32", endpoint.token)
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
                resolved = {key: (self.resolve(t), m) for key, (t, m) in bindings.items()}
                incoming = {key: (self.resolve(t), m) for key, (t, m) in other.items()}
                if resolved != incoming:
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
        ctor = self.constructor(expr, style, annotation)
        self.require(ctor.annotation, annotation, token)
        if style == "record":
            fields, rest = args
            indices = self.field_indices(ctor, fields, rest, token)
            assignments = zip(indices, (value for _, value in fields))
        else:
            rest = [index for index, child in enumerate(args) if child.kind == "rest"]
            if len(rest) > 1:
                raise args[rest[1]].token.error("a tuple pattern can contain only one '..'")
            if rest and len(args) - 1 <= len(ctor.fields):
                index = rest[0]
                omitted = len(ctor.fields) - len(args) + 1
                wildcard = Expr("wild", args[index].token)
                args = args[:index] + (wildcard,) * omitted + args[index + 1:]
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

    def match_type(self, expr, scope, expected=None):
        scrutinee, arms = expr.args
        annotation = self.type_of(scrutinee, scope)
        if isinstance(annotation, ReferenceType):
            raise scrutinee.token.error("match on a reference requires explicit dereferencing")
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
            bounds = [self.type_of(bound, scope) for bound in (start, end)]
            for bound in bounds:
                self.require(bound, "i32", token)
            if NEVER in bounds:
                test = NEVER
            scope = Scope(scope, {name.text: Binding("i32", mutable)})
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
                    if isinstance(test, ReferenceType):
                        raise condition.token.error("matching a reference requires explicit dereferencing")
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
        if expr.kind == "block" and expr.label:
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
        if isinstance(result, ReferenceType) and not reference_source(expr):
            raise expr.token.error(f"reference result from {expr.kind} is unsupported")
        self.program.expr_types[id(expr)] = result
        return result

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
        if kind == "literal":
            annotation, value = args
            if annotation == "i32" and value >= LIMIT:
                raise token.error("integer literal out of range for i32")
            return annotation
        if kind == "group":
            return self.type_of(args[0], scope, expected)
        if kind == "name":
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "unit":
                return self.constructor(expr, "unit", expected).annotation
            if expr.type_arguments is not None:
                raise token.error("type arguments require a generic call or constructor")
            return scope.find(token).value
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
                if value is None or source.kind not in {"bind", "wild"} or otherwise:
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
            target = find_control(self.loops, expr)
            if kind == "break":
                if args[0] and target.expr.kind not in {"loop", "block"}:
                    raise token.error("break values require loop or a labeled block")
                actual = self.type_of(args[0], scope) if args[0] else "()"
                target.exit_types.append((actual, token))
            return NEVER
        if kind == "unary":
            operand = args[0]
            if min_literal(expr):
                return "i32"
            actual = self.type_of(operand, scope)
            if token.text == "!" and actual == "bool":
                return "bool"
            if token.text == "!" and isinstance(actual, TypeVariable):
                result = TypeVariable("operator result", token)
                self.variables.append(result)
                self.unary_results.append((actual, result, token))
                return result
            self.numeric_operands.append((actual, token))
            return NEVER if actual == NEVER else "i32"
        if kind == "assign":
            left, right = args
            annotation = self.type_of(left, scope)
            self.require_place(left, write=True)
            if isinstance(annotation, ReferenceType):
                raise token.error("reassigning reference bindings is unsupported")
            actual = self.type_of(right, scope, annotation if token.text == "=" else None)
            if token.text == "=":
                self.require(actual, annotation, token)
            else:
                self.numeric_operands.extend(((annotation, token), (actual, token)))
            return NEVER if NEVER in {annotation, actual} else "()"
        if kind == "binary":
            hint = "bool" if token.text in {"&&", "||"} else None
            left = self.type_of(args[0], scope, hint)
            second_hint = left if token.text in COMPARE and left != NEVER else hint
            return self.binary_type(token, left, self.type_of(args[1], scope, second_hint))
        if kind == "call":
            return self.call_type(expr, scope, expected)
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
                if isinstance(actual, ReferenceType):
                    actual = actual.inner
                if mode == "display" and actual == "()":
                    raise token.error("() requires debug formatting")
                self.obligation(actual, "Debug" if mode == "debug" else "Display", token)
            return NEVER if NEVER in types else "()"
        raise AssertionError(kind)

    def call_type(self, expr, scope, expected):
        token, args = expr.token, expr.args
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
                raise token.error(f"unknown function {token.text!r}")
            arguments = self.specialize(function.type_parameters, expr.type_arguments, token)
            params = tuple(substitute(kind, arguments) for _, _, kind in function.params)
            result = substitute(function.result, arguments)
        if len(args) != len(params):
            raise token.error(f"{token.text} takes {len(params)} arguments")
        if expected is not None and arguments:
            self.require_argument(result, expected, token)
        actual_types = []
        for expr_arg, parameter in zip(args, params):
            hint = self.resolve(parameter)
            actual = self.type_of(expr_arg, scope, hint)
            self.require_argument(actual, parameter, expr_arg.token)
            actual_types.append(actual)
        self.program.call_types[id(expr)] = (params, arguments)
        return NEVER if NEVER in actual_types else self.resolve(result)

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
        self.numeric_operands.extend(((left, token), (right, token)))
        return NEVER if NEVER in {left, right} else "i32"


@dataclass(frozen=True, eq=False)
class Local:
    token: Token
    mutable: bool
    loan: Loan | None = None
    static: bool = False


@dataclass(frozen=True)
class Place:
    local: Local
    fields: tuple = ()
    holder: Local | None = field(default=None, compare=False)

    def child(self, field):
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
        for function in self.program.functions.values():
            scope = Scope()
            entry = FlowNode()
            cursor = entry
            self.return_roots = set()
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
                scope.bindings[token.text] = local
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
        if expr.kind == "call":
            return self.lower_call(expr, scope, cursor)
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
            return self.start_loan(cursor, place, mutable, expr.token, expiring=expiring)
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
        if expr.kind in {"unary", "binary"}:
            return all(self.promotable(operand) for operand in expr.args)
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
        function = self.program.functions.get(token.text)
        expected, _ = self.program.call_types[id(expr)]
        temporaries = []
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
            local = scope.find(Token(name, token.line, token.column))
            place = self.referent(local) if local.loan else Place(local)
            cursor, loan = self.start_loan(cursor, place, False, token)
            cursor, holder = self.keep_loan(cursor, loan, token)
            temporaries.append(holder)
        for holder in temporaries:
            cursor = self.emit(cursor, "read", holder, token)
        return cursor

    def move_action(self, place):
        if place.holder:
            return "deref_move"
        if place.local in self.guarded or any(place.overlaps(item) for item in self.guard_borrows):
            return "guard_move"
        return "move"

    def lower(self, expr, scope, cursor, *, consume=True):
        kind, token, args = expr.kind, expr.token, expr.args
        cursor, place = self.lower_address(expr, scope, cursor)
        if place is not None:
            return self.access(cursor, place, self.annotation(expr), token, consume=consume)
        if kind in {"literal", "name", "default"}:
            return cursor
        if kind == "borrow":
            cursor, _ = self.lower_reference(expr, scope, cursor)
            return cursor
        if kind in {"group", "field", "unary", "deref"}:
            return self.lower(args[0], scope, cursor)
        if kind == "clone":
            return self.lower(args[0], scope, cursor, consume=False)
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
            if isinstance(self.annotation(value), ReferenceType):
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
            cursor, origin = self.lower_address(value, scope, cursor)
            if origin is None:
                cursor = self.lower(value, scope, cursor)
            reads, moves, tests = self.pattern_accesses(pattern, origin, self.annotation(value))
            cursor = self.read_pattern(cursor, tests | reads, source.token)
            if otherwise:
                self.lower(otherwise, Scope(scope), cursor)
            cursor = self.move_pattern(cursor, moves, source.token)
            return self.declare(source, types, scope, cursor)
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
            return self.emit(cursor, action, target_place, token)
        if kind == "binary":
            if token.text in COMPARE and self.annotation(args[0]) not in {"i32", "bool"}:
                return self.lower_comparison(args, scope, cursor)
            left, right = args
            cursor = self.lower(left, scope, cursor)
            right_end = self.lower(right, scope, cursor)
            return self.join(cursor, right_end) if token.text in {"&&", "||"} else right_end
        if kind == "record":
            for _, value in args[0]:
                cursor = self.lower(value, scope, cursor)
            return cursor
        if kind == "call":
            cursor, _ = self.lower_call(expr, scope, cursor)
            return None if self.annotation(expr) == NEVER else cursor
        if kind == "print":
            return self.lower_print(token, args, scope, cursor)
        raise AssertionError(kind)

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
                body_entry, origin = self.lower_address(condition, scope, restart)
                if origin is None:
                    body_entry = self.lower(condition, scope, body_entry)
                target.in_condition = False
                pattern, types = self.program.patterns[id(source)]
                annotation = self.annotation(condition)
                reads, moves, tests = self.pattern_accesses(pattern, origin, annotation)
                body_entry = self.read_pattern(body_entry, tests | reads, source.token)
                if uncovered([(pattern,)], (annotation,), self.program) is not None:
                    self.connect(body_entry, finish)
                body_entry = self.move_pattern(body_entry, moves, source.token)
                scope = Scope(scope)
                body_entry = self.declare(source, types, scope, body_entry)
            if kind == "for":
                self.connect(restart, finish)
                local = Local(name, mutable)
                scope = Scope(scope, {name.text: local})
                body_entry = self.emit(restart, "declare", Place(local), token)
            self.connect(self.lower(body, scope, body_entry), restart)
        finally:
            self.loops.pop()
        return finish

    def lower_match(self, expr, scope, cursor, *, result_type=None, extend_to=None):
        scrutinee, arms = expr.args
        cursor, origin = self.lower_address(scrutinee, scope, cursor)
        if origin is None:
            cursor = self.lower(scrutinee, scope, cursor)
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
            ready = self.declare(source, types, inner, tested)
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

    def declare(self, source, types, scope, cursor, *, initialized=True, loan=None):
        action = "declare" if initialized else "uninitialized"
        for name, (_, mutable) in types.items():
            local = Local(Token(name, source.token.line, source.token.column), mutable, loan)
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
            if self.program.is_copy(annotation):
                reads[place] = None
            else:
                moves[place] = annotation
        elif isinstance(pattern.tag, Constructor):
            ctor = pattern.tag
            variant = "::" in ctor.path
            if variant and len(self.program.layout(ctor.annotation)[1]) > 1:
                tests[place] = None
            for child, (field, kind) in zip(pattern.children, ctor.fields):
                projected = place if opaque or variant else place.child(field)
                self.collect_pattern(child, projected, reads, moves, tests, kind, opaque=opaque or variant)
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
    if isinstance(value, Address):
        return display(value.read(), debug)
    if isinstance(value, Value):
        ctor = value.constructor
        name = ctor.path.rsplit("::", 1)[-1]
        if ctor.style == "unit" or not ctor.fields:
            return name
        fields = [display(item, True) for item in value.fields]
        if ctor.style == "tuple":
            return f"{name}({', '.join(fields)})"
        entries = ", ".join(f"{field}: {item}" for (field, _), item in zip(ctor.fields, fields))
        return f"{name} {{ {entries} }}"
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


def handles_jump(expr, signal):
    label = signal.args[0]
    return label is None or (expr.label is not None and expr.label.text == label.text)


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

    def call(self, name, values, arguments=None):
        function = self.functions[name]
        bindings = {
            param.text: Binding(value, mutable)
            for (param, mutable, _), value in zip(function.params, values)
            if param.text != "_"
        }
        previous = self.type_arguments
        self.type_arguments = {p: substitute(t, previous) for p, t in (arguments or {}).items()}
        try:
            return self.eval(function.body, Scope(bindings=bindings))
        except Return as signal:
            return signal.args[0]
        finally:
            self.type_arguments = previous

    def default_value(self, annotation):
        if annotation in PRIMITIVES:
            return {"i32": 0, "bool": False, "&str": "", "()": None}[annotation]
        constructors = self.program.layout(annotation)[1]
        name = annotation.name if isinstance(annotation, AppliedType) else annotation
        path = self.program.default_variants.get(name)
        ctor = next(ctor for ctor in constructors if not path or ctor.path == path)
        return Value(ctor, tuple(self.default_value(kind) for _, kind in ctor.fields))

    def compare(self, left, right):
        if isinstance(left, Address):
            return self.compare(left.read(), right.read())
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
        annotation = substitute(self.program.expr_types[id(expr)], self.type_arguments)
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
        return Address(Binding(self.eval(expr, scope), True))

    def eval(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "literal":
            return args[1]
        if kind == "default":
            annotation = self.program.expr_types[id(expr)]
            return self.default_value(substitute(annotation, self.type_arguments))
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
                scope.bindings[source.token.text] = Binding(self.placeholder(annotation), source.args[0])
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
                inner = Scope(scope, {name.text: Binding(value, mutable)})
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
            if min_literal(expr):
                return -LIMIT
            value = self.eval(args[0], scope)
            if token.text == "-":
                return checked(-value, token)
            return not value if isinstance(value, bool) else ~value
        if kind == "assign":
            left, right = args
            value = self.eval(right, scope)
            address = self.address(left, scope)
            if token.text != "=":
                value = binary(token.text[0], address.read(), value, token)
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
            return binary(token.text, value, self.eval(right, scope), token)
        if kind == "call":
            values = tuple(self.eval(arg, scope) for arg in args)
            ctor = self.program.constructors.get(token.text)
            if ctor and ctor.style == "tuple":
                return Value(self.runtime_constructor(expr, ctor), values)
            _, arguments = self.program.call_types[id(expr)]
            return self.call(token.text, values, arguments)
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
    Ownership(program).check()
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
