local M = {}

function M.setup()
  local lspconfig = require("lspconfig")
  local util = require("lspconfig.util")

  lspconfig.phpx_lsp.setup({
    cmd = { "phpx_lsp" },
    filetypes = { "phpx" },
    root_dir = util.root_pattern("deka.lock", "php_modules", ".git"),
  })
end

return M
