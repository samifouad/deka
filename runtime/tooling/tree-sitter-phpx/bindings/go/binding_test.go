package tree_sitter_phpx_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_phpx "github.com/samifouad/deka/tooling/tree-sitter-phpx/bindings/go"
)

func TestPHPGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_phpx.LanguagePHPX())
	if language == nil {
		t.Errorf("Error loading PHPX grammar")
	}

	sourceCode := []byte("<?php echo 'Hello, World!';")
	parser := tree_sitter.NewParser()
	defer parser.Close()
	parser.SetLanguage(language)

	tree := parser.Parse(sourceCode, nil)
	if tree == nil || tree.RootNode().HasError() {
		t.Errorf("Error parsing PHP")
	}
}

func TestPHPOnlyGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_phpx.LanguagePHPXOnly())
	if language == nil {
		t.Errorf("Error loading PHPX-only grammar")
	}

	sourceCode := []byte("echo 'Hello, World!';")
	parser := tree_sitter.NewParser()
	defer parser.Close()
	parser.SetLanguage(language)

	tree := parser.Parse(sourceCode, nil)
	if tree == nil || tree.RootNode().HasError() {
		t.Errorf("Error parsing PHP")
	}
}
