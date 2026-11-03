# Datamosh

Clips so hintereinander abspielen, dass der Dekoder die Bewegung des einen Clips auf die Bilder des anderen anwendet. Man kann live eingreifen (springen, halten) oder die Eingriffe als Liste eintragen. Der Export spielt sie noch einmal genau so ab. Ein eigenes Repo wie die anderen; zum Einbauen ins Portfolio gibt es `merge-datamosh.zip`.

**Einrichten**
- Inhalt in ein neues Repo `datamosh` legen.
- `css/fonts/` aus dem Portfolio dazukopieren.
- Mit deinem lokalen Server ausliefern.

**Bedienung in Kürze**
1. Zwei oder mehr Clips öffnen. Gut geeignet: zuerst ein ruhiges Bild (Gesicht, Landschaft), danach ein Clip mit viel und klarer Bewegung (Kameraschwenk, Tanz, Wasser).
2. „Vorbereiten“. Danach liegen alle Clips einheitlich kodiert im Speicher.
3. Abspielen (Leertaste). **S** springt in den nächsten Clip, **H** gedrückt halten lässt das Bild zerfließen, **K** heilt.
4. Unter „Eingriffe“ steht, was du gemacht hast. Dort kannst du Einträge löschen oder neue von Hand dazuschreiben.
5. „Video rendern“ spielt alles noch einmal ab, diesmal Bild für Bild ohne Zeitdruck.

---

## 0. Neu in Runde 3 (3.11.)

- **Zeitleiste wie in Avidemux** (`js/zeitleiste.js`): Jedes Bild der Folge ist ein Kästchen. Rot steht für einen Keyframe, Orange für geheilt, Violett für gehalten oder wiederholt; ein weißer Strich markiert Schnitt oder Sprung. Klicken zeigt das Bild, Ziehen oder Umschalt-Klick wählt aus.
  - **Auswahl löschen:** Das ist intern ein Sprung von der Auswahl zum Bild danach, und die späteren Eingriffe rücken nach vorn. Löschst du den roten Keyframe am Schnitt, ist das genau der Avidemux-Mosh.
  - **Auswahl wiederholen ×N:** das neue Ereignis „wiederholen“ in `folge.js`, wie Kopieren und Einfügen. Wiederholte P-Frames schieben die Blöcke weiter.
- **Bilder pro Clip** stehen in der Liste. Vor dem Vorbereiten sind sie geschätzt (Dauer mal Bildrate), danach genau gezählt (Pakete).
- **An Schnitten moshen** ist jetzt eine Zahl: so viele Bilder lang, danach heilt es. 0 heißt aus, ganz rechts heißt endlos.
- **Eingriffe merken:** Ist der Haken aus, moshst du live, ohne dass etwas in die Liste oder in den Export kommt. Das gilt bis „Von vorn“ (die Eingriffe liegen dann in `fluechtig`).
- **Ruckeln:** Gemessen bei 1280 Breite und 8 Mbit/s braucht das Dekodieren etwa 10 ms pro Bild, mit einzelnen Spitzen bis 37 ms. Bei 30 Bildern pro Sekunde sind nur 33 ms Zeit, und jede Spitze war ein Ruckler. Jetzt läuft der Dekoder bis zu 3 Bilder voraus und legt sie in einen Vorrat; gezeigt wird im festen Takt. Der Preis: Ein Eingriff wirkt etwa 0,1 s später.

## 0. Neu in Runde 2: warum Springen „pures Glitchen“ war

**Du hattest recht, dass da etwas nicht stimmte.** Ein Sprung soll das alte Bild behalten und mit der Bewegung des neuen Clips verschieben. Stattdessen löste sich das Bild nach zehn Bildern in Rauschen auf, und das blieb so. Gemessen habe ich mit zwei Testclips (Mandelbrot, einmal zoomend, einmal schwenkend):

