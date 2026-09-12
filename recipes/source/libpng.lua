return {
    name = "libpng", version = "1.6.50", revision = 1, executables = {}, dependencies = {"zlib.lua"},
    source = { url = "https://download.sourceforge.net/libpng/libpng-1.6.50.tar.xz", sha256 = "4df396518620a7aa3651443e87d1b2862e4e88cad135a8b93423e01706232307" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--disable-static"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
