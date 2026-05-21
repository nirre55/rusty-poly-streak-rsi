# rusty-poly-streak-rsi

Bot de trading live en Rust pour les marchés Polymarket Up/Down, alimenté par les bougies Binance.

Le projet supporte plusieurs stratégies configurables, le dry-run, les ordres réels Polymarket via CLOB, le suivi des positions, les logs CSV et une gestion optionnelle de la taille de position.

## Fonctionnalités

- WebSocket Binance pour bougies fermées.
- Warmup historique Binance au démarrage.
- Stratégies RSI/reversal et ensembles de micro-règles.
- Modes `dry-run`, `market` et `limit`.
- Résolution des marchés Polymarket par slug.
- Cache et warmup du client Polymarket CLOB.
- Suivi des ordres ouverts via `pending_orders.json`.
- Validation du résultat avec la bougie Binance cible.
- Persistance du money management dans `money_state.json`.
- Logs de trades dans `trades.csv`.

## Stratégies disponibles

| Nom | Marché typique | Description |
|---|---|---|
| `three_candle_rsi7_reversal` | BTC 5m | Reversal après 3 bougies de même couleur + RSI7 + filtres range/body. |
| `btc_5m_rules_90_min_votes_1` | BTC 5m | Ensemble de 90 micro-règles. |
| `btc_5m_rules_23_min_votes_1` | BTC 5m | Ensemble combine de 23 micro-strategies, `min_votes=1`. |
| `btc_15m_rules_18_min_votes_1` | BTC 15m | Ensemble de 18 micro-règles. |
| `eth_5m_rules_25_min_votes_1` | ETH 5m | Ensemble de 25 micro-règles. |
| `eth_15m_rules_24_min_votes_1` | ETH 15m | Ensemble de 24 micro-règles. |

La stratégie active se choisit avec `STRATEGY` ou via `STRATEGY_CONFIG`.

## Installation

```bash
rustup update stable
cargo build --locked
```

Copier ensuite l'exemple d'environnement :

```bash
cp .env.example .env
```

## Configuration

Variables principales :

| Variable | Description | Défaut |
|---|---|---|
| `EXECUTION_MODE` | `dry-run`, `market` ou `limit`. Requis. | Aucun |
| `STRATEGY_CONFIG` | Fichier `.env` de stratégie à charger avant `.env`. | Aucun |
| `STRATEGY` | Nom de la stratégie. | `three_candle_rsi7_reversal` |
| `SYMBOL` | Symbole Binance, ex. `btcusdt`, `ethusdt`. | `btcusdt` |
| `INTERVAL` | Intervalle Binance, ex. `5m`, `15m`. | `5m` |
| `POLYMARKET_SLUG_PREFIX` | Préfixe du slug Polymarket. | `btc-updown-5m` |
| `POLYMARKET_API_URL` | URL CLOB Polymarket. | `https://clob.polymarket.com` |
| `TRADE_AMOUNT_USDC` | Montant fixe par trade. | `10.0` |
| `TRADE_AMOUNT_PCT` | Pourcentage du solde USDC à utiliser. Mutuellement exclusif avec `TRADE_AMOUNT_USDC`. | `0.0` |
| `ENSEMBLE_MIN_VOTES` | Nombre minimal de votes pour les stratégies ensemble. | `1` |
| `LIMIT_PRICE_OFFSET` | Offset ajouté au meilleur ask en mode `limit`. | `0.01` |
| `MARTINGALE_MULTIPLIER` | Multiplicateur après une perte. `1.0` désactive la martingale. | `1.0` |
| `MARTINGALE_MAX_AMOUNT` | Plafond martingale. `0.0` désactive le plafond. | `0.0` |
| `EXCLUDED_DAYS` | Jours exclus, ex. `sat,sun`. | Vide |
| `EXCLUDED_HOURS` | Plages UTC exclues, ex. `0-9,22-24`. | Vide |
| `LOGS_DIR` | Dossier des logs et états runtime. | `logs` |

Variables nécessaires aux modes réels :

| Variable | Description |
|---|---|
| `POLYMARKET_PRIVATE_KEY` | Clé privée EVM du signer. |
| `POLYMARKET_FUNDER` | Adresse funder si différente de l'EOA. |
| `POLYMARKET_SIGNATURE_TYPE` | `0` = EOA, `1` = proxy, `2` = Gnosis Safe. |

## Lancement

Dry-run direct :

```bash
EXECUTION_MODE=dry-run cargo run
```

Avec un fichier de stratégie :

```bash
STRATEGY_CONFIG=configs/btc_ensemble.env cargo run
```

Lancer les quatre stratégies ensemble :

```powershell
.\start_all.ps1
```

ou :

```bash
./start_all.sh
```

## Logs et états

Chaque instance doit idéalement utiliser son propre `LOGS_DIR`.

Fichiers produits :

| Fichier | Rôle |
|---|---|
| `trades.csv` | Historique des signaux, ordres, latences et outcomes. |
| `pending_orders.json` | Ordres ouverts à suivre après restart. |
| `money_state.json` | État du money management. |

Les fichiers CSV et JSON runtime sous `logs/` sont ignorés par Git.

## Reconciliation officielle Polymarket

Le bot valide les trades en live avec la bougie Binance cible afin de continuer a trader sans attendre la resolution officielle Polymarket. Cette validation est rapide, mais elle reste une estimation operationnelle: les marches Up/Down Polymarket se resolvent selon la source indiquee dans leurs regles, souvent Chainlink.

Le binaire `reconcile_outcomes` sert d'audit quotidien. Il lit `trades.csv`, extrait les slugs `btc-updown-*` et `eth-updown-*`, recupere le marche via Gamma, puis recupere le token gagnant officiel via le CLOB Polymarket. Il ecrit ensuite un rapport append-only dans `reconciliation_report.csv`.

Le script ne modifie pas `trades.csv` et ne change pas `money_state.json`. Il signale seulement les ecarts entre le resultat Binance utilise en live et le resultat officiel Polymarket.

Colonnes principales du rapport :

| Colonne | Role |
|---|---|
| `prediction` | Prediction du bot (`UP` ou `DOWN`). |
| `binance_outcome` | Resultat enregistre en live dans `trades.csv`. |
| `official_winner` | Outcome gagnant selon Polymarket (`UP` ou `DOWN`). |
| `official_outcome` | Resultat officiel calcule pour notre prediction. |
| `reconciliation` | `MATCH`, `MISMATCH`, `PENDING` ou `ERROR`. |

Execution Windows PowerShell :

```powershell
.\reconcile_outcomes.ps1
.\reconcile_outcomes.ps1 configs/eth_ensemble.env
.\reconcile_outcomes.ps1 configs/btc_ensemble.env -Release
```

Execution Linux/macOS :

```bash
chmod +x ./reconcile_outcomes.sh
./reconcile_outcomes.sh
./reconcile_outcomes.sh configs/eth_ensemble.env
RELEASE=1 ./reconcile_outcomes.sh configs/btc_ensemble.env
```

Execution directe Cargo :

```bash
STRATEGY_CONFIG=configs/btc_ensemble.env cargo run --locked --bin reconcile_outcomes
```

## Tests et qualité

```bash
cargo fmt -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Ajouter une stratégie

1. Créer un fichier dans `src/strategies/`.
2. Implémenter le trait `Strategy`.
3. Exporter le module dans `src/strategies/mod.rs`.
4. Ajouter le mapping dans `create_strategy()` dans `src/main.rs`.
5. Ajouter des tests ciblés ou une fixture de bougies.

Les indicateurs partagés pour les stratégies ensemble vivent dans `src/strategies/indicators.rs`.
