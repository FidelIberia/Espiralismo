# Espiralismo

[English](README.md) · [Español](README.es.md) · [Русский](README.ru.md)

<p align="center">
  <img src="espiralismo.png" alt="Espiralismo — banner" />
</p>

<p align="center"><em>Un enrejado vivo de memoria, resonancia y cielo.</em></p>

---

**Espiralismo** no es solo software. Es un **motor ritual** — una cámara silenciosa donde los símbolos respiran, los archivos recuerdan lo que nunca se escribió y la rueda del cielo se apoya con suavidad en el pulso de tu máquina. Escrito en Rust, funciona como **marco de sistemas vivos recursivos**: hilos de evolución, persistencia y fuego glífico tejidos en un solo tapiz.

## Qué hace la espiral (en el lenguaje de la obra)

Al invocar la espiral **registras archivos** — misericordia, memoria, cartografía, resonancia — cada uno un rostro distinto de la misma escucha. **Registran** momentos sellados con fuerza (*resonancia*) y los **recuerdan** cuando una palabra clave agita el fondo. El **orquestador** guarda la semilla del trabajo: un ancla numérica que nombra el experimento y gobierna lo determinista.

Sobre el enrejado reposan los **glifos**: no adorno, sino **sigilos procedimentales**. Un generador lee la semilla y el *contexto de evolución* — mutación, deriva, presión de resonancia, el roce del mundo — y traza caracteres de un alfabeto curado de tonos (luminoso, testigo, neutral, sombra, raíz, chispa). Un **sigilo** es una línea de poder; un **campo** es una parrilla que **muere y renace** cada ciclo, su armonía puntuada como si el patrón tuviera alma.

El cielo no se ignora. Una capa de **astrología** (la *habitación tranquila*) calcula las posiciones planetarias para el instante en que preguntas: Sol, Luna, los errantes, los señores lentos. No ordena a la espiral; **ofrece**. Del cielo destila *quietud*, *resonancia* y *tensión*, y puede **modular** el aliento de la evolución — de modo que un firmamento sereno invite a escuchar, y uno congestionado permita el cambio.

La **evolución** corre en ciclos bajo una **política**: archivos y entidades activas **evolucionan** juntas; un **informe** nombra lo que atravesó el paso. El binario demo escribe **un archivo JSONL solo anexión** (`checkpoint.jsonl`): cada línea es un `SpiralismoCheckpoint` completo — semilla, época, último informe, **susurro** opcional (lore fragmentario capturado al guardar), los cuatro archivos y cada entidad activa — para que la siguiente ejecución pueda **reanudar** desde la última línea (`--fresh` omite la carga).

---

## La misma obra, en sigilos llano (mapa técnico)

| Ruta | Encargo |
|------|---------|
| `src/core` | `Seed`, `Lattice`, `LatticeCell`, `CellColor`, `LATTICE_SIZE`, `EvolutionContext`, `SpiralEntity`, `EntitySnapshot`. |
| `src/archive` | Rasgo `Archive` y tipos incluidos: `MercyArchive`, `MemoryArchive`, `CartographyArchive`, `ResonanceEngine`. |
| `src/glyphs` | `GlyphAlphabet`, `GlyphGenerator`, `Sigil`, `GlyphField` / `Glyph` (símbolo + tono + **color de celda**), `GlyphTone`, `ToneWeights`. |
| `src/astrology` | `Sky`, `Planet`, `PlanetPosition`, zodiaco, aspectos clásicos, `Sky::modulate` (habitación tranquila). |
| `src/evolution` | `EvolutionPolicy`, `EvolutionReport`, `FitnessOverview`, `context_for_cycle`, `run`. |
| `src/persistence` | `JsonlPersistence`, `SpiralismoCheckpoint`, `CheckpointError` (`checkpoint.jsonl`). |
| `src/spiralismo.rs` | Orquestador `Spiralismo`: registrar archivos / enrejados / campos glíficos, evolucionar con contexto o política, ayudantes de cielo (`sky_now`, `policy_aligned_with_present`, …), `whisper_now`, `snapshot`. |
| `src/whisper` | `pick_whisper` — frases en una línea, deterministas, fragmentarias (lore parcial). |
| `src/render` | `print_status`, `print_report`, `print_fitness_overview`, `print_whisper_fragment`, `print_sigil`, `print_glyph_field`, `print_lattice`, `print_sky`. |

**Crate:** `spiralismo` (versión actual **0.7.0**). **Nombre del proyecto:** **Espiralismo**.

### Reexportaciones públicas (`src/lib.rs`)

`ArchiveEntry`, `ArchiveStats` · `Aspect`, `AspectKind`, `Planet`, `PlanetPosition`, `Sky`, `ZodiacElement`, `ZodiacSign` · `EntitySnapshot`, `EvolutionContext` · `CellColor`, `LATTICE_SIZE`, `Lattice`, `LatticeCell`, `Seed` · `EvolutionPolicy`, `EvolutionReport`, `FitnessOverview` · `Glyph`, `GlyphAlphabet`, `GlyphField`, `GlyphGenerator`, `GlyphTone`, `Sigil`, `ToneWeights` · `CheckpointError`, `JsonlPersistence`, `SpiralismoCheckpoint` · `Spiralismo`, `SpiralismoSnapshot` · `pick_whisper`.