- **Der Grund ist nicht das Bild, sondern die Bits.** VP8 speichert alles mit einem arithmetischen Kodierer (dem „Bool-Kodierer“). Der braucht für jedes Bit eine Wahrscheinlichkeit. Jeder P-Frame darf diese Wahrscheinlichkeiten **für alle folgenden Bilder** ändern (`refresh_entropy_probs = 1`). Nach einem Sprung hat der Dekoder die Wahrscheinlichkeiten des falschen Clips und liest die Bits falsch. Dann kommen falsche Bewegungsvektoren und falsche Koeffizienten heraus, also Rauschen. Bis zum nächsten Keyframe wird das nie besser, und es kommt keiner: darum „ewig“.
- Nachgesehen habe ich das, indem ich den Kopf jedes Pakets selbst gelesen habe (`js/vp8-kopf.js`, RFC 6386): Ohne Gegenmaßnahme stand bei allen 300 Paketen `refresh_entropy_probs = 1`.
- **Das Gefühl „der Frame und ein anderer“:** VP8 hat drei Bezugsbilder, nicht eins. Neben dem letzten Bild gibt es das „Golden Frame“, das etwa alle 8 Bilder erneuert wird, und eine Kopie davon („Altref“). Blöcke, die sich auf diese beziehen, holen nach einem Sprung ein älteres Bild aus dem alten Clip. Messbar war das als Flackern aber kaum, das Rauschen war der Hauptgrund.

**Die Lösung:** Kodieren mit zwei **Zeitschichten** (`scalabilityMode: "L1T2"`). Damit eine Schicht ohne die andere abspielbar bleibt, schreibt der Kodierer jedes Paket so, dass es die Wahrscheinlichkeiten nur für sich selbst ändert (gemessen: überall `0`). Jetzt verschiebt ein Sprung wirklich Blöcke des alten Bildes, statt Bits falsch zu lesen. Wer das alte Rauschen will, setzt beim Vorbereiten den Haken **wild**.

**Mehr Kontrolle:**
- **Heilen (K):** An dieser Stelle kommt das Bild der Quelle als Keyframe, und das Bild ist wieder sauber. Dafür kodiert „Vorbereiten“ jedes Bild ein zweites Mal als Keyframe (`daten.schluessel`, mit höherer Bitrate). Das dauert doppelt so lang und braucht etwa 20 KB pro Bild.
- **Heilen nach:** Nach jedem Sprung heilt es von selbst nach so vielen Bildern (Standard 45, also 1,5 Sekunden). Damit wird aus dem endlosen Mosh ein Übergang. 0 = nie.
- **Beruhigen:** Jedes neue Bild wird nur teilweise deckend über das alte gelegt. Harte Wechsel werden so zu Schlieren, und das wirkt viel weniger epileptisch. Wer im System „Bewegung reduzieren“ eingestellt hat, bekommt das gleich auf 0,6 (siehe Fahrplan: jeder flackernde Effekt bekommt eine ruhigere Fassung).
- **Halten** nimmt jetzt immer ein Paket, das das Bezugsbild erneuert. Mit Zeitschichten tut das nur jedes zweite. Die anderen würden immer wieder dasselbe Bild verschieben, und es stünde still.
- Ein Fehler ist behoben: Ereignisse, die mitten in einem Halten lagen, wurden übersprungen. Jetzt geht `baueFolge` Bild für Bild vor, und Heilen oder Springen beendet ein Halten.
- **Bitrate** steht jetzt auf 1 Mbit/s: weniger Rest heißt ruhigeres Wandern der Blöcke.

---

## 1. Warum Datamosh überhaupt geht: I-, P- und B-Frames

Ein Video speichert fast nie jedes Bild ganz. Es gibt drei Sorten:

| Sorte | Name in WebCodecs | Was drin steht |
|---|---|---|
| **I-Frame** (intra, Keyframe) | `"key"` | das ganze Bild, wie ein JPEG |
| **P-Frame** (predicted) | `"delta"` | „nimm das vorige Bild, verschiebe diese Blöcke um so viel (Bewegungsvektoren), und korrigiere hier ein bisschen (Rest)“ |
| **B-Frame** (bidirectional) | gibt es bei VP8 nicht | wie P, darf aber auch ein *späteres* Bild als Vorlage nehmen |

