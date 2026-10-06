from build import note, p, rif, table, tip, ul, warn

S1 = p("While Phonestra is connected, the audio of the phone's apps comes out of the PC's speakers (or headphones). "
       "The phone stays silent.", lead=True) + ul([
    "This applies to all apps: music, videos, games, notification sounds, navigation.",
    "Audio starts a few seconds after the connection: it may be missing in the first moments.",
    "Audio and video stay in sync, recordings included (" + rif("Recording the screen") + "). While an app is "
    "playing sound, the picture waits for its sound: it appears a fraction of a second later than on the phone. "
    "When nothing is playing, the picture follows the mouse and keyboard right away.",
    "When Phonestra closes, any music or video that was playing is paused: the phone does not start playing again "
    "on its own through its loudspeaker.",
]) + note("some apps forbid capturing their audio. Their sound does not reach the PC: while Phonestra is connected "
          "they stay silent.", "Apps that forbid capture.")

S2 = p("Volume is adjusted on the PC, as for any other program: with the keyboard's volume keys or with the "
       "desktop mixer, where Phonestra shows up as a program playing sound.", lead=True) + \
    p("During the connection Phonestra turns the phone's media volume all the way up: some apps do not start their "
      "audio when the phone's volume is at zero. The phone still makes no sound, because the audio comes out only "
      "from the PC. When it closes, Phonestra restores the previous volume.") + \
    warn("in some cases, when Phonestra closes, the phone's media volume may stay at maximum. This is a known issue: "
         "if it happens, just lower it with the phone's keys.", "Volume left high.") + \
    tip("the phone's volume keys have no effect while you use Phonestra: what counts is the PC's volume.",
        "Phone keys.")

S3 = p("Calls stay on the phone: you answer and talk with the phone in hand.", lead=True) + \
    table(["Type of call", "Where you hear the voice", "Where you speak"], [
        ["Regular phone call", "From the phone", "Into the phone's microphone"],
        ["Call or video call of an app (for example WhatsApp) open in a Phonestra window",
         "From the phone", "Into the phone's microphone"],
    ], "«TAB» — Calls with Phonestra") + ul([
        "When a call comes in, Phonestra turns the phone's screen back on, so you can answer from the "
        "phone (" + rif("Incoming calls") + ").",
        "Android does not allow the voice of calls to be carried to the PC: even with Phonestra you hear it from the phone.",
        "The PC's microphone does not reach the phone.",
    ])

S4 = p("Apps open in a Phonestra window use the camera and microphone <b>of the phone</b>, as "
       "always.", lead=True) + ul([
    "A video call open in a window works: you see it on the PC, but it films and listens from the phone.",
    "To take photos with the " + "Camera" + " app in a window, point the phone and click to shoot.",
    "The phone does not become a webcam or a microphone for the PC's programs.",
])

CHAPTER = ("Audio, calls and camera", [
    ("Audio from the PC", S1),
    ("Volume", S2),
    ("Calls", S3),
    ("Camera and microphone", S4),
])