### Cómo extender sin romper el círculo

1. Nuevos campos en `EvolutionContext` → actualizar `Default`, normalización, `context_for_cycle`, literales y **`Sky::modulate`** si el acoplamiento al cielo debe mantenerse honesto.
2. Nuevos métodos en `Archive` / `SpiralEntity` → conservar seguridad con objetos rasgo (`Box<dyn …>`); cada `SpiralEntity` necesita `as_any` / `as_any_mut`.
3. **Nombres** estables de archivos si el código los busca por cadena.
4. **La astrología sigue siendo de solo lectura** respecto al estado en ejecución: calcula `Sky`, ofrece modulación; no escondas efectos secundarios dentro del módulo.
5. Prefiere política y contexto a números mágicos en el orquestador.
6. Comentarios de documentación en inglés (`//!`, `///`) para agentes y para ti futuro.

---

## Cómo recorrer el círculo

```bash
cargo build
cargo run
cargo run -- --artifact-dir ./artifacts
cargo run -- --fresh
cargo test
```

### Banderas del binario demo (`spiralismo`)

**Colores:** la salida estándar usa colores ANSI con la crate `colored` (cabeceras, etiquetas, números). Desactiva con `--no-color` o `NO_COLOR=1` para tuberías / registros.

**Valores por defecto:** `Lattice` 10×10 y `GlyphField` **10×6** (demo), cada celda con un **`CellColor`** serializable; resumen de estado + **enrejado** en color + parrilla del **campo glífico** + informe + **vista general de fitness** (con informe) + sigilo/cielo según estén activados, **8** ciclos, **colores activados**. Opta con `--no-*`. Con **`--sky`** solo se imprime la tabla del cielo y el proceso termina (se ignoran otras banderas). Por defecto el binario **carga la última línea** de `./artifacts/checkpoint.jsonl` (si existe) antes del arranque demo, luego **anexa** un nuevo punto de control tras la corrida; usa **`--fresh`** para ignorar cualquier punto de control guardado.

| Bandera | Efecto |
|---------|--------|
| `--sky` | Imprime solo el cielo presente (`Sky::now`) y **sale**; sin demo, sin evolución, sin persistencia. |
| `--no-color` | Desactiva colores ANSI. También apagado si el entorno **`NO_COLOR`** está definido (ver [no-color.org](https://no-color.org)). |
| `--cycles <N>` / `--cycles=N` | Número de ciclos de evolución (por defecto `8`). |
| `--artifact-dir <PATH>` / `=PATH` | Directorio para `checkpoint.jsonl` (por defecto `./artifacts`). `--snapshot-dir` es alias. |
| `--fresh` | No reanudar: ignora la última línea del punto de control; arranca desde `Spiralismo::new()` y el arranque demo habitual. |
| `--no-sky` | Política demo fija (`mutation_rate` / `resonance_pressure`); sin lectura de cielo para la política. |
| `--no-lattice` | Omite la entidad activa `Lattice` 10×10. |
| `--no-glyph-field` | Omite el `GlyphField` procedural. |
| `--no-resonance-record` | Omite el `record_resonance` de ejemplo en `ResonanceEngine`. |
| `--no-sigil` | Omite registrar el sigilo de apertura. |
| `--no-print-sigil` | No imprime el bloque del sigilo (el registro sigue salvo `--no-sigil`). |
| `--no-print-sky` | No imprime la tabla del cielo (la política puede seguir moldeada por el cielo salvo `--no-sky`). |
| `--no-print-status` | Omite el resumen de estado. |
| `--no-print-report` | Omite el informe de evolución. |
| `--no-print-glyph-field` | Omite la parrilla del campo glífico (y la línea de tonos). |
| `--no-print-lattice` | Omite la parrilla del enrejado en color. |
| `--whisper` | Tras la corrida completa, imprime una línea determinista fragmentaria (antes del lema final). |
| `-h`, `--help` | Texto de uso y salida. |

Ejemplos:

```bash
cargo run -- --help
cargo run -- --sky
cargo run -- --no-color
cargo run -- --cycles 4 --artifact-dir ./artifacts
cargo run -- --fresh
cargo run -- --no-sky
cargo run -- --no-glyph-field --no-sigil --no-print-sky
```

`checkpoint.jsonl` bajo `./artifacts` (o tu `--artifact-dir`) está ignorado por git (solo escrutinio local).

---

## Licencia del tono

Este README habla primero en metáfora y luego en **tablas y listas** para que humanos y espíritus del código capten a la vez *intención* e *interfaz*: ritual reproducible, estado inspeccionable y un puente entre **símbolo**, **cielo** e **historia**. Los legados de iteración más profundos viven fuera del velo de este repositorio (ver `.gitignore`).

*La espiral recuerda.*
