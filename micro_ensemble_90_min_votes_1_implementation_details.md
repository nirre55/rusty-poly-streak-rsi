# Implementation Details - 90 Micro-Strategy Ensemble

Ce document decrit le set officiel `90` avec `min_votes=1`, afin de pouvoir le recoder en une seule strategie unique.

## Identite

- Strategy ID officiel: `micro_ensemble_combined_90_min_votes_1`
- Source micro-strategies: `results\micro_strategies_over_65_combined_deduped.json`
- Rapport ensemble: `results\micro_ensemble_combined_deduped_min1.json`
- Label mode: `next_candle_color`
- Horizon: `1` bougie M5
- Decision delay: `0`
- Entry mode logique: `current_partial`
- Objectif: predire la couleur de la prochaine bougie M5.

## Regle De Vote

Pour chaque bougie courante:

1. Calculer toutes les features sur les donnees disponibles jusqu'a cette bougie.
2. Evaluer les 90 micro-strategies.
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

- Predictions: `8971`
- Coverage: `1.42%`
- Accuracy: `65.04%`
- GREEN precision: `64.93%`
- RED precision: `65.20%`
- Average votes: `1.97`
- Max votes: `19`
- Confusion matrix: `{"GREEN": {"GREEN": 3418, "RED": 1290}, "RED": {"GREEN": 1846, "RED": 2417}}`

### Test

- Predictions: `643`
- Coverage: `1.73%`
- Accuracy: `68.74%`
- GREEN precision: `67.49%`
- RED precision: `70.89%`
- Average votes: `2.36`
- Max votes: `17`
- Confusion matrix: `{"GREEN": {"GREEN": 274, "RED": 69}, "RED": {"GREEN": 132, "RED": 168}}`

## Composition Du Set

- Nombre total de micro-strategies: `90`
- Votes GREEN: `60`
- Votes RED: `30`
- Chaque micro-strategie contient exactement les conditions a respecter pour declencher son vote.

## Features A Implementer

- `close_z` utilisee `42` fois. Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`. Exemples: `close_z24, close_z48`
- `donch_low` utilisee `22` fois. Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`. Exemples: `donch_low144, donch_low72`
- `bb_pctb` utilisee `21` fois. Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`. Exemples: `bb_pctb`
- `atr` utilisee `20` fois. ATR sur N bougies divise par le close lorsque le nom finit par `_pct`. Exemples: `atr14_pct, atr72_pct`
- `body_sum` utilisee `20` fois. Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente. Exemples: `body_sum12, body_sum6`
- `hour` utilisee `19` fois. Heure UTC de la bougie courante, entre 0 et 23. Exemples: `hour`
- `rsi` utilisee `19` fois. Relative Strength Index sur N bougies. Exemples: `rsi14, rsi21, rsi8`
- `stoch_k` utilisee `19` fois. Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent. Exemples: `stoch_k12, stoch_k24`
- `ret` utilisee `16` fois. Rendement du close sur N bougies: `close / close.shift(N) - 1`. Exemples: `ret12, ret24, ret72`
- `donch_high` utilisee `13` fois. Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`. Exemples: `donch_high12, donch_high72`
- `dist_sma` utilisee `11` fois. Distance du close a la SMA N: `close / SMA(N) - 1`. Exemples: `dist_sma24`
- `cci` utilisee `8` fois. Commodity Channel Index sur N bougies, base sur le prix typique. Exemples: `cci12, cci24, cci72`
- `lower_wick` utilisee `8` fois. Meche basse courante normalisee: `(min(open, close) - low) / close`. Exemples: `lower_wick`
- `mfi` utilisee `7` fois. Money Flow Index sur N bougies, proche d'un RSI pondere par le volume. Exemples: `mfi14, mfi21, mfi8`
- `weekday` utilisee `7` fois. Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche. Exemples: `weekday`
- `volume_z` utilisee `6` fois. Z-score du volume sur N bougies. Exemples: `volume_z96`
- `macd_hist_pct` utilisee `5` fois. Histogramme MACD `(MACD - signal9)` divise par le close. Exemples: `macd_hist_pct`
- `green_count` utilisee `3` fois. Nombre de bougies vertes dans les N dernieres bougies. Exemples: `green_count6`
- `upper_wick` utilisee `3` fois. Meche haute courante normalisee: `(high - max(open, close)) / close`. Exemples: `upper_wick`
- `red_count` utilisee `1` fois. Nombre de bougies rouges dans les N dernieres bougies. Exemples: `red_count6`

