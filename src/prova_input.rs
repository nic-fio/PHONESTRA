//! Prove del modulo input del componente sul telefono, senza bisogno
//! dell'utente: `phonestra-prova input-componente <prova> [opzioni]`.
//!
//! - `appunti`: scrittura e lettura, niente avviso per le scritture di
//!   Phonestra, avviso per le copie «di un'altra app», copia sensibile senza
//!   testo, ascolto spento;
//! - `tocchi`: su uno schermo virtuale con le Impostazioni, rotellina,
//!   trascinamento (con coordinate da scalare), pizzico, tocco che apre una
//!   voce, «indietro», evento con misura vecchia scartato;
//! - `testo`: in un campo di testo (ricerca di Google, o `--app`/`--azione`):
//!   testo ASCII, Ctrl+A e Ctrl+C, poi incolla di lettere accentate, verificati
//!   rileggendo gli appunti;
//! - `tutte`: le tre di seguito.
//!
//! Gli effetti si verificano con la «firma» dello schermo di prova (immagine
//! rimpicciolita a 32×56 riquadri), con `dumpsys activity activities` e con gli
//! appunti. Alla fine, sempre: appunti dell'utente rimessi, schermo chiuso,
//! servizio chiuso, controllo che non resti niente. Il testo degli appunti
//! dell'utente non viene mai stampato.

use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};

use crate::adb::Adb;
use crate::componente::{self, Componente, Processo};
use crate::input_nostro::{self, Appunti, GIU, InputNostro, MOVIMENTO, SU, tipo};

pub const USO: &str = "input-componente appunti|tocchi|testo|tutte [--misura LxA] [--dpi D] [--app PACCHETTO] \
                       [--azione AZIONE[:PACCHETTO]] [--tocca X,Y]";

/// `KeyEvent.META_CTRL_ON | META_CTRL_LEFT_ON`.
const CTRL: u32 = 0x3000;
const TASTO_A: u32 = 29;
const TASTO_C: u32 = 31;
const TASTO_CANC: u32 = 67;
/// L'app Google, che apre la ricerca col campo di testo già attivo.
const RICERCA: &str = "com.google.android.googlequicksearchbox";
/// Differenza media di luminosità (in % del massimo) oltre la quale la schermata è cambiata.
const CAMBIATA: f32 = 1.5;

struct Opzioni {
    prova: String,
    misura: (u16, u16),
    dpi: u32,
    app: Option<String>,
    azione: Option<String>,
    tocca: Option<(i32, i32)>,
}

impl Opzioni {
    fn leggi(argomenti: &[String]) -> Result<Self> {
        let mut o = Self { prova: String::new(), misura: (720, 1280), dpi: 320, app: None, azione: None, tocca: None };
        let mut i = argomenti.iter();
        while let Some(a) = i.next() {
            let mut valore = || i.next().cloned().ok_or_else(|| anyhow!("manca il valore di {a}"));
            match a.as_str() {
                "--misura" => {
                    let v = valore()?;
                    let (l, h) = v.split_once('x').ok_or_else(|| anyhow!("misura come 720x1280"))?;
                    o.misura = (l.parse()?, h.parse()?);
                }
                "--dpi" => o.dpi = valore()?.parse()?,
                "--app" => o.app = Some(valore()?),
                "--azione" => o.azione = Some(valore()?),
                "--tocca" => {
                    let v = valore()?;
                    let (x, y) = v.split_once(',').ok_or_else(|| anyhow!("--tocca X,Y in pixel dello schermo di prova"))?;
                    o.tocca = Some((x.parse()?, y.parse()?));
                }
                p if o.prova.is_empty() && !p.starts_with("--") => o.prova = p.to_string(),
                altro => bail!("argomento sconosciuto: {altro} (uso: {USO})"),
            }
        }
        if !["appunti", "tocchi", "testo", "tutte"].contains(&o.prova.as_str()) {
            bail!("uso: {USO}");
        }
        Ok(o)
    }
}

