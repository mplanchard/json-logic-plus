let
  sources = import ./nix/sources.nix;
  moz_overlay = import sources.mozilla;
  pkgs = import sources.nixpkgs { overlays = [ moz_overlay ]; };

  # Set up a "rust" override that overrides the `extensions` attribute
  # in the mozilla overlay. You can update `stable` to something
  # else to use a different channel here. You can see what extensions
  # are available for your channel by pulling down
  # https://static.rust-lang.org/dist/channel-rust-<channel>.toml
  # and checking out which `pkg.<name>` components are enabled for
  # your arch. Default components are always installed by this overlay,
  # which are found at the bottom of the toml file.
  rust = (pkgs.latest.rustChannels.stable.rust.override {
    extensions = [
      "clippy-preview"
      "llvm-tools-preview" # for stepwise debugging
      "rust-src"
      "rust-std"
    ];
  });
in pkgs.mkShell {
  buildInputs = [
    # Rust and related tools
    rust
    pkgs.rust-analyzer

    # Needed for dev dependencies
    pkgs.openssl

    # For running and testing the python build
    pkgs.python3Minimal

    # For running and testing the JS build
    pkgs.nodejs-12_x

    # keep this line if you use bash
    pkgs.bashInteractive
  ];
  # Normally rustup provides access to the docs, but a) we're not providing it,
  # since we're installing our rust components manually via the overlay and
  # b) even if we did provide it, it wouldn't know where to find the docs
  # installed in the nix store, looking instead in the user's home directory.
  #
  # So, we define the alias:
  #   rust-docs
  #
  # Which will nab the correct rust docs for the currently installed rust version
  # and display them in the browser, using `xdg-open` for linux and `open` for mac
  shellHook = if pkgs.stdenv.isDarwin then ''
    alias rust-docs="open $(dirname $(dirname $(which rustc)))/share/doc/rust/html/index.html"
  '' else ''
    alias rust-docs="xdg-open $(dirname $(dirname $(which rustc)))/share/doc/rust/html/index.html"
  '';
}