## Details Des 90 Micro-Strategies

Lecture: une micro-strategie vote seulement si toutes ses conditions sont vraies simultanement.

### 1. micro_next_h1_red_2427

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `77.23%` sur `101` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, position dans le range ou meche.

Conditions:

- `stoch_k12 >= 98.87542722`
- `ret12 >= 0.02486548257`
- `lower_wick <= 0.001638794532`

Features utilisees:

- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.
- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-30 19:30:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 2. micro_next_h1_green_376

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `73.39%` sur `109` predictions
- Test: `80.00%` sur `20` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `bb_pctb <= -0.107140425`
- `hour == 13`
- `volume_z96 <= 0.7579850134`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-01-07 13:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 3. micro_next_h1_green_17965

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `72.92%` sur `96` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -239.1832969`
- `atr72_pct <= 0.0006406964493`
- `rsi8 >= 16.98439155`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-01 07:25:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 4. micro_next_h1_green_352

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `72.73%` sur `121` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_low144 <= 0.001457840018`
- `body_sum6 <= -0.01817602056`
- `volume_z96 >= 3.587853962`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-01-21 16:55:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 5. micro_next_h1_green_17161

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `72.62%` sur `84` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z48 <= -2.44769017`
- `atr72_pct <= 0.0006406964493`
- `body_sum12 <= -0.004124811926`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-18 23:00:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 6. micro_next_h1_green_1966

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.96%` sur `107` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.774117242`
- `atr72_pct <= 0.0006406964493`
- `stoch_k24 >= 10.74126157`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 7. micro_next_h1_green_292

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.43%` sur `119` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `rsi21 <= 33.28245704`
- `ret24 >= -0.005787322997`
- `dist_sma24 <= -0.005476785129`

Features utilisees:

- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `ret24`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `rsi21`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-23 14:35:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 8. micro_next_h1_green_58

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.28%` sur `94` predictions
- Test: `84.62%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `bb_pctb <= -0.2340435963`
- `hour == 13`
- `dist_sma24 >= -0.007134307442`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 9. micro_next_h1_green_1864

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.15%` sur `156` predictions
- Test: `71.43%` sur `21` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.774117242`
- `atr72_pct <= 0.0006406964493`
- `bb_pctb >= -0.2340435963`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-10 13:50:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 10. micro_next_h1_green_346

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.83%` sur `144` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0008266398454`
- `body_sum12 <= -0.01970911953`
- `volume_z96 >= 2.919374564`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-01-21 16:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 11. micro_next_h1_green_473

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.79%` sur `89` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche, filtre temporel.

Conditions:

- `donch_low144 <= 0.0006404179203`
- `hour == 14`
- `ret72 <= -0.01475596055`

Features utilisees:

- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `ret72`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.

Exemple test:

- Timestamp: `2026-01-07 14:15:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 12. micro_next_h1_red_151

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.73%` sur `164` predictions
- Test: `78.57%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_high12 >= -0.000239825686`
- `ret12 >= 0.01939351557`
- `lower_wick <= 0.001638794532`

Features utilisees:

- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.
- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.

Exemple test:

- Timestamp: `2026-01-14 14:50:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 13. micro_next_h1_green_14659

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.71%` sur `99` predictions
- Test: `75.00%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `cci12 <= -239.1832969`
- `close_z24 >= -2.058231615`
- `lower_wick <= 0.001092315747`

Features utilisees:

- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.

Exemple test:

