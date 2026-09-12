return {
    name = "lcms2", version = "2.17", revision = 1, executables = {},
    source = { url = "https://github.com/mm2/Little-CMS/releases/download/lcms2.17/lcms2-2.17.tar.gz", sha256 = "d11af569e42a1baa1650d20ad61d12e41af4fead4aa7964a01f93b08b53ab074" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--disable-static", "--without-jpeg", "--without-tiff"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
