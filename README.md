# Swaptora escrow program v1

Anchor-программа адресного атомарного обмена SOL, обычных SPL Token и обычных
непрограммируемых Metaplex NFT. Maker сразу помещает свою сторону сделки в
отдельный PDA-vault; только зафиксированный taker может атомарно исполнить оффер.

Program ID: `CUziHakzRiAYkYE5kz5Sb3DzyWor4p51QRpQ99HvKib8`.

## Что реализовано

- `initialize_config` — одноразовое создание неизменяемого `Config` PDA;
- `create_offer` — комиссия, создание Offer, ATA vault и полный депозит maker;
- `accept_offer` — атомарный перевод обеих сторон, закрытие vault ATA и возврат rent maker;
- `cancel_offer` — возврат maker до дедлайна;
- `claim_expired_offer` — возврат после дедлайна, допускается любой caller, но получатель всегда maker;
- TTL ровно 604800 секунд, до 8 позиций с каждой стороны;
- только classic SPL Token Program; Token-2022 отклоняется;
- принимается любой технически валидный classic SPL mint;
- NFT требует `NonFungible`, supply 1, decimals 0, amount 1 и canonical Metadata PDA;
- коллекция NFT может отсутствовать, быть непроверенной или иметь любой адрес;
- mint с freeze authority и frozen token account отклоняются;
- отсутствуют pause, изменение комиссии, emergency withdraw и другие admin-инструкции.

Платформенная комиссия списывается только при успешном `create_offer` и не
возвращается. Все исходные и конечные token accounts — canonical ATA classic
Token Program. Лишние SOL или токены, присланные в vault третьей стороной,
возвращаются maker при завершении, поэтому donation не даёт извлечь активы и не
оставляет их навсегда заблокированными.

Программа проверяет техническую идентичность активов, но не их рыночную
стоимость и не подлинность бренда. Клиент должен показывать точные mint-адреса,
статус коллекции и предупреждать пользователя о похожих или поддельных токенах.

## Сборка и тестирование

Требуемый стек: Rust 1.89+, Solana CLI 3.1.10/platform-tools v1.52, Anchor 1.1.2.
`Cargo.lock` обязателен: в нём закреплены версии, совместимые с SBPF Cargo.

```bash
NO_DNA=1 anchor --version
NO_DNA=1 cargo build-sbf --tools-version v1.52
NO_DNA=1 anchor run mollusk
NO_DNA=1 anchor idl build \
  -o idl/swaptora_contract_sol.json \
  -t types/swaptora_contract_sol.ts
```

Mollusk 0.14 выполняет собранный SBPF ELF, а не host-функцию. 13 тестов покрывают:

- SOL↔SOL и одноразовое исполнение;
- SPL↔SPL через Token/ATA CPI и возврат rent;
- NFT↔NFT с произвольными коллекциями и NFT без коллекции;
- смешанный обмен `2 NFT + SOL + SPL ↔ NFT + SPL`;
- атомарный rollback, если отсутствует один из нескольких активов taker;
- неверного taker, позднее принятие, cancel и permissionless reclaim;
- повторное accept/cancel/reclaim;
- пустые, дублированные и превышающие лимит массивы;
- ровно 8 разрешённых позиций на стороне и превышение лимита;
- подмену vault account;
- NFT без коллекции, Token-2022 и frozen classic token account;
- отсутствие административной поверхности изменения конфигурации.

Сгенерированные интерфейсы: [IDL](idl/swaptora_contract_sol.json) и
[TypeScript IDL type](types/swaptora_contract_sol.ts).

## PDA

```text
Config: ["config"]
Offer:  ["offer", maker, nonce.to_le_bytes()]
Vault:  ["vault", offer]
Vault token account: ATA(owner = Vault PDA, mint, classic Token Program)
```

`Offer` сохраняет обе стороны, fee snapshot, config version, timestamps, status
и canonical bumps. On-chain черновика нет: первый статус — `Active`.

Точная последовательность `remaining_accounts` описана в
[docs/ACCOUNT_SCHEMA.md](docs/ACCOUNT_SCHEMA.md). Клиент обязан передавать ровно
этот набор — лишние, недостающие и подменённые аккаунты отклоняются.

## Обязательный release gate

Текущий `RELEASE_INITIALIZER` — явный placeholder
`3qbR1eZRqXUWroWKKYhbDmR3FfqTHfqSU8zZSxtANzYh` (`[42; 32]`) для локального
Mollusk harness. У него намеренно нет сохранённого приватного ключа. До devnet
или mainnet необходимо заменить в `src/lib.rs` только публичный ключ на адрес
одноразового release initializer и заново собрать/опубликовать verified build.

До mainnet также обязательны: реальный fee receiver, devnet integration suite,
независимый аудит, публикация init transaction и окончательное снятие upgrade
authority. Полный чек-лист: [docs/RELEASE.md](docs/RELEASE.md).

Программа не подписывает и не отправляет транзакции, не хранит приватные ключи и
не включает relayer. Сетевую комиссию `accept_offer` в v1 оплачивает taker.
