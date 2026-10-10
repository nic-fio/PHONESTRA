# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import c, key, note, p, rif, steps, table, term, tip, ui, ul


S1 = table(["Requirement", "Details"], [
    ["PC", "64-bit Linux for Intel or AMD processors (x86_64), with a graphical desktop: GNOME, KDE, Xfce, Cinnamon or "
     "similar. Distributions from 2022 onward (for example Ubuntu 22.04, Debian 12, Fedora, Arch)."],
    ["Graphics", "The phone's video is decoded by the graphics card: Intel and AMD through VA-API, whose driver most "
     "distributions install by default; NVIDIA cards with the proprietary driver should work too, but have not been "
     "tried yet. Without it Phonestra decodes the video with the processor: it works, but on slower PCs fast "
     "scenes can be jerky."],
    ["Phone", "Android <b>14 or later</b>, any brand."],
    ["Network", "PC and phone on the <b>same Wi-Fi network</b>, for example your home network. A “guest” network "
     "or the phone's mobile data will not work. The PC may also be connected to the router with a network cable."],
    ["Audio", "The PC's speakers or headphones. Audio goes through the desktop's sound system (PipeWire or PulseAudio)."],
    ["To install", "Nothing: Phonestra is a single file, an AppImage, that contains everything it needs. You don't "
     "need " + c("adb") + " or any other program, nor the administrator password."],
], "«TAB» — Requirements") + \
    note("the phone must be <b>unlocked</b> while you use its apps from the PC: this is an Android security rule. "
         "Phonestra turns the phone's screen off while you use it from the PC and turns it back on at the end ("
         + rif("The phone's screen") + ").", "Unlocked phone.")

S2 = steps([
    "Open Phonestra's site in your browser: " + c("https://phonestra.nicfio.it") + ", section “Get it”.",
    "Download the " + c("Phonestra-<version>-x86_64.AppImage") + " file (the button “Download for Linux”).",
    "Move the file wherever you like, for example to your home folder. Phonestra runs from there: it does not need "
    "to be installed.",
    "Make the file executable, once only: in the file manager, right-click the file, " + "“Properties”" + ", "
    "the " + "“Permissions”" + " tab, the checkbox that allows running it as a program (the exact name varies from one "
    "desktop to another). Or from a terminal, with the command below.",
]) + term("""
$ chmod +x Phonestra-*-x86_64.AppImage
""", "Making the AppImage executable") + \
    tip("next to the file the site offers " + c("SHA256SUMS") + ", with the file's SHA-256 fingerprint. If you want "
        "to check that the downloaded file is intact, put both in the same folder and run "
        + c("sha256sum -c --ignore-missing SHA256SUMS") + ".", "Optional check.")

S3 = p("Start Phonestra by double-clicking the file, or from a terminal:", lead=True) + term("""
$ ./Phonestra-*-x86_64.AppImage
""") + table(["Situation", "What opens"], [
    ["First start: no phone connected so far", "The " + ui("Add a phone") + " window, which guides "
     "you through the first connection (" + rif("Connecting the phone") + ")."],
    ["Phone connected once before, switched on and on the network", "The drawer, Phonestra's main window: it "
     "connects by itself in a few seconds (" + rif("The drawer at a glance") + ")."],
    ["Phone connected before, but not reachable", "The drawer, with " + ui("Connecting to “<name>”…") + " and the notice " + ui("The phone must be on, unlocked and on the same Wi-Fi network.") + " Phonestra keeps looking for it "
     "in the background and connects as soon as it finds it."],
], "«TAB» — What opens at startup") + ul([
    "Phonestra does not add icons to the system menu: to open it again, start the file again.",
    "If Phonestra is already open, starting it a second time does not open a second Phonestra: it brings the drawer "
    "to the front (or reopens it if it had been closed).",
])

S4 = p("Phonestra closes when <b>all</b> of its windows are closed: the drawer and the app windows. "
       "If you close only the drawer, the open apps stay open.", lead=True) + steps([
    "Close the app windows and the drawer, with the " + ui("×") + " at the top right.",
    "Phonestra pauses any music or video the phone was playing, closes the apps open in "
    "windows and puts the phone back the way it was: screen back on, previous screen timeout and volume.",
    "After a few seconds the program exits.",
]) + note("if Phonestra was started from a terminal, " + key("Ctrl", "C") + " in the terminal also closes it in the "
          "same way, putting the phone back in order.", "From the terminal.") + \
    note("if the connection drops suddenly (PC turned off, Wi-Fi lost), the phone puts itself back in order "
         "a few seconds later; anything left over, Phonestra fixes at the next connection ("
         + rif("What changes on the phone") + ").", "Sudden shutdowns.")

S5 = steps([
    "Download the new release, as the first time, and make it executable.",
    "Close Phonestra, if it is open.",
    "Start the new release. The phone, preferences and favorites are kept: they live in "
    + c("~/.config/Phonestra") + ", not in the AppImage file.",
    "Delete the old release's file.",
]) + p("The version in use is shown in " + ui("About") + ", at the bottom of the drawer's sidebar.")

CHAPTER = ("Installation and first start", [
    ("Requirements", S1),
    ("Downloading Phonestra", S2),
    ("Starting Phonestra", S3),
    ("Closing Phonestra", S4),
    ("Updating Phonestra", S5),
])
