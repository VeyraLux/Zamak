//! Zamak for Zed: runs `zam lsp` for diagnostics, completion, hover, go to definition,
//! rename, and formatting. Highlighting comes from the tree-sitter grammar in `grammar/`
//! and works without `zam`.

use zed_extension_api::{self as zed, LanguageServerId, Result};

struct Zamak;

impl zed::Extension for Zamak {
    fn new() -> Self {
        Zamak
    }

    fn language_server_command(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // When the Zed settings set `lsp.zam.binary.path`, Zed starts that program itself
        // (with `lsp.zam.binary.arguments`) and doesn't call this function.
        let command = worktree.which("zam").ok_or_else(|| {
            "Zamak: can't find `zam` on PATH. Install it with `npm install -g @veyralux/zamak`, \
             or set `lsp.zam.binary.path` (and `\"arguments\": [\"lsp\"]`) in your Zed settings. \
             Highlighting works without it."
                .to_string()
        })?;
        Ok(zed::Command {
            command,
            args: vec!["lsp".into()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(Zamak);
