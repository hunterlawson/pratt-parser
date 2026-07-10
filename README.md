# Rust Pratt Parser

Parses strings into an expression tree using the [Pratt algorithm](https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing) for operator precidence parsing. The following syntax and features are supported:

- Variables: `X, Y, num_1, abc`
- Functions: `abs(x)` `complex_multiply(a, b)`
- Operators with custom precidence: `+ - * / ^`
- Custom error types that make it clear if it is a lexing or parsing error
- Easily walkable `Expr` tree types that supports unary and binary operators and functions with variable arguments

## Examples

### Parsing

The output of the following examples is a infix notation representation of the tree output from the parser:

```rust
let expr = "1 + 2 * 3";
let parsed_expr = parse(expr).unwrap().infix_notation();
// "(1 + (2 * 3))"

let expr = "-10^(-15 * 4.23) + -Z * 4";
let parsed_expr = parse(expr).unwrap().infix_notation();
// "(-(10 ^ (-15 * 4.23)) + (-Z * 4))"
```

### Errors

Custom `ParserError` type reveals what exactly went wrong when parsing the expression string:

```rust
let err = parse("(a + b").err().expect("missing ')'")
println("{err}");
// Parsing error: Expected: `)`, got: `Eof`

let err = parse("1 + 3.4.5 + 10").err().expect("malformed float")
println("{err}");
// Lexer error: Malformed number at pos 4: `3.4.5`
```
