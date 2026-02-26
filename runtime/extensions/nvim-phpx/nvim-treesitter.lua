local parser_config = require("nvim-treesitter.parsers").get_parser_configs()

parser_config.phpx = {
  install_info = {
    url = "/absolute/path/to/deka/tooling/tree-sitter-phpx",
    files = { "src/parser.c", "src/scanner.c" },
    branch = "main",
    generate_requires_npm = true,
  },
  filetype = "phpx",
}

vim.treesitter.language.register("phpx", "phpx")