/// Stato della prova: cosa rimettere a posto e l'esito dei controlli.
struct Banco {
    c: Componente,
    schermi: Vec<i32>,
    appunti_salvati: bool,
    riusciti: usize,
    falliti: Vec<String>,
}

impl Banco {
    fn controlla(&mut self, nome: &str, ok: bool, dettagli: impl AsRef<str>) {
        let d = dettagli.as_ref();
        let d = if d.is_empty() { String::new() } else { format!(" ({d})") };
        println!("  {} {nome}{d}", if ok { "ok" } else { "NO" });
        if ok {
            self.riusciti += 1;
        } else {
            self.falliti.push(nome.to_string());
        }
    }

    async fn prova(&mut self, comando: &str) -> Result<Vec<u8>> {
        Ok(self.c.richiesta(tipo::PROVA, comando.as_bytes().to_vec()).await.with_context(|| format!("«{comando}»"))?.dati)
    }

    async fn prova_testo(&mut self, comando: &str) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.prova(comando).await?).into_owned())
    }

    /// Schermo virtuale di prova, registrato per la chiusura.
    async fn apri(&mut self, o: &Opzioni) -> Result<(i32, InputNostro)> {
        let r = self.prova_testo(&format!("apri {} {} {}", o.misura.0, o.misura.1, o.dpi)).await?;
        let id: i32 = r.strip_prefix("display=").and_then(|v| v.trim().parse().ok()).ok_or_else(|| anyhow!("risposta inattesa: {r}"))?;
        self.schermi.push(id);
        println!("  schermo virtuale {id} ({}×{}, {} dpi)", o.misura.0, o.misura.1, o.dpi);
        Ok((id, InputNostro::new(self.c.mittente(), id)))
    }

    async fn chiudi_schermo(&mut self, id: i32) -> Result<()> {
        self.schermi.retain(|&s| s != id);
        self.prova(&format!("chiudi {id}")).await?;
        Ok(())
    }

    async fn firma(&mut self, id: i32) -> Result<Vec<u8>> {
        let f = self.prova(&format!("firma {id}")).await?;
        if f.len() < 2 || f.len() != 2 + f[0] as usize * f[1] as usize {
            bail!("firma dello schermo malformata ({} byte)", f.len());
        }
        Ok(f[2..].to_vec())
    }

    async fn salva_appunti(&mut self) -> Result<()> {
        let r = self.prova_testo("salva-appunti").await?;
        self.appunti_salvati = true;
        println!("  appunti dell'utente {r} nella memoria del servizio (il testo non passa dal PC)");
        Ok(())
    }

    /// Gli avvisi delle copie arrivati entro `attesa` (gli altri messaggi spontanei si stampano).
    async fn avvisi(&mut self, attesa: Duration) -> Vec<Appunti> {
        let limite = tokio::time::Instant::now() + attesa;
        let mut avvisi = Vec::new();
        while let Ok(Some(m)) = tokio::time::timeout_at(limite, self.c.ricevi()).await {
            match Appunti::da_avviso(&m) {
                Some(Ok(a)) => avvisi.push(a),
                Some(Err(e)) => println!("  avviso degli appunti malformato: {e:#}"),
                None => println!("  messaggio del servizio: tipo {:#04x} «{}»", m.tipo, m.testo()),
            }
        }
        avvisi
    }

    /// Rimette a posto: appunti, schermi, ascolto; errori solo stampati.
    async fn pulisci(&mut self) {
        if self.appunti_salvati {
            match self.prova_testo("ripristina-appunti").await {
                Ok(r) => println!("  pulizia: appunti dell'utente {r}"),
                Err(e) => println!("  pulizia: appunti dell'utente NON ripristinati: {e:#}"),
            }
            self.appunti_salvati = false;
        }
        for id in std::mem::take(&mut self.schermi) {
            match self.prova(&format!("chiudi {id}")).await {
                Ok(_) => println!("  pulizia: schermo {id} chiuso (task tolti)"),
                Err(e) => println!("  pulizia: schermo {id}: {e:#}"),
            }
        }
        let _ = input_nostro::ascolta_appunti(&mut self.c, false).await;
    }
}

