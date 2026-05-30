# PetKit settings research

参照ソース:

- `RobertD502/petkitaio`
- `RobertD502/home-assistant-petkit`
- `Jezza34000/py-petkit-api`
- `Jezza34000/homeassistant_petkit`

前提メモ:

- ここでの `Devices` は、Python 実装で実際に `update_setting` / `control_device` を送っている箇所、または `is_supported()` 相当の entity 判定から確認できた範囲。
- `FeederMini` は `update` の旧 endpoint を使い、複数キーで `settings.*` 形式の実 payload を使う。
- `K3` も `update` の旧 endpoint を使う (`py-petkit-api` / Rust 側実装一致)。
- `control_device` は `kv` の JSON とは別に form の `type` (`start` / `stop` / `continue` / `end` / `power` / `mode`) を送る。下表では `kv` 側の実 payload を `Key` 列にそのまま書く。
- `py-petkit-api` の新しい HA 統合は「型リスト固定」ではなく「値が `None` でない時だけ entity を出す」キーも多い。そのため一部の新機種キーは「推定込み」で整理している。

## Feeder (D3, D4, D4s, D4h, D4sh, Feeder, FeederMini)

| Key | Type | Devices | 説明 |
| --- | --- | --- | --- |
| `lightMode` / `settings.lightMode` | int (0/1) | D3, D4, D4s, D4h, D4sh, Feeder, FeederMini | インジケーター LED。`Feeder` / `FeederMini` は実 payload が `settings.lightMode`。 |
| `manualLock` / `settings.manualLock` | int (0/1) | D3, D4, D4s, D4h, D4sh, Feeder, FeederMini | チャイルドロック。`Feeder` / `FeederMini` は実 payload が `settings.manualLock`。 |
| `foodWarn` | int (0/1) | D4, D4s, D4h, D4sh | フード残量低下アラーム。古い HA では D4/D4s、新しい実装では YumShare 系でも確認。 |
| `feedSound` | int (0/1) | D4, D4h, D4sh | 給餌音。D4 系で使われるキー。 |
| `feedTone` | int (0/1) | D4s | 給餌音。D4s だけ別キー。 |
| `soundEnable` | int (0/1) | D3, D4h, D4sh | 音声付き給餌。 |
| `disturbMode` | int (0/1) | D3 | おやすみモード。古い HA では D3 専用。 |
| `systemSoundEnable` | int (0/1) | D3, D4h, D4sh | システム通知音。 |
| `surplus` | int | D3 | 残量しきい値。古い HA では 20-100g。 |
| `volume` | int | D3, D4h, D4sh | スピーカー音量。新しい HA では camera feeder と D3 で使用。 |
| `shortest` | int | D4s | 最短食事時間 (秒)。 |
| `selectedSound` | int | D3 | 再生する音声 ID。古い HA の sound select で確認。 |
| `feedNotify` | int (0/1) | D4h, D4sh | 給餌通知。 |
| `settings.feedNotify` | int (0/1) | FeederMini | 給餌通知。Mini は `settings.*`。 |
| `foodNotify` | int (0/1) | D4h, D4sh | 補充通知。 |
| `settings.foodNotify` | int (0/1) | FeederMini | 補充通知。Mini は `settings.*`。 |
| `desiccantNotify` | int (0/1) | D4h, D4sh | 乾燥剤交換通知。 |
| `settings.desiccantNotify` | int (0/1) | FeederMini | 乾燥剤交換通知。Mini は `settings.*`。 |
| `petNotify` | int (0/1) | D4h, D4sh | ペット接近通知。camera feeder 系。 |
| `eatNotify` | int (0/1) | D4h, D4sh | 食事通知。camera feeder 系。 |
| `moveNotify` | int (0/1) | D4h, D4sh | 動体通知。camera feeder 系。 |
| `lowBatteryNotify` | int (0/1) | D4h, D4sh | バッテリー低下通知。camera feeder 系。 |
| `camera` | int (0/1) | D4h, D4sh | カメラ有効化。 |
| `highlight` | int (0/1) | D4h, D4sh | ペットトラッキング。 |
| `timeDisplay` | int (0/1) | D4h, D4sh | 動画タイムスタンプ表示。 |
| `microphone` | int (0/1) | D4h, D4sh | マイク有効化。 |
| `night` | int (0/1) | D4h, D4sh | 暗視。 |
| `surplusControl` | int (0/1) | D3, D4h, D4sh | 残量制御の有効化。D4h/D4sh では `surplusStandard` と対で使う。 |
| `surplusStandard` | int enum | D4h, D4sh | 残量制御のレベル。新しい HA の select で使用。 |
| `eatSensitivity` | int enum | D4h, D4sh | AI 食事検知感度。 |
| `petSensitivity` | int enum | D4h, D4sh | AI ペット検知感度。 |
| `moveSensitivity` | int enum | D4h, D4sh | AI 動体検知感度。 |

