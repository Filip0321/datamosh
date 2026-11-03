# Rust-Spiegel

Bekannte Dateien noch einmal in Rust, **nicht eingebunden**: Die Seite benutzt weiter das JavaScript. Es geht ums Vergleichen und Lernen. Jede Rust-Datei heißt wie ihr Vorbild, oben steht, was sich ändert und warum.

| Rust | Vorbild | Was man daran lernt |
|---|---|---|
| `vp8-kopf/src/lib.rs` | `js/vp8-kopf.js` | feste Zahlentypen statt `number`, `struct` mit `&mut self` statt Closure, `Option` statt `undefined`, Tests in derselben Datei |

**Ausprobieren:** im Ordner `vp8-kopf` die Tests laufen lassen (`cargo test`). Die drei Tests benutzen echte Paketanfänge aus dem Browser und prüfen dasselbe, was der Datamosh braucht: Mit Zeitschichten ändert kein Paket die Wahrscheinlichkeiten.

**Später im Browser (WebAssembly):** Mit `wasm-bindgen` bekommt `lese_kopf` ein `#[wasm_bindgen]` davor, `wasm-pack build --target web` macht daraus ein Modul, das JavaScript importieren kann. Das lohnt sich erst bei Rechenarbeit auf der CPU (Delaunay, Pixel Sorting, das Aufräumen der Tiefe). Für diesen kleinen Kopf wäre der Weg über WebAssembly langsamer als das JavaScript.
