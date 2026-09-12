# Development toolchain/graphics bridge only: application libraries are built by Shadow.
FROM archlinux@sha256:61f7de2dd88cc4ba1fe36c24cfe1a503c3936984492d6405eeab013ce6ac68c5
RUN pacman -Syu --noconfirm --needed \
    rust python go cmake meson ninja patchelf curl git gperf \
    dbus libx11 libxrandr libxi libxinerama libxcursor libxkbcommon-x11 \
    wayland wayland-protocols libglvnd ncurses \
    && useradd --create-home builder
USER builder
ENV HOME=/work
WORKDIR /workspace
