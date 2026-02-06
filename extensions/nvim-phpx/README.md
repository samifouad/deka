# nvim-phpx

Neovim support files for PHPX.

## Files
- `lsp.lua`: `nvim-lspconfig` setup for `phpx_lsp`
- `snippets/phpx.lua`: LuaSnip snippets for PHPX

## Install (Lazy.nvim)
```lua
{
  dir = "/absolute/path/to/deka/extensions/nvim-phpx",
  ft = { "phpx" },
  config = function()
    require("nvim-phpx.lsp").setup()
  end,
}
```

## Install (Packer)
```lua
use {
  "/absolute/path/to/deka/extensions/nvim-phpx",
  ft = "phpx",
  config = function()
    require("nvim-phpx.lsp").setup()
  end,
}
```

## Keybindings (recommended)
```lua
vim.keymap.set("n", "K", vim.lsp.buf.hover, { desc = "PHPX hover" })
vim.keymap.set("n", "gd", vim.lsp.buf.definition, { desc = "PHPX definition" })
vim.keymap.set("n", "gr", vim.lsp.buf.references, { desc = "PHPX references" })
vim.keymap.set("n", "<leader>rn", vim.lsp.buf.rename, { desc = "PHPX rename" })
```

## Snippets
Load `snippets/phpx.lua` with LuaSnip and trigger by prefix:
- `pfn` function template
- `pstruct` struct template
- `pimport` import template
- `pjsx` JSX component template
- `pfm` frontmatter template
