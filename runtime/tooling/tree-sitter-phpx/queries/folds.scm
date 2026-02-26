(function_definition
  body: (compound_statement) @fold)

(struct_declaration
  body: (declaration_list) @fold)

(if_statement
  (compound_statement) @fold)

(foreach_statement
  (compound_statement) @fold)

(jsx_element) @fold
(jsx_fragment) @fold

(frontmatter) @fold
(template_section) @fold
