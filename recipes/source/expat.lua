return {
    name = "expat", version = "2.7.1", revision = 1, executables = {},
    source = { url = "https://github.com/libexpat/libexpat/releases/download/R_2_7_1/expat-2.7.1.tar.xz", sha256 = "354552544b8f99012e5062f7d570ec77f14b412a3ff5c7d8d0dae62c0d217c30" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--disable-static", "--without-docbook", "--without-tests", "--without-examples"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
