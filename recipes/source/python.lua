return {
    name = "python", version = "3.13.7", revision = 2, executables = {}, dependencies = {"zlib.lua", "openssl.lua", "libffi.lua", "expat.lua", "mpdecimal.lua", "bzip2.lua", "xz.lua"},
    source = { url = "https://www.python.org/ftp/python/3.13.7/Python-3.13.7.tar.xz", sha256 = "5462f9099dfd30e238def83c71d91897d8caa5ff6ebc7a50f14d4802cdaaa79a" },
    build = function(ctx)
        -- This private interpreter is for Kitty, not a general-purpose Python SDK.
        ctx.run({"sh", "-eu", "-c", [[
            printf '%s\n' '*disabled*' '_sqlite3 _curses _curses_panel readline _tkinter _dbm _gdbm _uuid' > Modules/Setup.local
        ]]})
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--enable-shared", "--with-ensurepip=no", "--with-system-expat", "--with-openssl=" .. ctx.deps.openssl})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
    end,
}
