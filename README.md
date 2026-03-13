NUCLEO-H753ZI + TB6612FNG — Quick Start

Summary
- Minimal firmware to drive one TB6612FNG channel from a NUCLEO-H753ZI.
- Uses `stm32h7xx-hal` and `embedded-hal` (0.2) — built for `thumbv7em-none-eabihf`.

Pin mapping (current firmware)
- PWMA (PWM speed): PB9 (CN7)
- AIN1 (direction): PB15 (CN7)
- AIN2 (direction): PB13 (CN7)
- STBY (enable):    PB12 (CN7)
- TB6612 VCC:        NUCLEO 3.3 V
- TB6612 GND:        NUCLEO GND and Battery negative (common ground)
- TB6612 VM:         Battery +12 V (motor supply)

Wiring notes
- Connect TB6612 GND to BOTH the NUCLEO GND and the battery negative (B−). This common ground is required for logic signals to be referenced correctly.
- VCC on TB6612 must be 3.3 V from the NUCLEO (not battery +12 V).
- STBY must be pulled HIGH (firmware drives `PB12` high) to enable outputs.
- Use the Morpho `CN7` cluster (PB9/PB15/PB13/PB12) for compact wiring as shown above.

Build & flash
- Ensure target and runner are set in `.cargo/config.toml` (probe-rs runner configured).
- Build and flash in one step (requires probe-rs and a connected ST-LINK):

```bash
cargo run --release
```

Troubleshooting checklist
1. Power off. Verify wiring matches the pin mapping above.
2. Confirm TB6612 VCC = 3.3 V and VM = battery +12 V.
3. Confirm continuity: NUCLEO GND ⇄ TB6612 GND ⇄ Battery negative.
4. Run firmware; verify STBY pin reads ~3.3 V when firmware starts.
5. Probe PWM pin (PB9) with logic probe / scope — should see PWM bursts during the demo.
6. If logic signals are present but motor doesn't spin, verify VM and motor connections (A01/A02) and that the motor can draw required current.

If you want, I can add a PNG wiring diagram and a CN7 pin-index reference into the repo.
