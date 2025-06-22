# Key Light Control

A Windows GUI application to control Elgato Key Lights via their local HTTP API, with **automatic camera detection** to turn lights on/off based on camera usage - a feature missing in the official app.
Built with [egui](https://github.com/emilk/egui), [eframe](https://github.com/emilk/egui/tree/master/crates/eframe), and [reqwest](https://github.com/seanmonstar/reqwest).

## Key Features

- **Automatic camera-based control**: Unlike the original Elgato application, this app automatically turns on lights when your camera is in use and turns them off when not - perfect for video calls and streaming!
- System tray icon for quick access and hiding the main window
- Toggle light on/off manually when needed
- Adjust brightness (0–100)
- Adjust color temperature (2900K–7000K, 50K steps)
- Auto-start with Windows option
- Periodic background polling and camera detection
- Settings saved in Windows registry

## Usage

1. **Configure IP and Port**  
   Enter your Elgato Key Light's IP address and port in the GUI.
2. **Control the Light**  
   - The light will automatically turn on when your camera is detected as active.
   - Use the toggle button to manually turn the light on or off when needed.
   - Adjust brightness and temperature with sliders.
   - All changes are sent instantly to the device.
3. **Auto-Start**  
   Enable "Start with Windows" to launch the app automatically.
4. **Tray Icon**  
   Minimize to tray and restore the window by clicking the tray icon.

## Download and Installation

1. **Download**  
   [Download the latest release](https://github.com/mikhail-zhadanov/key-light-control/releases/latest/download/key-light-control.zip)
2. **Installation**  
   - Simply unpack the ZIP file to any folder of your choice.
   - No formal installation required - just run the executable.

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