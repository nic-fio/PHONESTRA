from build import note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("The drawer's " + ui("Notifiche") + " (Notifications) page shows the phone's notifications, newest first. The number "
       "next to " + ui("Notifiche") + " in the sidebar says how many there are.", lead=True) + ul([
    "Notifications are grouped by app, in cards showing the app's name and how many notifications it has.",
    "For each app you see the last 2; the " + ui("altre N notifiche di <app> ›") + " (N more notifications from &lt;app&gt;) button shows the others.",
    "Each notification has an icon, title, text and time. Clicking a notification opens the app in its window.",
    "The " + ui("×") + " next to a notification hides it: " + ui("Nascondi (sul telefono resta)") + " (Hide — it stays on the phone). "
    + ui("Nascondi tutte") + " (Hide all), at the top, hides them all.",
    "With no notifications the page says " + ui("Nessuna notifica") + " (No notifications).",
    "Notifications refresh every few seconds, as long as the phone is connected.",
]) + note("hiding a notification removes it only from Phonestra: it stays on the phone. If the app updates it (for "
          "example a new message arrives in the same chat), it reappears.", "Hiding does not delete.") + \
    p("Ongoing notifications, the ones that stay while an app is working (for example music "
      "playing or a navigation app), do not appear, nor do notifications with no title and no text.")

S2 = p("When a new notification arrives on the phone, Phonestra shows a system alert, like those of the PC's other "
       "programs: it appears in the corner of the screen and stays in the desktop's notification list.",
       lead=True) + \
    table(["Part of the alert", "Content"], [
        ["Program", "Phonestra"],
        ["Icon", "The icon of the phone app"],
        ["Title", "The app's name and the notification's title (for example the sender)"],
        ["Text", "The notification's text"],
        ["Action", ui("Apri") + " (Open), or a click on the alert: opens the app in its window"],
    ], "«TAB» — Anatomy of an alert") + ul([
        "Alerts respect the desktop's “Do Not Disturb”.",
        "Alerts arrive only while Phonestra is open and connected to the phone.",
    ]) + warn("when the drawer opens, the notifications already on the phone may each produce an alert, "
              "as if they were new. This is a known issue: later alerts only concern new notifications.",
              "Alerts at startup.")

S3 = p("In the " + ui("Preferenze") + " (Preferences) page, " + ui("NOTIFICHE") + " (Notifications) tab, you choose what the "
       "alerts show.", lead=True) + \
    table(["Preference", "What it does", "Default"], [
        [ui("Avviso a comparsa") + " (Pop-up alert)", ui("Un avviso del sistema quando arriva una notifica sul telefono.") + " (A system alert when a notification arrives on the phone.) Off: "
         "notifications stay only in the " + ui("Notifiche") + " (Notifications) page.", "on"],
        [ui("Solo il nome dell'app") + " (App name only)", ui("Negli avvisi niente mittente né testo: utile se altri vedono il tuo "
         "schermo.") + " (No sender and no text in the alerts: useful if others see your screen.) The alert just says the app's name and " + ui("Nuova notifica") + " (New notification).", "off"],
        [ui("App che possono avvisare") + " (Apps that can alert)", ui("Scegli da quali app ricevere gli avvisi.") + " (Choose which apps to get alerts from.) The button says "
         + ui("tutte ›") + " (all) or " + ui("tutte tranne N ›") + " (all but N).", "all"],
    ], "«TAB» — Alert preferences") + steps([
        "Open " + ui("Preferenze") + " (Preferences) in the sidebar.",
        "In the " + ui("NOTIFICHE") + " (Notifications) tab, press the button next to " + ui("App che possono avvisare") + " (Apps that can alert).",
        "In the " + ui("App che possono avvisare") + " window, turn off the switch of the apps that should not "
        "produce alerts. The choice is saved immediately.",
    ]) + tip("apps muted here keep appearing in the " + ui("Notifiche") + " (Notifications) page: the choice applies only "
             "to pop-up alerts.", "Muted, not hidden.") + \
    note("Phonestra reads the title and text of all the phone's notifications, including those the phone hides "
         "on the lock screen. They stay on the PC, in Phonestra (" + rif("Privacy and security") + ").",
         "Private notifications.")

CHAPTER = ("Notifications", [
    ("The “Notifiche” page", S1),
    ("Pop-up alerts", S2),
    ("Choosing the alerts", S3),
])
