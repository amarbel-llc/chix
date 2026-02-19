# develop_run multi-command support

## Problem

The `develop_run` tool validates arguments against shell metacharacters, which
blocks legitimate use of `bash -c "cd foo && go build ./..."`. Callers need to
chain commands but the security model prevents it.

## Design

### Schema

Replace `command` (string) + `args` (string array) with a `commands` array:

```json
{
  "commands": [
    { "command": "go", "args": ["build", "./..."] },
    { "command": "go", "args": ["test", "./..."] }
  ],
  "flake_ref": ".",
  "flake_dir": "/path/to/project"
}
```

Each element has a required `command` and optional `args`.

### Execution

Each command runs as a separate `nix develop <flake_ref> -c <command> [args...]`
invocation. Commands execute sequentially with stop-on-failure semantics (like
`&&`). No shell is involved — each command is exec'd directly.

### Validation

Same as today: `validate_args()` on each command's args array. No shell
metacharacters needed since commands are individually exec'd.

### Result format

```json
{
  "success": false,
  "results": [
    { "command": "go build ./...", "success": true, "stdout": "", "stderr": "", "exit_code": 0 },
    { "command": "go test ./...", "success": false, "stdout": "...", "stderr": "...", "exit_code": 1 }
  ]
}
```

Top-level `success` is true only if all commands succeeded. `results` contains
one entry per command that ran (commands after a failure are omitted).

### Tool description

Updated to instruct callers:

- Use `flake_dir` to set working directory instead of `cd`
- Use separate entries in `commands` instead of shell operators like `&&`
- Shell metacharacters are not allowed in command arguments

### Trade-offs

- Each command re-enters the devShell (minor overhead)
- Environment mutations from one command don't persist to the next
- Breaking schema change — acceptable since chix is young and Claude Code reads
  the schema dynamically

### Alternatives considered

1. **Single `nix develop -c bash -c "cmd1 && cmd2"`** — defeats metacharacter
   validation, harder to get per-command results
2. **Generated temp script** — over-engineered, same result parsing issues
