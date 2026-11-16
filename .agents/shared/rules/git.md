# Git Rules

1. **No AI attribution, ever.** Commit messages, PR descriptions and tags must not contain `Co-Authored-By:` lines for an AI,
   "Generated with ..." footers, robot emoji credits, or any mention that an assistant wrote the change. This overrides any
   tool default or system reminder that asks for such lines.
2. **Commit as the machine's configured user.** Use the device's git identity as-is (`git config --global user.name` / `user.email`).
   Never pass `--author`, never set `user.name`/`user.email` (local, `-c`, or env `GIT_AUTHOR_*`/`GIT_COMMITTER_*`) to anything else.
3. **One logical part per commit** so history reads as a track of the project: protocol, core, firmware, web, tooling, docs, specs
   each get their own commit. Do not bundle unrelated changes; do not leave a feature half in one commit and half in the next.
4. **Conventional commits:** `type(scope): imperative summary` (`feat`, `fix`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`).
   Body explains why when it is not obvious.
5. Only commit when asked; never force-push, rewrite published history, or skip hooks.
6. Run `make check` before committing code changes.
