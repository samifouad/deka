package tree_sitter_phpx

// #cgo CFLAGS: -I../../php_only/src -std=c11 -fPIC
// #include "../../php_only/src/parser.c"
// #include "../../php_only/src/scanner.c"
import "C"

import "unsafe"

// Get the tree-sitter Language for PHPX-only.
func LanguagePHPXOnly() unsafe.Pointer {
	return unsafe.Pointer(C.tree_sitter_phpx_only())
}
