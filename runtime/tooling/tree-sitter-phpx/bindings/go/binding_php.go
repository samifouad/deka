package tree_sitter_phpx

// #cgo CFLAGS: -I../../php/src -std=c11 -fPIC
// #include "../../php/src/parser.c"
// #include "../../php/src/scanner.c"
import "C"

import "unsafe"

// Get the tree-sitter Language for the PHPX grammar.
func LanguagePHPX() unsafe.Pointer {
	return unsafe.Pointer(C.tree_sitter_phpx())
}
