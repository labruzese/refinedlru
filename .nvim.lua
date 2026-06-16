vim.api.nvim_create_autocmd("FileType", {
  pattern = "rust",
  callback = function(args)
    local root = vim.fs.root(args.buf, "Cargo.toml")
    if not root then return end
    local cmd = ("cd %s && cargo refinedrust && cd output && dune build --display short")
      :format(vim.fn.shellescape(root))
    vim.bo[args.buf].makeprg = vim.fn.escape(cmd, "%#")
  end,
})