/// Differenza media tra due firme, in % della luminosità massima.
pub fn differenza(a: &[u8], b: &[u8]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 100.0;
    }
    let somma: u64 = a.iter().zip(b).map(|(x, y)| x.abs_diff(*y) as u64).sum();
    somma as f32 / a.len() as f32 / 255.0 * 100.0
}

/// L'attività in primo piano sullo schermo `display`, da `dumpsys activity
/// activities` (prima riga `…ResumedActivity…` della sezione `Display #<id>`).
pub fn attivita_in_primo_piano(dumpsys: &str, display: i32) -> Option<String> {
    let mut dentro = false;
    for riga in dumpsys.lines().map(str::trim) {
        if let Some(resto) = riga.strip_prefix("Display #") {
            dentro = resto.split(|c: char| !c.is_ascii_digit()).next().and_then(|n| n.parse().ok()) == Some(display);
        } else if dentro && riga.contains("ResumedActivity") && riga.contains("ActivityRecord{") {
            return riga.split_whitespace().find(|p| p.contains('/')).map(|p| p.trim_end_matches('}').to_string());
        }
    }
    None
}

async fn attivita(adb: &Adb, display: i32) -> Option<String> {
    let d = adb.esegui("dumpsys activity activities").await.ok()?;
    attivita_in_primo_piano(&d, display)
}

fn conteggio(voci: &[(String, String)], chiave: &str) -> u64 {
    voci.iter().find(|(k, _)| k == chiave).and_then(|(_, v)| v.parse().ok()).unwrap_or(0)
}

async fn attendi(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

/// Esegue la prova indicata; ripulisce sempre, poi controlla che non resti niente.
pub async fn esegui(adb: &Adb, argomenti: &[String]) -> Result<()> {
    let o = Opzioni::leggi(argomenti)?;
    let c = Componente::avvia(adb).await?;
    println!("servizio avviato (pid {})", c.pid.map_or("?".into(), |p| p.to_string()));
    for voce in ["inject_input_event", "appunti", "display_manager", "permessi"] {
        println!("  autotest.{voce} = {}", c.ciao.valore(&format!("autotest.{voce}")).unwrap_or("?"));
    }
    let mut banco = Banco { c, schermi: Vec::new(), appunti_salvati: false, riusciti: 0, falliti: Vec::new() };
    let esito = async {
        if matches!(o.prova.as_str(), "appunti" | "tutte") {
            appunti(&mut banco).await?;
        }
        if matches!(o.prova.as_str(), "tocchi" | "tutte") {
            tocchi(&mut banco, adb, &o).await?;
        }
        if matches!(o.prova.as_str(), "testo" | "tutte") {
            testo(&mut banco, adb, &o).await?;
        }
        anyhow::Ok(())
    }
    .await;
    if let Err(e) = &esito {
        println!("errore durante la prova: {e:#}");
    }
    println!("pulizia:");
    banco.pulisci().await;
    match input_nostro::conteggi(&mut banco.c).await {
        Ok(v) => println!(
            "  conteggi del servizio: {}",
            v.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")
        ),
        Err(e) => println!("  conteggi non letti: {e:#}"),
    }
    let Banco { c, riusciti, mut falliti, .. } = banco;
    let fine = c.chiudi().await?;
    let uscito_bene = fine == Processo::Uscito(0);
    println!("  servizio: {fine:?}{}", if uscito_bene { "" } else { " (atteso: uscito con codice 0)" });
    let residui = attendi_pulizia(adb).await?;
    let schermi_rimasti = adb.esegui("dumpsys display | grep -c phonestra-prova-input").await.unwrap_or_default();
    let nessuno_schermo = schermi_rimasti.trim() == "0";
    for (nome, ok) in [
        ("servizio uscito con codice 0", uscito_bene),
        ("nessun processo o file del componente", residui.pulito()),
        ("nessuno schermo di prova rimasto", nessuno_schermo),
    ] {
        println!("  {} {nome}", if ok { "ok" } else { "NO" });
        if !ok {
            falliti.push(nome.into());
        }
    }
    for r in residui.processi.iter().chain(&residui.file) {
        println!("    rimasto: {r}");
    }
    esito?;
    if !falliti.is_empty() {
        bail!("{} controlli riusciti, non riusciti: {}", riusciti, falliti.join("; "));
    }
    println!("prova riuscita ({riusciti} controlli)");
    Ok(())
}

async fn attendi_pulizia(adb: &Adb) -> Result<componente::Residui> {
    let limite = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let r = componente::residui(adb).await?;
        if r.pulito() || std::time::Instant::now() >= limite {
            return Ok(r);
        }
        attendi(500).await;
    }
}

