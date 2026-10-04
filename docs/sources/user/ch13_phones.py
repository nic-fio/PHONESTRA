from build import note, p, rif, steps, tip, ui, ul, warn


S1 = p("Phonestra can remember several phones, but it uses <b>one at a time</b>. The phone in use is the "
       "“active” one; the others appear in the drawer's sidebar, under " + ui("I MIEI TELEFONI") + " (MY PHONES), marked "
       + ui("non attivo") + " (not active).", lead=True) + steps([
    "In the drawer, press " + ui("Aggiungi telefono") + " (Add phone), in the sidebar.",
    "The " + ui("Aggiungi un telefono") + " (Add a phone) window opens: follow the steps as for the first phone ("
    + rif("Connecting the phone") + ").",
    "At the end the drawer says " + ui("«<nome>» aggiunto: lo trovi in «I miei telefoni»") + " (“&lt;name&gt;” added: you will find "
    "it in “My phones”). The active phone does not change.",
]) + note("Phonestra only shows the phones connected with " + ui("Aggiungi telefono") + ", never those of other "
          "people on the same network.", "Only your own phones.")

S2 = steps([
    "In the sidebar, click the " + ui("non attivo") + " phone (the tooltip says "
    + ui("Passa a <nome>") + ", switch to &lt;name&gt;).",
    "If there are open apps, Phonestra asks for confirmation: " + ui("Chiudere N app di «<attivo>» e passare a «<altro>»?")
    + " (close N apps of &lt;active&gt; and switch to &lt;other&gt;?) with " + ui("Un solo telefono alla volta: le finestre delle app si chiudono.") + " (only one phone at a time: the app windows close). Press " + ui("Passa") + " (Switch).",
    "Phonestra closes the windows, puts the previous phone back as it was and restarts connected to the other phone.",
]) + tip("the phone you choose becomes the one Phonestra opens at the next starts.", "The next start.")

S3 = steps([
    "Click the phone pill, at the top of the drawer, and choose " + ui("Rinomina…") + " (Rename…).",
    "In the " + ui("Rinomina il telefono") + " (rename the phone) window, type the new name and press " + ui("Rinomina") + " (Rename).",
]) + p("The name changes only in Phonestra: the phone's name in its own settings stays as it was. Until you rename it, "
       "Phonestra calls it " + ui("Telefono") + " (Phone), or " + ui("Tablet") + ", in the language of the interface; "
       "with several phones it adds the model, for example " + ui("Telefono · Galaxy S23+") + " (Phone · Galaxy S23+). "
       "It does not use the name the phone gives itself, because that name is in the language of whoever wrote it. "
       "To go back to the default name, rename the phone with an empty name.")

S4 = p("“Forgetting” a phone removes it from Phonestra. This is useful, for example, when you change phones or give "
       "one away.", lead=True) + steps([
    "Click the phone pill and choose " + ui("Dimentica questo telefono…") + " (Forget this phone…).",
    "Read the " + ui("Dimenticare «<nome>»?") + " (forget &lt;name&gt;?) window and press " + ui("Dimentica") + " (Forget).",
    "Phonestra closes, putting the phone back as it was. If other phones remain, it restarts connected to the first "
    "one; otherwise, at the next start " + ui("Aggiungi un telefono") + " (Add a phone) opens.",
]) + warn("forgetting the phone does not disconnect it on the phone's side: the PC stays paired. To remove it, on "
          "the phone: " + ui("Debug wireless") + " › " + ui("Dispositivi associati") + " (Wireless debugging › Paired "
          "devices; or, if it had been connected with the cable, " + ui("Opzioni sviluppatore") + " › "
          + ui("Revoca autorizzazioni debug USB") + ", that is Developer options › Revoke USB debugging "
          "authorizations).",
          "On the phone too.") + \
    p("To connect it again, just use " + ui("Aggiungi telefono") + ".")

CHAPTER = ("Multiple phones", [
    ("Adding another phone", S1),
    ("Switching to another phone", S2),
    ("Renaming a phone", S3),
    ("Forgetting a phone", S4),
])
