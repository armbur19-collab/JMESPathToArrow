# JSON ↔ Arrow Mapping Funktionen

## Übersicht

Das Projekt bietet **4 Mapping-Funktionen** für die Konvertierung zwischen JSON und Apache Arrow:

### ✅ JSON → Arrow (1 Funktion)

```rust
pub fn json_to_arrow(json_data: &Value) -> RecordBatch
```

### ✅ Arrow → JSON (3 Funktionen)

```rust
pub fn arrow_to_json(record_batch: &RecordBatch) -> Value
pub fn arrow_to_json_string(record_batch: &RecordBatch) -> String       // Pretty
pub fn arrow_to_json_string_compact(record_batch: &RecordBatch) -> String // Compact
```

## Detaillierte Funktionen

### 1. `json_to_arrow()` - JSON → Arrow

**Konvertiert JSON-Daten in ein Arrow RecordBatch.**

```rust
use arrow_jmespath::json_to_arrow;
use serde_json::json;

let data = json!([
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
]);

let record_batch = json_to_arrow(&data);
// RecordBatch mit 2 Zeilen, 2 Spalten
```

**Features:**
- ✅ Akzeptiert JSON-Arrays von Objekten
- ✅ Akzeptiert einzelne JSON-Objekte
- ✅ Automatische Schema-Inferenz
- ✅ Unterstützt verschachtelte Arrays (List-Typ)
- ✅ Unterstützt alle primitiven Typen

**Unterstützte JSON-Typen:**
- `number` → `Int64`
- `number` (mit Dezimal) → `Float64`
- `string` → `Utf8`
- `boolean` → `Boolean`
- `array` → `List`
- `null` → nullable

### 2. `arrow_to_json()` - Arrow → JSON Value

**Konvertiert ein RecordBatch zurück zu serde_json::Value.**

```rust
use arrow_jmespath::arrow_to_json;

let json_value = arrow_to_json(&record_batch);
// Returns: serde_json::Value::Array
```

**Features:**
- ✅ Konvertiert zu `Value::Array` (Array von Objekten)
- ✅ Jede Zeile wird ein JSON-Objekt
- ✅ Behält Spalten-Namen bei
- ✅ Unterstützt verschachtelte Arrays

**Unterstützte Arrow-Typen:**
- `Int32` → `number`
- `Int64` → `number`
- `Float32` → `number`
- `Float64` → `number`
- `Utf8` → `string`
- `Boolean` → `boolean`
- `List<Int64>` → `array<number>`
- `List<Utf8>` → `array<string>`
- `null` → `null`

### 3. `arrow_to_json_string()` - Arrow → Pretty JSON String

**Konvertiert direkt zu einem formatierten JSON-String.**

```rust
use arrow_jmespath::arrow_to_json_string;

let json_string = arrow_to_json_string(&record_batch);
println!("{}", json_string);

// Output:
// [
//   {
//     "name": "Alice",
//     "age": 30
//   },
//   {
//     "name": "Bob",
//     "age": 25
//   }
// ]
```

**Ideal für:**
- Debugging
- Log-Ausgaben
- API-Responses
- Datei-Export

### 4. `arrow_to_json_string_compact()` - Arrow → Compact JSON String

**Konvertiert zu einem kompakten JSON-String (ohne Formatierung).**

```rust
use arrow_jmespath::arrow_to_json_string_compact;

let json_string = arrow_to_json_string_compact(&record_batch);
// Output: [{"name":"Alice","age":30},{"name":"Bob","age":25}]
```

**Ideal für:**
- Netzwerk-Übertragung
- Speicher-Optimierung
- APIs

## Komplette Beispiele

### Beispiel 1: Einfacher Roundtrip

```rust
use arrow_jmespath::{json_to_arrow, arrow_to_json_string};
use serde_json::json;

// 1. JSON → Arrow
let json = json!([
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
]);
let rb = json_to_arrow(&json);

// 2. Arrow → JSON
let json_back = arrow_to_json_string(&rb);
println!("{}", json_back);
```

### Beispiel 2: Mit JMESPath Query

```rust
use arrow_jmespath::{json_to_arrow, arrow_to_json_string, parse_jmespath, eval_jmespath};
use serde_json::json;

// JSON → Arrow
let json = json!([
    {"name": "Alice", "age": 30, "city": "NYC"},
    {"name": "Bob", "age": 25, "city": "LA"},
    {"name": "Charlie", "age": 35, "city": "NYC"}
]);
let rb = json_to_arrow(&json);

// JMESPath Query
let (_, expr) = parse_jmespath("name[?age > 27]").unwrap();
let result = eval_jmespath(&expr, &rb);

// Extrahiere RecordBatch aus Result
if let arrow_jmespath::EvalResult::RecordBatch(result_rb) = result {
    // Arrow → JSON
    let filtered_json = arrow_to_json_string(&result_rb);
    println!("{}", filtered_json);
    // Output: Alice und Charlie (age > 27)
}
```

