(function_definition) @function.outer
(function_definition body: (compound_statement) @function.inner)

(struct_declaration) @struct.outer
(struct_declaration body: (declaration_list) @struct.inner)

(jsx_element) @jsx.outer
(jsx_fragment) @jsx.outer
(jsx_element (jsx_child) @jsx.inner)
(jsx_fragment (jsx_child) @jsx.inner)

(simple_parameter) @parameter.outer
