"""A standard-library-only interpreter for a scalar subset of Rust.

Run: python3 rust_in_python.py program.rs
Demo: python3 rust_in_python.py --demo

Supported types include i32, usize, bool, string slices, arrays, structs, and enums.
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
    r'[0-9][0-9_]*(?:i32|usize)?|[A-Za-z_][A-Za-z_0-9]*|'
    r'\.\.=|\.\.|::|->|=>|==|!=|<=|>=|&&|\|\||[+*/%\-]=|[{}()[\]#$?.:,;=!+*/%&<>|\-]'
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
PRIMITIVES = {"i32", "usize", "bool", "&str", "()"}
LIMIT = 2**31
USIZE_LIMIT = 2**64
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


DELIMITERS = {"(": ")", "[": "]", "{": "}"}


@dataclass(frozen=True)
class TokenGroup:
    opening: Token
    items: tuple
    closing: Token


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
        else:
            yield tree


def tree_text(tree):
    return tree.opening.text if isinstance(tree, TokenGroup) else tree.text


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

    def expand(self, items, inherited=None, depth=0):
        if depth > 64:
            raise next(flatten_trees(items), Token("macro", 1, 1)).error("macro expansion recursion limit reached")
        macros = dict(inherited or {})
        result = []
        index = 0
        while index < len(items):
            item = items[index]
            if tree_text(item) == "macro_rules" and index + 3 < len(items):
                marker, name, body = items[index + 1:index + 4]
                if tree_text(marker) != "!" or not isinstance(name, Token) or not isinstance(body, TokenGroup):
                    raise item.error("invalid macro_rules definition")
                macros[name.text] = self.rules(body)
                index += 4
                if index < len(items) and tree_text(items[index]) == ";":
                    index += 1
                continue
            if (isinstance(item, Token) and item.text in macros and index + 2 < len(items)
                    and tree_text(items[index + 1]) == "!" and isinstance(items[index + 2], TokenGroup)):
                self.steps += 1
                if self.steps > 10_000:
                    raise item.error("macro expansion limit reached")
                expanded = self.invoke(item, macros[item.text], items[index + 2].items)
                index += 3
                result.extend(self.expand(expanded, macros, depth + 1))
                if (expanded and tree_text(expanded[0]) in {"#", "fn", "struct", "enum", "trait", "impl"}
                        and index < len(items) and tree_text(items[index]) == ";"):
                    index += 1
                continue
            if isinstance(item, TokenGroup):
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

    def invoke(self, token, rules, items):
        self.match_steps = 0
        for matcher, body in rules:
            for offset, captures in self.match(matcher, items, 0, {}, ()):
                if offset == len(items):
                    # Keep literal local declarations separate from caller tokens.
                    names = self.local_names(body)
                    renamed = {name: f"__macro_{self.steps}_{name}" for name in names}
                    return tuple(self.render(body, captures, (), renamed))
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

    @staticmethod
    def local_names(nodes):
        names = set()
        for index, node in enumerate(nodes):
            if isinstance(node, TokenGroup):
                names.update(MacroExpander.local_names(node.items))
            elif isinstance(node, MacroRepeat):
                names.update(MacroExpander.local_names(node.body))
            elif isinstance(node, Token) and node.text in {"let", "for"}:
                following = index + 1
                if following < len(nodes) and isinstance(nodes[following], Token) and nodes[following].text == "mut":
                    following += 1
                if following < len(nodes) and isinstance(nodes[following], Token):
                    name = nodes[following].text
                    if IDENT.fullmatch(name) and name not in RESERVED:
                        names.add(name)
        return names

    def render(self, nodes, captures, path, renamed):
        for index, node in enumerate(nodes):
            if isinstance(node, MacroCapture):
                for level in range(len(path), -1, -1):
                    key = (node.name.text, path[:level])
                    if key in captures:
                        fragment, items = captures[key]
                        if fragment in {"expr", "pat"}:
                            opening = replace(node.name, text="(")
                            closing = replace(node.name, text=")")
                            yield TokenGroup(opening, items, closing)
                        else:
                            yield from items
                        break
                else:
                    raise node.name.error(f"metavariable {node.name.text!r} has no value at this repetition depth")
            elif isinstance(node, TokenGroup):
                yield replace(node, items=tuple(self.render(node.items, captures, path, renamed)))
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
                    yield from self.render(node.body, captures, (*path, index_in_path), renamed)
            else:
                previous = nodes[index - 1] if index else None
                field = isinstance(previous, Token) and previous.text in {".", "::"}
                if node.text.startswith('"') and renamed:
                    template = re.sub(r"(?<!\{)\{([A-Za-z_][A-Za-z_0-9]*)(?=[:}])",
                                      lambda found: "{" + renamed.get(found[1], found[1]), node.text)
                    yield replace(node, text=template)
                else:
                    yield replace(node, text=renamed[node.text]) if node.text in renamed and not field else node


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
    value: int | bool
    kind: str | None = field(default=None, compare=False)
    boolean: bool = field(init=False)

    def __post_init__(self):
        object.__setattr__(self, "boolean", type(self.value) is bool)

    def __str__(self):
        return str(self.value).lower()


@dataclass(frozen=True)
class ArrayType:
    element: object
    length: object

    def __str__(self):
        return f"[{self.element}; {self.length}]"


@dataclass(frozen=True)
class AssociatedType:
    owner: object
    item: str
    trait: str | None = None

    def __str__(self):
        return f"{self.owner}::{self.item}"


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
    if isinstance(annotation, ReferenceType):
        return replace(annotation, inner=substitute(annotation.inner, arguments))
    if isinstance(annotation, AppliedType):
        return AppliedType(annotation.name, tuple(substitute(arg, arguments) for arg in annotation.arguments))
    if isinstance(annotation, ArrayType):
        return ArrayType(substitute(annotation.element, arguments), substitute(annotation.length, arguments))
    if isinstance(annotation, AssociatedType):
        return replace(annotation, owner=substitute(annotation.owner, arguments))
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
    traits: dict = field(default_factory=dict)
    impls: list = field(default_factory=list)
    call_functions: dict = field(default_factory=dict)
    dispatches: dict = field(default_factory=dict)

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
            annotation = replace(annotation, owner=owner)
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
                            resolved = substitute(impl.associated[annotation.item], arguments)
                            return self.normalize(resolved, (*seen, annotation))
            return annotation
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.normalize(annotation.inner, seen))
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.normalize(arg, seen) for arg in annotation.arguments))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.normalize(annotation.element, seen), annotation.length)
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
        if trait == "Sized":
            return True
        if isinstance(annotation, AssociatedType):
            normalized = self.normalize(annotation)
            if not isinstance(normalized, AssociatedType):
                return self.implements(normalized, trait, assumed, seen=seen)
            declaration = self.projection_trait(normalized)
            return bool(declaration and trait in self.trait_closure(declaration.associated[normalized.item]))
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
        if isinstance(annotation, ArrayType):
            if trait == "Display":
                return False
            if trait == "Default" and (not isinstance(annotation.length, ConstValue)
                                       or annotation.length.value > 32):
                return False
            if trait == "Default" and annotation.length.value == 0:
                return True
            return self.implements(annotation.element, trait, assumed)
        if annotation in PRIMITIVES:
            return trait != "Display" or annotation != "()"
        return trait in self.derives.get(annotation, {})

    def is_copy(self, annotation):
        return self.implements(annotation, "Copy")


class Parser:
    def __init__(self, source, *, tokens=None, fragment=False):
        self.tokens = iter(lex(source) if tokens is None else tokens)
        self.fragment = fragment
        self.lookahead = deque()
        self.self_type = None
        self.current_trait = None
        self.constants = {}
        self.current = next(self.tokens)
        self.type_parameters = {}
        self.trait_names = {name: name for name in TRAIT_MODULES if TRAIT_MODULES[name] != "fmt"}

    def advance(self):
        token = self.current
        if not token.text:
            raise token.error("unexpected end of input")
        self.current = self.lookahead.popleft() if self.lookahead else next(self.tokens)
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
        if token.text == "Self" and self.self_type is not None:
            if self.accept("::"):
                return AssociatedType(self.self_type, self.identifier().text, self.current_trait)
            return self.self_type
        if token.text == "<":
            owner = self.type_name(signature=signature)
            self.expect("as")
            trait = self.trait_path()
            self.expect(">")
            self.expect("::")
            return AssociatedType(owner, self.identifier().text, trait)
        if token.text in {"i32", "usize", "bool"}:
            return token.text
        if token.text == "[":
            element = self.type_name(static_string=static_string, signature=signature)
            self.expect(";")
            length = self.const_argument()
            self.expect("]")
            return ArrayType(element, length)
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
            if self.current.text == "::" and token.text in self.type_parameters:
                self.advance()
                return AssociatedType(self.type_parameters[token.text], self.identifier().text)
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
            name = self.identifier()
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
        self.where_clause(lifetimes)
        if receiver and isinstance(params[0][2], ReferenceType) and isinstance(result, ReferenceType) and not result.lifetime:
            return_inputs = (0,)
        else:
            return_inputs = self.return_inputs(name, params, result, lifetimes)
        body = None if declaration and self.accept(";") else self.block()
        function = Function(name, params, result, body, return_inputs, tuple(self.type_parameters.values()),
                            method_parameters=tuple(parameters.values()) if method else ())
        self.type_parameters = outer_parameters
        return function

    def constant_item(self):
        name = self.identifier()
        if name.text in self.constants:
            raise name.error(f"duplicate constant {name.text!r}")
        self.expect(":")
        annotation = self.type_name()
        if annotation not in {"i32", "usize", "bool"}:
            raise name.error("unsupported constant type")
        self.expect("=")
        value = self.const_expression(self.expression())
        if not isinstance(value, ConstValue) or (value.kind and value.kind != annotation):
            raise name.error(f"expected a {annotation} constant")
        if (annotation == "bool") != value.boolean:
            raise name.error(f"expected a {annotation} constant")
        if not value.boolean:
            checked(value.value, name, annotation)
        self.expect(";")
        self.constants[name.text] = ConstValue(value.value, annotation)

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
        offset = int(self.peek().startswith("'"))
        if self.peek(offset) == "mut":
            offset += 1
        return self.peek(offset) == "self"

    def self_parameter(self, lifetimes):
        borrowed = self.accept("&")
        lifetime = self.advance() if borrowed and self.current.text.startswith("'") else None
        if lifetime and lifetime.text not in lifetimes:
            raise lifetime.error("undeclared receiver lifetime")
        mutable = bool(self.accept("mut"))
        name = self.expect("self")
        annotation = self.self_type
        if borrowed:
            annotation = ReferenceType(annotation, mutable, lifetime.text if lifetime else None)
        return name, mutable and not borrowed, annotation

    def trait_item(self, program):
        name = self.identifier()
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
            self.accept("pub")
            if self.accept("type"):
                name = self.identifier()
                if not self.current_trait:
                    raise name.error("associated types require a trait")
                if name.text in associated:
                    raise name.error(f"duplicate associated type {name.text!r}")
                if declaration:
                    associated[name.text] = self.trait_bounds() if self.accept(":") else set()
                else:
                    self.expect("=")
                    associated[name.text] = self.type_name()
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
            methods[name.text] = function
            if function.body is not None:
                program.functions[internal.text] = function
        return methods, associated

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
            elif token.text == "const":
                name = self.identifier()
                if name.text in types:
                    raise name.error(f"duplicate generic parameter {name.text}")
                self.expect(":")
                kind = self.advance()
                if kind.text not in {"i32", "usize", "bool"}:
                    raise kind.error("unsupported const parameter type")
                types[name.text] = TypeParameter(name.text, const_type=kind.text)
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
        while self.current.text in {".", "["}:
            if opening := self.accept("["):
                index = self.expression()
                self.expect("]")
                left = Expr("index", opening, (left, index))
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
            self.expect("::")
            member = self.identifier()
            if self.current.text == "(":
                return Expr("call", member, self.arguments(), qualified=(self.self_type, self.current_trait))
            return Expr("name", member, qualified=(self.self_type, self.current_trait))
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
        if word == "[":
            if self.accept("]"):
                return Expr("array", token)
            first = self.expression()
            if self.accept(";"):
                count = self.const_argument()
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
        if word == "self" and (self.self_type is not None or self.fragment):
            return Expr("name", token)
        if IDENT.fullmatch(word) and word not in RESERVED:
            if self.accept("!"):
                opening = self.current.text
                if opening not in DELIMITERS:
                    raise self.current.error("expected a macro delimiter")
                if word != "println":
                    if not self.fragment:
                        raise token.error(f"unknown macro {word!r}")
                    self.skip_group()
                    return Expr("literal", token, ("i32", 0))
                return Expr("print", token, self.arguments(opening))
            path, type_arguments = self.expression_path(token)
            if path.text in INTEGER_CONSTANTS:
                return Expr("literal", path, ("i32", INTEGER_CONSTANTS[path.text]))
            if allow_record and self.current.text == "{":
                return Expr("record", path, (self.record_fields(self.expression, "name")[0],),
                            type_arguments=type_arguments)
            if self.current.text == "(":
                return Expr("call", path, self.arguments(), type_arguments=type_arguments)
            owner, separator, member = path.text.partition("::")
            if separator and owner in self.type_parameters:
                return Expr("name", replace(path, text=member), qualified=(self.type_parameters[owner], None))
            return Expr("name", path, type_arguments=type_arguments)
        raise token.error(f"unexpected token {word!r}")

    @staticmethod
    def literal(token, negative=False):
        word = token.text
        if word[0].isdigit():
            kind = "usize" if word.endswith("usize") else "i32"
            digits = word.removesuffix("usize").removesuffix("i32").replace("_", "").lstrip("0") or "0"
            if len(digits) > 20 or int(digits) >= USIZE_LIMIT:
                raise token.error("integer literal out of range")
            value = int(digits)
            return Expr("literal", token, (kind, -value if negative else value),
                        unsuffixed=not word.endswith(("i32", "usize")))
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
            word = self.current.text
            parameter = self.type_parameters.get(word)
            if self.accept("_"):
                arguments.append(None)
            elif (word in {"{", "-", "true", "false"} or word in self.constants or word[0].isdigit()
                  or (parameter and parameter.const_type)):
                arguments.append(self.const_argument())
            else:
                arguments.append(self.type_name())
            if not self.accept(","):
                break
        self.expect(">")
        return tuple(arguments)

    def const_argument(self):
        if self.accept("{"):
            value = self.const_expression(self.expression())
            self.expect("}")
            return value
        negative = self.accept("-")
        token = self.advance()
        if token.text in self.constants and not negative:
            return self.constants[token.text]
        parameter = self.type_parameters.get(token.text)
        if parameter and parameter.const_type and not negative:
            return parameter
        literal = self.literal(token, negative=bool(negative))
        if literal is None or literal.args[0] not in {"i32", "usize", "bool"}:
            raise token.error("expected a constant value or parameter")
        kind, value = literal.args
        return ConstValue(value, None if literal.unsuffixed else kind)

    def const_expression(self, expr):
        expr = ungroup(expr)
        if expr.kind == "const_value":
            return expr.args[0]
        if expr.kind == "literal" and expr.args[0] in {"i32", "usize", "bool"}:
            kind, value = expr.args
            return ConstValue(value, None if expr.unsuffixed else kind)
        if expr.kind == "unary" and expr.token.text == "-":
            value = self.const_expression(expr.args[0])
            if isinstance(value, ConstValue) and type(value.value) is int:
                return ConstValue(-value.value, value.kind)
        if expr.kind == "binary":
            left, right = (self.const_expression(arg) for arg in expr.args)
            word = expr.token.text
            if all(isinstance(arg, ConstValue) for arg in (left, right)):
                if left.boolean and right.boolean and word in {"&&", "||"}:
                    return ConstValue((left.value and right.value) if word == "&&"
                                      else (left.value or right.value), "bool")
                if word in COMPARE and left.boolean == right.boolean and (
                    not left.kind or not right.kind or left.kind == right.kind
                ):
                    return ConstValue(COMPARE[word](left.value, right.value), "bool")
                kind = left.kind or right.kind
                if (not left.boolean and not right.boolean and word in {*ARITHMETIC, "/", "%"}
                        and (not left.kind or not right.kind or left.kind == right.kind)):
                    return ConstValue(binary(word, left.value, right.value, expr.token, kind or "i32"), kind)
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
    while root.kind in {"field", "index"}:
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
            value = value.at(name)
        return value

    def write(self, value):
        if self.fields:
            value = self.binding.value.replace(self.fields, value)
        self.binding.value = value


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
            raise token.error(f"array index {index} out of bounds for length {len(self.items)}")
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
        self.array_integers = []
        self.parameters = {}
        self.ascriptions = []

    def check(self):
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
        for function in self.functions.values():
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
                if name.text in scope.bindings:
                    raise name.error(f"duplicate parameter {name.text!r}")
                scope.bindings[name.text] = Binding(annotation, mutable)
            self.result = function.result
            self.check_type(self.result, function.name, allow_never=True)
            result = self.type_of(function.body, scope, function.result)
            self.require_argument(result, function.result, function.body.token)
        self.finish_inference()

    def check_bounds(self, parameters, token):
        for parameter in parameters:
            for name in parameter.bounds:
                if name not in TRAIT_MODULES and name not in self.program.traits:
                    raise token.error(f"unknown trait {name!r}")

    def check_implementations(self):
        for trait in self.program.traits.values():
            self.check_bounds((trait.self_type,), trait.name)
            if trait.name.text in self.program.trait_closure(trait.bounds):
                raise trait.name.error("cyclic supertrait relationship")
            for bounds in trait.associated.values():
                for bound in bounds:
                    if bound not in TRAIT_MODULES and bound not in self.program.traits:
                        raise trait.name.error(f"unknown trait {bound!r}")
            for method in trait.methods.values():
                self.check_bounds(method.type_parameters, method.name)
                by_value = (method.result is trait.self_type
                            or any(kind is trait.self_type for _, _, kind in method.params))
                if method.body and by_value and "Sized" not in self.program.trait_closure(trait.self_type.bounds):
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
            if impl.trait is None and self.program.layout(impl.target) is None:
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
            for name, kind in impl.associated.items():
                self.check_type(kind, impl.token)
                if isinstance(self.resolve(kind), (ReferenceType, AssociatedType)):
                    raise impl.token.error("associated types must resolve to a non-reference type")
                for bound in sorted(trait.associated[name]):
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
                if (required_types != actual_types
                        or self.resolve(method.result) != self.resolve(substitute(required.result, bindings))
                        or method.return_inputs != required.return_inputs
                        or method.associated_const != required.associated_const
                        or self.has_receiver(method) != self.has_receiver(required)):
                    raise method.name.error(f"method {name!r} has an incompatible trait signature")

    def resolve(self, annotation):
        if isinstance(annotation, TypeVariable) and annotation in self.substitutions:
            result = self.resolve(self.substitutions[annotation])
            self.substitutions[annotation] = result
            return result
        if isinstance(annotation, ReferenceType):
            return replace(annotation, inner=self.resolve(annotation.inner))
        if isinstance(annotation, AppliedType):
            return AppliedType(annotation.name, tuple(self.resolve(arg) for arg in annotation.arguments))
        if isinstance(annotation, ArrayType):
            return ArrayType(self.resolve(annotation.element), self.resolve(annotation.length))
        if isinstance(annotation, AssociatedType):
            projected = replace(annotation, owner=self.resolve(annotation.owner))
            return self.program.normalize(projected)
        return annotation

    def occurs(self, variable, annotation):
        annotation = self.resolve(annotation)
        if isinstance(annotation, ReferenceType):
            return self.occurs(variable, annotation.inner)
        if isinstance(annotation, AppliedType):
            return any(self.occurs(variable, arg) for arg in annotation.arguments)
        if isinstance(annotation, ArrayType):
            return self.occurs(variable, annotation.element) or self.occurs(variable, annotation.length)
        if isinstance(annotation, AssociatedType):
            return self.occurs(variable, annotation.owner)
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
        if isinstance(actual, AppliedType) and isinstance(expected, AppliedType):
            if actual.name == expected.name and len(actual.arguments) == len(expected.arguments):
                for left, right in zip(actual.arguments, expected.arguments):
                    self.require(left, right, token)
                return
        if isinstance(actual, ArrayType) and isinstance(expected, ArrayType):
            self.require(actual.element, expected.element, token)
            self.require(actual.length, expected.length, token)
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
        elif isinstance(value, ConstValue) or getattr(value, "const_type", None):
            raise token.error("expected a type argument, found a constant")

    def check_const(self, value, kind, token):
        value = self.resolve(value)
        if isinstance(value, (TypeParameter, TypeVariable)) and value.const_type == kind:
            return
        if not isinstance(value, ConstValue) or (value.kind and value.kind != kind):
            raise token.error(f"expected a {kind} constant")
        if kind == "bool":
            if not value.boolean:
                raise token.error("expected a bool constant")
        elif value.boolean:
            raise token.error(f"expected a {kind} constant")
        else:
            checked(value.value, token, kind)

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
            if resolved not in {"i32", "usize"}:
                raise token.error(f"expected an integer, found {resolved}")
            checked(value, token, resolved)
        for operand, result, token in self.unary_results:
            operand = self.resolve(operand)
            if not isinstance(operand, TypeVariable):
                if operand not in {"i32", "usize", "bool"}:
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
        for annotation, expected, token in self.numeric_operands:
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
            if annotation.const_type:
                raise token.error("expected a type, found a const parameter")
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
            first = self.type_of(start, scope)
            last = self.type_of(end, scope, first if first == "usize" else None)
            integer = "usize" if "usize" in {first, last} else "i32"
            if integer == "usize" and start.unsuffixed:
                first = self.type_of(start, scope, integer)
            bounds = (first, last)
            for bound in bounds:
                self.require(bound, integer, token)
            if NEVER in bounds:
                test = NEVER
            scope = Scope(scope, {name.text: Binding(integer, mutable)})
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
        if id(expr) in self.program.call_functions or expr.kind == "length":
            cached = self.resolve(self.program.expr_types[id(expr)])
            if expected is not None:
                self.require_argument(cached, expected, expr.token)
            return cached
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
            hint = self.resolve(expected)
            if expr.unsuffixed and hint == "usize":
                annotation = hint
            elif expr.unsuffixed and isinstance(hint, TypeVariable) and hint.integer_literal:
                self.array_integers.append((hint, value, token))
                return hint
            if annotation in {"i32", "usize"}:
                checked(value, token, annotation)
            return annotation
        if kind == "const_value":
            return args[0].const_type
        if kind in {"array", "array_repeat"}:
            return self.array_type(expr, scope, expected)
        if kind == "index":
            base, index = args
            annotation = self.type_of(base, scope)
            if isinstance(annotation, ReferenceType):
                base = Expr("deref", base.token, (base, False))
                expr.args = (base, index)
                annotation = self.type_of(base, scope)
            if not isinstance(annotation, ArrayType):
                raise token.error(f"cannot index {annotation}")
            actual = self.type_of(index, scope, "usize")
            self.require(actual, "usize", index.token)
            return NEVER if actual == NEVER else annotation.element
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
                return self.constant_type(expr, scope, expected)
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
            if token.text == "!" and actual == "usize":
                return "usize"
            if token.text == "!" and isinstance(actual, TypeVariable):
                result = TypeVariable("operator result", token)
                self.variables.append(result)
                self.unary_results.append((actual, result, token))
                return result
            self.numeric_operands.append((actual, "i32", token))
            return NEVER if actual == NEVER else "i32"
        if kind == "assign":
            left, right = args
            annotation = self.type_of(left, scope)
            self.require_place(left, write=True)
            if isinstance(annotation, ReferenceType):
                raise token.error("reassigning reference bindings is unsupported")
            hint = annotation if token.text == "=" or right.unsuffixed else None
            actual = self.type_of(right, scope, hint)
            if token.text == "=":
                self.require(actual, annotation, token)
            else:
                integer = "usize" if annotation == "usize" or actual == "usize" else "i32"
                self.numeric_operands.extend(((annotation, integer, token), (actual, integer, token)))
            return NEVER if NEVER in {annotation, actual} else "()"
        if kind == "binary":
            hint = "bool" if token.text in {"&&", "||"} else None
            left = self.type_of(args[0], scope, hint)
            second_hint = left if token.text in COMPARE and left != NEVER else hint
            if args[1].unsuffixed and left == "usize":
                second_hint = "usize"
            right = self.type_of(args[1], scope, second_hint)
            if args[0].unsuffixed and right == "usize":
                left = self.type_of(args[0], scope, "usize")
            return self.binary_type(token, left, right)
        if kind == "call":
            return self.call_type(expr, scope, expected)
        if kind == "print":
            if not args:
                return "()"
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

    def array_type(self, expr, scope, expected):
        kind, token, args = expr.kind, expr.token, expr.args
        expected = self.resolve(expected)
        element = expected.element if isinstance(expected, ArrayType) else None
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
        if expected is not None:
            self.require(result, expected, token)
        return self.resolve(result)

    def call_type(self, expr, scope, expected):
        token, args = expr.token, expr.args
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
        self.program.call_functions[id(expr)] = function
        params = tuple(substitute(self.resolve(kind), arguments) for _, _, kind in function.params)
        result = substitute(self.resolve(function.result), arguments)
        return self.check_call(expr, scope, expected, params, result, arguments, first)

    def check_call(self, expr, scope, expected, params, result, arguments, first=None):
        token, args = expr.token, expr.args
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
            actual_types.append(actual)
        if expected is not None and defer_projection:
            self.require_argument(result, expected, token)
        self.program.call_types[id(expr)] = (params, arguments)
        return NEVER if NEVER in actual_types else self.resolve(result)

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
        if isinstance(base, ArrayType) and name == "len" and not rest:
            if isinstance(actual, ReferenceType):
                original = Expr("deref", expr.token, (original, False))
                self.type_of(original, scope)
            expr.kind, expr.args = "length", (original,)
            return "usize"
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
                    return self.receiver_call(expr, scope, expected, method, arguments, actual)
        candidates = [trait for trait in self.program.traits.values()
                      if name in trait.methods and self.has_receiver(trait.methods[name])
                      and self.implements(base, trait.name.text)]
        if len(candidates) > 1:
            raise expr.token.error(f"multiple traits provide method {name!r}")
        if candidates:
            trait = candidates[0]
            self.program.dispatches[id(expr)] = (trait.name.text, base, name)
            return self.receiver_call(expr, scope, expected, trait.methods[name],
                                      {trait.self_type: base}, actual)
        if name == "clone" and not rest:
            expr.kind = "clone"
            return self.check_expr(expr, scope, expected)
        raise expr.token.error(f"no method {name!r} for {base}")

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
        self.obligation(owner, trait_name, expr.token)
        self.program.dispatches[id(expr)] = (trait_name, owner, name)
        return self.function_call(expr, scope, expected, trait.methods[name], {trait.self_type: owner}, first)

    def call_on_type(self, expr, scope, expected, owner, name):
        for impl in self.program.impls:
            if impl.trait is not None or name not in impl.methods:
                continue
            args = {}
            if match_types(impl.target, owner, impl.parameters, args):
                for parameter, argument in args.items():
                    for trait in parameter.bounds:
                        self.obligation(argument, trait, expr.token)
                return self.function_call(expr, scope, expected, impl.methods[name], args)
        traits = [trait.name.text for trait in self.program.traits.values()
                  if name in trait.methods and self.implements(owner, trait.name.text)]
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
        integer = "usize" if "usize" in {left, right} else "i32"
        self.numeric_operands.extend(((left, integer, token), (right, integer, token)))
        return NEVER if NEVER in {left, right} else integer


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
        if expr.kind == "index":
            base, index = expr.args
            cursor, place = self.lower_address(base, scope, cursor)
            if place is not None:
                return self.lower(index, scope, cursor), replace(place, indexed=True)
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
        function = self.program.call_functions.get(id(expr))
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
        if place.indexed:
            return "indexed_move"
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
        if kind in {"literal", "name", "default", "const_value"}:
            return cursor
        if kind == "borrow":
            cursor, _ = self.lower_reference(expr, scope, cursor)
            return cursor
        if kind in {"group", "field", "unary", "deref"}:
            return self.lower(args[0], scope, cursor)
        if kind == "clone":
            return self.lower(args[0], scope, cursor, consume=False)
        if kind == "length":
            return self.lower(args[0], scope, cursor, consume=False)
        if kind in {"array", "index", "array_repeat"}:
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
            if target_place.indexed:
                cursor = self.emit(cursor, "read", target_place, token)
            return self.emit(cursor, action, target_place, token)
        if kind == "binary":
            if token.text in COMPARE and self.annotation(args[0]) not in {"i32", "usize", "bool"}:
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


def checked(value, token, integer="i32"):
    lower, upper = (0, USIZE_LIMIT) if integer == "usize" else (-LIMIT, LIMIT)
    if not lower <= value < upper:
        raise token.error(f"{integer} arithmetic overflow")
    return value


def binary(word, left, right, token, integer="i32"):
    if word in ARITHMETIC:
        return checked(ARITHMETIC[word](left, right), token, integer)
    if right == 0:
        raise token.error("division by zero")
    quotient = abs(left) // abs(right)
    if (left < 0) != (right < 0):
        quotient = -quotient
    checked(quotient, token, integer)
    return quotient if word == "/" else left - quotient * right


def display(value, debug):
    if isinstance(value, Address):
        return display(value.read(), debug)
    if isinstance(value, ArrayValue):
        return "[" + ", ".join(display(item, True) for item in value.items) + "]"
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
        self.type_arguments = {p: self.program.normalize(substitute(t, previous))
                               for p, t in (arguments or {}).items()}
        try:
            return self.eval(function.body, Scope(bindings=bindings))
        except Return as signal:
            return signal.args[0]
        finally:
            self.type_arguments = previous

    def default_value(self, annotation):
        if annotation in PRIMITIVES:
            return {"i32": 0, "usize": 0, "bool": False, "&str": "", "()": None}[annotation]
        if isinstance(annotation, ArrayType):
            return ArrayValue(tuple(self.default_value(annotation.element)
                                    for _ in range(annotation.length.value)))
        constructors = self.program.layout(annotation)[1]
        name = annotation.name if isinstance(annotation, AppliedType) else annotation
        path = self.program.default_variants.get(name)
        ctor = next(ctor for ctor in constructors if not path or ctor.path == path)
        return Value(ctor, tuple(self.default_value(kind) for _, kind in ctor.fields))

    def compare(self, left, right):
        if isinstance(left, Address):
            return self.compare(left.read(), right.read())
        if isinstance(left, ArrayValue):
            for a, b in zip(left.items, right.items):
                result = self.compare(a, b)
                if result:
                    return result
            return len(left.items) - len(right.items)
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
        return Address(Binding(self.eval(expr, scope), True))

    def eval(self, expr, scope):
        kind, token, args = expr.kind, expr.token, expr.args
        if kind == "literal":
            return args[1]
        if kind == "const_value":
            return substitute(args[0], self.type_arguments).value
        if kind == "array":
            return ArrayValue(tuple(self.eval(item, scope) for item in args))
        if kind == "array_repeat":
            value, length = args
            item = self.eval(value, scope)
            count = substitute(length, self.type_arguments).value
            if count > 1_000_000:
                raise token.error("array exceeds interpreter allocation limit")
            return ArrayValue((item,) * count)
        if kind == "index":
            base, index = args
            value = self.eval(base, scope)
            offset = self.eval(index, scope)
            return value.at(replace(token, text=str(offset)))
        if kind == "length":
            return len(self.eval(args[0], scope).items)
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
            if self.program.expr_types[id(expr)] == "usize":
                return (USIZE_LIMIT - 1) ^ value
            return not value if isinstance(value, bool) else ~value
        if kind == "assign":
            left, right = args
            value = self.eval(right, scope)
            address = self.address(left, scope)
            if token.text != "=":
                value = binary(token.text[0], address.read(), value, token, self.program.expr_types[id(left)])
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
            return binary(token.text, value, self.eval(right, scope), token, self.program.expr_types[id(expr)])
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
                impl, specialized = self.program.implementation(receiver, trait)
                if name in impl.methods:
                    prototype = function
                    function = impl.methods[name]
                    arguments = specialized | {actual: arguments[declared] for declared, actual in
                                               zip(prototype.method_parameters, function.method_parameters)}
                else:
                    arguments = arguments | {self.program.traits[trait].self_type: receiver}
            return self.call(function.name.text, values, arguments)
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
    trees, end = token_trees(lex(source))
    expanded = MacroExpander().expand(trees)
    program = Parser("", tokens=(*flatten_trees(expanded), end)).parse()
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
