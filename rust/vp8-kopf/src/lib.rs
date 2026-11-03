//! Den Kopf eines VP8-Pakets lesen (RFC 6386, Abschnitt 9): dasselbe wie `js/vp8-kopf.js`, in Rust.
//! Nicht eingebunden, sondern zum Vergleichen: Was ändert sich, wenn man dasselbe in Rust schreibt?
//!
//! Die wichtigsten Unterschiede zu JavaScript, alle hier zu sehen:
//!   - Typen stehen fest: `u8` (Byte), `u32`, `usize` (Index). In JS ist alles `number`, und
//!     `wert <<= 1` kann dort unbemerkt über 32 Bit laufen; hier sagt der Compiler, welche Breite gemeint ist.
//!   - Der Bool-Dekoder ist ein `struct` mit Methoden statt einer Funktion, die Variablen einfängt (Closure).
//!     `&mut self` heißt: diese Methode verändert den Dekoder. Wer nur liest, bekommt `&self`.
//!   - `daten.get(i)` gibt `Option<&u8>` zurück: Rust zwingt einen zu sagen, was passiert, wenn das Paket zu
//!     kurz ist (`unwrap_or(&0)`). In JS ergibt `daten[i]` dann still `undefined`, und `?? 0` vergisst man leicht.
//!   - Das Ergebnis ist ein `struct` mit festen Feldern statt eines Objekts, das jede Form haben kann.
//!   - Tests stehen in derselben Datei (`#[cfg(test)]`) und laufen mit `cargo test`.

/// Was der Datamosh über ein Paket wissen muss
#[derive(Debug, PartialEq)]
pub struct Kopf {
    pub key: bool,
    /// ändert dieses Paket die Wahrscheinlichkeiten für alle folgenden Bilder? (dann glitcht ein Sprung)
    pub wahrscheinlichkeiten_bleiben: bool,
    /// wird dieses Bild das neue Bezugsbild ("last")?
    pub erneuert_letztes: bool,
}

/// Der Bool-Dekoder: eine Zahl `wert` und ein Bereich `breite` (128 bis 255)
struct BoolDekoder<'a> {
    daten: &'a [u8],
    i: usize,
    wert: u32,
    breite: u32,
    bits: u32,
}

impl<'a> BoolDekoder<'a> {
    fn neu(daten: &'a [u8], start: usize) -> Self {
        let byte = |k: usize| *daten.get(k).unwrap_or(&0) as u32;
        BoolDekoder { daten, i: start + 2, wert: (byte(start) << 8) | byte(start + 1), breite: 255, bits: 0 }
    }

    /// ein Bit mit Wahrscheinlichkeit p/256, dass es 0 ist
    fn bit(&mut self, p: u32) -> u32 {
        let teil = 1 + (((self.breite - 1) * p) >> 8);
        let grenze = teil << 8;
        let ergebnis = if self.wert >= grenze {
            self.breite -= teil;
            self.wert -= grenze;
            1
        } else {
            self.breite = teil;
            0
        };
        while self.breite < 128 {
            self.wert <<= 1;
            self.breite <<= 1;
            self.bits += 1;
            if self.bits == 8 {
                self.bits = 0;
                self.wert |= *self.daten.get(self.i).unwrap_or(&0) as u32;
                self.i += 1;
            }
        }
        ergebnis
    }

    /// n Bits, das höchste zuerst (jedes mit p = 128, also 50 %)
    fn zahl(&mut self, n: u32) -> u32 {
        (0..n).fold(0, |x, _| (x << 1) | self.bit(128))
    }
}

