/**
 * @file Tree-sitter grammar for Zamak, used by the Zed extension.
 *
 * Blocks are delimited by indentation, as in Python. The external scanner in
 * `src/scanner.c` turns line breaks and indentation into `_newline`, `_indent`,
 * and `_dedent` tokens. Inside `( )`, `[ ]`, and `{ }` the grammar never asks for
 * those tokens, so line breaks there are ordinary whitespace, as in the compiler.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  catch: 1,
  lambda: 2,
  conditional: 3,
  or: 10,
  and: 11,
  not: 12,
  compare: 13,
  range: 14,
  bitor: 15,
  bitxor: 16,
  bitand: 17,
  shift: 18,
  plus: 19,
  times: 20,
  unary: 21,
  power: 22,
  postfix: 30,
  // Types: `A!B` binds tighter than `fn(...) -> R`, so `fn(&string) -> i64!string`
  // returns `i64!string`.
  fn_type: 1,
  error_type: 2,
  reference_type: 3,
};

const commaSep1 = (rule) => seq(rule, repeat(seq(',', rule)), optional(','));
const commaSep = (rule) => optional(commaSep1(rule));

module.exports = grammar({
  name: 'zamak',

  word: ($) => $.identifier,

  externals: ($) => [$._newline, $._indent, $._dedent, $._error_sentinel],

  extras: ($) => [$.comment, /[\s\f\uFEFF\u2060\u200B]/],

  supertypes: ($) => [$._expression, $._type, $._pattern],

  rules: {
    source_file: ($) => repeat($._statement),

    _statement: ($) => choice($._simple_statement, $._compound_statement),

    _simple_statement: ($) =>
      seq(
        choice(
          $.use_declaration,
          $.const_declaration,
          $.extern_function,
          $.extern_type,
          $.let_statement,
          $.assignment,
          $.augmented_assignment,
          $.return_statement,
          $.fail_statement,
          $.assert_statement,
          $.break_statement,
          $.continue_statement,
          $.pass_statement,
          $.expression_statement,
        ),
        $._newline,
      ),

    _compound_statement: ($) =>
      choice(
        $.function_definition,
        $.decorated_definition,
        $.struct_definition,
        $.enum_definition,
        $.interface_definition,
        $.if_statement,
        $.while_statement,
        $.for_statement,
      ),

    // After a block header's `:`: one statement on the same line, or an indented block.
    _suite: ($) => choice(alias($._simple_statement, $.block), seq($._indent, $.block)),

    block: ($) => seq(repeat($._statement), $._dedent),

    // ---------------------------------------------------------------- declarations

    use_declaration: ($) =>
      seq(optional($.visibility), 'use', field('path', $.module_path)),

    module_path: ($) => seq($.identifier, repeat(seq('/', $.identifier))),

    visibility: (_) => 'pub',

    const_declaration: ($) =>
      seq(
        optional($.visibility),
        'const',
        field('name', $.identifier),
        optional(seq(':', field('type', $._type))),
        '=',
        field('value', $._expression),
      ),

    extern_function: ($) =>
      seq(
        optional($.visibility),
        'extern',
        'fn',
        field('name', $.identifier),
        field('parameters', $.parameters),
        optional(seq('->', field('return_type', $._type))),
      ),

    extern_type: ($) =>
      seq(
        optional($.visibility),
        'extern',
        'type',
        field('name', $.identifier),
        ':',
        field('c_type', $.string),
        repeat(seq(',', $.extern_option)),
      ),

    extern_option: ($) => seq(field('name', $.identifier), '=', field('value', $._expression)),

    decorator: ($) => seq('@', field('name', $.identifier), $._newline),

    decorated_definition: ($) =>
      seq(repeat1($.decorator), field('definition', $.function_definition)),

    function_definition: ($) =>
      seq(
        optional($.visibility),
        'fn',
        field('name', $.identifier),
        optional(field('type_parameters', $.type_parameters)),
        field('parameters', $.parameters),
        optional(seq('->', field('return_type', $._type))),
        ':',
        field('body', $._suite),
      ),

    // A method in an interface: no body.
    function_signature: ($) =>
      seq(
        optional($.visibility),
        'fn',
        field('name', $.identifier),
        optional(field('type_parameters', $.type_parameters)),
        field('parameters', $.parameters),
        optional(seq('->', field('return_type', $._type))),
        $._newline,
      ),

    type_parameters: ($) => seq('[', commaSep1($.type_parameter), ']'),

    type_parameter: ($) =>
      seq(field('name', $.identifier), optional(seq(':', field('bound', $._type)))),

    parameters: ($) => seq('(', commaSep(choice($.self_parameter, $.parameter)), ')'),

    self_parameter: ($) => seq(optional(seq('&', optional('mut'))), $.self),

    parameter: ($) =>
      seq(
        field('name', $.identifier),
        optional(seq(':', field('type', $._type))),
        optional(seq('=', field('default', $._expression))),
      ),

    struct_definition: ($) =>
      seq(
        optional($.visibility),
        'struct',
        field('name', $.identifier),
        optional(field('type_parameters', $.type_parameters)),
        ':',
        field('body', choice(seq($.pass_statement, $._newline), $.struct_body)),
      ),

    struct_body: ($) =>
      seq(
        $._indent,
        repeat(
          choice(
            $.field_declaration,
            $.function_definition,
            $.decorated_definition,
            seq($.pass_statement, $._newline),
          ),
        ),
        $._dedent,
      ),

    field_declaration: ($) =>
      seq(
        optional($.visibility),
        field('name', $.identifier),
        ':',
        field('type', $._type),
        optional(seq('=', field('default', $._expression))),
        $._newline,
      ),

    enum_definition: ($) =>
      seq(
        optional($.visibility),
        'enum',
        field('name', $.identifier),
        optional(field('type_parameters', $.type_parameters)),
        ':',
        field('body', $.enum_body),
      ),

    enum_body: ($) =>
      seq(
        $._indent,
        repeat(
          choice(
            $.enum_variant,
            $.function_definition,
            $.decorated_definition,
            seq($.pass_statement, $._newline),
          ),
        ),
        $._dedent,
      ),

    enum_variant: ($) =>
      seq(
        field('name', $.identifier),
        optional(seq('(', commaSep(choice($.variant_field, $._type)), ')')),
        $._newline,
      ),

    variant_field: ($) => seq(field('name', $.identifier), ':', field('type', $._type)),

    interface_definition: ($) =>
      seq(
        optional($.visibility),
        'interface',
        field('name', $.identifier),
        optional(field('type_parameters', $.type_parameters)),
        ':',
        field('body', $.interface_body),
      ),

    interface_body: ($) =>
      seq(
        $._indent,
        repeat(choice($.function_signature, seq($.pass_statement, $._newline))),
        $._dedent,
      ),

    // ------------------------------------------------------------------ statements

    let_statement: ($) =>
      seq(
        'let',
        optional($.mutable),
        field('pattern', choice($.identifier, $.tuple_pattern)),
        optional(seq(':', field('type', $._type))),
        optional(seq('=', field('value', $._expression))),
      ),

    mutable: (_) => 'mut',

    assignment: ($) => seq(field('left', $._expression), '=', field('right', $._expression)),

    augmented_assignment: ($) =>
      seq(
        field('left', $._expression),
        field(
          'operator',
          choice('+=', '-=', '*=', '/=', '//=', '%=', '**=', '&=', '|=', '^=', '<<=', '>>='),
        ),
        field('right', $._expression),
      ),

    return_statement: ($) => prec.right(seq('return', optional($._expression))),

    fail_statement: ($) => seq('fail', $._expression),

    assert_statement: ($) =>
      seq('assert', field('condition', $._expression), optional(seq(',', field('message', $._expression)))),

    break_statement: (_) => 'break',
    continue_statement: (_) => 'continue',
    pass_statement: (_) => 'pass',

    expression_statement: ($) => $._expression,

    if_statement: ($) =>
      seq(
        'if',
        field('condition', $._expression),
        ':',
        field('consequence', $._suite),
        repeat(field('alternative', $.elif_clause)),
        optional(field('alternative', $.else_clause)),
      ),

    elif_clause: ($) =>
      seq('elif', field('condition', $._expression), ':', field('consequence', $._suite)),

    else_clause: ($) => seq('else', ':', field('body', $._suite)),

    while_statement: ($) =>
      seq('while', field('condition', $._expression), ':', field('body', $._suite)),

    for_statement: ($) =>
      seq(
        'for',
        field('left', $.loop_variables),
        'in',
        field('right', $._expression),
        ':',
        field('body', $._suite),
      ),

    loop_variables: ($) => seq(choice($.identifier, $.tuple_pattern), repeat(seq(',', choice($.identifier, $.tuple_pattern)))),

    // ----------------------------------------------------------------------- types

    _type: ($) =>
      choice(
        $.type_identifier,
        $.scoped_type,
        $.generic_type,
        $.list_type,
        $.map_type,
        $.set_type,
        $.tuple_type,
        $.reference_type,
        $.function_type,
        $.error_type,
      ),

    type_identifier: ($) => $.identifier,

    scoped_type: ($) => seq(field('module', $.identifier), '.', field('name', $.type_identifier)),

    generic_type: ($) =>
      seq(choice($.type_identifier, $.scoped_type), '[', commaSep1($._type), ']'),

    list_type: ($) => seq('[', $._type, optional(seq(';', $.integer)), ']'),

    map_type: ($) => seq('{', field('key', $._type), ':', field('value', $._type), '}'),

    set_type: ($) => seq('{', $._type, '}'),

    tuple_type: ($) => seq('(', commaSep($._type), ')'),

    reference_type: ($) =>
      prec(PREC.reference_type, seq('&', optional($.mutable), $._type)),

    function_type: ($) =>
      prec.right(
        PREC.fn_type,
        seq(
          optional('kept'),
          'fn',
          '(',
          commaSep($._type),
          ')',
          optional(seq('->', field('return_type', $._type))),
        ),
      ),

    error_type: ($) =>
      choice(
        prec.left(PREC.error_type, seq(field('value', $._type), '!', field('error', $._type))),
        prec.left(PREC.error_type, seq('!', field('error', $._type))),
      ),

    // -------------------------------------------------------------------- patterns

    _pattern: ($) =>
      choice(
        $.identifier,
        $.variant_pattern,
        $.tuple_pattern,
        $.integer,
        $.negative_integer,
        $.float,
        $.string,
        $.raw_string,
        $.character,
        $.true,
        $.false,
      ),

    negative_integer: ($) => seq('-', choice($.integer, $.float)),

    // `Circle(r)`, `Shape.Circle(r)`, `Option.None`. A bare name such as `None` or `r` is an identifier.
    variant_pattern: ($) =>
      choice(
        seq(
          field('variant', choice($.identifier, $.scoped_identifier)),
          '(',
          commaSep($._pattern),
          ')',
        ),
        field('variant', $.scoped_identifier),
      ),

    scoped_identifier: ($) => seq($.identifier, repeat1(seq('.', $.identifier))),

    tuple_pattern: ($) => seq('(', commaSep($._pattern), ')'),

    or_pattern: ($) => seq($._pattern, repeat1(seq('|', $._pattern))),

    match_arm: ($) =>
      seq(field('pattern', choice($._pattern, $.or_pattern)), ':', field('body', $._suite)),

    // ----------------------------------------------------------------- expressions

    _expression: ($) =>
      choice(
        $.identifier,
        $.self,
        $.integer,
        $.float,
        $.string,
        $.raw_string,
        $.bytes,
        $.character,
        $.true,
        $.false,
        $.null,
        $.ellipsis,
        $.list,
        $.dictionary,
        $.set,
        $.tuple,
        $.parenthesized_expression,
        $.call,
        $.attribute,
        $.subscript,
        $.try_expression,
        $.unary_expression,
        $.borrow_expression,
        $.not_expression,
        $.binary_expression,
        $.comparison,
        $.boolean_operator,
        $.range,
        $.conditional_expression,
        $.if_expression,
        $.lambda,
        $.match_expression,
        $.catch_expression,
      ),

    identifier: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
    self: (_) => 'self',
    true: (_) => choice('true', 'True'),
    false: (_) => choice('false', 'False'),
    null: (_) => 'null',
    // `...` stands for elided code in examples.
    ellipsis: (_) => '...',

    integer: (_) => token(choice(/0[xX][0-9a-fA-F_]+/, /0[bB][01_]+/, /0[oO][0-7_]+/, /[0-9][0-9_]*/)),

    float: (_) =>
      token(
        choice(
          /[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?/,
          /[0-9][0-9_]*[eE][+-]?[0-9]+/,
        ),
      ),

    character: (_) => token(/'([^'\\\n]|\\.)'/),

    comment: (_) => token(seq('#', /.*/)),

    // "...", f"...": escapes, `{{` and `}}`, and `{expression}` interpolation.
    string: ($) =>
      choice(
        seq(
          choice('"', 'f"'),
          repeat(
            choice(
              $.string_content,
              $.escape_sequence,
              $.brace_escape,
              $.interpolation,
            ),
          ),
          token.immediate('"'),
        ),
        // """...""" on one line; may contain `"`.
        $._triple_string,
      ),

    _triple_string: (_) =>
      token(seq(optional('f'), '"""', repeat(choice(/[^"\n]/, /"[^"\n]/, /""[^"\n]/)), '"""')),

    string_content: (_) => token.immediate(prec(2, /[^"\\{}\n]+/)),

    escape_sequence: (_) =>
      token.immediate(
        prec(2, /\\(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8}|[^xuU\n])/),
      ),

    brace_escape: (_) => token.immediate(prec(3, choice('{{', '}}'))),

    interpolation: ($) =>
      seq(
        token.immediate('{'),
        field('expression', $._expression),
        optional(field('format', $.format_spec)),
        '}',
      ),

    format_spec: (_) => seq(':', token.immediate(/[^{}"\n]*/)),

    // r"..." and r"""...""": no escapes, no interpolation.
    raw_string: (_) =>
      token(
        choice(
          seq('r"""', repeat(choice(/[^"\n]/, /"[^"\n]/, /""[^"\n]/)), '"""'),
          seq('r"', repeat(choice(/[^"\\\n]/, /\\[^\n]/)), '"'),
        ),
      ),

    // b"...": ASCII and escapes, no interpolation.
    bytes: ($) =>
      seq(
        'b"',
        repeat(choice(alias($._bytes_content, $.string_content), $.escape_sequence)),
        token.immediate('"'),
      ),

    _bytes_content: (_) => token.immediate(prec(2, /[^"\\\n]+/)),

    list: ($) => seq('[', commaSep($._expression), ']'),

    dictionary: ($) => seq('{', commaSep($.pair), '}'),

    pair: ($) => seq(field('key', $._expression), ':', field('value', $._expression)),

    set: ($) => seq('{', commaSep1($._expression), '}'),

    tuple: ($) =>
      seq('(', choice(seq($._expression, ','), seq($._expression, repeat1(seq(',', $._expression)), optional(','))), ')'),

    parenthesized_expression: ($) => seq('(', $._expression, ')'),

    call: ($) =>
      prec(PREC.postfix, seq(field('function', $._expression), field('arguments', $.arguments))),

    arguments: ($) => seq('(', commaSep(choice($._expression, $.keyword_argument)), ')'),

    // `name=value`, or `name: value` in a struct literal such as `Point(x: 1, y: 2)`.
    keyword_argument: ($) =>
      seq(field('name', $.identifier), choice('=', ':'), field('value', $._expression)),

    attribute: ($) =>
      prec(
        PREC.postfix,
        seq(field('object', $._expression), '.', field('attribute', choice($.identifier, $.integer))),
      ),

    subscript: ($) =>
      prec(
        PREC.postfix,
        seq(field('value', $._expression), '[', commaSep1(choice($._expression, $.slice)), ']'),
      ),

    slice: ($) => seq(optional($._expression), ':', optional($._expression)),

    try_expression: ($) => prec(PREC.postfix, seq($._expression, '?')),

    unary_expression: ($) =>
      prec(PREC.unary, seq(field('operator', choice('-', '~', '+')), field('argument', $._expression))),

    borrow_expression: ($) =>
      prec(PREC.unary, seq('&', optional($.mutable), field('argument', $._expression))),

    not_expression: ($) => prec(PREC.not, seq('not', field('argument', $._expression))),

    binary_expression: ($) => {
      const table = [
        [prec.left, PREC.bitor, '|'],
        [prec.left, PREC.bitxor, '^'],
        [prec.left, PREC.bitand, '&'],
        [prec.left, PREC.shift, choice('<<', '>>')],
        [prec.left, PREC.plus, choice('+', '-')],
        [prec.left, PREC.times, choice('*', '/', '//', '%')],
        [prec.right, PREC.power, '**'],
      ];
      return choice(
        ...table.map(([fn, p, operator]) =>
          // @ts-ignore
          fn(p, seq(field('left', $._expression), field('operator', operator), field('right', $._expression))),
        ),
      );
    },

    comparison: ($) =>
      prec.left(
        PREC.compare,
        seq(
          field('left', $._expression),
          field('operator', choice('==', '!=', '<', '<=', '>', '>=', 'in', alias(seq('not', 'in'), 'not in'))),
          field('right', $._expression),
        ),
      ),

    boolean_operator: ($) =>
      choice(
        prec.left(PREC.and, seq(field('left', $._expression), field('operator', 'and'), field('right', $._expression))),
        prec.left(PREC.or, seq(field('left', $._expression), field('operator', 'or'), field('right', $._expression))),
      ),

    range: ($) => prec.left(PREC.range, seq(field('start', $._expression), '..', field('end', $._expression))),

    // a if c else b
    conditional_expression: ($) =>
      prec.right(PREC.conditional, seq($._expression, 'if', $._expression, 'else', $._expression)),

    // if c: a elif d: b else: e
    if_expression: ($) =>
      prec.right(
        PREC.conditional,
        seq(
          'if',
          field('condition', $._expression),
          ':',
          field('consequence', $._expression),
          repeat(seq('elif', $._expression, ':', $._expression)),
          'else',
          ':',
          field('alternative', $._expression),
        ),
      ),

    // fn(x: i64) -> i64: x * 2
    lambda: ($) =>
      prec.right(
        PREC.lambda,
        seq(
          'fn',
          field('parameters', $.parameters),
          optional(seq('->', field('return_type', $._type))),
          ':',
          field('body', choice($._expression, $.fail_statement)),
        ),
      ),

    match_expression: ($) =>
      seq(
        'match',
        field('value', $._expression),
        ':',
        field('body', $.match_body),
      ),

    match_body: ($) => seq($._indent, repeat1($.match_arm), $._dedent),

    // f() catch e: value      — or an indented block after the `:`.
    catch_expression: ($) =>
      prec.right(
        PREC.catch,
        seq(
          field('value', $._expression),
          'catch',
          optional(field('error', $.identifier)),
          ':',
          field(
            'handler',
            choice(
              $._expression,
              $.fail_statement,
              $.return_statement,
              $.break_statement,
              $.continue_statement,
              seq($._indent, $.block),
            ),
          ),
        ),
      ),
  },
});
