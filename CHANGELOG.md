# Changelog

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

# [0.1.3] - 2026-04-21
### Changements
- **Refonte complète du README** : Détails et doc plus claire .
- **Optimisation du Cargo.toml** : Elargissement de la compatibilité avec `embassy-rp` (v0.4 à v0.10+).
- **Support RP235x** : Ajout de la feature pour la Raspberry Pi Pico 2 dans la doc et le README .

## [0.1.2] - 2026-04-21

### Ajouté
- Support officiel de la **Raspberry Pi Pico 2 (RP235x)** via la feature `rp235x`.
- Documentation mise à jour pour inclure la compatibilité RP235x.

### Modifié
- **Dépendances** : Élargissement de la plage de version pour `embassy-rp` (`>=0.4, <0.11`). 
- Ce changement permet d'utiliser les dernières versions de Crates.io (v0.10.0+) sans conflits de dépendances.
- Mise à jour de la feature interne pour le RP235x vers `rp235xa` pour correspondre aux standards d'Embassy v0.10.

### Corrigé
- Résolution des conflits de "links" lors de l'utilisation du driver dans un projet utilisant une version récente d'Embassy.