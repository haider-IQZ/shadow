return {
    name = "kitty", version = "0.48.2", revision = 3, executables = {"kitty", "kitten"},
    dependencies = {"python.lua", "harfbuzz.lua", "cairo.lua", "lcms2.lua", "xxhash.lua", "simde.lua"},
    source = { url = "https://github.com/kovidgoyal/kitty/releases/download/v0.48.2/kitty-0.48.2.tar.xz", sha256 = "792a2bbde715bb26839677875a44202f1c30e91a20351842e0a61b5c6a1e666e" },
    resources = {
        { path = "fonts/SymbolsNerdFontMono-Regular.ttf", url = "https://raw.githubusercontent.com/ryanoasis/nerd-fonts/v3.4.0/patched-fonts/NerdFontsSymbolsOnly/SymbolsNerdFontMono-Regular.ttf", sha256 = "f0f624d9b474bea1662cf7e862d44aebe1ae1f6c7f9cb7a0ca5d0e5ac9561c60" },
        { path = "LICENSE-NerdFont", url = "https://raw.githubusercontent.com/ryanoasis/nerd-fonts/v3.4.0/patched-fonts/NerdFontsSymbolsOnly/LICENSE", sha256 = "84a7a98c82140fb12c37fe42b93805baa16024cb3e5acc599b7ffe612c55d847" },
    },
    build = function(ctx)
        -- Set Python's private home inside Kitty's interpreter configuration,
        -- rather than leaking PYTHONHOME into every shell launched by the terminal.
        -- Generate code with the build interpreter first. The installed private
        -- home is not valid until the package reaches its final Cellar layout.
        ctx.run({ctx.deps.python .. "/bin/python3", "setup.py", "linux-package", "--prefix=" .. ctx.output, "--fontconfig-library=libfontconfig.so.1"})
        ctx.run({"sh", "-eu", "-c", [[
            export CPPFLAGS="$CPPFLAGS -DSET_PYTHON_HOME=\\\"$2\\\""
            exec "$1" setup.py linux-package --skip-code-generation "--prefix=$3" --fontconfig-library=libfontconfig.so.1
        ]], "shadow-kitty-build", ctx.deps.python .. "/bin/python3", "../../" .. ctx.relative_deps.python, ctx.output})
        ctx.run({"sh", "-eu", "-c", [[
            mv "$1/bin/kitty" "$1/bin/kitty.real"
            # Runtime paths are resolved from the installed wrapper, not the build prefix.
            {
                printf '%s\n' '#!/bin/sh' 'prefix=$(dirname "$(dirname "$(readlink -f "$0")")")'
                printf '%s\n' 'export FONTCONFIG_FILE="${FONTCONFIG_FILE:-/etc/fonts/fonts.conf}"' 'exec "$prefix/bin/kitty.real" "$@"'
            } > "$1/bin/kitty"
            chmod 755 "$1/bin/kitty"
        ]], "shadow-kitty", ctx.output})
        -- Preserve linked Go dependency sources and their redistribution notices.
        ctx.run({"go", "mod", "vendor"})
        ctx.run({ctx.deps.python .. "/bin/python3", "-c", [[
from pathlib import Path
import shutil, sys
root = Path(sys.argv[1]) / 'share/licenses/kitty/go-dependencies'
for path in Path('vendor').rglob('*'):
    if path.is_file() and path.name.upper().startswith(('LICENSE', 'COPYING', 'COPYRIGHT', 'NOTICE')):
        target = root / path.relative_to('vendor')
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
        ]], ctx.output})
        ctx.run({"sh", "-eu", "-c", [[
            cp "$(go env GOROOT)/LICENSE" "$1/share/licenses/kitty/LICENSE-Go"
            tar -czf "$2/kitty-0.48.2-go-vendor.tar.gz" vendor go.mod go.sum
        ]], "shadow-kitty-sources", ctx.output, ctx.source_archives})
    end,
}
