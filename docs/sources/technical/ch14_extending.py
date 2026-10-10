# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import c, code, note, p, rif, steps, warn

S1 = p("A new message touches both sides: the Java class of the part and the Rust module that uses it. These are "
       "the steps, in order.", lead=True) + steps([
    "Choose the number in the part's range (video " + c("0x40–0x4f") + ", input " + c("0x50–0x5f") + ") or open a "
    "new range of 16 for a new part.",
    "<b>Phone</b>: the constant in the part's class (" + c("static final int NOME = 0x47;") + " in "
    + c("Video.java") + "), the " + c("case") + " in the " + c("switch") + " of " + c("Servizio.comandi") + " (or "
    + c("Input.nostro") + " for input), the handler. If it is a request, respond with the same " + c("id") + " and the "
    + c("RISPOSTA") + " flag, or with " + c("ERRORE") + ".",
    "Rebuild the jar with " + c("android/helper/build.sh") + ".",
    "<b>PC</b>: the constant in " + c("componente::tipo") + " (or " + c("input_nostro::tipo") + "), the encoding of the "
    "content as a pure function, the method that uses it (" + c("Condiviso::domanda") + " for a request, "
    + c("Mittente::manda") + " for an event with no response).",
    "Add the name to the list of the " + c("nessun_tipo_usato_da_due_moduli") + " test: it checks that the number is "
    "unique, in the right range and the same in Java and in Rust.",
    "An encoding test with the expected bytes, then a test with " + c("phonestra-prova") + ".",
    "Describe the message in the table of its chapter in " + c("docs/sources/technical/") + ".",
]) + note("an old jar responds " + c("ERRORE") + " " + c("tipo sconosciuto") + " (unknown type) to a new message: the PC must know how to "
          "handle it. Change " + c("PROTOCOLLO") + " only if an existing message changes meaning.", "Compatibility.")

S2 = p("A part with a stream of its own, like audio and video, needs its own channel, opened with the "
       "preamble.", lead=True) + steps([
    "<b>Phone</b>: a handler " + c("static void gestisci(LocalSocket s, String tipo)") + " and its entry in "
    + c("Servizio.TIPI") + ". The handler runs on its own thread: it catches all errors (including the " + c("Error")
    + " of hidden APIs) and closes the socket with " + c("shutdownInput") + "/" + c("shutdownOutput") + " before "
    + c("close") + ", otherwise a read in progress on another thread keeps the socket open.",
    "<b>PC</b>: " + c("Condiviso::apritore().apri(\"tipo\")") + " gives a " + c("Canale") + " with the preamble already sent.",
    "If the part changes something on the phone, register the inverse action with the guardian.",
])

S3 = p("Everything a part changes on the phone must have its inverse action registered with the service's "
       "guardian (" + rif("The two guardians") + ").", lead=True) + code("""
// When you change something on the phone:
Servizio.custode().imposta("mia-azione", 450, "settings put system qualcosa 1");
// When you have put it back yourself:
Servizio.custode().togli("mia-azione");
""", "java", "A new action for the guardian") + \
    p("The command is a shell line run with " + c("sh -c") + " after the service dies, without an Android "
      "context. Choose the order so that the actions happen in the right sequence (panel 400, refresh rate 410, "
      "task 500) and test it with " + c("kill -9") + " on the service (" + c("phonestra-prova shell 'kill -9 <pid>'") + ").") + \
    warn("if the action changes a user setting, restore it only if it still holds Phonestra's value: "
         "the user may have changed it in the meantime (this is the screen timeout rule, prove §56).",
         "The user's value wins.")

CHAPTER = ("Extending Phonestra", [
    ("A new message on the command channel", S1),
    ("A new channel type", S2),
    ("A new action for the guardian", S3),
])
