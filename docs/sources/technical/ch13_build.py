from build import c, flow, note, p, rif, steps, table, term, tip, warn

S1 = p("Building Phonestra, its component and the manuals requires these tools. The reference system is "
       "Debian 13 “trixie”.", lead=True) + \
    table(["Tool", "What for", "Notes"], [
        ["Stable Rust (2024 edition)", "Everything on the PC", c("cargo") + " lives in " + c("~/.cargo/bin") + ": add it "
         "to the " + c("PATH") + " in commands."],
        ["GTK 4.12+, libadwaita 1.5+, GStreamer 1.x (with " + c("-dev") + ")", "Building and running on the development PC",
         "Required plugins: base (with " + c("opusdec") + "), good, bad (with " + c("openh264dec")
         + ") and " + c("gst-plugin-gtk4") + " (" + c("gtk4paintablesink") + "). No FFmpeg: " + c("gstreamer1.0-libav")
         + " is not needed."],
        ["JDK (" + c("javac") + ")", "Building the component", "Compiled with " + c("--release 11") + "; any recent "
         "JDK will do."],
        ["D8 from R8 9.4.26", "From Java classes to dex", "In " + c("strumenti/r8.jar") + " (not in the repository: URL "
         "and SHA-256 fingerprint in " + c("android/helper/build.sh") + ")."],
        [c("podman"), "Building the AppImage", "Container " + c("phonestra-appimage") + " ("
         + rif("The AppImage container") + ")."],
        ["Python 3 with " + c("pygments") + " and Pillow", "Generating the two manuals", c("python3 docs/sources/build.py")
         + " writes " + c("docs/Technical Manual.html") + " and " + c("docs/User Manual.html")
         + "; packages " + c("python3-pygments") + " and " + c("python3-pil") + " (Pillow is needed for the official logo "
         "on the cover). It is also used by " + c("cargo test") + "."],
    ], "«TAB» — The tools needed on the PC") + \
    p("Testing requires a phone with Android 14 or later, Wireless debugging turned on and the PC on the same Wi-Fi "
      "network.") + \
    note("three things live outside the repository. " + c("strumenti/r8.jar") + " is downloaded with the command written in "
         + c("android/helper/build.sh") + ". The image of the " + c("phonestra-appimage") + " container is rebuilt with "
         + c("podman build") + ". The developer's configuration (" + c("~/.config/Phonestra") + ": ADB key and "
         "paired phones) stays on the PC: on a new PC the phone must be paired again. The AppImages "
         "published on the site are rebuilt from the code.", "What a clone does not bring.")

S2 = p("A few commands cover everyday work; " + c("cargo test") + " also checks that the manuals are "
       "in step with the code.", lead=True) + term("""
$ export PATH=$HOME/.cargo/bin:$PATH
$ cargo build   # program and test tool
$ cargo test   # tests on the PC, without a phone; also checks the manuals
$ cargo clippy --all-targets   # no warnings allowed
$ cargo run --bin phonestra   # runs Phonestra from the sources
$ android/helper/build.sh   # rebuilds the phone component
$ python3 docs/sources/build.py   # regenerates the two manuals
""", "Everyday commands") + \
    tip("change the code → " + c("cargo test") + " (seconds) → if you touched the component, " + c("android/helper/build.sh")
        + " and the " + c("phonestra-prova") + " test of that piece → the real Phonestra on the phone. Before every commit: "
        + c("cargo build") + ", " + c("cargo test") + " and " + c("cargo clippy") + " with no warnings.",
        "The everyday loop.") + \
    warn("the manuals are changed in the sources in " + c("docs/sources/") + ", never in the HTML files in " + c("docs/")
         + ": the next generation would wipe out the change, and " + c("cargo test") + " fails until the published "
         "files match the sources (" + rif("The manuals and their checks") + ").",
         "The manuals are generated.")