/// Appunti senza schermo: scrittura, lettura, avvisi.
async fn appunti(b: &mut Banco) -> Result<()> {
    println!("# appunti");
    b.salva_appunti().await?;
    input_nostro::ascolta_appunti(&mut b.c, true).await?;
    let scritto = "Phonestra: prova degli appunti àèìòù €";
    input_nostro::scrivi_appunti(&mut b.c, scritto).await?;
    let letto = input_nostro::leggi_appunti(&mut b.c).await?;
    b.controlla("scrittura e rilettura (anche accentate)", letto == Appunti::Testo(scritto.into()), format!("{letto:?}"));
    let avvisi = b.avvisi(Duration::from_millis(1500)).await;
    b.controlla("nessun avviso per la scrittura di Phonestra", avvisi.is_empty(), format!("{avvisi:?}"));
    // Lo stesso testo di nuovo: il servizio non lo riscrive (niente doppioni nella cronologia).
    input_nostro::scrivi_appunti(&mut b.c, scritto).await?;
    let avvisi = b.avvisi(Duration::from_millis(800)).await;
    b.controlla("stesso testo di nuovo: nessun avviso", avvisi.is_empty(), format!("{avvisi:?}"));

    let esterno = "copia di prova da un'altra app";
    b.prova(&format!("esterno 0 {esterno}")).await?;
    let avvisi = b.avvisi(Duration::from_secs(2)).await;
    b.controlla("copia di un'altra app: avviso col testo", avvisi.contains(&Appunti::Testo(esterno.into())), format!("{avvisi:?}"));
    b.prova("esterno 1 segreto di prova").await?;
    let avvisi = b.avvisi(Duration::from_secs(2)).await;
    b.controlla("copia sensibile: avviso senza testo", avvisi == vec![Appunti::Sensibili], format!("{avvisi:?}"));
    let letto = input_nostro::leggi_appunti(&mut b.c).await?;
    b.controlla("copia sensibile: la lettura non dà il testo", letto == Appunti::Sensibili, format!("{letto:?}"));
    input_nostro::ascolta_appunti(&mut b.c, false).await?;
    b.prova("esterno 0 dopo lo stop dell'ascolto").await?;
    let avvisi = b.avvisi(Duration::from_millis(1500)).await;
    b.controlla("ascolto spento: nessun avviso", avvisi.is_empty(), format!("{avvisi:?}"));

    let r = b.prova_testo("ripristina-appunti").await?;
    b.appunti_salvati = false;
    let dopo = match input_nostro::leggi_appunti(&mut b.c).await? {
        Appunti::Testo(t) => format!("testo di {} caratteri", t.chars().count()),
        altro => format!("{altro:?}"),
    };
    println!("  appunti dell'utente {r}: ora {dopo}");
    Ok(())
}

