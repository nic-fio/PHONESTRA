# Phonestra landing page — brief for the mockups

Twenty mockups of the same landing page, **same content, twenty visual directions**, for
the owner to choose one. The chosen one becomes the page at `https://phonestra.nicfio.it`.
The method is the one used for NESH and EFI Partition Manager on 2 October 2026.

## Rules (all mockups)

- One **self-contained HTML file** per mockup: `site/mockups/NN-slug.html` (in
  `/home/nicfio/Documenti/PHONESTRA/`). No JavaScript, no external requests (no CDN, no
  Google Fonts, no remote images): CSS in a `<style>`, images inline as `data:` URIs, fonts
  embedded as `data:` URIs from `/usr/share/fonts`, subset with
  `/tmp/claude-1000/-home-nicfio/0e80c400-428a-4147-98e6-10e95186e0f4/scratchpad/ftv/bin/pyftsubset`
  to Basic Latin plus the Italian letters `àèéìòù` and `·—–’“”«»→←©×…›‹⋮✓`; or system font
  stacks. Local fonts: Cantarell, Quicksand, URW Gothic, P052, C059, URW Bookman, Nimbus
  Sans, Nimbus Sans Narrow, DejaVu Sans/Serif/Sans Mono, Liberation Sans/Serif/Mono, Noto
  Mono (look in `/usr/share/fonts` for others, e.g. Inter or Noto Sans, if present). They
  are for exploring only: the real site will use fonts licensed for the web. Each file
  under 1.5 MB.
- English only. Elegant, professional, appealing — a page an ordinary Linux user trusts
  and wants to try. Responsive: it must look right at 1440 px and at 390 px (phone), with
  no horizontal scrolling of the page.
- **The logo** (never redrawn, never recoloured), in `site/mockups/_shared/`:
  `phonestra-logo-horizontal-light.png` (700×210: the symbol and «Phonestra» in dark navy,
  transparent background, for light backgrounds), `phonestra-logo-horizontal-dark.png`
  (the same with white text, for dark backgrounds), `phonestra-logo-transparent.png` and
  `phonestra-logo-dark-background.png` (700×627, square layout, symbol above the word), and
  `phonestra-symbol.png` (400×400, only the symbol: a phone circled by a ring, with pixels
  flying off its corner; works on light and dark). Logo colours: cyan #19c3f0 → blue
  #2a6cf5 → violet #7a3cf0, text navy #0d1340. The palette of the page is free.
- **No real screenshots exist that can be shown** (they would show someone's apps and
  notifications). The visuals are **illustrations of Phonestra's interface**, drawn by you
  in HTML/CSS/inline SVG: a Linux desktop with the drawer and one or two app windows. Base
  them on the reference images in `site/mockups/_shared/reference/` (the interface
  designs the program was built from: `drawer-glass.png` the drawer, `app-window.png` app
  windows, `drawer-glass-notifications.png`, `receive-files.png`, `guide-list-4.png` the
  first-connection window). Do **not** embed the reference PNGs themselves (one still
  shows an old provisional name in its title bar: the window title is **Phonestra**).
  Use the generic app icons in `site/mockups/_shared/*.svg` (bank, calendar, camera,
  email, maps, messages, music, notes, photos, settings, weather…); **never** the logos of
  real apps or brands (no WhatsApp, Google, Samsung… logos), and call the phone
  «Telefono», not a brand model. Invent harmless app content (a chat, a map, a playlist),
  never real people's data. A small caption such as «Illustration» is welcome.
- **The interface of the program is in Italian.** The page is in English; where it shows
  or quotes the interface, use the Italian labels exactly as below, with the English in
  parentheses or in a caption when it helps. Say it plainly somewhere visible: «The
  interface is in Italian; the manuals are in English.»
- **Only true statements.** Use the facts below; do not invent features, users,
  testimonials, numbers, benchmarks, ratings, logos of companies, or claims of support
  (no «works with every phone», no «Windows/macOS»). Do not write «fast», «lightweight»,
  «seamless», «zero latency», «production-ready», «battle-tested», «trusted by».
- **An independent project:** do not mention any other project of the author, or other
  programs that do similar things. Only `https://nicfio.it` may appear, small, as the
  author's site in the footer.

## What Phonestra is

**Phonestra**: a Linux program that brings the apps of your Android phone to your PC.
Each app opens in its own window, like a PC program, and is used with mouse and keyboard.
The phone stays on the desk: it only needs to be on, unlocked and on the same Wi-Fi
network as the PC. **No app to install on the phone**, no cable needed. The apps keep
running on the phone, with their data and their accounts: Phonestra shows their picture
on the PC and sends clicks and keystrokes to the phone. The name: *phone* + *finestra*
(Italian for «window»).