S3 = p("The phone component is built outside " + c("cargo") + ", with a script: " + c("javac") + ", then D8. "
       "The resulting jar goes into the repository.", lead=True) + term("""
$ curl -L -o strumenti/r8.jar https://dl.google.com/android/maven2/com/android/tools/r8/9.4.26/r8-9.4.26.jar
$ android/helper/build.sh
creato android/phonestra-helper.jar
""", "Building the component") + \
    p("The script compiles with " + c("javac --release 11") + " the fake Android classes in " + c("stub/") + " (signatures "
      "only: they serve the compiler and do not end up in the jar; the phone has the real ones) and the sources in "
      + c("src/phonestra/") + ", then D8 converts them to dex with " + c("--min-api 34") + " and " + c("--lib")
      + " pointing at the JDK.") + \
    p("When you use a new public Android class, add its stub with only the methods used. Hidden APIs are "
      "called through reflection and usually have no stub; the exceptions are the hidden classes we extend, which the "
      "compiler must know: " + c("android.app.TaskStackListener") + " (" + c("EventiApp.java") + ") and "
      + c("android.content.IOnPrimaryClipChangedListener") + " (" + c("Input.java") + ").") + \
    warn("after the build the jar must be committed together with the sources: the program embeds it with "
         + c("include_bytes!") + ", and an old jar makes the phone run the previous code.", "The jar goes in the commit.")

S4 = p("A single file that starts on all common distributions: it is built on an old glibc and carries GTK, "
       "libadwaita and GStreamer with it, but uses the system's graphics drivers.", lead=True) + \
    p(c("packaging/Containerfile") + " starts from Ubuntu 22.04 (glibc 2.35), so the executable requires at most "
      "glibc 2.34 and starts even on 2022 systems. The GTK and libadwaita of Ubuntu 22.04 are too old: the "
      "container builds into " + c("/opt/phonestra") + " wayland 1.22, wayland-protocols 1.36, glib 2.80, graphene "
      "1.10, GTK 4.14 and libadwaita 1.5 (" + c("packaging/build.sh") + ", meson), plus " + c("gst-plugin-gtk4") + " 0.13.5 from "
      "gst-plugins-rs.") + \
    term("""
$ podman build -t phonestra-appimage packaging
$ podman run --rm -v .:/phonestra:Z phonestra-appimage sh -c 'CARGO_TARGET_DIR=target/appimage cargo build --release && packaging/collect.sh'
""", "Building the AppImage") + \
    warn(c("collect.sh") + " on its own packages whatever executable it finds in " + c("target/appimage/release")
         + ", even an old one: after every code change, " + c("cargo build --release")
         + " must be run again in the container first (on 28 Sep 2026 an old version was packaged).", "Always both steps.") + \
    p("The result is " + c("target/appimage/Phonestra-<versione>-x86_64.AppImage") + "; the version comes from "
      + c("Cargo.toml") + ".")

S5 = p(c("collect.sh") + " takes the executable built in the container and turns it into an AppImage, in five steps.",
       lead=True) + \
    flow([("Executable", "and libs via ldd", "navy"), ("Plugins", "GStreamer, images, GIO", "blue"),
          ("Fallback", "libraries set aside", "blue"), ("patchelf", "$ORIGIN", "blue"),
          ("appimagetool", "static runtime", "light")],
         "«FIG» — The steps of " + c("collect.sh")) + steps([
    "Copies the executable and, recursively with " + c("ldd") + ", the libraries it uses, except those that must come "
    "from the system: glibc, " + c("libstdc++") + ", " + c("libgcc_s") + ", graphics drivers and the libraries that load them (GL, "
    "EGL, DRM, gbm, Vulkan), " + c("libX11") + " and " + c("libxcb") + ", fontconfig, freetype, " + c("libexpat")
    + ", PipeWire, ALSA, udev.",
    "Copies the GStreamer plugins that are needed (" + c("coreelements") + ", " + c("app") + ", "
    + c("typefindfunctions") + ", " + c("playback") + ", " + c("videoparsersbad") + ", " + c("videoconvert") + ", "
    + c("videoscale") + ", " + c("opus") + ", " + c("audioconvert") + ", " + c("audioresample")
    + ", " + c("autodetect") + ", " + c("pulseaudio") + ", " + c("isomp4") + ", " + c("vaapi") + ", " + c("va")
    + ", " + c("nvcodec") + ", " + c("openh264") + ") and " + c("gtk4") + ", with " + c("gst-plugin-scanner")
    + ". There is no FFmpeg (" + c("libav") + ") since version 1.2.0: H.264 is decoded by the graphics card (VA-API, "
    "NVDEC) and otherwise by OpenH264, Opus by libopus.",
    "Copies the image loaders (PNG, JPEG, SVG), the GSettings schemas, the fallback Adwaita icons "
    "and the dconf GIO module: without it, GTK does not read the desktop settings (the window title bar was left "
    "with only the X). Then the licenses, in " + c("usr/share/doc/phonestra/") + ": " + c("LICENSE.md")
    + ", " + c("NOTICE.md") + " and " + c("third-party/") + ", with the Ubuntu copyright file of every package a "
    "copied file comes from, the license texts they point to, those of the libraries built in the container "
    "(saved by " + c("build.sh") + "), the crates' ones (" + c("rust-licenses.py") + ") and the AppImage runtime's.",
    "Moves into " + c("usr/lib/riserva") + " the libraries that the system's drivers also use (wayland, zlib, zstd, "
    "libxml2, libffi, libelf, libva, the " + c("libxcb-*") + " ones…): they are used only if the system lacks them or has "
    "versions that are too old.",
    "Fixes the paths with " + c("patchelf") + " (" + c("$ORIGIN") + "), adds " + c("AppRun") + ", the icon "
    "(" + c("logos/icons/phonestra-256.png") + ") and a " + c(".desktop") + " file with " + c("NoDisplay=true")
    + " (needed by " + c("appimagetool") + ", not installed), and creates the AppImage with the static runtime (no "
    + c("libfuse2") + ").",
])