- Timestamp: `2026-01-18 07:10:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 14. micro_next_h1_red_13228

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.53%` sur `207` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `macd_hist_pct >= 0.001865948161`
- `stoch_k12 >= 97.6614989`
- `close_z24 >= 2.292740233`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-02-05 23:00:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 15. micro_next_h1_green_10746

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.53%` sur `190` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `stoch_k24 <= 2.898900732`
- `donch_high12 <= -0.02994953395`
- `mfi8 <= 14.51065973`

Features utilisees:

- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-29 15:10:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 16. micro_next_h1_green_13948

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.50%` sur `139` predictions
- Test: `83.33%` sur `18` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `bb_pctb <= -0.107140425`
- `atr72_pct <= 0.0004654084234`
- `atr14_pct >= 0.0002988690878`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 17. micro_next_h1_green_17884

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.08%` sur `127` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `bb_pctb <= -0.173645982`
- `hour == 13`
- `mfi8 >= 25.84167143`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 18. micro_next_h1_green_16661

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.07%` sur `137` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -209.9115581`
- `atr72_pct <= 0.0007461976822`
- `atr14_pct >= 0.0006619818413`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.

Exemple test:

- Timestamp: `2026-01-17 16:20:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 19. micro_next_h1_green_14148

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `71.43%` sur `245` predictions
- Test: `70.00%` sur `20` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.774117242`
- `atr72_pct <= 0.0007461976822`
- `cci72 <= -130.1004463`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci72`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-03 07:20:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 20. micro_next_h1_green_12448

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.93%` sur `143` predictions
- Test: `75.00%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `cci12 <= -239.1832969`
- `atr72_pct <= 0.0007461976822`
- `stoch_k24 >= 7.498316732`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci12`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-10 13:50:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 21. micro_next_h1_green_14963

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.72%` sur `109` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.487372513`
- `atr72_pct <= 0.0004654084234`
- `cci24 >= -192.094143`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `cci24`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 22. micro_next_h1_green_14818

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.72%` sur `142` predictions
- Test: `70.59%` sur `17` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.774117242`
- `atr14_pct <= 0.0007499679689`
- `close_z48 <= -3.032681751`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-01 04:15:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 23. micro_next_h1_red_18015

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.60%` sur `273` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `stoch_k12 >= 95.36043284`
- `macd_hist_pct >= 0.002366750995`
- `close_z24 >= 2.046146229`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 19:45:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 24. micro_next_h1_red_5396

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.32%` sur `365` predictions
- Test: `81.25%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `macd_hist_pct >= 0.001865948161`
- `stoch_k12 >= 95.36043284`
- `mfi21 >= 73.95404425`

Features utilisees:

- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `mfi21`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 19:30:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 25. micro_next_h1_red_18153

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `74.05%` sur `131` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `mfi8 >= 94.13387702`
- `donch_high12 >= -0.0001214530509`
- `stoch_k24 <= 99.45945562`

Features utilisees:

- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-13 05:15:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 26. micro_next_h1_red_139

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.16%` sur `214` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `macd_hist_pct >= 0.001865948161`
- `stoch_k12 >= 98.87542722`
- `ret72 <= 0.03681466689`

Features utilisees:

- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `ret72`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-02-03 20:20:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 27. micro_next_h1_green_17655

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.15%` sur `188` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `bb_pctb <= -0.2340435963`
- `dist_sma24 >= -0.004431553752`
- `cci72 <= -153.295298`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `cci72`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.

Exemple test:

- Timestamp: `2026-01-09 07:55:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 28. micro_next_h1_green_229

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.08%` sur `152` predictions
- Test: `72.22%` sur `18` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `bb_pctb <= -0.2340435963`
- `dist_sma24 >= -0.004431553752`
- `rsi14 <= 27.52356397`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `rsi14`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-09 07:55:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 29. micro_next_h1_green_16929

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.05%` sur `84` predictions
- Test: `75.00%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `bb_pctb <= -0.107140425`
- `atr72_pct <= 0.0006406964493`
- `donch_low72 >= 0.001652921292`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 30. micro_next_h1_green_2073

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.89%` sur `135` predictions
- Test: `71.43%` sur `21` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.774117242`
- `atr72_pct <= 0.0006406964493`
- `mfi8 <= 18.35512826`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `mfi8`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-10 21:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 31. micro_next_h1_red_16428

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.86%` sur `395` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, position dans le range ou meche.

Conditions:

- `stoch_k12 >= 95.36043284`
- `body_sum12 >= 0.01911639022`
- `donch_high72 <= -0.001380130085`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `donch_high72`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 19:30:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 32. micro_next_h1_green_17660

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.80%` sur `250` predictions
- Test: `69.23%` sur `26` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `bb_pctb <= -0.2340435963`
- `dist_sma24 >= -0.004431553752`
- `mfi14 <= 27.45320025`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `mfi14`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-09 07:55:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 33. micro_next_h1_green_1940

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.79%` sur `157` predictions
- Test: `71.43%` sur `21` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z24 <= -2.774117242`
- `atr72_pct <= 0.0006406964493`
- `green_count6 >= 2`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `green_count6`: Nombre de bougies vertes dans les N dernieres bougies.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 34. micro_next_h1_green_16047

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.75%` sur `112` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z48 <= -2.668322486`
- `atr72_pct <= 0.0004654084234`
- `mfi14 <= 31.37307417`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `mfi14`: Money Flow Index sur N bougies, proche d'un RSI pondere par le volume.

