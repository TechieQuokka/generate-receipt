# generate-receipt

Generate ESC/POS thermal receipts from JSON data + a simple template ("skin") language. Renders bold/underline/alignment/size, dividers, images, QR codes, dot-leader rows, loops, conditionals, and a `match`/`case` block for per-item-type formatting.

Output is raw ESC/POS bytes: write to a `.bin` file, or send directly over TCP to a real printer or an emulator like [escpresso](https://github.com/jflaflamme/escpresso) on `localhost:9100`.

## Build

```bash
cargo build --release
```

## Usage

```bash
cargo run -- \
  --data examples/burgerking_receipt.json \
  --skin skins/coffee_shop.skin \
  --tax 10 --total --sum --item-count \
  --send
```

### CLI options

| Flag | Description |
| --- | --- |
| `--data <path>` | Receipt data JSON file (required) |
| `--skin <path>` | Skin/template file (required) |
| `--tax <percent>` | Tax rate, e.g. `10` for 10% |
| `--tax-mode <inclusive\|exclusive>` | Whether item prices already include tax (default: `exclusive`) |
| `--sum` | Show pre-tax/pre-discount subtotal |
| `--total` | Show final total (tax, discount, service charge applied) |
| `--average` | Show average price per item |
| `--discount <value>` | Overall discount, e.g. `10%` or `1000` (fixed amount) |
| `--service-charge <percent>` | Service charge in percent |
| `--currency <code>` | Currency code, default `KRW` |
| `--item-count` | Show total item count |
| `--footer-text <text>` | Footer line; repeatable. Prefix with `qr:` to render as a QR code instead of text |
| `--out <path>` | Save rendered bytes to a `.bin` file |
| `--send` | Send bytes to `localhost:9100` (real printer or emulator) |
| `--cpl <n>` | Characters per line — `32` for 58mm paper, `48` for 80mm (default: `48`) |
| `--margin <n>` | Left/right margin in characters (default: `2`) |

## Data format

See `examples/*.json`. Top-level fields:

- `store`: name, address, phone, business_number, owner_name, website, logo_text, logo_path, qr_content
- `meta`: receipt_number, timestamp, cashier, order_type, table_number
- `items[]`: name, price, quantity, item_type (`normal` | `bundle` | `discount`), emphasis, strikethrough, note, children (recursive, for bundle sub-items)

## Skin (template) syntax

See `skins/coffee_shop.skin` for a full reference skin using every directive.

| Directive | Description |
| --- | --- |
| `{{path.to.value}}` | Interpolate a value from the data (dot-path lookup) |
| `@align:left\|center\|right` | Set alignment |
| `@bold:on\|off` | Toggle bold |
| `@size:normal\|double` | Toggle font size |
| `@underline:on\|off` | Toggle underline |
| `@divider` | Horizontal rule |
| `@blank` | Blank line |
| `@image:{{path}}` | Embed an image (resolved to a file path) |
| `@qr:{{content}}` | Render a QR code |
| `@row:{{left}}\|{{right}}` | Two-column line, dot-leader filled between left and right |
| `@cut` | Paper cut |
| `@if:condition` / `@else` / `@endif` | Conditional block |
| `@foreach:list as var` / `@endforeach` | Loop over an array |
| `@match:value` / `@case:x` / `@endmatch` | Branch on a value |

Computed aggregates (`computed.sum`, `computed.tax`, `computed.total`, etc.) and CLI-supplied `footer` entries are injected into the data context alongside `store`/`meta`/`items` before rendering.

## Examples

```bash
# Save to a file instead of sending
cargo run -- --data examples/sample_receipt.json --skin skins/coffee_shop.skin --out receipt.bin

# Full aggregate breakdown with tax + discount + service charge
cargo run -- \
  --data examples/burgerking_receipt.json \
  --skin skins/coffee_shop.skin \
  --tax 10 --tax-mode exclusive \
  --discount 10% --service-charge 5 \
  --sum --total --average --item-count \
  --footer-text "Follow us @bluebean" \
  --footer-text "qr:https://bluebean.example.com/survey" \
  --send
```

## Testing with an emulator

[escpresso](https://github.com/jflaflamme/escpresso) is a virtual ESC/POS printer with a GUI preview, useful for testing without real hardware:

```bash
cargo install escpresso
escpresso
```

It opens a TCP server on `localhost:9100`; `--send` targets that port.

> **Note:** escpresso currently parses but does not apply code page / Kanji mode commands, so non-Latin text (e.g. Korean) will render incorrectly in the emulator preview. This is an emulator limitation, not a bug in this tool.

## License

TBD
