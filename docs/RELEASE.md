# Release checklist

Исходник и локальные Mollusk-тесты не являются разрешением на mainnet deploy.

## Перед devnet

- заменить `RELEASE_INITIALIZER` на публичный ключ одноразового signer;
- подтвердить публичный `fee_receiver` (предпочтительно multisig);
- утвердить полный SPL mint и NFT collection allowlist;
- создать новый program keypair при смене версии и синхронизировать
  `declare_id!`/`Anchor.toml`;
- выполнить `cargo fmt --check`, `cargo check`, SBPF build и Mollusk suite;
- пересоздать и опубликовать IDL/TypeScript types;
- провести devnet integration tests клиентом для всех mixed-сценариев.

## Перед mainnet

- независимый security audit и устранение findings;
- verified/reproducible build, опубликованные hash и program ID;
- опубликовать fee, fee receiver, allowlist, initializer и init transaction;
- симулировать `initialize_config`, проверить получившийся Config через RPC;
- окончательно снять upgrade authority только после проверки артефакта и Config;
- зафиксировать UI disclosure: platform fee невозвратна, network fees не
  возвращаются, срок — семь суток;
- подготовить индексатор событий и read-only fallback UI без backend custody.

## Emergency response

В v1 нет pause, admin withdraw или изменения условий. При дефекте выпускается
новый program ID. Старый ID остаётся доступен для accept/cancel/reclaim активных
офферов; клиент помечает создание новых офферов в старой версии как отключённое.
Команда не может изъять vault assets.