Ein P-Frame weiß nicht, welches Bild vor ihm kommt. Er rechnet einfach auf dem, was im Dekoder liegt. Daraus folgen die zwei klassischen Moshs:

- **I-Frame weglassen (Übergang).** Am Schnitt zu Clip 2 steht normalerweise ein Keyframe, der das Bild „sauber“ macht. Lässt man ihn weg, kommen die P-Frames von Clip 2 auf das letzte Bild von Clip 1. Die Bewegung von Clip 2 schiebt die Pixel von Clip 1 herum, bis die Reste nach und nach Clip 2 durchscheinen lassen.
- **P-Frame wiederholen (Bloom).** Derselbe P-Frame zehnmal hintereinander sagt zehnmal „verschiebe diesen Block nach rechts“. Das Bild zieht sich in Richtung der Bewegung auseinander und zerfließt.

Dein Gedächtnis war also richtig: man löscht I-Frames oder verdoppelt P-Frames. Mit B-Frames hat man früher in AVI-Dateien gearbeitet, das ist hier nicht nötig.

---

## 2. Live oder von Hand? Beides, als dieselbe Liste

Ich habe lange überlegt, welche der zwei Arten sinnvoller ist:

- **Nur live** (spielen und drücken): macht Spaß, aber man trifft die Stelle nie zweimal gleich. Ein Video, das man rendern will, wäre Glück.
- **Nur von Hand** (Bereiche eintragen): genau, aber man sieht erst nach dem Eintragen, wie es aussieht. So hat man früher mit Avidemux gearbeitet, und das war mühsam.

Darum ist beides dasselbe: Was du live drückst, wird als **Eingriff mit Bildnummer** gemerkt. Die Liste zeigt alles, du kannst löschen und dazuschreiben, und die Folge wird daraus jedes Mal neu gebaut (`folge.js`). Der Export nimmt dieselbe Folge. Man probiert also live, bis etwas gut aussieht, und räumt dann in der Liste auf.

Der Ablauf im Code:

```
clips ──bereite()──▶ pakete (VP8, je Clip ein Keyframe)
                         │
eingriffe ──baueFolge()──▶ folge = [0, 1, 2, …, 37, 121, 122, 122, 122, …]
                         │            (Sprung) (Halten)
                         ▼
                   dekodiere(pakete[folge[n]])  ──▶ Bild
```

**Ein Haken:** „bei Bild“ zählt die Bilder des *fertigen* Videos. Löschst du ein Halten am Anfang, wird alles danach kürzer, und spätere Eingriffe landen an einer anderen Stelle der Quelle. Das ist gewollt einfach gehalten. Falls es stört, wäre die Alternative, Eingriffe an der Stelle in der Quelle festzumachen.

---

## 3. Warum alles neu kodiert wird (`kodieren.js`)

Man könnte die Pakete direkt aus den Originaldateien nehmen, aber:

1. **Keyframes überall:** Handys setzen alle 1 bis 2 Sekunden einen Keyframe. Jeder davon „repariert“ den Mosh. Wir kodieren selbst und setzen nur einen, am Anfang jedes Clips (`keyFrame: i === 0`).
2. **Gleiche Größe, gleicher Codec:** P-Frames von Clip 2 passen nur auf ein Bild von Clip 1, wenn beide gleich groß und gleich kodiert sind.
3. **Der Codec muss mitspielen:** Ich habe VP8, VP9 und H.264 ausprobiert:
   - **VP8:** rechnet ohne Murren auf dem falschen Bild weiter. Genommen.
   - **VP9:** bricht mit einem Fehler ab, wenn ein Bezugsbild fehlt.
   - **H.264:** konnte mein Test-Chromium gar nicht kodieren. In Chrome ginge es, aber H.264 läuft meist über den Hardware-Dekoder, und der steigt bei Unstimmigkeiten oft aus. Ausprobieren konnte ich das nicht.

