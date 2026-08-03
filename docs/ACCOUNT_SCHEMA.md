# Account schema v1

Все списки ниже идут после фиксированных Anchor accounts из IDL. Активы
обрабатываются строго в порядке, записанном в `maker_assets`/`taker_assets`.
SOL не добавляет dynamic accounts.

## `create_offer`

Сначала для каждого non-SOL maker asset:

```text
SPL: [mint, maker_source_ata(w), vault_ata(w)]
NFT: [mint, maker_source_ata(w), vault_ata(w), metadata]
```

Затем для каждого non-SOL taker asset (проверка условий без требования текущего
владения):

```text
SPL: [mint]
NFT: [mint, metadata]
```

`vault_ata` создаётся idempotent CPI, payer = maker. `maker_source_ata` обязан
быть canonical ATA maker.

## `accept_offer`

Сначала taker assets, переводимые maker:

```text
SPL: [mint, taker_source_ata(w), maker_destination_ata(w)]
NFT: [mint, taker_source_ata(w), maker_destination_ata(w), metadata]
```

Отсутствующий `maker_destination_ata` создаётся idempotent CPI, payer = taker.

Затем maker assets из escrow:

```text
SPL: [mint, vault_ata(w), taker_destination_ata(w), maker_refund_ata(w)]
NFT: [mint, vault_ata(w), taker_destination_ata(w), maker_refund_ata(w), metadata]
```

Отсутствующий `taker_destination_ata` создаётся с payer = taker. Точный объём
оффера поступает taker, случайный surplus того же mint возвращается в
`maker_refund_ata`, после чего пустой vault ATA закрывается в пользу maker.

## `cancel_offer` / `claim_expired_offer`

Для каждого non-SOL maker asset:

```text
[mint, vault_ata(w), maker_destination_ata(w)]
```

Возвращается весь баланс vault token account (включая donation), ATA закрывается
в пользу maker. Весь SOL-баланс canonical vault PDA также направляется maker.

## Общие проверки

- token/ATA program IDs фиксированы на classic SPL Token и Associated Token;
- mint, ATA owner и ATA address выводятся программой независимо;
- Metadata owner и PDA проверяются против Metaplex Token Metadata program;
- NFT collection не является условием допуска и может отсутствовать;
- source balance должен быть достаточен, vault balance — не меньше commitment;
- account count должен совпасть точно;
- все CPI выполняются в одной Solana transaction, ошибка откатывает их целиком.
