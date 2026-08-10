{
  description = "bevy runtime environment";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      buildPackages = with pkgs; [
        clang
        pkg-config
      ];
      runtimePackages = with pkgs; [
        wayland
        libxkbcommon
        xkeyboard-config
        vulkan-loader
        alsa-lib
        udev
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = buildPackages ++ runtimePackages;
        shellHook = ''
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimePackages}:$LD_LIBRARY_PATH"
          export XKB_CONFIG_ROOT="${pkgs.xkeyboard-config}/share/X11/xkb"
        '';
      };
    };
}