Exemple test:

- Timestamp: `2026-01-10 14:10:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 35. micro_next_h1_green_14202

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.75%` sur `144` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_low144 <= 0.0006404179203`
- `donch_high12 <= -0.02387268949`
- `green_count6 <= 1`

Features utilisees:

- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `green_count6`: Nombre de bougies vertes dans les N dernieres bougies.

Exemple test:

- Timestamp: `2026-01-21 16:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 36. micro_next_h1_red_17486

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.72%` sur `195` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, filtre temporel.

Conditions:

- `rsi8 >= 79.78754453`
- `hour == 21`
- `red_count6 >= 2`

Features utilisees:

- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `red_count6`: Nombre de bougies rouges dans les N dernieres bougies.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-03 21:15:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 37. micro_next_h1_red_16450

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.70%` sur `246` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `stoch_k12 >= 95.36043284`
- `body_sum12 >= 0.01911639022`
- `cci72 >= 301.1917591`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `cci72`: Commodity Channel Index sur N bougies, base sur le prix typique.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-14 14:50:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 38. micro_next_h1_green_213

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.69%` sur `99` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0008266398454`
- `ret24 <= -0.03454574655`
- `upper_wick >= 0.001054938882`

Features utilisees:

- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `ret24`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `upper_wick`: Meche haute courante normalisee: `(high - max(open, close)) / close`.

Exemple test:

- Timestamp: `2026-01-29 15:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 39. micro_next_h1_red_18222

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.67%` sur `83` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `stoch_k12 >= 95.36043284`
- `atr14_pct >= 0.005489115066`
- `atr72_pct <= 0.003517992609`

Features utilisees:

- `atr14_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-02-02 15:35:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 40. micro_next_h1_red_5239

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.66%` sur `351` predictions
- Test: `80.00%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `macd_hist_pct >= 0.001865948161`
- `stoch_k12 >= 95.36043284`
- `stoch_k24 <= 96.70846245`

Features utilisees:

- `macd_hist_pct`: Histogramme MACD `(MACD - signal9)` divise par le close.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 19:55:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 41. micro_next_h1_green_16955

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.58%` sur `331` predictions
- Test: `69.77%` sur `43` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `bb_pctb <= -0.107140425`
- `atr72_pct <= 0.0006406964493`
- `stoch_k24 >= 4.53517561`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-01 07:25:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 42. micro_next_h1_green_15546

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.53%` sur `232` predictions
- Test: `68.75%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `stoch_k24 <= 1.091431392`
- `body_sum6 <= -0.01392502169`
- `green_count6 <= 1`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `green_count6`: Nombre de bougies vertes dans les N dernieres bougies.
- `stoch_k24`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 16:55:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 43. micro_next_h1_red_14262

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.50%` sur `419` predictions
- Test: `70.83%` sur `24` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, position dans le range ou meche.

Conditions:

- `stoch_k12 >= 95.36043284`
- `body_sum6 >= 0.01753783257`
- `lower_wick <= 0.001638794532`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-21 19:30:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 44. micro_next_h1_green_416

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.45%` sur `168` predictions
- Test: `69.23%` sur `26` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `close_z24 <= -2.487372513`
- `hour == 13`
- `volume_z96 <= 1.219114982`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 45. micro_next_h1_red_7539

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `69.60%` sur `352` predictions
- Test: `68.42%` sur `19` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, position dans le range ou meche.

