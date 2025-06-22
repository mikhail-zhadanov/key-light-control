# Key Light Control

A Windows GUI application to control Elgato Key Lights via their local HTTP API.
Built with [egui](https://github.com/emilk/egui), [eframe](https://github.com/emilk/egui/tree/master/crates/eframe), and [reqwest](https://github.com/seanmonstar/reqwest).

## Features

- System tray icon for quick access and hiding the main window
- Toggle light on/off
- Adjust brightness (0–100)
- Adjust color temperature (2900K–7000K, 50K steps)
- Auto-start with Windows option
- Periodic background polling and control
- Settings saved in Windows registry

## Usage

1. **Configure IP and Port**  
   Enter your Elgato Key Light's IP address and port in the GUI.
2. **Control the Light**  
   - Use the toggle button to turn the light on or off.
   - Adjust brightness and temperature with sliders.
   - All changes are sent instantly to the device.
3. **Auto-Start**  
   Enable "Start with Windows" to launch the app automatically.
4. **Tray Icon**  
   Minimize to tray and restore the window by clicking the tray icon.

## Building

```sh
cargo build --release
```

## Running

```sh
cargo run --release
```

## Project Structure

```
src/
  main.rs         # Application entry point
  ui.rs           # egui UI logic
  settings.rs     # Registry settings load/save
  background.rs   # Background worker for polling/control
  utils/
    light.rs      # Elgato Key Light API logic
    icon.rs       # Icon loading and conversion
    camera.rs     # Camera access detection
assets/
  TrayIconLit.png
  TrayIconUnlit.png
```

## Elgato Key Light API

- Communicates with the `/elgato/lights` endpoint.
- Example request:
  ```json
  {
    "numberOfLights": 1,
    "lights": [
      {
        "on": 1,
        "brightness": 50,
        "temperature": 200
      }
    ]
  }
  ```
- Temperature is mapped between API value (143–344) and Kelvin (2900–7000K).
  - 2900K = 344, 7000K = 143 (the mapping is reversed compared to Kelvin)

## License

MIT

---

*This project is not affiliated with Elgato or Corsair.*