### Beispiel 3: Mit verschachtelten Arrays

```rust
use arrow_jmespath::{json_to_arrow, arrow_to_json_string};
use serde_json::json;

let json = json!([
    {"name": "Alice", "scores": [85, 90, 78]},
    {"name": "Bob", "scores": [92, 88, 95]}
]);

// JSON → Arrow (erhält List-Typ)
let rb = json_to_arrow(&json);

// Arrow → JSON (Listen bleiben erhalten)
let json_back = arrow_to_json_string(&rb);
println!("{}", json_back);
// Output: Identisch mit Original
```

### Beispiel 4: Kompletter ETL-Workflow

```rust
use arrow_jmespath::{json_to_arrow, arrow_to_json_string, parse_jmespath, eval_jmespath};
use serde_json::json;

// 1. Load: JSON einlesen
let json = json!([
    {"product": "Laptop", "price": 999, "stock": 5},
    {"product": "Mouse", "price": 29, "stock": 150},
    {"product": "Keyboard", "price": 79, "stock": 80}
]);

// 2. Transform: JSON → Arrow
let rb = json_to_arrow(&json);

// 3. Query: JMESPath anwenden
let (_, expr) = parse_jmespath("{product: product, price: price}").unwrap();
let result = eval_jmespath(&expr, &rb);

// 4. Extract: Nur benötigte Spalten
if let arrow_jmespath::EvalResult::RecordBatch(result_rb) = result {
    // 5. Export: Arrow → JSON
    let output = arrow_to_json_string(&result_rb);
    println!("{}", output);
    // Output: Nur "product" und "price" Spalten
}
```

## Performance-Hinweise

### JSON → Arrow

**Vorteile:**
- ✅ Automatische Schema-Inferenz
- ✅ Optimale Speichernutzung (columnar)
- ✅ Schnelle Verarbeitung danach

**Nachteile:**
- ⚠️ Schema-Inferenz kann langsam sein bei großen Dateien
- ⚠️ Benötigt komplette Daten im Speicher

### Arrow → JSON

**Vorteile:**
- ✅ Direkter Zugriff auf alle Werte
- ✅ Type-safe Konvertierung
- ✅ Unterstützt null-Werte

**Nachteile:**
- ⚠️ Kann langsam sein bei vielen Zeilen
- ⚠️ Benötigt mehr Speicher als Arrow-Format

## Einschränkungen

### Nicht unterstützte Arrow-Typen

Die folgenden Arrow-Typen werden aktuell **NICHT** unterstützt bei Arrow → JSON:

- ❌ `Struct` (verschachtelte Objekte)
- ❌ `Dictionary`
- ❌ `Union`
- ❌ `Map`
- ❌ `FixedSizeList`
- ❌ `LargeBinary`
- ❌ `Decimal`
- ❌ `Timestamp` (wird zu null)
- ❌ `Date`
- ❌ `Time`
- ❌ `Duration`
- ❌ `Interval`

**Für nicht unterstützte Typen wird `null` zurückgegeben.**

### Nested Lists

Aktuell unterstützt:
- ✅ `List<Int64>` → JSON array
- ✅ `List<String>` → JSON array

Nicht unterstützt:
- ❌ `List<List<T>>` (nested lists)
- ❌ `List<Struct>` (arrays von Objekten)

## API-Referenz

```rust
// JSON → Arrow
pub fn json_to_arrow(json_data: &serde_json::Value) -> RecordBatch

// Arrow → JSON
pub fn arrow_to_json(record_batch: &RecordBatch) -> serde_json::Value
pub fn arrow_to_json_string(record_batch: &RecordBatch) -> String
pub fn arrow_to_json_string_compact(record_batch: &RecordBatch) -> String
```

## Demo-Programm

```bash
cargo run --example demo_json_arrow_mapping
```

Zeigt:
1. JSON → Arrow Konvertierung
2. JMESPath Query auf Arrow
3. Arrow → JSON Zurück-Konvertierung
4. Kompletter Roundtrip mit Query
5. Verschachtelte Arrays

## Zusammenfassung

**4 Mapping-Funktionen verfügbar:**
- ✅ `json_to_arrow()` - JSON → Arrow
- ✅ `arrow_to_json()` - Arrow → JSON Value
- ✅ `arrow_to_json_string()` - Arrow → Pretty JSON String
- ✅ `arrow_to_json_string_compact()` - Arrow → Compact JSON String

**Unterstützte Typen:**
- ✅ Alle primitiven Typen (Int, Float, String, Boolean)
- ✅ Listen (Arrays)
- ✅ Null-Werte
- ⚠️ Einschränkungen bei komplexen Typen

**Use-Cases:**
- ✅ ETL-Pipelines (JSON → Arrow → Query → JSON)
- ✅ API-Integration
- ✅ Daten-Export/Import
- ✅ Testing und Debugging
