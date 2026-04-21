[![crates.io](https://img.shields.io/crates/v/embassy-rp-gl5528.svg)](https://crates.io/crates/embassy-rp-gl5528)
[![docs.rs](https://docs.rs/embassy-rp-gl5528/badge.svg)](https://docs.rs/embassy-rp-gl5528)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

# embassy-rp-gl5528

Driver async `no_std` minimaliste pour la photorésistance **GL5528** (LDR)  testé sur microcontrôleur **RP2040**, basé sur le framework [Embassy](https://embassy.dev).

---


# 📄 Historique et Compatibilité
Ce projet suit de près l'évolution de l'écosystème Embassy pour garantir le support des nouvelles puces comme la RP2350.

Dernière version stable conseillée : Il est fortement recommandé d'utiliser la version 0.1.3(ou supérieure). Les versions précédentes étaient trop rigides sur les dépendances et peuvent causer des conflits de compilation.

**Important : Cette crate est compatible avec une large plage de versions (v0.4.0 à v0.10.0+).** Assurez-vous que votre projet utilise une version d' embassy-rp incluse dans cette plage.

Confiance et Évolution : Je fais pleinement confiance aux développeurs d'Embassy pour la stabilité de leurs APIs. Cependant, le monde de l'embarqué bouge vite : si vous testez ce driver et rencontrez le moindre défaut ou problème de compilation, n'hésitez pas à ouvrir une Issue GitHub. Votre aide est précieuse pour améliorer cet outil !

Changelog : Pour voir le détail des changements et l'évolution du support Pico 2, consultez le fichier CHANGELOG.md.

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

**Important : Cette crate est compatible avec une large plage de versions (v0.4.0 à v0.10.0+). Assurez-vous que votre projet utilise une version d' embassy-rp incluse dans cette plage.**

```toml
[dependencies.embassy-rp-gl5528]
version = "0.1.3"
```

---

# Features
Par défaut, la crate utilise la feature rp2040. Si vous utilisez un Raspberry Pi Pico 2 ou une autre carte basée sur le RP2350, vous devez désactiver les features par défaut et activer rp235x.

**Pour le RP2040 (Pico 1)**
**Feature rp2040 activée par défaut**
````
[dependencies.embassy-rp-gl5528]
version = "0.1.3"
````


**Pour le RP235x (Pico 2)**
Si vous utilisez la nouvelle Pico 2, désactivez les fonctionnalités par défaut (qui ciblent la RP2040) et activez la feature rp235x

````
[dependencies]
embassy-rp-gl5528 = { version = "0.1.3", default-features = false, features = ["rp235x"] }
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
        // raw : 0 (obscurité) → 4095 (pleine lumière) ou 16383 sur RP235x (14 bits )
        
    }
}
```

### Conversion valeur brute → tension

```rust
let voltage = (raw as f32 / 4095.0) * 3.3;

// Sur RP235x (14 bits)
let voltage = (raw as f32 / 16383.0) * 3.3;
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

### async fn read_raw(&mut self) -> u16  
Lit la valeur ADC brute du capteur.

- RP2040 : 12 bits (0..=4095).

- RP235x : 14 bits (0..=16383).

- Retourne 0 en cas d'erreur ADC.

---

## Compatibilité

| Dépendance    | Version    |
|---------------|------------|
| `embassy-rp`  | 0.4 à 0.10+|
| Rust edition  | 2024       |
| `no_std`      |  ✓         |

---


# Exemple Pico 2040 , Affichage avec la Oled :
 Utilise [`embassy-ssd1306`](https://crates.io/crates/embassy-ssd1306)  et [`sigmoid-q15`](https://crates.io/crates/sigmoid-q15)

````rust 
#![no_std]
#![no_main]

use cortex_m_rt as _;
use embassy_executor::Spawner;
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async};
use embassy_time::{Timer, Duration}; 
use {panic_halt as _, embassy_rp as _};

use embassy_ssd1306::Ssd1306;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::I2C0; 
use rp2040_linker as _; 

// GPIO et ADC
use embassy_rp::gpio::{Output, Level, Pull};
use embassy_rp::adc::{Adc, Config as AdcConfig, Channel, InterruptHandler as AdcInterruptHandler};
use embassy_rp::i2c::InterruptHandler as I2cInterruptHandler;

// Lumière et sigmoide 
use sigmoid_q15::sigmoid_q15;
use embassy_rp_gl5528::Gl5528; 

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2cInterruptHandler<I2C0>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

#[embassy_executor::task]
async fn system_task(
    mut oled: Ssd1306<I2cDevice<'static, NoopRawMutex, I2c<'static, I2C0, Async>>>,
    mut light_sensor: Gl5528<'static>,
) {
    if let Ok(_) = oled.init().await {
        oled.clear();
        let _ = oled.flush().await;
    }

    let test_points: [i16; 5] = [i16::MIN, -16384, 0, 16384, i16::MAX];
    let mut idx = 0;

    loop {
        oled.clear();
        oled.draw_rect(0, 0, 127, 63, true);

        let x_input = test_points[idx];
        let y_output = sigmoid_q15(x_input);

        oled.draw_str(10, 1, b"Sigmoid Q15");
        oled.draw_str(10, 3, b"In :");
        oled.draw_i16(50, 3, x_input);
        oled.draw_str(10, 5, b"Out:");
        oled.draw_i16(50, 5, y_output);

        // UTILISATION DU CAPTEUR 
        let lux_raw = light_sensor.read_raw().await;
        
        oled.draw_str(10, 7, b"Lumiere:");
        if lux_raw > 2000 {
            oled.draw_str(75, 7, b"Light");
        } else {
            oled.draw_str(75, 7, b"Dark");
        }

        let _ = oled.flush().await;
        idx = (idx + 1) % test_points.len();
        
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    
    // I2C & OLED
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 400_000; 
    let i2c_bus = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, i2c_config);

    static I2C_BUS: static_cell::StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, Async>>> = static_cell::StaticCell::new();
    let i2c_mutex = I2C_BUS.init(Mutex::new(i2c_bus));

    let i2c_dev_oled = I2cDevice::new(i2c_mutex);
    let oled = Ssd1306::new(i2c_dev_oled, 0x3C);
    let mut led = Output::new(p.PIN_25, Level::Low);

    // ADC & CAPTEUR (VIA TA CRATE)
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let p26 = Channel::new_pin(p.PIN_26, Pull::None);

    let light_sensor = Gl5528::new(adc, p26);
    
    spawner.spawn(system_task(oled, light_sensor)).unwrap();
    //blink led pico pin 25
    loop {
        led.toggle();
        Timer::after_millis(200).await;
    };
}

````

## Licence

Ce projet est distribué sous licence **GPL-2.0-or-later**.  
Voir le fichier [LICENSE](LICENSE) pour les détails complets.

---

# 🦅 À propos
Développé et testé par Jorge Andre Castro