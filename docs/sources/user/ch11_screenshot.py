from build import c, key, note, p, rif, steps, table, tip, ui, ul


S1 = p("A screenshot captures the app in a window at the phone's resolution, without the window's title bar.",
       lead=True) + table(["Command", "Where", "Result"], [
    [ui("Screenshot (salvato e copiato)") + " (Screenshot (saved and copied))", "Button with the camera icon in the window's title bar",
     "Saves the image in " + c("Immagini/Phonestra") + " and copies it to the PC's clipboard: "
     + ui("Screenshot in Immagini/Phonestra e negli appunti") + " (Screenshot in Pictures/Phonestra and in the clipboard)."],
    [ui("Copia screenshot") + " (Copy screenshot)", ui("Altri comandi") + " (More commands, ⋮), or " + key("Ctrl", "Shift", "C"), "Copies it only to the "
     "clipboard: " + ui("Screenshot copiato negli appunti") + " (Screenshot copied to the clipboard)."],
], "«TAB» — Screenshots") + ul([
    "The file is named after the app, the date and the time: for example " + c("Mappe 2026-09-30 10.42.05.png") + ".",
    "The image on the clipboard can be pasted right away into another PC program (an email, a chat, a document).",
    "If the app does not show anything yet, " + ui("Nessuna immagine da fotografare") + " (No image to capture) appears.",
]) + note("a protected screen (" + rif("Protected screens") + ") comes out black in the screenshot too.",
          "Protected screens.")

S2 = steps([
    "In the app's window press " + ui("Registra lo schermo") + " (Record the screen; the button with the red dot).",
    "The button turns into a red pill with the elapsed time, for example " + ui("● 0:42") + ".",
    "To stop, press the button again: " + ui("Registrazione salvata in Video/Phonestra") + " (Recording saved in Video/Phonestra) appears.",
]) + ul([
    "The video is an MP4 file in " + c("Video/Phonestra") + ", named after the app, the date and the time, for example "
    + c("Mappe 2026-09-30 10.42.05.mp4") + ".",
    "The video is saved exactly as it arrives from the phone, without re-encoding: it is small and does not slow down the PC.",
    "It includes audio too, which is the audio of the whole phone, not just of the app.",
    "During recording the app does not change shape: if you resize the window the image adapts, but the video "
    "keeps its initial size. " + ui("Ruota") + " (Rotate) does not work.",
    "If you close the window during a recording, the video is saved before the window closes.",
]) + tip("to record with audio, it is best to wait a few seconds after connecting: audio starts shortly after "
         "the picture (" + rif("Audio from the PC") + ").", "With audio.")

CHAPTER = ("Screenshots and recording", [
    ("Taking a screenshot", S1),
    ("Recording the screen", S2),
])