/// Tocchi, rotellina, dita, «indietro» sulle Impostazioni in uno schermo virtuale.
async fn tocchi(b: &mut Banco, adb: &Adb, o: &Opzioni) -> Result<()> {
    println!("# tocchi");
    let (id, mut input) = b.apri(o).await?;
    let (l, a) = o.misura;
    let (x, y) = (l as i32 / 2, a as i32 / 2);
    let app = o.app.clone().unwrap_or_else(|| "com.android.settings".into());
    println!("  {}", b.prova_testo(&format!("avvia {id} {app}")).await?);
    attendi(2500).await;
    let prima = input_nostro::conteggi(&mut b.c).await?;
    let a0 = attivita(adb, id).await;
    println!("  in primo piano: {}", a0.as_deref().unwrap_or("?"));
    let f0 = b.firma(id).await?;

    // Rotellina: 5 scatti in giù, poi in su fino in cima (anche frazionari).
    input.scorri(x, y, l, a, 0.0, -5.0).await?;
    attendi(1200).await;
    let f1 = b.firma(id).await?;
    let d = differenza(&f0, &f1);
    b.controlla("la rotellina fa scorrere", d > CAMBIATA, format!("differenza {d:.1} %"));
    for _ in 0..4 {
        input.scorri(x, y, l, a, 0.0, 0.25).await?;
    }
    input.scorri(x, y, l, a, 0.0, 16.0).await?;
    attendi(1200).await;
    let f2 = b.firma(id).await?;
    println!("  -- rotellina in su fino in cima: differenza dall'inizio {:.1} %", differenza(&f0, &f2));

    // Trascinamento col dito generico, con coordinate su un'immagine grande la
    // metà: il telefono le scala (come per lo schermo del telefono in «specchio»).
    let (lm, am) = (l / 2, a / 2);
    let xm = lm as i32 / 2;
    let da = am as i32 * 3 / 4;
    let verso = am as i32 / 4;
    input.dito(GIU, xm, da, lm, am).await?;
    for passo in 1..=10 {
        attendi(16).await;
        input.dito(MOVIMENTO, xm, da + (verso - da) * passo / 10, lm, am).await?;
    }
    attendi(16).await;
    input.dito(SU, xm, verso, lm, am).await?;
    attendi(1200).await;
    let f3 = b.firma(id).await?;
    let d = differenza(&f2, &f3);
    b.controlla("il trascinamento (coordinate scalate) fa scorrere", d > CAMBIATA, format!("differenza {d:.1} %"));
    input.scorri(x, y, l, a, 0.0, 16.0).await?;
    input.scorri(x, y, l, a, 0.0, 16.0).await?;

    // Pizzico a due dita come lo zoom di Phonestra, in un solo invio.
    let mut dita = vec![(10, GIU, x - 40, y), (11, GIU, x + 40, y)];
    for passo in 1..=3 {
        dita.push((10, MOVIMENTO, x - 40 - 30 * passo, y));
        dita.push((11, MOVIMENTO, x + 40 + 30 * passo, y));
    }
    dita.push((11, SU, x + 130, y));
    dita.push((10, SU, x - 130, y));
    input.dita_insieme(&dita, l, a).await?;
    attendi(1500).await;

    // Un tocco nella lista apre una voce; «indietro» torna.
    let f4 = b.firma(id).await?;
    let (tx, ty) = (x, a as i32 * 6 / 10);
    input.tocco(GIU, tx, ty, l, a).await?;
    attendi(80).await;
    input.tocco(SU, tx, ty, l, a).await?;
    attendi(2000).await;
    let a1 = attivita(adb, id).await;
    let f5 = b.firma(id).await?;
    let d_tocco = differenza(&f4, &f5);
    b.controlla(
        "il tocco apre una voce",
        a1 != a0 || d_tocco > 3.0,
        format!("{} → {}, differenza {d_tocco:.1} %", a0.as_deref().unwrap_or("?"), a1.as_deref().unwrap_or("?")),
    );
    input.indietro().await?;
    attendi(2000).await;
    let a2 = attivita(adb, id).await;
    let f6 = b.firma(id).await?;
    let d_indietro = differenza(&f4, &f6);
    let tornato = if a1 != a0 { a2 == a0 } else { d_indietro < d_tocco };
    b.controlla(
        "«indietro» torna alla schermata di prima",
        tornato,
        format!("{}, differenza dalla schermata di prima {d_indietro:.1} %", a2.as_deref().unwrap_or("?")),
    );

    // Un evento calcolato su una misura vecchia (proporzioni diverse) si scarta.
    let intermedi = input_nostro::conteggi(&mut b.c).await?;
    input.tocco(GIU, 10, 10, l + 200, a).await?;
    input.tocco(SU, 10, 10, l + 200, a).await?;
    let dopo = input_nostro::conteggi(&mut b.c).await?;
    let scartati = conteggio(&dopo, "scartati") - conteggio(&intermedi, "scartati");
    b.controlla("tocco con misura vecchia scartato", scartati == 2, format!("scartati {scartati}"));
    let iniettati = conteggio(&dopo, "iniettati") - conteggio(&prima, "iniettati");
    let falliti = conteggio(&dopo, "falliti") - conteggio(&prima, "falliti");
    b.controlla(
        "tutti gli eventi iniettati senza errori",
        falliti == 0 && iniettati >= 30,
        format!("iniettati {iniettati}, falliti {falliti}, ultimo errore «{}»", valore(&dopo, "ultimo_errore")),
    );
    b.chiudi_schermo(id).await?;
    Ok(())
}

