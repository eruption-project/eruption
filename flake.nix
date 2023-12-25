{
  # Eruption NixOS flake
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    naersk,
    nixpkgs,
    self,
    utils,
  }:
  # We do this for all systems - namely x86_64-linux, aarch64-linux,
  # x86_64-darwin and aarch64-darwin
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      naersk-lib = pkgs.callPackage naersk {};

      eruption = pkgs.rustPlatform.buildRustPackage rec {
        pname = "eruption";
        version = "3119861a912fc976c4b24cfee5e41f6c32bf75c2";
        src = pkgs.fetchFromGitHub {
          owner = "eruption-project";
          repo = pname;
          rev = version;
          hash = "sha256-J3ov5izyachxw6g/Bx/nFsSRotN4qT1LTWAR0JQJyFs=";
        };
        cargoLock = {
          lockFile = "${src}/Cargo.lock";
          outputHashes = {
            "hidapi-2.4.1" = "sha256-jzZalweZRr/Ob+7XhHFFruPEXzmSkYZLP5RhdNq9CE4=";
            "rust-pulsectl-0.2.7" = "sha256-jkZJiTbCkPCe20d08ExY/VmdFOaV3GxMxMVOnXl2HlM=";
          };
        };
        nativeBuildInputs = with pkgs; [
          pkg-config
          protobuf
          installShellFiles
          libxkbcommon
        ];
        PROTOC = "${pkgs.protobuf}/bin/protoc";
        buildInputs = with pkgs; [
          dbus
          gcc
          gtk3
          gtk3
          gtksourceview4
          hidapi
          libxkbcommon
          libevdev
          libpulseaudio
          libusb1
          lua
          lua54Packages.luasocket
          systemd
        ];
        postInstall = ''
          installManPage support/man/*.{8,5,1}
          installShellCompletion --bash support/shell/completions/en_US/*.bash-completion
          installShellCompletion --fish support/shell/completions/en_US/*.fish-completion
          installShellCompletion --zsh support/shell/completions/en_US/*.zsh-completion
        '';
      };
    in rec {
      # Build via "nix build .#default"
      packages = {
        default = eruption;
        eruption_git = naersk-lib.buildPackage {
          # The build dependencies
          buildInputs = with pkgs; [
            cmake
            dbus
            gcc
            gtk3
            gtk3
            gtksourceview4
            hidapi
            libevdev
            libpulseaudio
            libusb1
            lua
            lua54Packages.luasocket
            pkgconf
            protobuf
            systemd
            xorg.xrandr
            xorg.xorgserver
          ];
          src = ./.;
        };
        inherit eruption;
      };
      # Enter devshell with all the tools via "nix develop"
      # or "nix-shell"
      devShells.default = with pkgs;
        mkShell {
          buildInputs = [
            git
            eruption
          ];
        };
    });
}
