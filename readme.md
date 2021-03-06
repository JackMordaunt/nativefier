# nativefier

> Create native apps for your favourite websites.

Inspired by [jiahaog/nativefier](https://github.com/jiahaog/nativefier).

## Features  

- Tiny generated app size, `3MB` instead of `80MB`
- Native webviews (Webkit, MSHTML) instead of bundling a whole browser (looking at you, Electron)  
- MacOS (Windows and Linux planned)  
- Standalone binary which does not require a toolchain (unlike [jiahaog/nativefier](https://github.com/jiahaog/nativefier) which requires the `nodejs` toolchain)  

## Caveats  

- Compatibility with websites is dependent on the built-in webview for the OS  

## Structure

- `gui` contains the frontend code which is compiled to `wasm`.
- `webview` serves the `gui`, connecting it to `nativefier` core and the OS.
- `nativefier` is the core logic that produces the bundle.

## Key Components

- [x] Delineate between execution modes (bundle vs bundler).
- [x] Detect appropriate icon for website.
- [ ] Support common web icon formats.  
  - [x] png
  - [x] ico  
  - [ ] svg  
- [x] Support icon override.  
- [x] Replace dependency [`icns`](https://github.com/jackmordaunt/icns) with [`icns-rs`](https://github.com/jackmordaunt/icns-rs) for pure Rust goodness.  
- [ ] Create simple and elegant GUI (make `nativefier` accessible to those that can't use the command line).
- [ ] Integrate with chrome via the Chrome DevTools Protocol.
- [ ] Integrate with this [icon repository](https://github.com/jiahaog/nativefier-icons).  
- [ ] Inject JS/CSS for customisable experience.  

## ie11 GUI with asmjs-unknown-emscripten

- Needs polyfil for Math.random()
- It seems like my prepended js "runtime.js" is NOT being prepended/activated 
- ?? 