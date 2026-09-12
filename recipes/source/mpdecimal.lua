return {
    name = "mpdecimal", version = "4.0.1", revision = 1, executables = {},
    source = { url = "https://www.bytereef.org/software/mpdecimal/releases/mpdecimal-4.0.1.tar.gz", sha256 = "96d33abb4bb0070c7be0fed4246cd38416188325f820468214471938545b1ac8" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--disable-cxx"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
