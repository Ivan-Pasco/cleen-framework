# Frame i18n Specification (16)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 1.0
**Location:** `/documents/specification/16_frame_i18n.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

## 1. Purpose

`frame.i18n` provides internationalization (i18n) support for Clean Language applications. It covers translation key lookup, plural form selection via CLDR rules, locale detection, and locale-aware formatting of numbers, currencies, and dates. The same API works on both the server (reads translation files from disk) and in the browser (fetches translation files over HTTP).

**Goals:**
- Simple `t(key, params)` API for translators and developers.
- Plural forms without special syntax — just key suffixes.
- Locale detection from HTTP headers, cookies, query parameters, or manual override.
- Standard `Intl`-based formatting on all platforms.

---

## 2. File Layout

```
app/i18n/           # Owned by frame.i18n plugin
  en.json           # English translations (BCP 47 tag as filename)
  fr.json           # French translations
  de.json           # German translations
  es-MX.json        # Mexican Spanish translations
main.cln            # Contains locale: configuration block
```

Translation files are loaded at startup by `_i18n_load()`. The server reads them from disk; the browser fetches them as JSON.

---

## 3. Configuration — `locale:` Block

The `locale:` block configures the i18n system. It appears at the top level of `main.cln`:

```clean
locale:
    default   = "en"
    fallback  = "en"
    detection = "header"
    path      = "app/i18n"
```

| Field | Default | Description |
|-------|---------|-------------|
| `default` | `"en"` | Active locale BCP 47 tag used when detection finds no match |
| `fallback` | `"en"` | Locale tried when a key is absent in the active locale |
| `detection` | `"header"` | How the active locale is selected per request (see §7) |
| `path` | `"app/i18n"` | Path to the translation JSON files |

The plugin expands `locale:` into a `start:` block that calls `_i18n_load()` for each locale and `_i18n_set_locale()` with the default locale.

---

## 4. Translation File Format

Translation files are JSON objects with dot-separated keys:

```json
{
    "common": {
        "save": "Save",
        "cancel": "Cancel",
        "delete": "Delete"
    },
    "errors": {
        "required": "{field} is required",
        "minLength": "{field} must be at least {min} characters",
        "serverError": "Something went wrong. Please try again."
    },
    "users": {
        "greeting": "Hello, {name}!",
        "count_zero": "No users",
        "count_one": "One user",
        "count_other": "{count} users"
    }
}
```

### 4.1 Interpolation

Use `{placeholder}` syntax for variable substitution. Placeholders are replaced by values from the params JSON object passed to `t()`:

```clean
// Translates "errors.required" with {field} = "Email"
string msg = t("errors.required", "{\"field\": \"Email\"}")
// → "Email is required"
```

### 4.2 Plural Forms

Plural keys append a CLDR category suffix to the base key. The plural category is selected based on the count value and the active locale's plural rules:

| Suffix | CLDR category | When used |
|--------|--------------|-----------|
| `_zero` | zero | count == 0 (languages with zero form) |
| `_one` | one | Singular (count == 1 in most languages) |
| `_two` | two | Dual form (Arabic, Hebrew, etc.) |
| `_few` | few | Paucal (Slavic languages, etc.) |
| `_many` | many | Many (Polish, Russian, etc.) |
| `_other` | other | Default / catch-all — required for all pluralised keys |

At minimum, provide `_one` and `_other` for English content. Always provide `_other` — it is the fallback for any missing category.

---

## 5. The `t()` Function

`t(key, params)` is the primary translation function:

```clean
// Simple key lookup
string label = t("common.save", "{}")

// With interpolation
string greeting = t("users.greeting", "{\"name\": \"" + user.name + "\"}")

// Nested key path
string error = t("errors.required", "{\"field\": \"Password\"}")
```

Signature: `t(key: string, params: string) -> string`

- `key` — dot-separated key path into the translation JSON
- `params` — JSON object string for `{placeholder}` substitution
- Returns the translated string, or the `key` itself if not found in any locale

---

## 6. The `tc()` Function — Plurals

`tc(key, count, params)` selects the correct plural form:

```clean
// "No users" / "One user" / "3 users"
string label = tc("users.count", users.length(), "{}")

// With additional params
string msg = tc("cart.items", cart.size(), "{\"total\": \"" + total + "\"}")
```

Signature: `tc(key: string, count: integer, params: string) -> string`

The function:
1. Determines the CLDR plural category for `count` using the active locale's rules.
2. Appends the category suffix: `key_zero`, `key_one`, `key_few`, `key_many`, or `key_other`.
3. Falls back through `key_other` if the specific category key is missing.
4. `count` is always available as `{count}` in the resolved template.

---

## 7. Locale Detection

On each server request, the active locale is selected in this order:

1. **Query parameter:** `?locale=fr` overrides all other methods.
2. **Cookie:** The `cl_locale` cookie value, if present and a loaded locale.
3. **Accept-Language header:** Parsed and matched against loaded locales using BCP 47 quality values.
4. **Default locale:** The `default` value from the `locale:` block.

When `detection = "manual"` is configured, only explicit `i18n.setLocale()` calls change the locale; the first three detection methods are skipped.

In the browser, the locale is a module-level variable. `i18n.setLocale()` changes it and fires a `cl-locale-change` custom event on `window`.

---

## 8. Locale Management Functions

```clean
// Read the active locale
string locale = i18n.locale()
// → "en", "fr-CA", etc.

