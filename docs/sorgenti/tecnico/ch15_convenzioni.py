from build import c, p, rif, table, term, ul, warn

S1 = p("Phonestra's code reads like its project documents: in Italian, with the why next to the how. The two "
       "manuals are the exception: they are written in English.",
       lead=True) + ul([
    "Italian everywhere else: names, comments, program messages, the documents in the repository (README, "
    "SPECIFICHE, memoria/). Only the text of the two manuals is in English: the pages in docs/ and the chapters "
    "they are generated from in docs/sorgenti/. Plain words, short sentences.",
    "Comments explain the why and point to the source: " + c("SPECIFICHE §7.3") + ", " + c("prove §49") + " ("
    + c("memoria/prove-collegamento.md") + "), " + c("memoria/componente.md") + ".",
    "Rust: " + c("anyhow") + " for errors, with " + c("context") + " saying what was being done; no "
    + c("unwrap") + " where a different phone might answer something else.",
    "Java: one class per piece, reflection only in " + c("Nascoste") + " and in the pieces that use it, errors caught "
    "in the thread that produces them.",
    "Third-party code: never copied. scrcpy and AOSP are read as documentation.",
])

S2 = p("Every commit leaves the project built, tested and documented.", lead=True) + term("""
$ cargo build && cargo test && cargo clippy --all-targets
""") + p("No clippy warnings. If you changed the component, the rebuilt jar goes into the same commit. If you "
         "changed a behavior, update the chapter that describes it in " + c("docs/sorgenti/tecnico/") + " (and, if the user sees it, in " + c("docs/sorgenti/utente/") + ")"
         + " and regenerate the manuals (" + c("python3 docs/sorgenti/build.py") + "); if you added a source "
         "file, give it a row in the map (" + c("ch17_mappa.py") + ", " + rif("Appendix B — File map") + "): "
         + c("cargo test") + " requires it.")

S3 = p("The repository is public: nothing that identifies people, phones or networks.", lead=True) + warn("the repository is public. No names of people, serial numbers, Wi-Fi network names, addresses, "
          "real screenshots that are not blurred. Tests use fake values (for example " + c("R5CT0000000") + "); the files "
          "produced by the tests (" + c("phonestra-prova.*") + ") are ignored by git.", "Personal data.")

S4 = p("The project's why lives in " + c("memoria/") + ", one file per topic. Decisions are written there, "
       "together with their why.", lead=True) + table(["File", "What goes in it"], [
    [c("memoria/prossima-sessione.md"), "Where to pick up again: read at the start of every work session"],
    [c("memoria/decisioni-utente.md"), "The decisions and their why, including the rejected alternatives"],
    [c("memoria/registro-problemi.md"), "Problem → cause → solution → status"],
    [c("memoria/prove-collegamento.md"), "The measurements on the phone, in numbered sections (§)"],
    [c("memoria/componente.md") + ", " + c("adb.md") + ", " + c("api-android.md") + ", " + c("studio/"),
     "Design and study of the component and of the transport"],
    [c("memoria/interfaccia.md"), "User interface rules"],
], "«TAB» — Where decisions are recorded") + \
    p("Before proposing an alternative, check that it has not already been rejected (for example Flatpak, icons in "
      "the system menu, dark themes as a style, several phones active at the same time).")

CHAPTER = ("Conventions", [
    ("Language and style", S1),
    ("Before every commit", S2),
    ("Personal data", S3),
    ("Recording decisions", S4),
])