## Litter Box (T3, T4, T5, T6, T7) — update_setting

| Key | Type | Devices | 説明 |
| --- | --- | --- | --- |
| `lightMode` | int (0/1) | T3, T4, T5, T6, T7 | 本体表示 / ライト。 |
| `manualLock` | int (0/1) | T3, T4, T5, T6, T7 | チャイルドロック。 |
| `disturbMode` | int (0/1) | T3, T4, T5, T6, T7 | おやすみモード。 |
| `autoWork` | int (0/1) | T3, T4, T5, T6, T7 | 自動清掃。 |
| `avoidRepeat` | int (0/1) | T3, T4, T5, T6, T7 | 連続清掃回避。 |
| `fixedTimeClear` | int (0/1) | T3, T4, T5, T6, T7 | 定期清掃。 |
| `kitten` | int (0/1) | T3, T4, T5, T6, T7 | 子猫モード。 |
| `stillTime` | int | T3, T4, T5, T6, T7 | ペット退出後の待機時間 (秒)。T7 は HA 側で 20 分以上に制限。 |
| `autoIntervalMin` | int enum | T3, T4, T5, T6, T7 | Avoid repeat clean の間隔。 |
| `autoRefresh` | int (0/1) | T3, T4 (+K3 連携時) | 自動脱臭。古い HA では T3 または K3 連携付き T4。 |
| `fixedTimeRefresh` | int (0/1) | T3, T4 (+K3 連携時) | 定期脱臭。古い HA では T3 または K3 連携付き T4。 |
| `sandType` | int enum | T3, T4, T5, T6 | 猫砂タイプ。新しい HA では T7 を除外。 |
| `underweight` | int (0/1) | T3, T4 | 軽量体重の計測/通知制御。古い HA では Pura X / MAX、 新しい HA でも camera litter を除外。 |
| `downpos` | int (0/1) | T4 | Continuous rotation。古い HA の Pura MAX 専用。 |
| `deepClean` | int (0/1) | T4 | Deep clean。古い HA の Pura MAX 専用。 |
| `deepRefresh` | int (0/1) | T3, T4 | Deep deodorizing (旧 Pura Air 系)。 |
| `autoSpray` | int (0/1) | T5, T6 | 自動脱臭 (N60 系)。 |
| `fixedTimeSpray` | int (0/1) | T5, T6 | 定期脱臭 (N60 系)。 |
| `deepSpray` | int (0/1) | T5, T6 | Deep deodorizing (N60 系)。`py-petkit-api` README でも T5/T6 と明記。 |
| `sandSaving` | int (0/1) | T5, T6, T7 | 猫砂節約モード。新しい py 系実装のみで確認。 |
| `bury` | int (0/1) | T3, T4, T5, T6 | 排泄物カバー。新しい HA では T7 を除外。 |
| `volume` | int | T5, T6 | スピーカー音量。新しい HA の共通 number では T5/T6 で使用。 |
| `petNotify` | int (0/1) | T5, T6, T7 | ペット訪問通知。新しい py 系実装のみで確認。 |
| `litterFullNotify` | int (0/1) | T5, T6, T7 | ゴミ満杯通知。 |
| `petInNotify` | int (0/1) | T5, T6, T7 | ペット入室通知。 |
| `workNotify` | int (0/1) | T5, T6, T7 | 清掃/動作通知。 |
| `deodorantNotify` | int (0/1) | T4, T5, T6 | N50 脱臭剤通知。`RESET_N50_DEODORIZER` 対応機種と整合。 |
| `sprayNotify` | int (0/1) | T5, T6, T7 | N60 脱臭剤通知。 |
| `lackSandNotify` | int (0/1) | T5, T6, T7 | 猫砂不足通知。 |
| `logNotify` | int (0/1) | T5, T6, T7 | 動作ログ通知。 |
| `camera` | int (0/1) | T5, T6, T7 | カメラ有効化。camera litter 系。 |
| `timeDisplay` | int (0/1) | T5, T6, T7 | 動画タイムスタンプ表示。 |
| `microphone` | int (0/1) | T5, T6, T7 | マイク有効化。 |
| `night` | int (0/1) | T5, T6 | 暗視。新しい HA は T7 を明示除外。 |
| `cameraLight` | int (0/1) | T5, T6, T7 | カメラライト。camera litter 系。 |
| `homeMode` | int (0/1) | T5, T6, T7 | プライバシーモード。 |
| `cameraOff` | int (0/1) | T5, T6, T7 | プライバシー用のカメラ OFF。 |
| `cameraInward` | int (0/1) | T6 | カメラを内向きにするプライバシー制御。`py-petkit-api` README で T6 明記。 |
| `noSound` | int (0/1) | T5, T6, T7 | プライバシー用のマイク OFF。 |
| `lightAssist` | int (0/1) | T7 (推定) | 補助ライト。新しい py 系 litter schema のみで確認。 |
| `toiletNotify` | int (0/1) | T7 (推定) | 排泄通知。新しい py 系 litter schema のみで確認。 |
| `toiletLight` | int (0/1) | T7 (推定) | トイレライト。新しい py 系 litter schema のみで確認。 |
| `phDetection` | int (0/1) | T7 (推定) | 尿 pH 検知。新しい py 系 litter schema / README の AI 項目。 |
| `voice` | int (0/1) | T7 (推定) | 鳴き声検知。新しい py 系 litter schema / README の AI 項目。 |
| `softMode` | int (0/1) | T7 (推定) | 軟便検知。新しい py 系 litter schema / README の AI 項目。 |
| `softModeClean` | int (0/1) | T7 (推定) | 軟便検知時の清掃停止。新しい HA だけで確認。 |
| `highlight` | int (0/1) | T3, T4 | ペット追跡。新しい HA は camera litter (`T5/T6/T7`) を除外。 |
| `lackLiquidNotify` | int (0/1) | T5, T6, T7 (推定) | 消臭液不足通知。新しい py 系実装のみで確認。 |
| `systemSoundEnable` | int (0/1) | T5, T6, T7 (推定) | システム通知音。新しい py 系実装のみで確認。 |