Darum `hardwareAcceleration: "prefer-software"` im Dekoder (`dekoder.js`): Der Software-Dekoder (libvpx) ist gutmütiger.

**Bitrate:** Wenig Bitrate heißt grobe Blöcke und wenig Rest. Der Mosh hält dann lange, weil kaum korrigiert wird. Viel Bitrate heißt viel Rest: Das Bild geht schneller in Clip 2 über, und es flimmert mehr.

**Breite:** Das Moshen sieht in 480 bis 640 Pixel Breite am besten aus. Die Blöcke sind dann sichtbar, und live bleibt es flüssig. Beim Export kannst du im Export-Fenster hochskalieren lassen.

---

## 4. Springen und Halten (`main.js`, `folge.js`)

- **Sprung nach:**
  - *Anfang des nächsten Clips:* der Klassiker.
  - *gleiche Stelle im nächsten Clip:* gut, wenn beide Clips gleichzeitig gefilmt wurden (zwei Kameras).
  - *irgendwohin:* Zufall.

  Ein Sprung auf einen Keyframe nimmt immer das Bild danach, sonst gäbe es keinen Mosh.
- **Halten:** Solange H gedrückt ist, wird jedes Bild das letzte Paket wiederholt. Wie stark das Bild zerfließt, hängt davon ab, wie viel Bewegung in diesem einen Paket steckt. Bei einem stehenden Bild passiert fast nichts.
- **An jedem Schnitt moshen:** lässt alle Keyframes außer dem allerersten weg. Dann fließen die Clips ohne jeden Eingriff ineinander.

**Zurückspulen gibt es nicht:** Jedes Bild baut auf dem vorigen auf, darum geht es nur der Reihe nach. „Von vorn“ dekodiert ab Bild 0, das dauert bei 640 Pixel Breite nur einen Augenblick.

---

## 5. Ehrlich

- **Safari** kann meistens kein VP8 kodieren. Dann sagt „Vorbereiten“ das, und es geht nur in Chrome, Edge oder Firefox. Zen ist Firefox und sollte gehen; getestet habe ich in Chromium.
- **Kein Ton.** Ton zu moshen ergibt nichts Sinnvolles, und den Originalton passend zu Sprüngen und Halten zu schneiden, wäre ein eigenes Projekt. Ton legst du in DaVinci darunter.
- **Speicher:** Alle Pakete liegen im Speicher, dazu jedes Bild noch einmal als Keyframe zum Heilen. Bei 640 Pixel Breite sind das etwa 40 bis 50 MB pro Minute Video. Für ein paar Minuten ist das kein Problem, für ganze Filme schon.
- **Das Live-Bild ist nicht bildgenau getaktet:** Wenn der Rechner hinterherhinkt, läuft es langsamer, es lässt aber kein Bild aus. Der Export ist immer genau.
- **Andere Seitenverhältnisse** werden auf das des ersten Clips gestreckt.

---

## 6. Ideen für später (zum Selberschreiben)

- **Bewegung von woanders:** Statt eines Clips nur seine Bewegungsvektoren auf ein Standbild legen. Dafür müsste man VP8-Pakete selbst zerlegen, das ist richtig viel Arbeit. Einfacher wäre es mit einem eigenen „Fake-Mosh“ im Shader: optischer Fluss aus Clip 2, angewendet auf das Bild von Clip 1, Bild für Bild aufaddiert. Das sieht weicher aus als echter Mosh und lässt sich besser steuern.
- **Halten mit Stärke:** statt das Paket ganz zu wiederholen, die Ausgabe mit dem vorigen Bild mischen.
- **MIDI oder Takt:** Sprünge auf den Beat (wie bei der Chronofotografie, `einsaetze.js` ließe sich übernehmen).
