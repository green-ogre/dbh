## Winny Itch Io Template

1. Run this command to build the project:
```
wasm-pack build --release --target web
```
2. Copy the `pkg` directory, `index.html`, and any resources into a new folder.
3. Compress the new folder (.zip).
4. Upload to itch.
  - Check `This file will be played in the browser`.
  - Set `Viewport Dimensions` to the window size matching the `WindowPlugin`'s `window_size` field.
