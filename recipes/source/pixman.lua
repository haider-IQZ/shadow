return {
    name = "pixman", version = "0.46.4", revision = 1, executables = {},
    source = { url = "https://cairographics.org/releases/pixman-0.46.4.tar.gz", sha256 = "d09c44ebc3bd5bee7021c79f922fe8fb2fb57f7320f55e97ff9914d2346a591c" },
    build = function(ctx)
        ctx.run({"meson", "setup", "build", "--prefix=" .. ctx.prefix, "--libdir=lib", "--buildtype=release", "--wrap-mode=nodownload", "-Dtests=disabled", "-Ddemos=disabled", "-Dlibpng=disabled"})
        ctx.run({"ninja", "-C", "build", "-j" .. ctx.jobs})
        ctx.run({"env", "DESTDIR=" .. ctx.destdir, "ninja", "-C", "build", "install"})
    end,
}
