(function_definition
  (visibility)? @context
  "fn" @context
  name: (identifier) @name) @item

(function_signature
  "fn" @context
  name: (identifier) @name) @item

(extern_function
  "extern" @context
  "fn" @context
  name: (identifier) @name) @item

(extern_type
  "extern" @context
  "type" @context
  name: (identifier) @name) @item

(struct_definition
  (visibility)? @context
  "struct" @context
  name: (identifier) @name) @item

(enum_definition
  (visibility)? @context
  "enum" @context
  name: (identifier) @name) @item

(interface_definition
  (visibility)? @context
  "interface" @context
  name: (identifier) @name) @item

(enum_variant
  name: (identifier) @name) @item

(field_declaration
  name: (identifier) @name) @item

(const_declaration
  (visibility)? @context
  "const" @context
  name: (identifier) @name) @item
