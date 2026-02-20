{
  description = "Nix MCP server and skills for Claude Code";

  inputs = {
    nixpkgs-master.url = "github:NixOS/nixpkgs/b28c4999ed71543e71552ccfd0d7e68c581ba7e9";
    nixpkgs.url = "github:NixOS/nixpkgs/23d72dabcb3b12469f57b37170fcbc1789bd7457";
    utils.url = "https://flakehub.com/f/numtide/flake-utils/0.1.102";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    fh.url = "https://flakehub.com/f/DeterminateSystems/fh/*.tar.gz";
    rust.url = "github:friedenberg/eng?dir=devenvs/rust";
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , rust-overlay
    , crane
    , fh
    , rust
    , nixpkgs-master
    ,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = [ ];
          nativeBuildInputs = [ ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        chix-unwrapped = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        fhPkg = fh.packages.${system}.default;

        formatNixHook = pkgs.writeShellScript "format-nix" ''
          set -euo pipefail
          input=$(cat)
          file_path=$(${pkgs.jq}/bin/jq -r '.tool_input.file_path // empty' <<< "$input")
          if [[ -n "$file_path" && "$file_path" == *.nix ]]; then
            ${pkgs.nixfmt-rfc-style}/bin/nixfmt "$file_path" 2>/dev/null || true
          fi
        '';

        # Note: We don't bundle nix itself - we use the system nix.
        # This ensures compatibility with Determinate Nix settings like lazy-trees.
        chix =
          pkgs.runCommand "chix"
            {
              nativeBuildInputs = [ pkgs.makeWrapper ];
            }
            ''
              mkdir -p $out/bin
              makeWrapper ${chix-unwrapped}/bin/chix $out/bin/chix \
                --prefix PATH : ${
                  pkgs.lib.makeBinPath [
                    fhPkg
                    pkgs.cachix
                    pkgs.nil
                  ]
                }

              mkdir -p $out/share/purse-first/chix/hooks
              cp ${./.claude-plugin/plugin.json} $out/share/purse-first/chix/plugin.json
              install -m 755 ${formatNixHook} $out/share/purse-first/chix/hooks/format-nix
            '';
      in
      {
        packages = {
          default = chix;
          chix = chix;
          unwrapped = chix-unwrapped;
        };

        devShells.default = rust.devShells.${system}.default.overrideAttrs (oldAttrs: {
          nativeBuildInputs = (oldAttrs.nativeBuildInputs or [ ]) ++ [
            fhPkg
            pkgs.nil
            pkgs.just
            pkgs.gum
          ];
        });
      }
    );
}
