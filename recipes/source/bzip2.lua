return {
    name = "bzip2", version = "1.0.8", revision = 1, executables = {},
    source = { url = "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz", sha256 = "ab5a03176ee106d3f0fa90e381da478ddae405918153cca248e682cd0c4a2269" },
    build = function(ctx)
        ctx.run({"make", "-f", "Makefile-libbz2_so", "-j" .. ctx.jobs})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "PREFIX=" .. ctx.output, "install"})
        ctx.run({"sh", "-eu", "-c", [[
            cp libbz2.so.1.0.8 "$1/lib/"
            ln -s libbz2.so.1.0.8 "$1/lib/libbz2.so.1.0"
            ln -s libbz2.so.1.0.8 "$1/lib/libbz2.so"
        ]], "shadow-bzip2", ctx.output})
    end,
}
