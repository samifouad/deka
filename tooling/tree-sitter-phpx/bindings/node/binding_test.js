/// <reference types="node" />

const assert = require("node:assert");
const { describe, it } = require("node:test");

const Parser = require("tree-sitter");
const { phpx, phpx_only } = require("../..");

describe("PHPX", () => {
  const parser = new Parser();
  parser.setLanguage(phpx);

  it("should be named phpx", () => {
    assert.strictEqual(parser.getLanguage().name, "phpx");
  });

  it("should parse source code", () => {
    const sourceCode = "<?php echo 'Hello, World!';";
    const tree = parser.parse(sourceCode);
    assert(!tree.rootNode.hasError);
  });
});

describe("PHPX Only", () => {
  const parser = new Parser();
  parser.setLanguage(phpx_only);

  it("should be named phpx_only", () => {
    assert.strictEqual(parser.getLanguage().name, "phpx_only");
  });

  it("should parse source code", () => {
    const sourceCode = "echo 'Hello, World!';";
    const tree = parser.parse(sourceCode);
    assert(!tree.rootNode.hasError);
  });
});
