# TCC Results Summary
Total valid runs: 24
Benchmarks: ['CG', 'EP', 'LU']
Strategies: ['DEGRADED', 'MANA_noFT', 'REPLACE', 'noFT']

## CG Class C

| Workers | Strategy | N | Trigger (s) | FT Wall Time (s) | Mop/s total | Phase1 (s) | Phase2 (s) | Phase3 (s) | Recovery (s) | Run Cost (spot $) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | noFT (native) | 1 | — | 34.3 | 4698.2 | — | — | — | — | 0.0019 |
| 2 | MANA noFT | 1 | — | 107.2 | 1416.4 | — | — | — | — | 0.0060 |
| 2 | REPLACE | 1 | 3s | 283.4 | 530.5 | 26.6 | 190.6 | 53.7 | 271.0 | 0.0158 |
| 2 | DEGRADED | 1 | 2s | 170.4 | 894.9 | 26.6 | 23.1 | 110.3 | 160.0 | 0.0095 |
| 4 | noFT (native) | 1 | — | 20.9 | 8288.8 | — | — | — | — | 0.0018 |
| 4 | MANA noFT | 1 | — | 32.8 | 5345.8 | — | — | — | — | 0.0029 |
| 4 | REPLACE | 1 | 2s | 256.8 | 576.6 | 26.5 | 185.4 | 38.1 | 250.0 | 0.0226 |
| 4 | DEGRADED | 1 | 2s | 93.7 | 1743.3 | 26.6 | 21.7 | 36.9 | 85.2 | 0.0083 |

## EP Class D

| Workers | Strategy | N | Trigger (s) | FT Wall Time (s) | Mop/s total | Phase1 (s) | Phase2 (s) | Phase3 (s) | Recovery (s) | Run Cost (spot $) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | noFT (native) | 1 | — | 330.5 | 417.3 | — | — | — | — | 0.0184 |
| 2 | MANA noFT | 1 | — | 492.6 | 281.4 | — | — | — | — | 0.0274 |
| 2 | REPLACE | 1 | 45s | 583.7 | 239.2 | 16.3 | 196.4 | 314.7 | 527.4 | 0.0325 |
| 2 | DEGRADED | 1 | 45s | 537.4 | 260.0 | 16.3 | 21.7 | 444.4 | 482.4 | 0.0299 |
| 4 | noFT (native) | 1 | — | 167.0 | 831.4 | — | — | — | — | 0.0147 |
| 4 | MANA noFT | 1 | — | 249.0 | 562.3 | — | — | — | — | 0.0219 |
| 4 | REPLACE | 1 | 45s | 409.1 | 342.7 | 16.4 | 185.6 | 151.5 | 353.5 | 0.0360 |
| 4 | DEGRADED | 1 | 45s | 294.6 | 479.8 | 16.3 | 21.6 | 205.0 | 243.0 | 0.0260 |

## LU Class C

| Workers | Strategy | N | Trigger (s) | FT Wall Time (s) | Mop/s total | Phase1 (s) | Phase2 (s) | Phase3 (s) | Recovery (s) | Run Cost (spot $) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | noFT (native) | 1 | — | 146.9 | 14271.0 | — | — | — | — | 0.0082 |
| 2 | MANA noFT | 1 | — | 242.1 | 8606.0 | — | — | — | — | 0.0135 |
| 2 | REPLACE | 1 | 45s | 397.3 | 5280.2 | 26.5 | 185.0 | 130.5 | 342.0 | 0.0221 |
| 2 | DEGRADED | 1 | 45s | 302.3 | 6932.5 | 26.6 | 21.7 | 198.9 | 247.2 | 0.0168 |
| 4 | noFT (native) | 1 | — | 80.7 | 26437.9 | — | — | — | — | 0.0071 |
| 4 | MANA noFT | 1 | — | 134.6 | 15689.0 | — | — | — | — | 0.0119 |
| 4 | REPLACE | 1 | 45s | 360.1 | 5810.4 | 26.5 | 220.9 | 57.5 | 305.0 | 0.0317 |
| 4 | DEGRADED | 1 | 45s | 190.2 | 11180.4 | 26.5 | 21.6 | 88.8 | 136.9 | 0.0168 |

