# Implementation Details - Micro-Strategy Ensemble

Ce document decrit un set de `23` micro-strategies avec `min_votes=1`, afin de pouvoir le recoder en une seule strategie unique.

## Identite

- Strategy ID officiel: `btcusdt_5m_combined`
- Source micro-strategies: `results\workflows\btcusdt_5m\combined_micro_strategies.json`
- Rapport ensemble: `results\workflows\btcusdt_5m\ensemble_combined.json`
- Label mode: `next_candle_color`
- Horizon: `1` bougie 5M
- Decision delay: `0`
- Entry mode logique: `current_partial`
- Objectif: predire la couleur de la prochaine bougie 5M.

## Regle De Vote

Pour chaque bougie courante:

1. Calculer toutes les features sur les donnees disponibles jusqu'a cette bougie.
2. Evaluer les 23 micro-strategies.
3. Chaque micro-strategie active ajoute `+1` au compteur GREEN ou RED selon sa direction.
4. Si `green_votes > red_votes`, la prediction finale est `GREEN`.
5. Si `red_votes > green_votes`, la prediction finale est `RED`.
6. Si `green_votes == red_votes`, ou si aucun vote n'est actif, on skip la bougie.
7. Comme `min_votes=1`, un seul vote suffit tant qu'il n'y a pas egalite.

Pseudo-code:

```python
green_votes = 0
red_votes = 0
for rule in micro_strategies:
    if all(condition_is_true(condition) for condition in rule.conditions):
        if rule.prediction_direction == 'GREEN':
            green_votes += 1
        else:
            red_votes += 1

if green_votes > red_votes and green_votes + red_votes >= 1:
    prediction = 'GREEN'
elif red_votes > green_votes and green_votes + red_votes >= 1:
    prediction = 'RED'
else:
    prediction = 'SKIP'
```

## Performance Officielle

### Backtest

- Predictions: `2136`
- Coverage: `0.34%`
- Accuracy: `68.45%`
- GREEN precision: `68.21%`
- RED precision: `68.83%`
- Average votes: `1.32`
- Max votes: `6`
- Confusion matrix: `{"GREEN": {"GREEN": 899, "RED": 255}, "RED": {"GREEN": 419, "RED": 563}}`

### Test

- Predictions: `200`
- Coverage: `0.54%`
- Accuracy: `75.00%`
- GREEN precision: `76.12%`
- RED precision: `72.73%`
- Average votes: `1.43`
- Max votes: `5`
- Confusion matrix: `{"GREEN": {"GREEN": 102, "RED": 18}, "RED": {"GREEN": 32, "RED": 48}}`

## Composition Du Set

- Nombre total de micro-strategies: `23`
- Votes GREEN: `15`
- Votes RED: `8`
- Chaque micro-strategie contient exactement les conditions a respecter pour declencher son vote.

## Features A Implementer

- `rsi` utilisee `10` fois. Relative Strength Index sur N bougies. Exemples: `rsi7, rsi8`
- `range_atr14` utilisee `7` fois. Feature specifique. Exemples: `range_atr14`
- `body_ratio` utilisee `6` fois. Feature specifique. Exemples: `body_ratio`
- `green_streak` utilisee `6` fois. Feature specifique. Exemples: `green_streak`
- `hour` utilisee `6` fois. Heure UTC de la bougie courante, entre 0 et 23. Exemples: `hour`
- `stoch_k` utilisee `6` fois. Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent. Exemples: `stoch_k12, stoch_k24`
- `atr` utilisee `5` fois. ATR sur N bougies divise par le close lorsque le nom finit par `_pct`. Exemples: `atr14_pct, atr72_pct`
- `cci` utilisee `4` fois. Commodity Channel Index sur N bougies, base sur le prix typique. Exemples: `cci12, cci24`
- `body_sum` utilisee `3` fois. Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente. Exemples: `body_sum12, body_sum6`
- `close_z` utilisee `3` fois. Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`. Exemples: `close_z24, close_z48`
- `donch_low` utilisee `3` fois. Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`. Exemples: `donch_low144, donch_low72`
- `macd_hist_pct` utilisee `3` fois. Histogramme MACD `(MACD - signal9)` divise par le close. Exemples: `macd_hist_pct`
- `weekday` utilisee `3` fois. Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche. Exemples: `weekday`
- `body` utilisee `2` fois. Corps de la bougie courante, normalise par le close: `(close - open) / close`. Exemples: `body`
- `body_abs_pct` utilisee `2` fois. Feature specifique. Exemples: `body_abs_pct`
- `lower_wick` utilisee `2` fois. Meche basse courante normalisee: `(min(open, close) - low) / close`. Exemples: `lower_wick`
- `lower_wick_body` utilisee `2` fois. Feature specifique. Exemples: `lower_wick_body`
- `mfi` utilisee `2` fois. Money Flow Index sur N bougies, proche d'un RSI pondere par le volume. Exemples: `mfi8`
- `red_streak` utilisee `2` fois. Feature specifique. Exemples: `red_streak`
- `volume_ratio20` utilisee `2` fois. Feature specifique. Exemples: `volume_ratio20`
- `bb_pctb` utilisee `1` fois. Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`. Exemples: `bb_pctb`
- `dist_sma` utilisee `1` fois. Distance du close a la SMA N: `close / SMA(N) - 1`. Exemples: `dist_sma24`
- `donch_high` utilisee `1` fois. Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`. Exemples: `donch_high12`
- `ha_body` utilisee `1` fois. Feature specifique. Exemples: `ha_body`
- `ret` utilisee `1` fois. Rendement du close sur N bougies: `close / close.shift(N) - 1`. Exemples: `ret12`
- `volume_z` utilisee `1` fois. Z-score du volume sur N bougies. Exemples: `volume_z96`

