return {
    name = "xz", version = "5.8.4", revision = 1, executables = {},
    source = { url = "https://github.com/tukaani-project/xz/releases/download/v5.8.4/xz-5.8.4.tar.gz", sha256 = "0014c7886930454fe8bd4228665b51af55eeae560ea135c9c4cd33f55b2591d9" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--disable-static", "--disable-nls", "--disable-doc", "--disable-xz", "--disable-xzdec", "--disable-lzmadec", "--disable-lzmainfo", "--disable-scripts"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