## Litter Box (T3, T4, T5, T6, T7) — control_device

| Key | Type | Devices | 説明 |
| --- | --- | --- | --- |
| `{"start_action": 0}` | object<int enum> | T3, T4, T5, T6, T7 | 清掃開始 (`LBCommand.CLEANING`)。 |
| `{"start_action": 9}` | object<int enum> | T4, T5 | メンテナンスモード開始 (`LBCommand.MAINTENANCE`)。 |
| `{"end_action": 9}` | object<int enum> | T4, T5 | メンテナンスモード終了。 |
| `{"start_action": 1}` | object<int enum> | T3, T4, T5, T6 | 猫砂排出 (`LBCommand.DUMPING`)。新しい HA は T7 を除外。 |
| `{"stop_action": <work_mode>}` | object<int> | T3, T4, T5, T6, T7 | 進行中動作の一時停止。値は現在の `work_mode`。 |
| `{"continue_action": <work_mode>}` | object<int> | T3, T4, T5, T6, T7 | 一時停止中動作の再開。 |
| `{"end_action": <work_mode>}` | object<int> | T3, T4, T5, T6, T7 | 進行中動作のリセット/中断。 |
| `{"start_action": 2}` | object<int enum> | T3, T4 (+K3 連携時), T5, T7 | 脱臭開始 (`LBCommand.ODOR_REMOVAL`)。`py-petkit-api` README でも T3/T4/T5/T7 と記載。 |
| `{"start_action": 10}` | object<int enum> | T5, T6, T7 | N60 脱臭剤リセット (`LBCommand.RESET_N60_DEODOR`)。 |
| `{"start_action": 4}` | object<int enum> | T3, T4, T5, T6, T7 | 猫砂ならし (`LBCommand.LEVELING`)。 |
| `{"power_action": 1}` / `{"power_action": 0}` | object<int (0/1)> | T3, T4, T5, T6, T7 | 電源 ON/OFF。 |
| `{"start_action": 7}` | object<int enum> | T4 (+K3 連携時) | ライト ON (`LBCommand.LIGHT`)。古い HA の Pura MAX + Pura Air ボタンで確認。 |
| `{"start_action": 8}` | object<int enum> | T4 (+K3 連携時) | 旧 `RESET_MAX_DEODOR`。古い HA の Pura MAX + Pura Air で確認。 |