## Details Des 23 Micro-Strategies

Lecture: une micro-strategie vote seulement si toutes ses conditions sont vraies simultanement.

### 1. micro_next_h1_red_2935

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `77.23%` sur `101` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, position dans le range ou meche.

Conditions:

- `stoch_k12 >= 98.87542775`
- `ret12 >= 0.02486548978`
- `lower_wick <= 0.001638796436`

Features utilisees:

- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.
- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-30 19:30:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 2. micro_next_h1_green_12421

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `73.49%` sur `83` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z48 <= -2.447691672`
- `atr72_pct <= 0.0006406963614`
- `body_sum12 <= -0.004124824725`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-18 23:00:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 3. micro_next_h1_green_2423

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `73.47%` sur `98` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine position dans le range ou meche.

Conditions:

- `donch_low144 <= 0.0006404171868`
- `ha_body <= -0.007640254912`
- `body_abs_pct >= 0.007312429007`

Features utilisees:

- `body_abs_pct`: Feature calculee par le moteur de recherche micro-strategies.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `ha_body`: Feature calculee par le moteur de recherche micro-strategies.

Exemple test:

- Timestamp: `2026-01-21 16:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 4. micro_next_h1_green_13747

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `72.92%` sur `96` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -239.1833565`
- `atr72_pct <= 0.0006406963614`
- `rsi8 >= 16.98438998`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-01 07:25:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 5. micro_next_h1_green_12900

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `74.24%` sur `132` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_low144 <= 0.001091384106`
- `body_sum6 <= -0.01817603669`
- `volume_z96 >= 2.919388313`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-01-21 16:55:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 6. micro_next_h1_green_11992

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.35%` sur `171` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `stoch_k24 <= 2.898892702`
- `macd_hist_pct <= -0.002344176743`
- `range_atr14 >= 1.403339542`

Features utilisees:

- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `range_atr14`: Feature calculee par le moteur de recherche micro-strategies.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 16:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 7. micro_next_h1_green_11304

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.28%` sur `94` predictions
- Test: `84.62%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `bb_pctb <= -0.2340438348`
- `hour == 13`
- `dist_sma24 >= -0.007134313431`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 8. micro_next_h1_green_10662

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.03%` sur `107` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.487374544`
- `atr72_pct <= 0.0004654080978`
- `cci24 >= -192.0944316`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci24`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 9. micro_next_h1_green_13386

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.83%` sur `144` predictions
- Test: `84.62%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0008266367569`
- `body_abs_pct >= 0.007312429007`
- `mfi8 <= 14.51065799`

Features utilisees:

- `body_abs_pct`: Feature calculee par le moteur de recherche micro-strategies.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-21 16:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 10. micro_next_h1_red_10207

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.79%` sur `178` predictions
- Test: `91.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `macd_hist_pct >= 0.00186594868`
- `stoch_k12 >= 97.66150155`
- `green_streak >= 4`

Features utilisees:

- `green_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-02-02 07:05:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 11. micro_next_h1_red_12028

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.78%` sur `154` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `stoch_k24 >= 98.04361321`
- `body_sum12 >= 0.02432418065`
- `green_streak >= 3`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `green_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-02-03 21:00:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 12. micro_next_h1_green_10764

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.71%` sur `99` predictions
- Test: `75.00%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `cci12 <= -239.1833565`
- `close_z24 >= -2.058232069`
- `lower_wick <= 0.001092316795`

Features utilisees:

- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.

Exemple test:

- Timestamp: `2026-01-18 07:10:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 13. micro_next_h1_green_8172

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.53%` sur `190` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `stoch_k24 <= 2.898892702`
- `donch_high12 <= -0.02994954907`
- `mfi8 <= 14.51065799`

Features utilisees:

- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-29 15:10:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 14. micro_next_h1_red_6357

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.26%` sur `232` predictions
- Test: `70.59%` sur `17` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `macd_hist_pct >= 0.00186594868`
- `stoch_k12 >= 95.36034773`
- `rsi8 >= 86.81303658`

Features utilisees:

- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `rsi8`: Relative Strength Index sur N bougies.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 19:30:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 15. micro_next_h1_green_11645

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.07%` sur `137` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -209.9116877`
- `atr72_pct <= 0.0007461972567`
- `atr14_pct >= 0.0006619825044`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.

Exemple test:

- Timestamp: `2026-01-17 16:20:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 16. struct_wick_volume_rebound_green_rsi25_wick4.0_vol2.0__weekday_2

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.89%` sur `79` predictions
- Test: `75.00%` sur `8` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `body <= 0`
- `rsi7 <= 25`
- `lower_wick_body >= 4`
- `volume_ratio20 >= 2`
- `weekday == 2`

Features utilisees:

- `body`: Corps de la bougie courante, normalise par le close: `(close - open) / close`.
- `lower_wick_body`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.
- `volume_ratio20`: Feature calculee par le moteur de recherche micro-strategies.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-07 00:40:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 17. struct_streak_rsi_rebound_red_s5_rsi75_atr1.5_body0.75__weekday_3

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.30%` sur `114` predictions
- Test: `87.50%` sur `8` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `green_streak >= 5`
- `rsi7 >= 75`
- `range_atr14 >= 1.5`
- `body_ratio >= 0.75`
- `weekday == 3`

Features utilisees:

- `body_ratio`: Feature calculee par le moteur de recherche micro-strategies.
- `green_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `range_atr14`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-01 09:15:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 18. struct_streak_rsi_rebound_red_s4_rsi75_atr1.0_body0.75__hour_1

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.04%` sur `97` predictions
- Test: `75.00%` sur `8` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `green_streak >= 4`
- `rsi7 >= 75`
- `range_atr14 >= 1`
- `body_ratio >= 0.75`
- `hour == 1`

Features utilisees:

- `body_ratio`: Feature calculee par le moteur de recherche micro-strategies.
- `green_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `range_atr14`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-05 01:05:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 19. struct_streak_rsi_rebound_red_s4_rsi75_atr0.8_body0.75__hour_11

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.68%` sur `99` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `green_streak >= 4`
- `rsi7 >= 75`
- `range_atr14 >= 0.8`
- `body_ratio >= 0.75`
- `hour == 11`

Features utilisees:

- `body_ratio`: Feature calculee par le moteur de recherche micro-strategies.
- `green_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `range_atr14`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-01 11:55:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 20. struct_streak_rsi_rebound_green_s3_rsi30_atr1.5_body0.75__hour_21

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.67%` sur `99` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `red_streak >= 3`
- `rsi7 <= 30`
- `range_atr14 >= 1.5`
- `body_ratio >= 0.75`
- `hour == 21`

Features utilisees:

- `body_ratio`: Feature calculee par le moteur de recherche micro-strategies.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `range_atr14`: Feature calculee par le moteur de recherche micro-strategies.
- `red_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-10 21:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 21. struct_streak_rsi_rebound_red_s6_rsi70_atr0.8_body0.75__weekday_5

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.91%` sur `88` predictions
- Test: `66.67%` sur `9` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `green_streak >= 6`
- `rsi7 >= 70`
- `range_atr14 >= 0.8`
- `body_ratio >= 0.75`
- `weekday == 5`

Features utilisees:

- `body_ratio`: Feature calculee par le moteur de recherche micro-strategies.
- `green_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `range_atr14`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-03 20:45:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 22. struct_streak_rsi_rebound_green_s5_rsi30_atr1.5_body0.75__late

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.85%` sur `82` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `red_streak >= 5`
- `rsi7 <= 30`
- `range_atr14 >= 1.5`
- `body_ratio >= 0.75`
- `hour in [21, 22, 23]`

Features utilisees:

- `body_ratio`: Feature calculee par le moteur de recherche micro-strategies.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `range_atr14`: Feature calculee par le moteur de recherche micro-strategies.
- `red_streak`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-10 21:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 23. struct_wick_volume_rebound_green_rsi30_wick1.5_vol1.5__hour_22

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `64.34%` sur `143` predictions
- Test: `78.57%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `body <= 0`
- `rsi7 <= 30`
- `lower_wick_body >= 1.5`
- `volume_ratio20 >= 1.5`
- `hour == 22`

Features utilisees:

- `body`: Corps de la bougie courante, normalise par le close: `(close - open) / close`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `lower_wick_body`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi7`: Relative Strength Index sur N bougies.
- `volume_ratio20`: Feature calculee par le moteur de recherche micro-strategies.

Exemple test:

- Timestamp: `2026-01-14 22:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

## Notes D'Implementation

- Les conditions doivent etre calculees sans regarder la prochaine bougie.
- La colonne cible `target_green` sert uniquement a evaluer, jamais a predire.
- Les features rolling doivent utiliser les bougies deja connues jusqu'a la bougie courante.
- Une bougie doji future est consideree neutre dans l'evaluation officielle et ne doit pas compter comme RED.
- Pour reproduire exactement les resultats actuels, garder les memes formules de features que `discover_micro_strategies.py`.
