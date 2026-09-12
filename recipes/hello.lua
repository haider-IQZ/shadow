-- Trusted local recipe. Build commands are NOT sandboxed.
return {
    name = "hello",
    version = "1.0.0",
    revision = 1,
    build = function(ctx)
        ctx.run({"sh", "-eu", "-c", [[
            mkdir -p "$1/bin"
            "${CC:-cc}" -O2 -Wall -Wextra -Werror "$SHADOW_RECIPE_DIR/hello.c" -o "$1/bin/hello"
        ]], "shadow-build", ctx.destdir})
    end,
}
