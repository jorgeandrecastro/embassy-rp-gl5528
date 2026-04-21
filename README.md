[![crates.io](https://img.shields.io/crates/v/embassy-rp-gl5528.svg)](https://crates.io/crates/embassy-rp-gl5528)
[![docs.rs](https://docs.rs/embassy-rp-gl5528/badge.svg)](https://docs.rs/embassy-rp-gl5528)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

# embassy-rp-gl5528

Driver async `no_std` minimaliste pour la photorésistance **GL5528** (LDR)  testé sur microcontrôleur **RP2040**, basé sur le framework [Embassy](https://embassy.dev).

---

## Description

La **GL5528** est une photorésistance (*Light Dependent Resistor*) dont la résistance varie selon l'intensité lumineuse ambiante :

| Condition       | Résistance approx. |
|-----------------|--------------------|
| Pleine lumière  | ~1 kΩ              |
| Obscurité       | ~1 MΩ+             |

Ce driver encapsule la lecture ADC asynchrone Embassy et expose une API simple pour intégrer ce capteur dans vos projets embarqués RP2040 , RP2350 .

---

## Câblage

Montez la GL5528 en diviseur de tension avec une résistance de tirage de **10 kΩ** :

```
3.3V
 │
[GL5528]
 │
 ├──── GP26 (ADC0) par exemple
 │
[R 10kΩ]
 │
GND
```

> La tension sur GP26 augmente avec la luminosité.

---

## Installation

Ajoutez la dépendance dans votre `Cargo.toml` svp régardez Features par défaut la feature est faite pour la pico 2040 :

**Important : Cette crate est conçue pour fonctionner avec Embassy v0.6 Assurez-vous que votre projet principal utilise la même version pour éviter les conflits de types!!**

```toml
[dependencies.embassy-rp-gl5528]
version = "0.1.0"
```

---

# Features
Par défaut, la crate utilise la feature rp2040. Si vous utilisez un Raspberry Pi Pico 2 ou une autre carte basée sur le RP2350, vous devez désactiver les features par défaut et activer rp235x.

**Pour le RP2040 (Pico 1)**

````
[dependencies.embassy-rp-gl5528]
version = "0.1.0"
Feature rp2040 activée par défaut
````

**Pour le RP235x (Pico 2)**
````
[dependencies.embassy-rp-gl5528]
version = "0.1.0"
default-features = false
features = ["rp235x"]
````

## Utilisation

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
use embassy_rp::bind_interrupts;
use embassy_rp::adc::InterruptHandler;
use embassy_rp_gl5528::Gl5528;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);

    let mut sensor = Gl5528::new(adc, channel);

    loop {
        let raw = sensor.read_raw().await;
        // raw : 0 (obscurité) → 4095 (pleine lumière)
    }
}
```

### Conversion valeur brute → tension

```rust
let voltage = (raw as f32 / 4095.0) * 3.3;
```

### Conversion valeur brute → résistance LDR

Avec une résistance de tirage `R_pull = 10 kΩ` :

```rust
// Si LDR est au 3.3V et 10k au GND :
let r_ldr = 10_000.0 * ((3.3 - voltage) / voltage);
```

---

## API

### `Gl5528::new(adc, channel) -> Self`

Crée le driver en prenant possession de l'ADC Embassy et du canal correspondant.

### `async fn read_raw(&mut self) -> u16`

Lit la valeur ADC brute (12 bits, `0..=4095`). Retourne `0` en cas d'erreur.

---

## Compatibilité

| Dépendance    | Version |
|---------------|---------|
| `embassy-rp`  | 0.6     |
| Rust edition  | 2024    |
| `no_std`      |  ✓      |

---

## Licence

Ce projet est distribué sous licence **GPL-2.0-or-later**.  
Voir le fichier [LICENSE](LICENSE) pour les détails complets.

---

# 🦅 À propos
Développé et testé par Jorge Andre Castro