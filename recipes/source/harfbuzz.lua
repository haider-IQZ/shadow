return {
    name = "harfbuzz", version = "11.4.5", revision = 2, executables = {}, dependencies = {"freetype.lua"},
    source = { url = "https://github.com/harfbuzz/harfbuzz/releases/download/11.4.5/harfbuzz-11.4.5.tar.xz", sha256 = "0f052eb4ab01d8bae98ba971c954becb32be57d7250f18af343b1d27892e03fa" },
    build = function(ctx)
        ctx.run({"meson", "setup", "build", "--prefix=" .. ctx.prefix, "--libdir=lib", "--buildtype=release", "--wrap-mode=nodownload", "-Dglib=disabled", "-Dgobject=disabled", "-Dicu=disabled", "-Dcairo=disabled", "-Dfreetype=enabled", "-Dtests=disabled", "-Ddocs=disabled", "-Dutilities=disabled"})
        ctx.run({"ninja", "-C", "build", "-j" .. ctx.jobs})
        ctx.run({"env", "DESTDIR=" .. ctx.destdir, "ninja", "-C", "build", "install"})
    end,
}
