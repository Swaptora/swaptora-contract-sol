/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/swaptora_contract_sol.json`.
 */
export type SwaptoraContractSol = {
  "address": "CUziHakzRiAYkYE5kz5Sb3DzyWor4p51QRpQ99HvKib8",
  "metadata": {
    "name": "swaptoraContractSol",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Immutable v1 addressed escrow for atomic Solana asset swaps"
  },
  "instructions": [
    {
      "name": "acceptOffer",
      "discriminator": [
        227,
        82,
        234,
        131,
        1,
        18,
        48,
        2
      ],
      "accounts": [
        {
          "name": "taker",
          "writable": true,
          "signer": true
        },
        {
          "name": "maker",
          "writable": true
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "offer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "offer.maker",
                "account": "offer"
              },
              {
                "kind": "account",
                "path": "offer.nonce",
                "account": "offer"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "offer"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "cancelOffer",
      "discriminator": [
        92,
        203,
        223,
        40,
        92,
        89,
        53,
        119
      ],
      "accounts": [
        {
          "name": "maker",
          "writable": true,
          "signer": true
        },
        {
          "name": "offer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "offer.maker",
                "account": "offer"
              },
              {
                "kind": "account",
                "path": "offer.nonce",
                "account": "offer"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "offer"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "claimExpiredOffer",
      "discriminator": [
        211,
        8,
        111,
        146,
        136,
        151,
        233,
        238
      ],
      "accounts": [
        {
          "name": "caller",
          "signer": true
        },
        {
          "name": "maker",
          "writable": true
        },
        {
          "name": "offer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "offer.maker",
                "account": "offer"
              },
              {
                "kind": "account",
                "path": "offer.nonce",
                "account": "offer"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "offer"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "createOffer",
      "discriminator": [
        237,
        233,
        192,
        168,
        248,
        7,
        249,
        241
      ],
      "accounts": [
        {
          "name": "maker",
          "writable": true,
          "signer": true
        },
        {
          "name": "taker"
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "offer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "maker"
              },
              {
                "kind": "arg",
                "path": "nonce"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "offer"
              }
            ]
          }
        },
        {
          "name": "feeReceiver",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "nonce",
          "type": "u64"
        },
        {
          "name": "takerAddress",
          "type": "pubkey"
        },
        {
          "name": "makerAssets",
          "type": {
            "vec": {
              "defined": {
                "name": "assetItem"
              }
            }
          }
        },
        {
          "name": "takerAssets",
          "type": {
            "vec": {
              "defined": {
                "name": "assetItem"
              }
            }
          }
        }
      ]
    },
    {
      "name": "initializeConfig",
      "discriminator": [
        208,
        127,
        21,
        1,
        194,
        190,
        196,
        70
      ],
      "accounts": [
        {
          "name": "releaseInitializer",
          "writable": true,
          "signer": true
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "feeReceiver",
          "type": "pubkey"
        },
        {
          "name": "platformFeeLamports",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "config",
      "discriminator": [
        155,
        12,
        170,
        224,
        30,
        250,
        204,
        130
      ]
    },
    {
      "name": "offer",
      "discriminator": [
        215,
        88,
        60,
        71,
        170,
        162,
        73,
        229
      ]
    }
  ],
  "events": [
    {
      "name": "configInitialized",
      "discriminator": [
        181,
        49,
        200,
        156,
        19,
        167,
        178,
        91
      ]
    },
    {
      "name": "offerAccepted",
      "discriminator": [
        81,
        238,
        238,
        115,
        140,
        18,
        8,
        20
      ]
    },
    {
      "name": "offerCancelled",
      "discriminator": [
        45,
        42,
        175,
        214,
        51,
        192,
        154,
        9
      ]
    },
    {
      "name": "offerCreated",
      "discriminator": [
        31,
        236,
        215,
        144,
        75,
        45,
        157,
        87
      ]
    },
    {
      "name": "offerExpiredReclaimed",
      "discriminator": [
        237,
        156,
        246,
        141,
        25,
        89,
        120,
        103
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "makerEqualsTaker",
      "msg": "Maker and taker must be different wallets"
    },
    {
      "code": 6001,
      "name": "invalidAssetList",
      "msg": "Asset list is empty or contains an invalid item"
    },
    {
      "code": 6002,
      "name": "duplicateMint",
      "msg": "An asset mint appears more than once on the same side"
    },
    {
      "code": 6003,
      "name": "tooManyAssets",
      "msg": "Asset count exceeds the immutable v1 limit"
    },
    {
      "code": 6004,
      "name": "unsupportedTokenStandard",
      "msg": "NFT token standard is unsupported"
    },
    {
      "code": 6005,
      "name": "invalidNftAmount",
      "msg": "NFT amount must be exactly one"
    },
    {
      "code": 6006,
      "name": "invalidTokenAccount",
      "msg": "Token account, mint, metadata, owner, or token program is invalid"
    },
    {
      "code": 6007,
      "name": "insufficientBalance",
      "msg": "Source account has insufficient balance"
    },
    {
      "code": 6008,
      "name": "unauthorizedMaker",
      "msg": "Only the offer maker may perform this action"
    },
    {
      "code": 6009,
      "name": "unauthorizedTaker",
      "msg": "Only the addressed offer taker may accept"
    },
    {
      "code": 6010,
      "name": "offerNotActive",
      "msg": "Offer is not active"
    },
    {
      "code": 6011,
      "name": "offerExpired",
      "msg": "Offer has expired"
    },
    {
      "code": 6012,
      "name": "offerNotExpired",
      "msg": "Offer has not expired"
    },
    {
      "code": 6013,
      "name": "vaultBalanceMismatch",
      "msg": "Vault holds less than the committed amount or is malformed"
    },
    {
      "code": 6014,
      "name": "invalidRecipientAccount",
      "msg": "Recipient account does not match the committed owner and mint"
    },
    {
      "code": 6015,
      "name": "feeConfigurationInvalid",
      "msg": "Immutable fee configuration or receiver account is invalid"
    },
    {
      "code": 6016,
      "name": "unauthorizedInitializer",
      "msg": "Release initializer does not match the key compiled into this release"
    },
    {
      "code": 6017,
      "name": "arithmeticOverflow",
      "msg": "Arithmetic overflow"
    },
    {
      "code": 6018,
      "name": "invalidRemainingAccounts",
      "msg": "Unexpected, missing, duplicate, or writable remaining account"
    }
  ],
  "types": [
    {
      "name": "assetItem",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "kind",
            "type": {
              "defined": {
                "name": "assetKind"
              }
            }
          },
          {
            "name": "mint",
            "docs": [
              "Pubkey::default() is the canonical sentinel for native SOL."
            ],
            "type": "pubkey"
          },
          {
            "name": "amount",
            "docs": [
              "Lamports for SOL, base units for SPL, exactly one for NFT."
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "assetKind",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "sol"
          },
          {
            "name": "splToken"
          },
          {
            "name": "nft"
          }
        ]
      }
    },
    {
      "name": "config",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "feeReceiver",
            "type": "pubkey"
          },
          {
            "name": "platformFeeLamports",
            "type": "u64"
          },
          {
            "name": "maxAssetsPerSide",
            "type": "u8"
          },
          {
            "name": "version",
            "type": "u16"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "configInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "config",
            "type": "pubkey"
          },
          {
            "name": "feeReceiver",
            "type": "pubkey"
          },
          {
            "name": "platformFeeLamports",
            "type": "u64"
          },
          {
            "name": "version",
            "type": "u16"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offer",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "taker",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u64"
          },
          {
            "name": "status",
            "type": {
              "defined": {
                "name": "offerStatus"
              }
            }
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "expiresAt",
            "type": "i64"
          },
          {
            "name": "makerAssets",
            "type": {
              "vec": {
                "defined": {
                  "name": "assetItem"
                }
              }
            }
          },
          {
            "name": "takerAssets",
            "type": {
              "vec": {
                "defined": {
                  "name": "assetItem"
                }
              }
            }
          },
          {
            "name": "platformFeeLamports",
            "type": "u64"
          },
          {
            "name": "configVersion",
            "type": "u16"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "vaultBump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "offerAccepted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offer",
            "type": "pubkey"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "taker",
            "type": "pubkey"
          },
          {
            "name": "assetsHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offerCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offer",
            "type": "pubkey"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "taker",
            "type": "pubkey"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offerCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offer",
            "type": "pubkey"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "taker",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u64"
          },
          {
            "name": "expiresAt",
            "type": "i64"
          },
          {
            "name": "assetsHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offerExpiredReclaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offer",
            "type": "pubkey"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "taker",
            "type": "pubkey"
          },
          {
            "name": "caller",
            "type": "pubkey"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offerStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "active"
          },
          {
            "name": "accepted"
          },
          {
            "name": "cancelled"
          },
          {
            "name": "reclaimed"
          }
        ]
      }
    }
  ]
};