S6 = p(c("packaging/AppRun") + " prepares the environment and launches " + c("usr/bin/phonestra") + ": the system's "
       "libraries and the bundled ones must not mix.", lead=True) + \
    table(["Setting", "Why"], [
        ["Only the bundled GStreamer plugins, registry in " + c("~/.cache/Phonestra"), "The system's plugins are "
         "built for a different GStreamer."],
        ["Image loaders with the paths of this run", "The loaders file has absolute paths "
         "(" + c("@APPDIR@") + " replaced at startup)."],
        ["No system GIO modules", "Built for a different glib, they break programs."],
        [c("GSK_RENDERER=gl"), "The GTK 4.14 default gets gradients wrong with older Mesa; a value set by the user "
         "wins."],
        ["System icons before ours", "Ours are only a fallback."],
        ["Fallback libraries linked into " + c("~/.cache/Phonestra/riserva") + " only when needed",
         "For example a " + c("libwayland-client") + " older than 1.21."],
    ], "«TAB» — What AppRun sets up")

S7 = p(c("packaging/test-distributions.sh") + " starts the AppImage in Ubuntu 22.04, Debian 12, Fedora 43 "
       "and Arch containers, connected to the PC's Wayland display and graphics card, with an empty configuration: Phonestra "
       "must open “Add a phone”. With " + c("PHONESTRA_FOTO") + " the window is saved as PNG and the script "
       "reports the errors in the log.", lead=True)

S8 = p("A new release is published in four steps, always in the same order.", lead=True) + steps([
    "Version in " + c("Cargo.toml") + " (for release candidates: " + c("1.0.0-rc.N") + "), a section in "
    + c("CHANGELOG.md") + " (in English: it becomes the site's " + c("changes.html") + " page), "
    + c("python3 docs/sources/build.py") + " (the version appears in the manuals), commit and push.",
    "AppImage from the container, startup and connection test, SHA-256 fingerprint.",
    "Publication on " + c("https://phonestra.nicfio.it") + " with " + c("site/publish.sh")
    + ", which takes the AppImage just built or the published one (see the script's header; first " + c("--build") + " alone, to look at " + c("site/public/") + "; before publishing, the search-engine "
    "check: description, canonical, robots, icons, " + c("og.png") + ", structured data, sitemap), then the annotated "
    "git tag " + c("v<versione>") + ".",
    "Copy of the AppImage into the user's home (" + c("~/Phonestra-<versione>-x86_64.AppImage") + "): that is the one "
    "the user runs. Since 1.0.0 the site replaces the GitHub Releases (the repository is private).",
])

CHAPTER = ("Build and release", [
    ("Required tools", S1),
    ("Everyday commands", S2),
    ("Building the component", S3),
    ("The AppImage container", S4),
    ("What collect.sh does", S5),
    ("AppRun", S6),
    ("Testing on distributions", S7),
    ("Publishing a release", S8),
])
