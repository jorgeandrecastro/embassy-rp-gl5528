// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later
//! # embassy-rp-gl5528
//!
//! Driver async `no_std` minimaliste pour la photorésistance **GL5528** (LDR)
//! sur microcontrôleur RP2040, basé sur le framework [Embassy](https://embassy.dev).
//!
//! ## Description du composant
//!
//! La GL5528 est une photorésistance (LDR — *Light Dependent Resistor*) dont la
//! résistance varie entre ~1 kΩ (forte lumière) et ~10 MΩ (obscurité totale).
//! Elle est typiquement utilisée dans un diviseur de tension avec une résistance
//! fixe (10 kΩ recommandée) pour convertir la variation de résistance en tension
//! lisible par un ADC.
//!
//! ## Schéma de câblage
//!
//! ```text
//! 3.3V
//!  │
//! [GL5528]
//!  │
//!  ├──── GP26 (ADC0)
//!  │
//! [R 10kΩ]
//!  │
//! GND
//! ```
//!
//! La tension sur la broche ADC augmente avec la luminosité (la GL5528 devient
//! moins résistante, le point milieu monte vers 3,3 V).
//!
//! ## Exemple d'utilisation
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! use embassy_executor::Spawner;
//! use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
//! use embassy_rp::bind_interrupts;
//! use embassy_rp::peripherals::ADC;
//! use embassy_rp::adc::InterruptHandler;
//! use embassy_rp_gl5528::Gl5528;
//!
//! bind_interrupts!(struct Irqs {
//!     ADC_IRQ_FIFO => InterruptHandler;
//! });
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let p = embassy_rp::init(Default::default());
//!
//!     let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
//!     let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);
//!
//!     let mut sensor = Gl5528::new(adc, channel);
//!
//!     loop {
//!         let raw = sensor.read_raw().await;
//!         // Valeur entre 0 (obscurité) et 4095 (pleine lumière sur 12 bits)
//!         let _ = raw;
//!     }
//! }
//! ```
//!
//! ## Calcul de luminosité
//!
//! La valeur brute ADC (12 bits, 0–4095) peut être convertie en tension :
//!
//! ```text
//! V = raw × 3.3 / 4095
//! ```
//!
//! La résistance de la LDR s'en déduit (diviseur de tension, R_pull = 10 kΩ) :
//!
//! ```text
//! R_ldr = R_pull × V / (3.3 - V)
//! ```
//!
//! ## Caractéristiques
//!
//! | Paramètre              | Valeur                    |
//! |------------------------|---------------------------|
//! | Tension d'alimentation | 3,3 V (RP2040)            |
//! | Résolution ADC         | 12 bits (0–4095)          |
//! | Résistance lumière     | ~1 kΩ @ 10 lux            |
//! | Résistance obscurité   | ~1 MΩ minimum             |
//! | Résistance de tirage   | 10 kΩ recommandée         |
//!
//! ## `no_std`
//!
//! Cette crate ne dépend pas de la bibliothèque standard et est conçue pour
//! tourner sur des microcontrôleurs bare-metal avec le runtime Embassy.

#![no_std]
#![forbid(unsafe_code)]

use embassy_rp::adc::{Adc, Async, Channel};

/// Driver pour la photorésistance GL5528 via l'ADC du RP2040.
///
/// Ce driver encapsule un canal ADC Embassy et fournit une lecture asynchrone
/// de la valeur brute du capteur.
///
/// # Exemple
///
/// ```rust,no_run
/// # use embassy_rp::adc::{Adc, Channel};
/// # use embassy_rp_gl5528::Gl5528;
/// // Création du driver (adc et channel obtenus depuis les périphériques Embassy)
/// // let mut sensor = Gl5528::new(adc, channel);
/// // let raw: u16 = sensor.read_raw().await;
/// ```
pub struct Gl5528<'d> {
    adc: Adc<'d, Async>,
    channel: Channel<'d>,
}

impl<'d> Gl5528<'d> {
    /// Crée une nouvelle instance du driver GL5528.
    ///
    /// # Arguments
    ///
    /// * `adc` — Périphérique ADC Embassy en mode asynchrone.
    /// * `channel` — Canal ADC connecté à la broche de lecture de la GL5528.
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// # use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
    /// # use embassy_rp_gl5528::Gl5528;
    /// // let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    /// // let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);
    /// // let sensor = Gl5528::new(adc, channel);
    /// ```
    #[inline]
    pub fn new(adc: Adc<'d, Async>, channel: Channel<'d>) -> Self {
        Self { adc, channel }
    }

    /// Lit la valeur brute du convertisseur ADC.
    ///
    /// Retourne un entier non signé 16 bits dans l'intervalle `0..=4095`
    /// (résolution 12 bits du RP2040). Une valeur élevée correspond à une
    /// forte luminosité ; une valeur faible correspond à une faible luminosité
    /// ou à l'obscurité.
    ///
    /// En cas d'erreur ADC, la valeur `0` est retournée.
    ///
    /// # Retour
    ///
    /// * `u16` — Valeur ADC brute, entre `0` (obscurité) et `4095` (lumière max).
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// # use embassy_rp_gl5528::Gl5528;
    /// # async fn example(mut sensor: Gl5528<'_>) {
    /// let raw: u16 = sensor.read_raw().await;
    ///
    /// // Conversion en tension (V)
    /// let voltage = raw as f32 * 3.3 / 4095.0;
    ///
    /// // Conversion en résistance LDR (kΩ), avec R_pull = 10 kΩ
    /// if voltage < 3.3 {
    ///     // Si LDR est au 3.3V et R_pull (10k) au GND :
  ///     let r_ldr = 10.0 * (3.3 - voltage) / voltage;
    ///     let _ = r_ldr;
    /// }
    /// # }
    /// ```
    #[inline]
    pub async fn read_raw(&mut self) -> u16 {
        self.adc.read(&mut self.channel).await.unwrap_or(0)
    }
}