For people who use Linux on their PC and an Android phone, and want the phone's apps —
chats, music, maps, everyday apps — on the big screen with a real keyboard.

Tagline ideas (choose or adapt, keep them true): «Your phone's apps, in Linux windows.» —
«Android apps on your Linux desktop, over Wi-Fi.» — «Nothing to install on the phone.» —
«The phone stays on the desk.»

## Highlights (all true today, version 1.0.0-rc.8)

- **Each app in its own window**: several apps at once, ordinary desktop windows — move
  them, tile them side by side, full screen; they appear in the taskbar with the app's
  name. Click = tap, wheel = scroll, Esc = Back.
- **Over Wi-Fi, nothing installed on the phone**: it uses Android's own *Wireless
  debugging*, turned on once. A small program is copied to a temporary folder on the
  phone at every connection and deleted at the end.
- **The drawer**: Phonestra's main window, with the phone's apps (search, favourites, all
  apps), the notifications, and the phone's real screen, live, usable with the mouse
  (home screen, notification shade, quick settings, widgets).
- **Sound on the PC**: the audio of the phone's apps comes out of the PC's speakers or
  headphones, in sync with the picture; the phone stays silent.
- **Keyboard and clipboard**: type with the PC keyboard; copy and paste text in both
  directions (passwords marked as sensitive are not passed).
