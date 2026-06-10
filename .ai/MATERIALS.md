# Hardware References for the Prototype (BOM)

This document lists the exact part numbers recommended for initial testing of the **Dartdetect** prototype, whether using the microphone or piezoelectric sensor approach.

---

## 1. Development Boards (ESP32)

Two boards are recommended to validate wireless communication via the ESP-NOW protocol.

| Role | Recommended Part | Key Features | Why this choice? |
| :--- | :--- | :--- | :--- |
| **Transmitter** (Battery-powered target) | **DFRobot FireBeetle 2 ESP32-E** (or S3 version) | Ultra-efficient LDO regulator, integrated JST battery connector. Consumption ~13 µA (Deep Sleep) / ~0.8 mA (Light Sleep). | Specifically designed for low power consumption. Prevents battery drain at rest. Fully compatible with Rust. |
| **Receiver** (USB gateway to PC) | **ESP32 DevKit V1** (Generic NodeMCU / WROOM-32) | Very affordable, all pins accessible on breadboard, continuous power via micro-USB or USB-C. | Permanently powered by the server, this module only needs to receive ESP-NOW frames and forward them over serial. |

---

## 2. Acoustic Capture (Experimental Options)

### Option A: Airborne Digital Microphones (High Precision)
Ideal if you want to process the audio signal of the dart's "thud" traveling through the room air.
* **Primary Reference: INMP441 (I2S Module)**
  * *Description:* Omnidirectional microphone with direct I2S digital output.
  * *Advantage:* The signal is already digitized at the source. The ESP32 reads it directly in a background task via its I2S controller (DMA) without overloading the processor.
* **Premium Alternative: Adafruit ICS-43434** (or SPH0645)
  * *Description:* High-quality MEMS I2S microphone offering excellent signal-to-noise ratio for precisely detecting the impact wavefront.

### Option B: Piezoelectric Sensors (Contact Microphones)
Ideal for isolating the target from room noise (music, voices) by only listening to the internal vibration of the wood/plastic.
* **Raw Component: Piezoelectric Discs (20mm or 27mm)**
  * *Description:* Simple brass/ceramic disc with pre-soldered wires (very affordable).
  * *Note:* Requires a small amplification or comparator circuit (e.g., LM393 chip) to convert the impact into a $0\text{V} - 3.3\text{V}$ logic signal readable by the ESP32.
* **Module Reference (Ready-to-use): KY-031 Vibration Module** (or equivalent based on an LM393 comparator)
  * *Description:* Integrates a piezo disc, a sensitivity adjustment potentiometer (blue trim pot), and a direct digital output (`DO`). Ideal for coding in Rust immediately without complex electronics work.

---

## 3. Power & Energy (For the target transmitter)

| Type | Recommended Part | Capacity / Role | Notes |
| :--- | :--- | :--- | :--- |
| **Li-Ion Cell** (Battery format) | **Panasonic NCR18650B** (3.7V) | 3400 mAh | The industry reliability standard. Offers maximum autonomy (up to one year depending on the ESP32 sleep mode). |
| **Li-Po Cell** (Flat format) | **Li-Po 3.7V Battery** with JST-PH 2.0mm connector | 2000 to 3000 mAh | Lightweight and flat, easy to velcro or integrate behind the target casing. Plugs directly into the FireBeetle 2 board. |
| **Charger Module** (If using a standard ESP32) | **TP4056 Module with protection** (USB-C version) | Charge controller & BMS | Essential if your ESP32 board lacks a battery connector. Protects the cell against deep discharge and manages USB-C charging. |

---

## 4. Lab Tools (For debugging)

To validate that your timings are measured to the microsecond during your initial dart throws, these tools are highly recommended:
* **A large Breadboard** (solderless prototyping board) accompanied by a set of Dupont jumper wires (Male-to-Male and Male-to-Female).
* **An 8-channel USB Logic Analyzer (Saleae-compatible / 24 MHz)**
  * *Use:* A very affordable little box (10–15 €) that plugs into your PC. It lets you visually display the signals from your 4 microphones on screen, allowing you to visually verify the time deltas ($t_1, t_2, t_3, t_4$) before even sending them to your Rust server.