Conditions:

- `stoch_k12 >= 95.36043284`
- `body_sum12 >= 0.02432417868`
- `lower_wick <= 0.001638794532`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.

Exemple test:

- Timestamp: `2026-01-30 19:15:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 46. micro_next_h1_red_16521

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.39%` sur `193` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, filtre temporel.

Conditions:

- `stoch_k12 >= 95.36043284`
- `ret12 >= 0.01939351557`
- `weekday == 4`

Features utilisees:

- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `stoch_k12`: Stochastique %K sur N bougies: position du close entre le plus bas et le plus haut recent.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-30 19:15:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 47. micro_next_h1_green_283

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.38%` sur `272` predictions
- Test: `69.57%` sur `23` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `close_z24 <= -2.774117242`
- `dist_sma24 >= -0.004431553752`
- `close_z48 <= -3.385888687`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.

Exemple test:

- Timestamp: `2026-01-09 07:55:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 48. micro_next_h1_green_15889

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.37%` sur `98` predictions
- Test: `68.75%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `bb_pctb <= -0.06443813881`
- `atr72_pct <= 0.0004654084234`
- `rsi21 >= 39.3931585`

Features utilisees:

- `atr72_pct`: ATR sur N bougies divise par le close lorsque le nom finit par `_pct`.
- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `rsi21`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 49. micro_next_h1_green_317

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.00%` sur `100` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche, filtre temporel.

Conditions:

- `close_z24 <= -2.774117242`
- `donch_low72 <= 0.001228070749`
- `hour == 6`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-08 06:15:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 50. micro_next_h1_green_323

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `68.52%` sur `216` predictions
- Test: `67.86%` sur `28` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `donch_low144 <= 0.001457840018`
- `close_z48 <= -3.385888687`
- `donch_high12 >= -0.007190350995`

Features utilisees:

- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-04 19:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 51. micro_next_h1_red_481

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.76%` sur `152` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `body_sum6 >= 0.008390907843`
- `donch_high12 >= -0.0003773777571`
- `lower_wick <= 0`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.

Exemple test:

- Timestamp: `2026-02-12 13:55:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 52. micro_next_h1_green_325

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.69%` sur `195` predictions
- Test: `73.33%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `donch_low144 <= 0.001457840018`
- `close_z48 <= -3.385888687`
- `dist_sma24 <= -0.0156322462`

Features utilisees:

- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-29 15:05:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 53. micro_next_h1_green_372

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.68%` sur `99` predictions
- Test: `75.00%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche, filtre temporel.

Conditions:

- `close_z24 <= -2.304024546`
- `hour == 22`
- `upper_wick <= 9.223945209e-08`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `upper_wick`: Meche haute courante normalisee: `(high - max(open, close)) / close`.

Exemple test:

- Timestamp: `2026-01-10 22:05:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 54. micro_next_h1_red_218

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.57%` sur `111` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `close_z24 >= 2.783849801`
- `hour == 9`
- `close_z48 <= 3.429563387`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-02 09:45:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 55. micro_next_h1_green_397

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.57%` sur `515` predictions
- Test: `69.44%` sur `36` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z24 <= -2.774117242`
- `body_sum12 >= -0.005758468918`
- `body_sum6 <= -0.004965357046`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.

Exemple test:

- Timestamp: `2026-01-07 14:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 56. micro_next_h1_green_402

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.50%` sur `120` predictions
- Test: `70.59%` sur `17` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, filtre temporel.

Conditions:

- `close_z24 <= -2.774117242`
- `body_sum12 >= -0.005758468918`
- `hour == 13`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 57. micro_next_h1_red_303

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.21%` sur `183` predictions
- Test: `76.47%` sur `17` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine position dans le range ou meche.

Conditions:

- `donch_high12 >= -0.000509590257`
- `donch_low72 <= 0.0005943956918`
- `lower_wick <= 3.702010378e-07`

Features utilisees:

- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.

Exemple test:

