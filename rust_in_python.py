"""A standard-library-only interpreter for a scalar subset of Rust.

Run: python3 rust_in_python.py program.rs
Demo: python3 rust_in_python.py --demo

Supported types: i32, bool, &str (string literals), and ().
Supported syntax: functions, initialized let bindings, mut, blocks, if/else,
while, loop, for over integer ranges, return, break, continue, scalar operators,
and println! with {}, {:?}, or named captures. Integers use checked i32 arithmetic.
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
    r'\.\.=|\.\.|->|==|!=|<=|>=|&&|\|\||[+*/%\-]=|[{}():,;=!+*/%&<>\-]'
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
BLOCK_EXPRESSIONS = {"block", "if", "while", "loop", "for"}
LIMIT = 2**31
NEVER = "!"


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

    def type_name(self, *, result=False):
        token = self.advance()
        if token.text in {"i32", "bool"}:
            return token.text
        if token.text == "&":
            lifetime = self.accept("'static")
            self.expect("str")
            if result and not lifetime:
                raise token.error("use &'static str to return a string literal")
            return "&str"
        if token.text == "(":
            self.expect(")")
            return "()"
        raise token.error(f"unsupported type {token.text!r}")

    def parse(self):
        functions = {}
        while self.current.text:
            self.accept("pub")
            self.expect("fn")
            name = self.identifier()
            if name.text in functions:
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
            result = self.type_name(result=True) if self.accept("->") else "()"
            functions[name.text] = Function(name, params, result, self.block())
        return functions

    def block(self):
        token = self.expect("{")
        statements = []
        tail = None
        while self.current.text != "}":
            if self.accept(";"):
                continue
            let = self.accept("let")
            if let:
                mutable = bool(self.accept("mut"))
                name = self.advance() if self.current.text == "_" else self.identifier()
                annotation = self.type_name() if self.accept(":") else None
                self.expect("=")
                value = self.expression()
                self.expect(";")
                statements.append(Expr("let", name, (mutable, annotation, value)))
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

    def expression(self, minimum=1):
        left = self.prefix()
        compared = False
        while PRECEDENCE.get(self.current.text, 0) >= minimum:
            token = self.advance()
            precedence = PRECEDENCE[token.text]
            if precedence == 4:
                if compared:
                    raise token.error("comparison operators cannot be chained")
                compared = True
            right = self.expression(precedence if precedence == 1 else precedence + 1)
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

    def prefix(self):
        if self.current.text == "{":
            return self.block()
        token = self.advance()
        word = token.text
        if word == "if":
            condition = self.expression()
            yes = self.block()
            no = None
            if self.accept("else"):
                no = self.prefix() if self.current.text == "if" else self.block()
            return Expr("if", token, (condition, yes, no))
        if word == "while":
            return Expr("while", token, (self.expression(), self.block()))
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
            end = self.expression()
            return Expr("for", token, (name, mutable, start, end, inclusive, self.block()))
        if word in {"return", "break", "continue"}:
            if word == "return" and self.current.text not in {";", "}", ")", ","}:
                return Expr(word, token, (self.expression(),))
            return Expr(word, token)
        if word in {"-", "!"}:
            return Expr("unary", token, (self.expression(7),))
        if word == "(":
            if self.accept(")"):
                return Expr("literal", token, ("()", None))
            value = self.expression()
            self.expect(")")
            return Expr("group", token, (value,))
        if word in {"true", "false"}:
            return Expr("literal", token, ("bool", word == "true"))
        if word[0].isdigit():
            digits = word.removesuffix("i32").replace("_", "").lstrip("0") or "0"
            if len(digits) > 10 or int(digits) > LIMIT:
                raise token.error("integer literal out of range for i32")
            return Expr("literal", token, ("i32", int(digits)))
        if word.startswith('"'):
            return Expr("literal", token, ("&str", decode_string(token)))
        if IDENT.fullmatch(word) and word not in RESERVED:
            if self.accept("!"):
                if word != "println":
                    raise token.error(f"unsupported macro {word!r}")
                return Expr("print", token, self.arguments())
            if self.current.text == "(":
                return Expr("call", token, self.arguments())
            return Expr("name", token)
        raise token.error(f"unexpected token {word!r}")


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
    while expr.kind == "group":
        expr = expr.args[0]
    if expr.kind != "name":
        raise expr.token.error("invalid assignment target")
    binding = scope.find(expr.token)
    if not binding.mutable:
        raise expr.token.error(f"cannot assign to immutable binding {expr.token.text!r}")
    return binding


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
    def __init__(self, functions):
        self.functions = functions
        self.result = "()"
        self.loops = []

    def check(self):
        main = self.functions.get("main")
        if main is None:
            raise RustError("missing fn main()")
        if main.params or main.result != "()":
            raise main.name.error("main must take no arguments and return ()")
        for function in self.functions.values():
            scope = Scope()
            for name, mutable, annotation in function.params:
                if name.text in scope.bindings:
                    raise name.error(f"duplicate parameter {name.text!r}")
                scope.bindings[name.text] = Binding(annotation, mutable)
            self.result = function.result
            result = self.type_of(function.body, scope)
            require(result, function.result, function.body.token)

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
            return scope.find(token).value
        if kind == "block":
            statements, tail = args
            inner = Scope(scope)
            diverges = False
            for statement in statements:
                diverges |= self.type_of(statement, inner) == NEVER
            result = self.type_of(tail, inner) if tail else "()"
            return NEVER if diverges else result
        if kind == "let":
            mutable, annotation, value = args
            actual = self.type_of(value, scope)
            if annotation:
                require(actual, annotation, token)
            if token.text != "_":
                scope.bindings[token.text] = Binding(annotation or actual, mutable)
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
            binding = target(left, scope)
            if token.text != "=":
                require(binding.value, "i32", token)
            actual = self.type_of(right, scope)
            require(actual, binding.value, token)
            return NEVER if actual == NEVER else "()"
        if kind == "binary":
            return self.binary_type(token, *(self.type_of(arg, scope) for arg in args))
        if kind == "call":
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
    def __init__(self, functions, stdout):
        self.functions = functions
        self.stdout = stdout

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
            return scope.find(token).value
        if kind == "block":
            statements, tail = args
            inner = Scope(scope)
            for statement in statements:
                self.eval(statement, inner)
            return self.eval(tail, inner) if tail else None
        if kind == "let":
            mutable, _, value = args
            result = self.eval(value, scope)
            if token.text != "_":
                scope.bindings[token.text] = Binding(result, mutable)
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
            binding = target(left, scope)
            binding.value = value if token.text == "=" else binary(
                token.text[0], binding.value, value, token
            )
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
            return self.call(token.text, [self.eval(arg, scope) for arg in args])
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
    functions = Parser(source).parse()
    Checker(functions).check()
    Interpreter(functions, sys.stdout if stdout is None else stdout).call("main", [])


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
