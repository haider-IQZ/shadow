return {
    name = "cairo", version = "1.18.4", revision = 2, executables = {}, dependencies = {"pixman.lua", "fontconfig.lua", "libpng.lua"},
    source = { url = "https://cairographics.org/releases/cairo-1.18.4.tar.xz", sha256 = "445ed8208a6e4823de1226a74ca319d3600e83f6369f99b14265006599c32ccb" },
    build = function(ctx)
        ctx.run({"meson", "setup", "build", "--prefix=" .. ctx.prefix, "--libdir=lib", "--buildtype=release", "--wrap-mode=nodownload", "-Dglib=disabled", "-Dxlib=disabled", "-Dxcb=disabled", "-Dtests=disabled", "-Dgtk_doc=false", "-Dspectre=disabled", "-Dsymbol-lookup=disabled"})
        ctx.run({"ninja", "-C", "build", "-j" .. ctx.jobs})
        ctx.run({"env", "DESTDIR=" .. ctx.destdir, "ninja", "-C", "build", "install"})
    end,
}