fn valore<'a>(voci: &'a [(String, String)], chiave: &str) -> &'a str {
    voci.iter().find(|(k, _)| k == chiave).map_or("", |(_, v)| v.as_str())
}

/// Scrittura in un campo di testo, verificata rileggendo gli appunti dopo Ctrl+A e Ctrl+C.
async fn testo(b: &mut Banco, adb: &Adb, o: &Opzioni) -> Result<()> {
    println!("# testo");
    let (id, mut input) = b.apri(o).await?;
    let avvio = match (&o.app, &o.azione) {
        (Some(app), _) => format!("avvia {id} {app}"),
        (None, Some(azione)) => format!("azione {id} {}", azione.replace(':', " ")),
        // La ricerca di Google, indicata per pacchetto: senza, con due app di
        // ricerca comparirebbe la scelta dell'app.
        (None, None) => format!("azione {id} android.search.action.GLOBAL_SEARCH {RICERCA}"),
    };
    println!("  {}", b.prova_testo(&avvio).await?);
    attendi(3000).await;
    println!("  in primo piano: {}", attivita(adb, id).await.as_deref().unwrap_or("?"));
    if let Some((tx, ty)) = o.tocca {
        input.tocco(GIU, tx, ty, o.misura.0, o.misura.1).await?;
        input.tocco(SU, tx, ty, o.misura.0, o.misura.1).await?;
        attendi(1000).await;
    }
    if let Ok(ime) = adb.esegui("dumpsys input_method").await {
        for riga in ime.lines().map(str::trim).filter(|r| r.starts_with("mServedView") || r.starts_with("mCurFocusedWindow")).take(3) {
            println!("  -- {}", riga.chars().take(160).collect::<String>());
        }
    }
    b.salva_appunti().await?;
    input_nostro::ascolta_appunti(&mut b.c, true).await?;

    // Testo ASCII un carattere alla volta, come lo manda oggi la finestra.
    let ascii = "Phonestra prova 123";
    for c in ascii.chars() {
        input.testo(&c.to_string()).await?;
    }
    attendi(800).await;
    seleziona_e_copia(&mut input).await?;
    attendi(1200).await;
    let letto = input_nostro::leggi_appunti(&mut b.c).await?;
    b.controlla("testo ASCII, Ctrl+A e Ctrl+C", letto == Appunti::Testo(ascii.into()), format!("{letto:?}"));
    let avvisi = b.avvisi(Duration::from_millis(1500)).await;
    b.controlla(
        "la copia fatta nell'app arriva al PC",
        avvisi.contains(&Appunti::Testo(ascii.into())),
        format!("{avvisi:?}"),
    );

    // Incolla (appunti + PASTE) al posto della selezione, poi testo ASCII: la
    // copia finale è diversa da quello che ha messo Phonestra, quindi prova
    // davvero l'incolla.
    tasto_con(&mut input, TASTO_A, CTRL).await?;
    input.incolla("àèìòù €").await?;
    attendi(800).await;
    for c in " fine".chars() {
        input.testo(&c.to_string()).await?;
    }
    attendi(800).await;
    seleziona_e_copia(&mut input).await?;
    attendi(1200).await;
    let letto = input_nostro::leggi_appunti(&mut b.c).await?;
    b.controlla("incolla di lettere accentate", letto == Appunti::Testo("àèìòù € fine".into()), format!("{letto:?}"));

    // Campo svuotato, tastiera e ricerca chiuse.
    tasto_con(&mut input, TASTO_A, CTRL).await?;
    tasto_con(&mut input, TASTO_CANC, 0).await?;
    input.indietro().await?;
    input.indietro().await?;
    attendi(500).await;
    b.chiudi_schermo(id).await?;
    Ok(())
}

