return {
    name = "openssl", version = "3.5.2", revision = 1, executables = {},
    source = { url = "https://github.com/openssl/openssl/releases/download/openssl-3.5.2/openssl-3.5.2.tar.gz", sha256 = "c53a47e5e441c930c3928cf7bf6fb00e5d129b630e0aa873b08258656e7345ec" },
    build = function(ctx)
        ctx.run({"./Configure", "--prefix=" .. ctx.prefix, "--libdir=lib", "--openssldir=/etc/ssl", "shared", "no-tests"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install_sw"})
    end,
}