補足:

- `LitterCommand.RESET_N50_DEODORIZER` は `control_device` ではなく専用 endpoint (`deodorantReset`) を使うため、上表には含めていない。
- 古い `petkitaio` / HA では `pause_clean`, `resume_clean`, `start_maintenance`, `exit_maintenance`, `pause_litter_dump`, `resume_litter_dump` などを enum 化していたが、wire 上は上表の `start/stop/continue/end_action` + 数値に正規化される。

## Water Fountain (W4, W5, Ctw2, Ctw3)

`update_setting` で送っている kv は、調査した Python 実装では確認できなかった。

| Key | Type | Devices | 説明 |
| --- | --- | --- | --- |
| _(none confirmed)_ | - | W4, W5, Ctw2, Ctw3 | `petkitaio` / 両 HA 実装 / `py-petkit-api` では、fountain 制御は BLE `FountainAction` (`POWER_ON/OFF`, `PAUSE/CONTINUE`, mode, DND, light, `RESET_FILTER`) を使っていた。`settings` モデル (`lampRingSwitch`, `lampRingBrightness`, `noDisturbingSwitch` など) は存在するが、`update_setting` 送信箇所は見つかっていない。 |

## Air Purifier (K2, K3)

| Key | Type | Devices | 説明 |
| --- | --- | --- | --- |
| `lightMode` | int (0/1) | K2, K3 | ライト ON/OFF。古い HA と新しい HA の共通 light entity で確認。 |
| `sound` | int (0/1) | K2, K3 | 通知音 ON/OFF。 |

補足:

- `py-petkit-api` の `purifier_container.Settings` には `autoWork`, `fixedTimeRefresh`, `lackNotify`, `manualLock`, `tempUnit` なども定義されているが、調査範囲では Python 実装から実際に `update_setting` 送信している箇所は確認できなかった。
- `control_device` は K2 で明確に使われており、`{"power_action": 0/1}` と `{"mode_action": 0..3}` (`auto/silent/standard/strong`) を送る。`ACTIONS_MAP` 上は K3 も `control_device` 対応に含まれるが、新しい HA fan entity は K2 のみを露出していた。

## Pet (`pet_update_setting`)

| Key | Type | Devices | 説明 |
| --- | --- | --- | --- |
| `weight` | int | Pet | 体重。`py-petkit-api` README では grams、古い HA は UI で kg/lb を変換して送信。 |