async fn tasto_con(input: &mut InputNostro, codice: u32, meta: u32) -> Result<()> {
    input.tasto(GIU, codice, meta).await?;
    input.tasto(SU, codice, meta).await
}

async fn seleziona_e_copia(input: &mut InputNostro) -> Result<()> {
    tasto_con(input, TASTO_A, CTRL).await?;
    attendi(200).await;
    tasto_con(input, TASTO_C, CTRL).await
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn differenza_tra_firme() {
        assert_eq!(differenza(&[0, 0], &[0, 0]), 0.0);
        assert_eq!(differenza(&[0, 255], &[255, 255]), 50.0);
        assert_eq!(differenza(&[0], &[0, 0]), 100.0);
    }

    #[test]
    fn attivita_dello_schermo() {
        let d = "ACTIVITY MANAGER ACTIVITIES (dumpsys activity activities)\n\
                 Display #0 (activities from top to bottom):\n\
                   * Task{1 #2 type=home}\n\
                     mResumedActivity: ActivityRecord{a1 u0 com.sec.android.app.launcher/.activities.LauncherActivity t2}\n\
                 Display #57 (activities from top to bottom):\n\
                   * Task{2 #41 type=standard}\n\
                   Resumed activities in task display areas (from top to bottom):\n\
                     ResumedActivity: ActivityRecord{b2 u0 com.android.settings/.SubSettings t41}\n";
        assert_eq!(attivita_in_primo_piano(d, 57).as_deref(), Some("com.android.settings/.SubSettings"));
        assert_eq!(attivita_in_primo_piano(d, 0).as_deref(), Some("com.sec.android.app.launcher/.activities.LauncherActivity"));
        assert_eq!(attivita_in_primo_piano(d, 5), None);
    }

    #[test]
    fn opzioni_della_prova() {
        let a: Vec<String> = ["tocchi", "--misura", "800x1200", "--tocca", "10,20"].iter().map(|s| s.to_string()).collect();
        let o = Opzioni::leggi(&a).unwrap();
        assert_eq!((o.prova.as_str(), o.misura, o.tocca), ("tocchi", (800, 1200), Some((10, 20))));
        assert!(Opzioni::leggi(&["boh".to_string()]).is_err());
        assert!(Opzioni::leggi(&["testo".to_string(), "--dpi".to_string()]).is_err());
    }
}