// Change the active locale
i18n.setLocale("fr")
// Server: scoped to the current request
// Browser: persists for the module lifetime; fires cl-locale-change
```

---

## 9. Locale-Aware Formatting

### 9.1 Numbers

```clean
// Format with active locale
string n = i18n.formatNumber(1234567.89, "", "{}")
// en-US → "1,234,567.89"
// de-DE → "1.234.567,89"

// Explicit locale and options
string n2 = i18n.formatNumber(0.1234, "en-US", "{\"minimumFractionDigits\": 2, \"maximumFractionDigits\": 2}")
// → "0.12"
```

Signature: `i18n.formatNumber(value: number, locale: string, options: string) -> string`

Pass `""` for `locale` to use the active locale. `options` is a JSON object corresponding to `Intl.NumberFormat` options.

### 9.2 Currencies

```clean
// Format as currency
string price = i18n.formatCurrency(49.99, "USD", "")
// en-US → "$49.99"
// de-DE → "49,99 $"

string euros = i18n.formatCurrency(1299.0, "EUR", "de-DE")
// → "1.299,00 €"
```

Signature: `i18n.formatCurrency(value: number, currency: string, locale: string) -> string`

`currency` is an ISO 4217 code (`"USD"`, `"EUR"`, `"GBP"`, `"JPY"`, etc.). Pass `""` for `locale` to use the active locale.

### 9.3 Dates

```clean
// Format a Unix timestamp
integer now = _time_now()
string date = i18n.formatDate(now, "medium", "")
// en-US → "Jun 2, 2026"
// fr-FR → "2 juin 2026"

string full = i18n.formatDate(now, "full", "ja-JP")
// → "2026年6月2日火曜日"
```

Signature: `i18n.formatDate(timestamp: number, style: string, locale: string) -> string`

| Style | Example (en-US) |
|-------|----------------|
| `"short"` | `6/2/26` |
| `"medium"` | `Jun 2, 2026` |
| `"long"` | `June 2, 2026` |
| `"full"` | `Tuesday, June 2, 2026` |

---

## 10. Using `t()` in HTML Templates

Translation functions are available inside `{{ }}` interpolation in HTML pages:

```html
<!-- app/web/pages/dashboard.html -->
<h1>{{ t("users.greeting", params) }}</h1>
<p>{{ tc("users.count", userCount, "{}") }}</p>
<button>{{ t("common.save", "{}") }}</button>
```

The companion `.cln` file sets up the variables:

```clean
// app/web/pages/dashboard.cln
functions:
    any load(Request req)
        return {
            params: "{\"name\": \"" + req.auth.userId + "\"}",
            userCount: User.count()
        }
```

---

## 11. RTL Language Support

When `i18n.setLocale()` is called with a right-to-left locale, the runtime automatically sets `dir="rtl"` on the document root:

| Language | BCP 47 tag | Direction |
|----------|-----------|-----------|
| Arabic | `ar`, `ar-*` | RTL |
| Hebrew | `he` | RTL |
| Persian | `fa` | RTL |
| Urdu | `ur` | RTL |

Server-rendered HTML includes `data-locale-dir="rtl"` on the root element when the selected locale is RTL, enabling CSS targeting without JavaScript:

```html
<html lang="ar" data-locale-dir="rtl">
```

```css
[data-locale-dir="rtl"] {
    direction: rtl;
    text-align: right;
}
```

---

## 12. Complete Example

### Translation files

```json
// app/i18n/en.json
{
    "nav": {
        "home": "Home",
        "about": "About",
        "contact": "Contact"
    },
    "products": {
        "title": "Products",
        "count_zero": "No products found",
        "count_one": "One product found",
        "count_other": "{count} products found",
        "price": "Price: {price}"
    },
    "errors": {
        "notFound": "The page you are looking for does not exist."
    }
}
```

```json
// app/i18n/fr.json
{
    "nav": {
        "home": "Accueil",
        "about": "À propos",
        "contact": "Contact"
    },
    "products": {
        "title": "Produits",
        "count_zero": "Aucun produit trouvé",
        "count_one": "Un produit trouvé",
        "count_other": "{count} produits trouvés",
        "price": "Prix : {price}"
    },
    "errors": {
        "notFound": "La page que vous recherchez n'existe pas."
    }
}
```

### `main.cln`

```clean
locale:
    default   = "en"
    fallback  = "en"
    detection = "header"
    path      = "app/i18n"
```

### Endpoint using locale detection

```clean
// app/server/api/products.cln
endpoints:
    GET "/api/products" :
        list<Product> products = Product.find:
            where: active == true
        string countMsg = tc("products.count", products.length(), "{}")
        return json({ products: products, message: countMsg, locale: i18n.locale() })
```

### Page with formatted price

```clean
// app/web/pages/product.cln
functions:
    any load(Request req)
        string id = req.params.id
        Product p = Product.first:
            where: id == id.toInteger()
        string priceStr = i18n.formatCurrency(p.price, "USD", "")
        return {
            product: p,
            priceLabel: t("products.price", "{\"price\": \"" + priceStr + "\"}")
        }
```

```html
<!-- app/web/pages/product.html -->
<article>
    <h1>{{ product.name }}</h1>
    <p>{{ priceLabel }}</p>
</article>
```

---

**End of Document 16 — Frame i18n Specification**