/// liest den Kopf; `None`, wenn das Paket zu kurz ist
pub fn lese_kopf(daten: &[u8]) -> Option<Kopf> {
    if daten.len() < 12 {
        return None;
    }
    let marke = daten[0] as u32 | (daten[1] as u32) << 8 | (daten[2] as u32) << 16;
    let key = marke & 1 == 0;
    let mut b = BoolDekoder::neu(daten, if key { 10 } else { 3 }); // Keyframe: Startcode und Größe dazwischen
    let mut l = |n| b.zahl(n);
    if key {
        l(2); // Farbraum, Clamping
    }
    if l(1) == 1 {
        // Segmente
        let karte = l(1);
        if l(1) == 1 {
            l(1);
            for _ in 0..4 {
                if l(1) == 1 {
                    l(8);
                }
            }
            for _ in 0..4 {
                if l(1) == 1 {
                    l(7);
                }
            }
        }
        if karte == 1 {
            for _ in 0..3 {
                if l(1) == 1 {
                    l(8);
                }
            }
        }
    }
    l(1 + 6 + 3); // Filterart, -stärke, -schärfe
    if l(1) == 1 && l(1) == 1 {
        for _ in 0..8 {
            if l(1) == 1 {
                l(7);
            }
        }
    }
    l(2); // Partitionen
    l(7); // Quantisierer
    for _ in 0..5 {
        if l(1) == 1 {
            l(5);
        }
    }
    if key {
        return Some(Kopf { key, wahrscheinlichkeiten_bleiben: l(1) == 1, erneuert_letztes: true });
    }
    let golden = l(1);
    let alt = l(1);
    if golden == 0 {
        l(2);
    }
    if alt == 0 {
        l(2);
    }
    l(2); // Vorzeichen
    let bleiben = l(1) == 1;
    Some(Kopf { key, wahrscheinlichkeiten_bleiben: bleiben, erneuert_letztes: l(1) == 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    // echte Paketanfänge aus dem Test im Browser (Chromium, VP8): ohne und mit Zeitschichten (L1T2)
    const OHNE_KEY: [u8; 24] = [16, 0, 1, 157, 1, 42, 128, 2, 104, 1, 2, 71, 8, 133, 133, 136, 153, 132, 136, 9, 131, 187, 142, 211];
    const OHNE_P: [u8; 24] = [145, 36, 0, 8, 17, 4, 0, 28, 6, 46, 168, 121, 19, 222, 60, 241, 114, 69, 219, 128, 210, 181, 255, 211];
    const L1T2_KEY: [u8; 24] = [144, 214, 0, 157, 1, 42, 128, 2, 104, 1, 57, 39, 0, 11, 28, 36, 12, 44, 44, 68, 204, 36, 64, 148];
    const L1T2_P_OHNE_BEZUG: [u8; 24] = [17, 52, 0, 13, 28, 36, 12, 44, 44, 68, 204, 36, 65, 184, 0, 6, 239, 41, 247, 187, 51, 173, 77, 49];
    const L1T2_P_MIT_BEZUG: [u8; 24] = [241, 58, 0, 228, 161, 47, 220, 84, 23, 175, 16, 79, 79, 177, 10, 47, 227, 132, 224, 50, 185, 222, 254, 32];

    #[test]
    fn ohne_zeitschichten_bleiben_die_wahrscheinlichkeiten() {
        assert_eq!(lese_kopf(&OHNE_KEY), Some(Kopf { key: true, wahrscheinlichkeiten_bleiben: true, erneuert_letztes: true }));
        assert_eq!(lese_kopf(&OHNE_P), Some(Kopf { key: false, wahrscheinlichkeiten_bleiben: true, erneuert_letztes: true }));
    }

    #[test]
    fn mit_zeitschichten_nicht() {
        assert_eq!(lese_kopf(&L1T2_KEY), Some(Kopf { key: true, wahrscheinlichkeiten_bleiben: false, erneuert_letztes: true }));
        assert_eq!(lese_kopf(&L1T2_P_OHNE_BEZUG), Some(Kopf { key: false, wahrscheinlichkeiten_bleiben: false, erneuert_letztes: false }));
        assert_eq!(lese_kopf(&L1T2_P_MIT_BEZUG), Some(Kopf { key: false, wahrscheinlichkeiten_bleiben: false, erneuert_letztes: true }));
    }

    #[test]
    fn zu_kurz() {
        assert_eq!(lese_kopf(&[1, 2, 3]), None);
    }
}
