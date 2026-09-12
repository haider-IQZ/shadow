return {
    name = "libffi", version = "3.5.2", revision = 1, executables = {},
    source = { url = "https://github.com/libffi/libffi/releases/download/v3.5.2/libffi-3.5.2.tar.gz", sha256 = "f3a3082a23b37c293a4fcd1053147b371f2ff91fa7ea1b2a52e335676bac82dc" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--disable-static", "--disable-docs"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
