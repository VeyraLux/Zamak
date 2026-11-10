; Later patterns take precedence over earlier ones, so general rules come first.

(identifier) @variable

; Names written in CamelCase are types, variants, or constructors; ALL_CAPS are constants.
((identifier) @type
  (#match? @type "^[A-Z]"))

((identifier) @constant
  (#match? @constant "^_*[A-Z][A-Z0-9_]*$"))

(self) @variable.special

; ---------------------------------------------------------------- declarations

(function_definition
  name: (identifier) @function)

(function_signature
  name: (identifier) @function)

(extern_function
  name: (identifier) @function)

(struct_definition
  name: (identifier) @type)

(enum_definition
  name: (identifier) @type)

(interface_definition
  name: (identifier) @type)

(extern_type
  name: (identifier) @type)

(type_parameter
  name: (identifier) @type)

(enum_variant
  name: (identifier) @variant)

(field_declaration
  name: (identifier) @property)

(variant_field
  name: (identifier) @property)

(parameter
  name: (identifier) @variable.parameter)

(const_declaration
  name: (identifier) @constant)

(module_path
  (identifier) @namespace)

(decorator
  "@" @attribute
  name: (identifier) @attribute)

; ----------------------------------------------------------------------- types

(type_identifier
  (identifier) @type)

((type_identifier
  (identifier) @type.builtin)
  (#match? @type.builtin "^(i64|f64|bool|string|bytes)$"))

(scoped_type
  module: (identifier) @namespace)

; ----------------------------------------------------------------- expressions

(attribute
  attribute: (identifier) @property)

(call
  function: (identifier) @function)

(call
  function: (attribute
    attribute: (identifier) @function.method))

; `Point(x: 1)`, `Shape.Circle(2.0)`: a call to a CamelCase name builds a value.
((call
  function: (identifier) @constructor)
  (#match? @constructor "^[A-Z]"))

((call
  function: (attribute
    attribute: (identifier) @constructor))
  (#match? @constructor "^[A-Z]"))

; `Shape.Empty`, `Option.None`
((attribute
  object: (identifier) @type
  attribute: (identifier) @variant)
  (#match? @type "^[A-Z]")
  (#match? @variant "^[A-Z]"))

((call
  function: (identifier) @function.builtin)
  (#match? @function.builtin
    "^(print|println|len|abs|min|max|sum|sorted|str|int|float|copy|range|input|i64|f64|bool|string|bytes|set)$"))

(keyword_argument
  name: (identifier) @variable.parameter)

(variant_pattern
  variant: (identifier) @variant)

(variant_pattern
  variant: (scoped_identifier
    (identifier) @variant))

(match_arm
  pattern: (identifier) @variant
  (#match? @variant "^[A-Z]"))

((match_arm
  pattern: (identifier) @variable.special)
  (#eq? @variable.special "_"))

(catch_expression
  error: (identifier) @variable.parameter)

; -------------------------------------------------------------------- literals

(integer) @number
(float) @number
(negative_integer) @number

[
  (true)
  (false)
] @boolean

(null) @constant.builtin

(character) @string.special

(string) @string
(raw_string) @string
(bytes) @string

(string_content) @string
(escape_sequence) @string.escape
(brace_escape) @string.escape

(interpolation
  "{" @punctuation.special
  "}" @punctuation.special)

(format_spec) @string.special

(comment) @comment

(ellipsis) @comment

; -------------------------------------------------------------------- keywords

[
  "fn"
  "struct"
  "enum"
  "interface"
  "const"
  "let"
  "use"
  "extern"
  "type"
  "kept"
  (visibility)
  (mutable)
] @keyword

[
  "if"
  "elif"
  "else"
  "while"
  "for"
  "in"
  "match"
  "catch"
  "return"
  "fail"
  "assert"
  (break_statement)
  (continue_statement)
  (pass_statement)
] @keyword

[
  "and"
  "or"
  "not"
] @keyword.operator

(comparison
  operator: "not in" @keyword.operator)

; ------------------------------------------------------------------- operators

[
  "+"
  "-"
  "*"
  "/"
  "//"
  "%"
  "**"
  "=="
  "!="
  "<"
  "<="
  ">"
  ">="
  "="
  "+="
  "-="
  "*="
  "/="
  "//="
  "%="
  "**="
  "&="
  "|="
  "^="
  "<<="
  ">>="
  "&"
  "|"
  "^"
  "~"
  "<<"
  ">>"
  "->"
  ".."
  "?"
  "!"
] @operator

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  "."
  ":"
  ";"
] @punctuation.delimiter
