# Implementation Details - Micro-Strategy Ensemble

Ce document decrit un set de `25` micro-strategies avec `min_votes=1`, afin de pouvoir le recoder en une seule strategie unique.

## Identite

- Strategy ID officiel: `ethusdt_5m_micro_deduped`
- Source micro-strategies: `results\workflows\ethusdt_5m\micro_strategies_deduped.json`
- Rapport ensemble: `results\workflows\ethusdt_5m\ensemble_micro_deduped.json`
- Label mode: `next_candle_color`
- Horizon: `1` bougie 5M
- Decision delay: `0`
- Entry mode logique: `current_partial`
- Objectif: predire la couleur de la prochaine bougie 5M.

## Regle De Vote

Pour chaque bougie courante:

1. Calculer toutes les features sur les donnees disponibles jusqu'a cette bougie.
2. Evaluer les 25 micro-strategies.
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

- Predictions: `2891`
- Coverage: `0.46%`
- Accuracy: `67.97%`
- GREEN precision: `67.78%`
- RED precision: `68.61%`
- Average votes: `1.30`
- Max votes: `7`
- Confusion matrix: `{"GREEN": {"GREEN": 1517, "RED": 205}, "RED": {"GREEN": 721, "RED": 448}}`

### Test

- Predictions: `250`
- Coverage: `0.67%`
- Accuracy: `72.80%`
- GREEN precision: `71.98%`
- RED precision: `75.00%`
- Average votes: `1.32`
- Max votes: `5`
- Confusion matrix: `{"GREEN": {"GREEN": 131, "RED": 17}, "RED": {"GREEN": 51, "RED": 51}}`

## Composition Du Set

- Nombre total de micro-strategies: `25`
- Votes GREEN: `19`
- Votes RED: `6`
- Chaque micro-strategie contient exactement les conditions a respecter pour declencher son vote.

## Features A Implementer

- `stoch_k` utilisee `10` fois. Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent. Exemples: `stoch_k12, stoch_k24, stoch_k72`
- `close_z` utilisee `7` fois. Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`. Exemples: `close_z24, close_z48`
- `mfi` utilisee `7` fois. Money Flow Index sur N bougies, proche d'un RSI pondere par le volume. Exemples: `mfi14, mfi21, mfi8`
- `atr` utilisee `6` fois. ATR sur N bougies divise par le close lorsque le nom finit par `_pct`. Exemples: `atr14_pct, atr72_pct`
- `bb_pctb` utilisee `6` fois. Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`. Exemples: `bb_pctb`
- `cci` utilisee `6` fois. Commodity Channel Index sur N bougies, base sur le prix typique. Exemples: `cci12, cci24`
- `rsi` utilisee `6` fois. Relative Strength Index sur N bougies. Exemples: `rsi21, rsi7, rsi8`
- `body_abs_pct` utilisee `5` fois. Feature specifique. Exemples: `body_abs_pct`
- `donch_low` utilisee `5` fois. Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`. Exemples: `donch_low144, donch_low72`
- `body_sum` utilisee `2` fois. Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente. Exemples: `body_sum12, body_sum6`
- `hour` utilisee `2` fois. Heure UTC de la bougie courante, entre 0 et 23. Exemples: `hour`
- `lower_wick_body` utilisee `2` fois. Feature specifique. Exemples: `lower_wick_body`
- `ret` utilisee `2` fois. Rendement du close sur N bougies: `close / close.shift(N) - 1`. Exemples: `ret12, ret72`
- `volume_z` utilisee `2` fois. Z-score du volume sur N bougies. Exemples: `volume_z96`
- `weekday` utilisee `2` fois. Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche. Exemples: `weekday`
- `body_ratio` utilisee `1` fois. Feature specifique. Exemples: `body_ratio`
- `donch_high` utilisee `1` fois. Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`. Exemples: `donch_high12`
- `lower_wick` utilisee `1` fois. Meche basse courante normalisee: `(min(open, close) - low) / close`. Exemples: `lower_wick`
- `upper_wick_body` utilisee `1` fois. Feature specifique. Exemples: `upper_wick_body`
- `williams_r` utilisee `1` fois. Williams %R sur N bougies: autre mesure de position dans le range recent. Exemples: `williams_r12`