- **Notifications on the PC**: on the «Notifiche» page and as desktop alerts (with an
  option to show only the app's name).
- **Files both ways**: drag files onto the drawn phone to send them; «Ricevi file…»
  browses the phone and copies files to the PC; drop an `.apk` to install an app.
- **Screenshots and recording**: a screenshot of an app is saved and copied to the
  clipboard; screen recording of an app, with its audio.
- **The phone's screen turns off** while you use it from the PC (battery, privacy) and
  turns back on at the end — and when a call comes in.
- **Everything put back**: screen timeout, media volume, audio, the apps opened in
  windows — all restored when Phonestra closes, even after a sudden disconnection.
- **Private**: no internet, no account, no advertising, no usage statistics: Phonestra
  only talks to the phone on the home network, over an encrypted connection.
- **Protected screens stay protected**: screens that apps protect (banking, passwords,
  paid video) never reach the PC; Phonestra shows «Schermata protetta» instead and does
  not get around it.
- **One file**: an AppImage (about 97 MB) with everything inside; no `adb` or other
  program to install, no administrator password.

## Exact interface texts (Italian, use these verbatim)

The drawer: title bar «Phonestra»; the phone pill in the middle: «Telefono · collegato via
Wi-Fi» (green dot). Sidebar: «App», «Notifiche» (with a count), heading «TELEFONI» with
«Telefono» «attivo» and «+ Aggiungi telefono», heading «STRUMENTI» with «Installa app…»,
«Invia file…», «Ricevi file…», then «Preferenze», «Informazioni». The page: search box
«Cerca un'app», sections «PREFERITI» and «TUTTE LE APP». On the right, the drawn phone
with the phone's live screen.

App window: a top bar with «‹» (Back), the app's name and below it «Telefono», a
screenshot button, a record button, «⋮» with «Ruota» (Ctrl+R), «Copia screenshot»
(Ctrl+Maiusc+C), «Chiudi app» (Ctrl+W), and the window's ×.

Messages (verbatim): «Il telefono deve essere acceso, sbloccato e sulla stessa rete
Wi-Fi.» — «Schermata protetta» / «Questa app non permette di mostrare questa schermata
fuori dal telefono.» — «Collegamento perso» / «Riprovo da solo in sottofondo.»

First connection, the window «Aggiungi un telefono», heading «Prepara il telefono», four
items: 1 «Telefono e PC sulla stessa rete Wi-Fi», 2 «Sblocca le Opzioni sviluppatore»,
3 «Spegni le protezioni che bloccano il collegamento», 4 «Debug wireless: accendilo e
associa questo PC». Then the 6-digit pairing code shown by the phone. Done once per phone;
afterwards Phonestra connects by itself in a few seconds.

## How it works (for a small diagram, three steps)

1. **Once, on the phone**: Developer options → Wireless debugging on, pair the PC with the
   6-digit code («Aggiungi un telefono» guides you step by step; a cable route exists for
   phones that need it).
2. **Start Phonestra** on the PC: it finds the phone on the Wi-Fi network and connects
   (encrypted).
3. **Open apps in windows**: picture and sound to the PC; clicks, keys, files to the phone.

## Requirements (exact)

- PC: 64-bit Linux on Intel or AMD (x86_64) with a graphical desktop (GNOME, KDE, Xfce,
  Cinnamon or similar); distributions from 2022 on (for example Ubuntu 22.04, Debian 12,
  Fedora, Arch).
- Phone: **Android 14 or later**, any brand.
- Network: PC and phone on the same Wi-Fi network (not a «guest» network, not mobile
  data); the PC may be on a cable to the router.

## Limits — say them honestly (keep them visible, not hidden)

- The phone must stay on, unlocked and on the same Wi-Fi network.
- Calls are heard and made on the phone (Android does not carry call audio to the PC);
  the PC's microphone and webcam do not reach the phone.
- One phone active at a time (several can be paired, and you switch between them).
- Protected screens stay black on the PC, by design; apps that forbid audio capture stay
  silent.
- Wireless debugging must be turned on by hand on the phone, once: Android does not let a
  program do it.

## Status — say it honestly

Version **1.0.0-rc.8**, a release candidate for 1.0. Used every day by the author, and
tried so far on two Samsung Galaxy phones with Android 14 and later; the AppImage starts
on Ubuntu 22.04, Debian 12, Fedora 43 and Arch. Other brands are expected to work (it uses
only standard Android features) but have not been tried yet. **Call to action**: «Tried it
with your phone? Send a report» → `mailto:phonestra@nicfio.it` (subject «Phonestra
report»): phone model, Android version, Linux distribution, what worked and what did not.
A phone where everything worked is as useful a report as one where nothing did.

## Get it

- **Download** `Phonestra-1.0.0-rc.8-x86_64.AppImage` (about 97 MB) →
  `download/Phonestra-1.0.0-rc.8-x86_64.AppImage`; checksums `download/SHA256SUMS`;
  licence files `download/LICENSE.txt`, `download/NOTICE.txt`.
- Then, once (exact):
  ```
  $ chmod +x Phonestra-*-x86_64.AppImage
  $ ./Phonestra-*-x86_64.AppImage
  ```
  or make it executable from the file manager (Properties › Permissions) and
  double-click it. At the first start the window «Aggiungi un telefono» guides you.
- Check the download: `sha256sum -c --ignore-missing SHA256SUMS`.

## Documentation

User Manual (`User%20Manual.html`): installation, the first connection, everyday use,
troubleshooting. Technical Manual (`Technical%20Manual.html`): architecture, the
component on the phone, video, audio, input, the AppImage. Licence (`licence.html`).

## Licence (free)

**Free for noncommercial use.** PolyForm Noncommercial License 1.0.0, from the version after
1.3.0: personal use, study, hobby projects, schools, charities and public bodies. Commercial
use, including use at work inside a company, needs a written licence: phonestra@nicfio.it.
Small print: «Versions 1.0.0 to 1.3.0 keep the Phonestra Freeware Licence, versions up to
1.0.0-rc.8 their personal-use licence.» No prices, no «Buy», no «Pro».

## Sections (in this order, adapt the layout)

1. Header: logo, links (Features, How it works, Get it, Docs, Licence), a «Download»
   button (#get).
2. Hero: tagline, one sentence, buttons «Download for Linux» (#get) and «Read the manual»,
   and the illustration: a Linux desktop with the drawer and an app window.
3. How it works: the three steps (a small diagram: phone ⇄ Wi-Fi ⇄ PC).
4. Features (the highlights).
5. A closer look: two or three illustrations with captions (the drawer; an app window with
   its «⋮» menu; the «Aggiungi un telefono» window or «Ricevi file…»).
6. Privacy and the phone: nothing installed, everything put back, no internet, protected
   screens stay protected.
7. Requirements and honest limits.
8. Status and the report call to action.
9. Get it: download, the two commands, checksum.
10. Documentation.
11. Licence (short).
12. Footer: «Phonestra · Version 1.0.0-rc.8 · © 2026 Nicola Fiorillo», links to the
    manuals and the licence, a small link to nicfio.it.

## Checking your work

Render each page with Chrome headless and LOOK at the images before delivering:
`google-chrome --headless=new --disable-gpu --hide-scrollbars --window-size=1440,8000 --screenshot=OUT.png file:///path/NN-slug.html`
and `--window-size=390,14000`. Fix overflow, overlaps, contrast, cut text, empty areas,
illustrations that break on phones (scale them down or simplify them there). Save a
preview for the gallery: `site/mockups/previews/NN-slug.png` (`--window-size=1440,900`).
Write only in `site/mockups/` and in your own scratch subfolder
`/tmp/claude-1000/-home-nicfio-Documenti-PHONESTRA/37c861f8-3a06-470a-a799-de35916ec6a5/scratchpad/pm-<letter>/`.