- Timestamp: `2026-01-24 10:20:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 58. micro_next_h1_green_418

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.12%` sur `146` predictions
- Test: `71.43%` sur `14` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `rsi8 <= 31.93496681`
- `ret72 >= 0.0242546669`
- `body_sum12 >= -0.007039969776`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `ret72`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-21 22:55:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 59. micro_next_h1_green_333

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.92%` sur `133` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `donch_low72 <= 0.0008266398454`
- `dist_sma24 <= -0.0156322462`
- `ret72 >= -0.02315688552`

Features utilisees:

- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `ret72`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.

Exemple test:

- Timestamp: `2026-01-21 16:40:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 60. micro_next_h1_red_207

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.90%` sur `142` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `rsi8 >= 79.78754453`
- `hour == 21`
- `rsi14 <= 73.34429789`

Features utilisees:

- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `rsi14`: Relative Strength Index sur N bougies.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-16 21:05:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 61. micro_next_h1_green_406

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.81%` sur `238` predictions
- Test: `69.23%` sur `26` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z24 <= -2.774117242`
- `ret12 >= -0.005712097907`
- `rsi21 <= 31.37459303`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `rsi21`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-03 07:20:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 62. micro_next_h1_green_294

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.76%` sur `355` predictions
- Test: `67.57%` sur `37` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `rsi21 <= 33.28245704`
- `ret24 >= -0.005787322997`
- `bb_pctb <= -0.173645982`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `ret24`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `rsi21`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-07 09:10:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 63. micro_next_h1_green_226

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.67%` sur `195` predictions
- Test: `70.37%` sur `27` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `bb_pctb <= -0.2340435963`
- `dist_sma24 >= -0.004431553752`
- `body_sum6 <= -0.004083019831`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.

Exemple test:

- Timestamp: `2026-01-18 23:00:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 64. micro_next_h1_green_384

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.67%` sur `222` predictions
- Test: `70.37%` sur `27` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `close_z24 <= -2.774117242`
- `rsi21 >= 31.37459303`
- `donch_low72 <= 0.0001481980796`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `rsi21`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-07 14:15:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 65. micro_next_h1_red_165

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `67.50%` sur `200` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `close_z24 >= 3.068414947`
- `weekday == 5`
- `close_z48 >= 3.429563387`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-02-07 12:15:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 66. micro_next_h1_red_190

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.87%` sur `163` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche, filtre temporel.

Conditions:

- `donch_high72 >= -0.000212151614`
- `hour == 12`
- `bb_pctb <= 1.061948757`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `donch_high72`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-01 12:00:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 67. micro_next_h1_red_318

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.95%` sur `118` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `body_sum6 >= 0.01364096683`
- `lower_wick <= 0`
- `upper_wick <= 0.001347937908`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `lower_wick`: Meche basse courante normalisee: `(min(open, close) - low) / close`.
- `upper_wick`: Meche haute courante normalisee: `(high - max(open, close)) / close`.

Exemple test:

- Timestamp: `2026-01-13 22:30:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 68. micro_next_h1_green_440

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.96%` sur `224` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, filtre temporel.

Conditions:

- `close_z24 <= -2.487372513`
- `hour == 11`
- `body_sum6 >= -0.004965357046`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-14 11:15:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 69. micro_next_h1_green_477

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `70.48%` sur `105` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `bb_pctb <= -0.06443813881`
- `hour == 13`
- `weekday == 5`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-10 13:50:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 70. micro_next_h1_green_250

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.58%` sur `398` predictions
- Test: `83.33%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `rsi8 <= 23.24758078`
- `donch_low144 >= 0.03033879208`
- `close_z24 <= -2.304024546`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-02-04 05:50:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 71. micro_next_h1_green_281

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.49%` sur `191` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `rsi8 <= 31.93496681`
- `donch_low72 >= 0.03468189691`
- `donch_high72 >= -0.02252162313`

Features utilisees:

- `donch_high72`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-02-03 22:20:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 72. micro_next_h1_green_215

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.47%` sur `346` predictions
- Test: `68.75%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `close_z24 <= -2.058231615`
- `donch_low72 >= 0.02558085724`
- `donch_high72 >= -0.02252162313`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_high72`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-21 22:55:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 73. micro_next_h1_green_271

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.42%` sur `137` predictions
- Test: `91.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche, filtre temporel.

Conditions:

- `bb_pctb <= -0.2340435963`
- `donch_low72 <= 0.001228070749`
- `weekday == 1`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-20 05:00:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 74. micro_next_h1_green_354

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.35%` sur `208` predictions
- Test: `68.18%` sur `22` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z24 <= -2.774117242`
- `body_sum6 >= -0.002956606273`
- `rsi21 <= 35.7929937`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `rsi21`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-10 21:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 75. micro_next_h1_red_169

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.23%` sur `154` predictions
- Test: `76.92%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, filtre temporel.

Conditions:

- `close_z24 >= 3.068414947`
- `weekday == 5`
- `ret12 <= 0.004280476703`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-03 12:45:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 76. micro_next_h1_red_245

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.23%` sur `228` predictions
- Test: `70.59%` sur `17` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `close_z24 >= 3.068414947`
- `close_z48 <= 1.903776951`
- `donch_low144 <= 0.03033879208`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-19 11:35:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 77. micro_next_h1_green_312

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.05%` sur `215` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `close_z24 <= -2.487372513`
- `hour == 22`
- `volume_z96 <= 0.7579850134`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-01-14 22:35:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 78. micro_next_h1_green_223

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `66.00%` sur `150` predictions
- Test: `66.67%` sur `18` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `bb_pctb <= -0.2340435963`
- `dist_sma24 >= -0.004431553752`
- `donch_low144 <= 0.001091384768`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-18 01:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 79. micro_next_h1_red_307

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.78%` sur `225` predictions
- Test: `68.75%` sur `16` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `rsi8 >= 79.78754453`
- `weekday == 5`
- `rsi21 <= 64.95549342`

Features utilisees:

- `rsi21`: Relative Strength Index sur N bougies.
- `rsi8`: Relative Strength Index sur N bougies.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-24 22:10:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 80. micro_next_h1_red_132

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.76%` sur `403` predictions
- Test: `68.18%` sur `22` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche, filtre temporel.

Conditions:

- `close_z24 >= 2.783849801`
- `weekday == 5`
- `donch_high72 <= -0.001004677022`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_high72`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `weekday`: Jour de la semaine: 0=lundi, 1=mardi, ..., 6=dimanche.

Exemple test:

- Timestamp: `2026-01-31 23:20:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 81. micro_next_h1_green_174

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.76%` sur `257` predictions
- Test: `66.67%` sur `15` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur.

Conditions:

- `bb_pctb <= -0.2340435963`
- `rsi14 >= 35.01130025`
- `dist_sma24 <= -0.003142104413`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `dist_sma24`: Distance du close a la SMA N: `close / SMA(N) - 1`.
- `rsi14`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-12 17:30:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 82. micro_next_h1_green_103

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.76%` sur `368` predictions
- Test: `66.67%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche.

Conditions:

- `bb_pctb <= -0.005731108736`
- `donch_low72 >= 0.02923394723`
- `donch_low144 >= 0.04226528076`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `donch_low72`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-02-06 03:45:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 83. micro_next_h1_green_258

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.75%` sur `254` predictions
- Test: `68.42%` sur `19` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, filtre temporel.

Conditions:

- `bb_pctb <= -0.107140425`
- `hour == 22`
- `ret24 >= -0.005787322997`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `ret24`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.

Exemple test:

- Timestamp: `2026-01-10 22:05:00+00:00`
- Prediction: `GREEN`
- Actual: `RED`
- Correct: `False`

### 84. micro_next_h1_green_396

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.72%` sur `423` predictions
- Test: `66.67%` sur `39` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, position dans le range ou meche.

Conditions:

- `close_z24 <= -2.774117242`
- `body_sum12 >= -0.005758468918`
- `donch_low144 <= 0.001091384768`

Features utilisees:

- `body_sum12`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.

Exemple test:

- Timestamp: `2026-01-07 14:05:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 85. micro_next_h1_green_414

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.70%` sur `207` predictions
- Test: `66.67%` sur `21` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente, filtre temporel.

Conditions:

