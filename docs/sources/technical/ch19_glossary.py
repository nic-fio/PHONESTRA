from build import c, dl, p, rif


def v(titolo):
    return " See " + rif(titolo) + "."


A_L = [
    ("adbd", "The phone's ADB daemon: it accepts connections from the PC and starts the services (" + c("shell") + ", "
     + c("sync:") + ", " + c("localabstract:") + ")." + v("The ADB client")),
    ("Annex B", "The H.264/H.265 stream format with the start code " + c("00 00 00 01") + " in front of every unit."
     + v("The video:<id> channel")),
    ("app_process", "The Android program that starts Java code outside an app; the shell uses it too."
     + v("What the component is")),
    ("ART", "Android Runtime: the virtual machine that runs the dex."),
    ("AudioPolicy", "Hidden API for routing audio; with loopback it sends the apps' sound to a recorder "
     "instead of the loudspeaker." + v("The recipe")),
    ("Channel", "A logical connection inside the ADB connection, to a service on the phone; all channels "
     "share the same connection." + v("Channels and flow control")),
    ("Component", "Everything Phonestra runs on the phone: the jar " + c("phonestra-helper.jar") + "."
     + v("The on-phone component")),
    ("Connection", "The active phone and its ADB connection (" + c("collegamento.rs") + "), kept "
     "alive as long as Phonestra stays open." + v("Life of a connection")),
    ("Delayed ack", "ADB extension with more data in flight per channel and acknowledgements that carry the "
     "number of bytes received." + v("Channels and flow control")),
    ("dex", "The format of compiled Java code for Android (" + c("classes.dex") + "), produced by D8."
     + v("Building the component")),
    ("Drawer", "Phonestra's main window with the phone's apps (" + c("cassetto.rs") + ")."
     + v("The drawer")),
    ("Guardian", "A shell process that puts the phone back in order when whatever it is tied to ends."
     + v("The two guardians")),
    ("Heartbeat", "The " + c("BATTITO") + " message, sent every second in both directions: after 5 s of "
     "silence each side considers the other gone." + v("Heartbeat and exit codes")),
    ("Helper", "The component's jar when used for short commands (app list, wallpaper, thumbnails, "
     "measurements)." + v("What the component is")),
    ("In hand", "Panel state: the user is using the phone with their hands, so the panel stays on."
     + v("The phone in hand")),
    ("Loopback", "Capture of the audio coming out of the apps, while the phone itself stays silent." + v("The recipe")),
]

M_Z = [
    ("mDNS", "DNS on the local network without a server: this is how the phone announces Wireless debugging."
     + v("Network discovery: mDNS")),
    ("Mirror", "The copy of the phone's main screen, drawn in the drawer."
     + v("Sessions: virtual display and mirror")),
    ("Panel", "The phone's physical screen, turned off while the apps are used from the PC."
     + v("Turning the panel off")),
    ("Preamble", "The first bytes of every channel of the service: the secret and the channel type."
     + v("Channels and preamble")),
    ("3-second round", "The periodic check of the connection: lock, calls and notifications in a "
     "single command." + v("The 3-second round")),
    ("Service", "The component's long-running process, one per connection (" + c("phonestra-servizio")
     + ")." + v("Starting the service")),
    ("Session", "A virtual display or the mirror, with its " + c("video:<id>") + " channel."
     + v("Sessions: virtual display and mirror")),
    ("Surface", "The graphics buffer the virtual display draws into and the encoder reads from."
     + v("Encoder and keyframe")),
    ("uid 2000", "The ADB shell user: its permissions are the component's permissions."
     + v("Service security")),
    ("Virtual display", "An extra display, created by the component, where the app of a window "
     "runs." + v("Sessions: virtual display and mirror")),
    ("Wireless debugging", "ADB over Wi-Fi in Android 11+, with TLS and pairing by code."
     + v("Wi-Fi connection and TLS")),
]

CHAPTER = ("Glossary", [
    ("Terms A–L", p("Definitions of the technical terms used in the manual. The code of Phonestra is written in "
                    "Italian: names from the code appear in monospace, as they are written in the sources. The "
                    "cross-references point to the section with the details.", lead=True) + dl(A_L, "gloss")),
    ("Terms M–Z", dl(M_Z, "gloss")),
])
