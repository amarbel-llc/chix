# chix

default:
    @just --list

# Check skill markdown files with shellcheck where applicable
check:
    nix develop --command shellcheck skills/nix-codebase/examples/*.nix 2>/dev/null || true

# Format shell scripts
fmt:
    nix develop --command shfmt -w -i 2 -ci skills/nix-codebase/examples/*.bash 2>/dev/null || true

# Clean build artifacts
clean:
    rm -rf result
