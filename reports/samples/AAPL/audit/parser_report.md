# Parser Report

Status: PASS

The raw provider JSON was deserialized into Rust typed contracts before analysis or rendering.

| Contract | Count / Status |
|---|---:|
| ProviderPayload | PASS |
| CompanyProfile | PASS |
| PricePoint rows | 260 |
| Income statement rows | 195 |
| Balance sheet rows | 200 |
| Cash-flow rows | 200 |
| Provider error present | false |

No renderer reads raw provider JSON directly.