- `close_z24 <= -2.487372513`
- `hour == 13`
- `body_sum6 >= -0.004083019831`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-01 13:45:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

### 86. micro_next_h1_red_461

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.69%` sur `102` predictions
- Test: `75.00%` sur `12` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, filtre temporel.

Conditions:

- `rsi14 >= 77.03278368`
- `hour == 11`
- `volume_z96 <= 2.919374564`

Features utilisees:

- `hour`: Heure UTC de la bougie courante, entre 0 et 23.
- `rsi14`: Relative Strength Index sur N bougies.
- `volume_z96`: Z-score du volume sur N bougies.

Exemple test:

- Timestamp: `2026-02-14 11:55:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 87. micro_next_h1_red_203

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.68%` sur `169` predictions
- Test: `68.42%` sur `19` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `rsi8 >= 73.82299439`
- `ret72 <= -0.01475596055`
- `close_z48 <= 1.450800747`

Features utilisees:

- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `ret72`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.
- `rsi8`: Relative Strength Index sur N bougies.

Exemple test:

- Timestamp: `2026-01-06 20:20:00+00:00`
- Prediction: `RED`
- Actual: `RED`
- Correct: `True`

### 88. micro_next_h1_red_421

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.66%` sur `265` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine momentum ou pression recente, position dans le range ou meche.

Conditions:

- `body_sum6 >= 0.01364096683`
- `donch_high12 >= -0.000509590257`
- `ret24 <= 0.0173612306`

Features utilisees:

- `body_sum6`: Somme des corps normalises sur N bougies. Negatif = pression vendeuse recente; positif = pression acheteuse recente.
- `donch_high12`: Distance du close au plus haut Donchian N: `close / rolling_high(N) - 1`.
- `ret24`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.

Exemple test:

- Timestamp: `2026-01-30 18:50:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 89. micro_next_h1_red_439

- Vote: `RED`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.57%` sur `244` predictions
- Test: `69.23%` sur `13` predictions
- Fonctionnement: Cette micro-strategie vote `RED` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, momentum ou pression recente.

Conditions:

- `close_z24 >= 2.783849801`
- `close_z48 <= 1.450800747`
- `ret12 >= 0.004280476703`

Features utilisees:

- `close_z24`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `close_z48`: Z-score du close sur N bougies: `(close - moyenneN) / ecart_typeN`.
- `ret12`: Rendement du close sur N bougies: `close / close.shift(N) - 1`.

Exemple test:

- Timestamp: `2026-01-21 19:25:00+00:00`
- Prediction: `RED`
- Actual: `GREEN`
- Correct: `False`

### 90. micro_next_h1_green_277

- Vote: `GREEN`
- Label mode: `next_candle_color`
- Horizon: `1` bougie
- Backtest: `65.56%` sur `270` predictions
- Test: `66.67%` sur `18` predictions
- Fonctionnement: Cette micro-strategie vote `GREEN` quand toutes ses conditions sont vraies. Elle combine extreme statistique/oscillateur, position dans le range ou meche, filtre temporel.

Conditions:

- `bb_pctb <= -0.107140425`
- `hour == 11`
- `donch_low144 <= 0.004305280217`

Features utilisees:

- `bb_pctb`: Position du close dans les bandes de Bollinger 20/2: `(close - lower_band) / (upper_band - lower_band)`.
- `donch_low144`: Distance du close au plus bas Donchian N: `close / rolling_low(N) - 1`.
- `hour`: Heure UTC de la bougie courante, entre 0 et 23.

Exemple test:

- Timestamp: `2026-01-14 11:15:00+00:00`
- Prediction: `GREEN`
- Actual: `GREEN`
- Correct: `True`

## Notes D'Implementation

- Les conditions doivent etre calculees sans regarder la prochaine bougie.
- La colonne cible `target_green` sert uniquement a evaluer, jamais a predire.
- Les features rolling doivent utiliser les bougies deja connues jusqu'a la bougie courante.
- Une bougie doji future est consideree neutre dans l'evaluation officielle et ne doit pas compter comme RED.
- Pour reproduire exactement les resultats actuels, garder les memes formules de features que `discover_micro_strategies.py`.
