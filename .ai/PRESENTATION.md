# dart_scoring — Présentation du module

## Rôle dans le projet

`dart_scoring` est le module d'orchestration central de **Dartdetect**. Il agit comme le **point d'entrée unique** pour tout calcul de score : il reçoit des données brutes (coordonnées d'impact ou timings capteurs), orchestre la chaîne localisation → interprétation, et retourne un score ou un `ShotResult` prêt à être injecté dans le moteur de jeu.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│                dart_scoring                 │
│                                             │
│   calculate_score_for_impact_point()        │
│   calculate_score_for_sensors_timings()     │
│                                             │
│   ┌──────────┐    ┌──────────────┐          │
│   │ calcula- │    │ interpré-    │          │
│   │ tion     │───►│ tation       │          │
│   │ (locator)│    │ (score)      │          │
│   └──────────┘    └──────────────┘          │
└─────────────────────────────────────────────┘
          ▲                    │
          │                    ▼
   dart_calculator      dart_interpretor
   (physique/DoA)      (secteurs/règles)
```

Le module expose deux fonctions publiques qui couvrent les deux cas d'usage du projet :

### 1. `calculate_score_for_impact_point(impact_point: Point) -> Result<u32, String>`

Mode **simulation** — utilisé par `dart_simu_cli` pour tester sans matériel.

Pipeline interne :
1. Calcule les timings simulés via `ImpactSimulator::from_point()` (distance parcourue par le son à 343 m/s + bruit de fond)
2. Passe ces timings à `ImpactLocator::locate()` qui résout la position par **descente de gradient** (algorithme `Solver`)
3. Convertit la position trouvée en score via `Score`

### 2. `calculate_score_for_sensors_timings(t1, t2, t3, t4) -> Result<(Point, u32), String>`

Mode **temps réel** — utilisé par `dart_server` quand il reçoit des données des capteurs piézo.

Pipeline interne :
1. Reçoit les 4 timings capteurs (en microsecondes) directement du réseau
2. Localise l'impact via le solveur
3. Retourne à la fois les coordonnées **et** le score

---

## Flux de données complet

```
Timings capteurs (t1..t4)
        │
        ▼
┌─────────────────┐
│  ImpactLocator  │  (dart_calculator)
│  ┌───────────┐  │
│  │  Solver   │  │  Descente de gradient
│  │  (gradient │  │  pour minimiser l'erreur
│  │  descent)  │  │  L² entre distances mesurées
│  └───────────┘  │  et distances candidates
│        │        │
│  ⇩ coordonnées  │
│  Point {x, y}   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Score          │  (dart_interpretor)
│  ┌───────────┐  │
│  │ board_    │  │  Angle → secteur (1..20)
│  │ angle()   │  │  (ex: 0° = haut = 20)
│  └───────────┘  │
│  ┌───────────┐  │
│  │ multiplier │  │  Rayon → multiplicateur
│  │ _from_     │  │  (single/double/triple/bulls)
│  │ radius()   │  │
│  └───────────┘  │
│        │        │
│  ⇩ score u32    │
└────────┬────────┘
         │
         ▼
   Score ou ShotResult
         │
         ▼
  dart_game::GameSession.apply_shot()
```

---

## Détail des sous-composants

### Capteurs et coordonnées

Le plateau est modélisé dans un repère **500 mm × 500 mm** :
- Origine (0,0) = coin **bas-gauche**
- (500,500) = coin **haut-droit**
- Centre du bull = (250, 250)

Quatre capteurs piézo sont placés aux quatre coins :

| Capteur | Position        |
|---------|-----------------|
| C1      | (0, 500)        |
| C2      | (500, 500)      |
| C3      | (500, 0)        |
| C4      | (0, 0)          |

### Solveur par descente de gradient

Le solveur (`Solver` dans `dart_calculator`) implémente un algorithme itératif qui :

1. **Initialise** une grille de points candidats (pas de 50 mm)
2. **Évalue** l'erreur quadratique (L²) entre :
   - Les distances calculées depuis les timings réels
   - Les distances calculées depuis chaque point candidat
3. **Affine** avec une descente de gradient sur le meilleur candidat
4. **Snappe** au point entier le plus proche

**Performances** (relevées dans `main.rs`) :

| Version | Itérations | Temps       |
|---------|------------|-------------|
| Sans gradient | 125 606 | 36,19 ms |
| **Avec gradient** | **13** | **297 µs** |

Soit un facteur ×10 000 en rapidité, rendant le système utilisable en temps réel.

### Interprétation du score

Le module `dart_interpretor` convertit les coordonnées localisées en score en deux étapes :

**Angle → Secteur** (dans `zone.rs`) :
- Division du cercle en 20 secteurs de 18° chacun
- Ordre des secteurs selon la norme d'une cible officielle : 20, 1, 18, 4, 13, 6, 10, 15, 2, 17, 3, 19, 7, 16, 8, 11, 14, 9, 12, 5
- 0° = haut de la cible = secteur 20

**Rayon → Multiplicateur** (dans `multiplicator.rs`) :

| Rayon (mm) | Zone         | Multiplicateur |
|------------|--------------|----------------|
| ≤ 6,35     | Double bull  | 2              |
| ≤ 15,9     | Single bull  | 1              |
| ≤ 97       | Single (int.)| 1              |
| 97 – 105   | Triple ring  | 3              |
| 105 – 162  | Single (ext.)| 1              |
| 162 – 170  | Double ring  | 2              |
| 170 – 225  | Bord noir    | 0              |
| > 225      | Hors cible   | 0              |

### Intégration avec le moteur de jeu

`Score::get_shortresult()` produit un `ShotResult { sector, multiplier }` qui peut être passé à `GameSession::apply_shot()`. Cela permet d'enchaîner :

```
Timings capteurs
  → dart_scoring::calculate_score_for_sensors_timings()
    → Score::get_shortresult()
      → GameSession::apply_shot()
        → ShotOutcome { score, is_bust, turn_over, game_over, message }
```

---

## Points d'extension

- **Nouveaux capteurs** : la signature `[SensorPos; 4]` est extensible. Le solveur et le simulateur sont paramétrés par la position des capteurs.
- **Nouveaux modes de jeu** : le trait `GameVariant` (dans `dart_game`) permet d'implémenter n'importe quel format (Cricket, Killer, etc.) sans toucher à `dart_scoring`.
- **Bruit capteur** : `ImpactSimulator::from_point()` ajoute déjà un bruit de fond (`baseline_time_us`). Une modélisation plus fine du bruit piézo peut y être intégrée.

---

## Dépendances dans le projet

```
dart_engine
 ├── dart_core          ← Types partagés (Point, SensorPos)
 ├── dart_calculator    ← Solveur, simulateur (distance, erreur, gradient)
 ├── dart_interpretor   ← Scoring (secteur, multiplicateur)
 ├── dart_scoring       ← Orchestrateur (ce module)
 └── dart_game          ← Sessions de jeu, validation des règles
```

En production, `dart_server` appelle `calculate_score_for_sensors_timings()` à chaque réception de données capteurs. `dart_ui` affiche le score retourné en temps réel via WebSocket.
`
