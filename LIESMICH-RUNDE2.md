# Datamosh, Runde 2: nur die Änderungen (2.11.)

Ins Repo `datamosh` kopieren und überschreiben. Was und warum steht in `LIESMICH.md`, Abschnitt 0.

| Datei | |
|---|---|
| `js/vp8-kopf.js` | **neu**: liest den Kopf jedes VP8-Pakets (Bool-Dekoder nach RFC 6386) |
| `js/kodieren.js` | Zeitschichten (L1T2), damit Sprünge Blöcke verschieben statt Rauschen zu machen; zweiter Kodierer mit jedem Bild als Keyframe (zum Heilen); „wild“ |
| `js/folge.js` | Ereignis „heilen“, „Heilen nach“, Halten nimmt ein Bezugspaket, Bild für Bild (Fehler: Ereignisse mitten im Halten wurden übersprungen) |
| `js/main.js` | Taste K, Beruhigen, wild, Meldung ohne Zeitschichten |
| `index.html` | Heilen-Knopf, Heilen nach, Beruhigen, wild, Bitrate 1 |
| `LIESMICH.md` | Abschnitt 0 neu |
