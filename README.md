# MZ-Textbausteine

**Autor:** Marcel Zimmer<br>
**Web:** [www.marcelzimmer.de](https://www.marcelzimmer.de)<br>
**X:** [@marcelzimmer](https://x.com/marcelzimmer)<br>
**GitHub:** [@marcelzimmer](https://github.com/marcelzimmer)<br>
**Version:** 1.0.0<br>
**Sprache:** Rust<br>
**Plattform:** Windows<br>
**Lizenz:** MIT

---

## Überblick

MZ-Textbausteine ist ein systemweiter Textkürzel-Expander für Windows. Das Programm läuft unsichtbar im Hintergrund und ersetzt vordefinierte Kürzel automatisch durch längere Texte - in jeder Anwendung, jedem Textfeld und jedem Browser.

Die Konfiguration erfolgt über eine einzige TOML-Datei [mz-textbausteine.toml], die neben der .exe liegt. Im Infobereich der Taskleiste erscheint ein Icon, über das ein kleines Über-Fenster geöffnet und die Anwendung beendet werden kann. Ansonsten arbeitet das Programm still im Hintergrund und hängt sich in die Windows-Tastatureingabe ein.

---

## Warum MZ-Textbausteine?

Viele kennen Textbausteine aus SAP - dort lassen sich häufig benötigte Texte hinterlegen und per Kürzel abrufen. Das ist praktisch, hat aber einen entscheidenden Nachteil: Die Bausteine funktionieren nur innerhalb von SAP. Sobald man in Outlook eine E-Mail schreibt, im Browser ein Formular ausfüllt oder in einem anderen Programm arbeitet, sind sie weg.

MZ-Textbausteine löst genau dieses Problem. Die Kürzel stehen in **praktisch jeder Anwendung** zur Verfügung - egal ob SAP, Outlook, Teams, Word oder ein Webformular. Einmal eingerichtet, spart das täglich Zeit: Grussformeln, E-Mail-Adressen, Datumsangaben, Signaturen - alles ist mit wenigen Tasten abrufbar, überall und sofort.

Die Idee ist nicht neu: Apple bietet mit den **macOS-Textersetzungen** seit Jahren systemweite Kürzel direkt aus den Systemeinstellungen. Unter **Omarchy Linux** übernimmt CapsLock die Rolle einer Compose-Taste für ähnliche Sequenzen. Windows hat von Haus aus nichts Vergleichbares. MZ-Textbausteine bringt beide Konzepte zusammen: kurze Auslöser direkt im Tippfluss oder alternativ über CapsLock-Tastenfolgen - in einer einzigen schlanken Anwendung.

Das Programm arbeitet still im Hintergrund. Ein dezentes Icon in der Taskleiste zeigt nur, dass es läuft - mehr Bedienung gibt es nicht. Kürzel tippen, Text erscheint - in praktisch jeder App auf dem Rechner.

---

## Funktionsweise

MZ-Textbausteine kennt zwei Eingabemodi.

### Auslöser-Modus

Ein Auslöser ist eine kurze Buchstabenfolge, die beim Tippen automatisch erkannt und ersetzt wird. Sobald der Auslöser vollständig eingegeben wurde, löscht das Programm ihn zeichenweise und tippt die Ersetzung an seiner Stelle.

**Beispiel:** `@e ` wird sofort zu `vorname.nachname@example.com`.

Das abschliessende Leerzeichen gehört zum Auslöser und verhindert versehentliche Auslösungen mitten in einem Wort.

### Tastenfolge-Modus [CapsLock]

Im Tastenfolge-Modus wird CapsLock als Einleitungstaste verwendet. Nach dem Drücken von CapsLock wartet das Programm auf eine festgelegte Tastenfolge. Wird eine bekannte Folge erkannt, tippt das Programm die Ersetzung. Wird eine unbekannte Taste gedrückt oder ist die Folge nicht erreichbar, beendet sich der Modus ohne Auswirkung.

**Beispiel:** CapsLock - Space - m tippt `Mit freundlichen Grüssen` gefolgt von zwei Zeilenumbrüchen und `Marcel Zimmer`.

Der Tastenfolge-Modus beeinflusst CapsLock nicht als Feststelltaste - diese Funktion entfällt, solange das Programm läuft.

---

## Tray-Icon

Sobald das Programm läuft, erscheint ein Icon im Infobereich der Taskleiste [rechts unten]. Beim Überfahren mit der Maus wird der Tooltip "MZ-Textbausteine" angezeigt. Das Icon ist die einzige sichtbare Bedienoberfläche.

### Linksklick

Ein Linksklick öffnet dasselbe Kontextmenü wie der Rechtsklick.

### Rechtsklick

Ein Rechtsklick auf das Tray-Icon öffnet ein Kontextmenü mit zwei Einträgen:

- **Über MZ-Textbausteine** - öffnet das Info-Fenster mit Versionsnummer, Autorname und Schaltflächen zum Schliessen oder Beenden
- **Beenden** - zeigt eine Sicherheitsabfrage vor dem tatsächlichen Beenden

### Beenden

Sowohl die Schaltfläche "Beenden" im Über-Fenster als auch der Menüpunkt "Beenden" im Tray-Kontextmenü zeigen vor dem tatsächlichen Beenden eine Sicherheitsabfrage:

> Nach dem Beenden stehen die Textbausteine nicht mehr zur Verfügung.<br>
> Möchten Sie MZ-Textbausteine wirklich schliessen?

Erst nach Bestätigung mit "Ja" wird das Programm beendet, das Tray-Icon entfernt und der Tastatur-Hook freigegeben. Bis dahin bleibt die Anwendung mit allen Kürzeln aktiv.

---

## Bekannte Einschränkungen

Im neuen **Windows-11-Notepad** [mit Tabs und Rechtschreibprüfung] werden die per `SendInput` gesendeten Unicode-Zeichen von Notepads eigener Eingabe-Pipeline gefiltert oder verändert, sodass die Ersetzungen dort verstümmelt erscheinen. Dieses Verhalten ist Notepad-spezifisch und in allen anderen getesteten Anwendungen nicht reproduzierbar.

---

## Platzhalter

In der Ersetzung können folgende Platzhalter verwendet werden:

| Platzhalter    | Ausgabe                       | Beispiel    |
|----------------|-------------------------------|-------------|
| `{{datum}}`    | Aktuelles Datum               | `30.04.2026`|
| `{{zeit}}`     | Aktuelle Uhrzeit              | `14:35`     |
| `{{zeit_sek}}` | Aktuelle Uhrzeit mit Sekunden | `14:35:42`  |

Zeilenumbrüche in der Ersetzung werden mit `\n` geschrieben und als Enter-Taste gesendet.

---

## Konfiguration

Die Konfigurationsdatei `mz-textbausteine.toml` liegt im selben Verzeichnis wie die .exe. Änderungen werden erst nach einem Neustart des Programms wirksam.

### Aufbau eines Auslöser-Eintrags

```toml
[[textbaustein]]
ausloeser = "@e "
ersetzung = "vorname.nachname@example.com"
```

### Aufbau eines Tastenfolge-Eintrags

```toml
[[textbaustein]]
tastenfolge = "space m"
ersetzung = "Mit freundlichen Grüssen\n\nMarcel Zimmer"
```

Erlaubte Tasten im Tastenfolge-Modus: `space` sowie alle Buchstaben `a` bis `z`.

### Aktuelle Kürzel

**Auslöser:**

| Kürzel   | Ausgabe                                          |
|----------|--------------------------------------------------|
| `@@ `    | `vorname.nachname@example.com`                          |
| `@e `    | `vorname.nachname@example.com`                          |
| `@w `    | `www.marcelzimmer.de`                            |
| `@g `    | `GitHub @marcelzimmer`                           |
| `@x `    | `X @marcelzimmer`                                |
| `@d `    | Aktuelles Datum                                  |
| `@u `    | Aktuelle Uhrzeit                                 |
| `@us `   | Aktuelle Uhrzeit mit Sekunden                    |
| `@du `   | Datum @ Uhrzeit                                  |
| `@mfg `  | `Mit freundlichen Grüssen` + Leerzeile + `Marcel Zimmer` |

**Tastenfolgen [CapsLock + Space + ...]:**

| Taste | Ausgabe                                          |
|-------|--------------------------------------------------|
| `e`   | `vorname.nachname@example.com`                          |
| `w`   | `www.marcelzimmer.de`                            |
| `g`   | `GitHub @marcelzimmer`                           |
| `x`   | `X @marcelzimmer`                                |
| `d`   | Aktuelles Datum                                  |
| `u`   | Aktuelle Uhrzeit                                 |
| `s`   | Aktuelle Uhrzeit mit Sekunden                    |
| `t`   | Datum @ Uhrzeit                                  |
| `m`   | `Mit freundlichen Grüssen` + Leerzeile + `Marcel Zimmer` |

---

## Build

### Voraussetzungen

- Rust [stable, Edition 2021]
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) mit der Komponente „Desktop development with C++" [enthält MSVC-Compiler und Windows SDK]

### Debug-Build

```cmd
cargo build
target\debug\mz-textbausteine.exe
```

### Release-Build

```cmd
cargo build --release
target\release\mz-textbausteine.exe
```

Die Release-Binary ist durch `opt-level = "z"`, LTO und `strip = true` auf minimale Dateigrösse optimiert.

### Installation

Die .exe sowie die `mz-textbausteine.toml` in ein gemeinsames Verzeichnis legen. Für den Autostart kann eine Verknüpfung der .exe in den Windows-Autostart-Ordner gelegt werden:

```
%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
```

---

## Lizenz

### MIT-Lizenz - Nutzung, Rechte und Pflichten

MZ-Textbausteine steht unter der **MIT-Lizenz**. Der vollständige Lizenztext befindet sich in der Datei `LICENSE`.

**Was die MIT-Lizenz erlaubt:**

- **Nutzung** - Die Software darf frei genutzt werden, auch **kommerziell**.
- **Modifikation** - Der Quellcode darf verändert und angepasst werden.
- **Weitergabe** - Die Software darf weitergegeben und weiterverteilt werden, auch in veränderter Form.

**Was ausdrücklich ausgeschlossen ist:**

- **Haftung ist ausgeschlossen.** Der Autor haftet nicht für Schäden, Datenverlust, Fehlfunktionen oder sonstige Folgen, die durch die Nutzung dieser Software entstehen - weder direkt noch indirekt.
- **Keine Gewährleistung.** Die Software wird **„wie sie ist"** bereitgestellt, ohne jegliche Garantie auf Funktionsfähigkeit, Eignung für einen bestimmten Zweck oder Fehlerfreiheit.
- **Keine Support-Pflicht.** Es besteht keinerlei Verpflichtung, Fehler zu beheben, Fragen zu beantworten, Updates bereitzustellen oder irgendeine Form von Wartung oder Support zu leisten.

**Pflichten bei der Nutzung:**

- **Eigenverantwortliche Code-Prüfung:** Wer diese Software einsetzt, ist **selbst dafür verantwortlich**, den Quellcode zu lesen, zu verstehen und zu prüfen. Eine Nutzung ohne vorherige Prüfung erfolgt auf eigenes Risiko.

---

*Diese README wurde am 02.05.2026 erstellt [Version 1.0.0].*
