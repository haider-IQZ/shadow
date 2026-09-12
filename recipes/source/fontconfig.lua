return {
    name = "fontconfig", version = "2.17.1", revision = 2, executables = {}, dependencies = {"freetype.lua", "expat.lua"},
    source = { url = "https://gitlab.freedesktop.org/fontconfig/fontconfig/-/archive/2.17.1/fontconfig-2.17.1.tar.gz", sha256 = "82e73b26adad651b236e5f5d4b3074daf8ff0910188808496326bd3449e5261d" },
    build = function(ctx)
        ctx.run({"meson", "setup", "build", "--prefix=" .. ctx.prefix, "--libdir=lib", "--buildtype=release", "--wrap-mode=nodownload", "-Ddoc=disabled", "-Dtests=disabled", "-Dtools=disabled", "-Dcache-build=disabled"})
        ctx.run({"ninja", "-C", "build", "-j" .. ctx.jobs})
        ctx.run({"env", "DESTDIR=" .. ctx.destdir, "ninja", "-C", "build", "install"})
    end,
}
