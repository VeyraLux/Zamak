(function_definition
  body: (_) @function.inside) @function.around

(lambda
  body: (_) @function.inside) @function.around

(struct_definition
  body: (_) @class.inside) @class.around

(enum_definition
  body: (_) @class.inside) @class.around

(interface_definition
  body: (_) @class.inside) @class.around

(comment)+ @comment.around