## Details Des 25 Micro-Strategies

Lecture: une micro-strategie vote seulement si toutes ses conditions sont vraies simultanement.

### 1. micro_next_h1_green_348

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `73.26%` sur `86` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `stoch_k24 <= 0.5443385043`
- `hour == 5`
- `rsi21 <= 39.11398072`

Features utilisees:

- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `rsi21`: Relative Strength Index sur N bougies.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-03 05:10:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 2. micro_next_h1_red_509

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.52%` sur `173` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `close_z24 >= 3.082851148`
- `weekday == 3`
- `mfi21 <= 66.52204642`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `mfi21`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-01 22:55:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 3. micro_next_h1_green_219

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.19%` sur `104` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0005704541149`
- `bb_pctb <= -0.24090522`
- `close_z48 >= -2.696757558`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-11 23:00:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 4. micro_next_h1_green_374

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.08%` sur `127` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `stoch_k12 <= 2.325581395`
- `bb_pctb <= -0.24090522`
- `body_abs_pct <= 0.003473998504`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `body_abs_pct`: Feature calculee par le moteur de recherche micro-strategies.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-05 04:10:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 5. micro_next_h1_green_644

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.70%` sur `99` predictions
- Test: `91.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0005704541149`
- `cci12 <= -213.8206725`
- `stoch_k72 >= 2.887028121`

Features utilisees:

- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `stoch_k72`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-07 09:15:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 6. micro_next_h1_red_591

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.42%` sur `121` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `rsi8 >= 84.02165584`
- `atr72_pct <= 0.0009239513225`
- `lower_wick_body >= 0.01797752809`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `lower_wick_body`: Feature calculee par le moteur de recherche micro-strategies.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-10 10:00:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 7. micro_next_h1_green_723

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `72.13%` sur `122` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `stoch_k24 <= 5.132606156`
- `ret72 >= 0.02599541236`
- `rsi7 <= 24.46140344`

Features utilisees:

- `ret72`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `rsi7`: Relative Strength Index sur N bougies.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 22:35:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 8. micro_next_h1_green_1030

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `72.83%` sur `92` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `donch_low144 <= 0.0006709289818`
- `bb_pctb <= -0.24090522`
- `volume_z96 <= 2.912906413`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-01-07 09:10:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 9. micro_next_h1_green_220

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.93%` sur `103` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0005704541149`
- `bb_pctb <= -0.24090522`
- `body_sum6 >= -0.005522189783`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-09 08:00:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 10. micro_next_h1_red_296

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.83%` sur `154` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `bb_pctb >= 1.242274122`
- `weekday == 3`
- `volume_z96 <= 2.098463339`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `volume_z96`: Z-score du volume sur N bougies.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-02-05 09:05:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 11. micro_next_h1_red_668

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `72.09%` sur `86` predictions
- Test: `68.75%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `rsi8 >= 80.23066448`
- `close_z48 <= 1.456773235`
- `mfi14 <= 73.70646328`

Features utilisees:

- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `mfi14`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-24 22:10:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 12. micro_next_h1_green_52

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.71%` sur `163` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `mfi8 <= 7.65525868`
- `body_abs_pct >= 0.01206610733`
- `stoch_k72 <= 10.166951`

Features utilisees:

- `body_abs_pct`: Feature calculee par le moteur de recherche micro-strategies.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `stoch_k72`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 16:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 13. micro_next_h1_red_262

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.60%` sur `121` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z48 >= 3.429940156`
- `body_sum12 <= 0.005753340807`
- `rsi21 >= 67.95174536`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `rsi21`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-01 23:20:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 14. micro_next_h1_red_854

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.38%` sur `136` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `rsi8 >= 80.23066448`
- `atr14_pct <= 0.000953452966`
- `mfi21 >= 78.27879154`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `mfi21`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-10 09:55:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 15. micro_next_h1_green_920

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.26%` sur `167` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `donch_high12 <= -0.03797357864`
- `mfi21 <= 13.8331558`
- `close_z48 <= -3.063615586`

Features utilisees:

- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `mfi21`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-29 15:10:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 16. micro_next_h1_green_258

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.94%` sur `131` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -243.4867158`
- `atr14_pct <= 0.001277921698`
- `mfi21 <= 33.22794653`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `mfi21`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-04 15:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 17. micro_next_h1_green_518

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.84%` sur `255` predictions
- Test: `78.57%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `stoch_k12 <= 2.325581395`
- `hour == 11`
- `williams_r12 >= -99.31370042`

Features utilisees:

- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.
- `williams_r12`: Williams %R sur N bougies: autre mesure de position dans le range recent.

Exemple test:

- Timestamp: `2026-01-10 11:50:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 18. micro_next_h1_green_1157

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.83%` sur `115` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -243.4867158`
- `atr72_pct <= 0.001411663091`
- `body_abs_pct >= 0.003473998504`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `body_abs_pct`: Feature calculee par le moteur de recherche micro-strategies.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.

Exemple test:

- Timestamp: `2026-01-11 23:00:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 19. micro_next_h1_green_187

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.81%` sur `320` predictions
- Test: `83.33%` sur `18` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `mfi8 <= 13.48098558`
- `body_abs_pct >= 0.01206610733`
- `atr72_pct <= 0.00646353357`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `body_abs_pct`: Feature calculee par le moteur de recherche micro-strategies.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-20 22:30:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 20. micro_next_h1_green_215

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.80%` sur `177` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0005704541149`
- `upper_wick_body >= 5.117647059`
- `lower_wick >= 1.612549971e-05`

Features utilisees:

- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.
- `upper_wick_body`: Feature calculee par le moteur de recherche micro-strategies.

Exemple test:

- Timestamp: `2026-01-10 22:05:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 21. micro_next_h1_green_1215

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.79%` sur `208` predictions
- Test: `85.71%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `stoch_k12 <= 0.6862995766`
- `close_z24 <= -2.808854839`
- `body_abs_pct <= 0.003473998504`

Features utilisees:

- `body_abs_pct`: Feature calculee par le moteur de recherche micro-strategies.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-01 04:05:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 22. micro_next_h1_green_533

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.65%` sur `170` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `stoch_k72 <= 6.986747793`
- `body_ratio <= 0.03305785124`
- `cci12 >= -89.88475125`

Features utilisees:

- `body_ratio`: Feature calculee par le moteur de recherche micro-strategies.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `stoch_k72`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-14 11:55:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 23. micro_next_h1_green_861

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.59%` sur `216` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `ret12 <= -0.03224343338`
- `stoch_k12 <= 3.779328959`
- `cci24 <= -175.7548746`

Features utilisees:

- `cci24`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-19 00:10:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 24. micro_next_h1_green_556

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.55%` sur `151` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `stoch_k24 <= 1.644587669`
- `bb_pctb <= -0.24090522`
- `lower_wick_body >= 0.01797752809`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `lower_wick_body`: Feature calculee par le moteur de recherche micro-strategies.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-07 09:10:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 25. micro_next_h1_green_99

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.50%` sur `160` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -243.4867158`
- `atr14_pct <= 0.001072510421`
- `close_z24 <= -2.519107999`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-18 07:10:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

## Notes D'Implementation

- Les conditions doivent etre calculees sans regarder la prochaine bougie.
- La colonne cible `target_green` sert uniquement a evaluer, jamais a predire.
- Les features rolling doivent utiliser les bougies deja connues jusqu'a la bougie courante.
- Une bougie doji future est consideree neutre dans l'evaluation officielle et ne doit pas compter comme RED.
- Pour reproduire exactement les resultats actuels, garder les memes formules de features que `discover_micro_strategies.py`.
