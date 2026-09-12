return {
    name = "freetype", version = "2.14.1", revision = 2, executables = {}, dependencies = {"zlib.lua", "libpng.lua"},
    source = { url = "https://download.savannah.gnu.org/releases/freetype/freetype-2.14.1.tar.xz", sha256 = "32427e8c471ac095853212a37aef816c60b42052d4d9e48230bab3bdf2936ccc" },
    build = function(ctx)
        ctx.run({"./configure", "--prefix=" .. ctx.prefix, "--disable-static", "--with-harfbuzz=no", "--with-bzip2=no", "--with-brotli=no", "--with-zlib=yes", "--with-png=yes"})
        ctx.run({"make", "-j" .. ctx.jobs})
        ctx.run({"make", "DESTDIR=" .. ctx.destdir, "install"})
        ctx.run({"mkdir", "-p", ctx.output .. "/share/licenses/freetype"})
        ctx.run({"cp", "docs/FTL.TXT", "docs/GPLv2.TXT", ctx.output .. "/share/licenses/freetype/"})
    end,
}
