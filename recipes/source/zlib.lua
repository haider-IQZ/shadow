return {
    name = "zlib", version = "1.3.1", revision = 1, executables = {},
    source = { url = "https://zlib.net/fossils/zlib-1.3.1.tar.gz", sha256 = "9a93b2b7dfdac77ceba5a558a580e74667dd6fede4585b91eefb60f03b72df23" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
