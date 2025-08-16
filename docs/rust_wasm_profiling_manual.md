# Manual: Profiling Rust/WebAssembly Apps in the Browser

This guide walks you through how to **profile Rust applications compiled to WebAssembly (WASM)** when running inside a web browser.  
It focuses on **Chrome**, but includes short notes for Firefox and Safari.

---

## 1) Building for Profiling (Keeping Debug Info)

When profiling, your goal is to **preserve symbol names and debug information** so the profiler can map runtime activity back to your source code.

In your `Cargo.toml`, create a dedicated profile for profiling builds:

```toml
[profile.profiling]
inherits = "release"
opt-level = 2        # Reasonable optimization for realistic performance
debug = 2            # Preserve DWARF info and function names
strip = "none"       # Do NOT strip symbols
lto = "off"          # Turn off link-time optimization (easier debugging)
```

### Why this matters
If you strip symbols or compile without debug info, your flame charts in DevTools will show entries like `(wasm-function #42)` instead of real function names. This makes the profile far less useful.

### Building
Depending on your toolchain:

**wasm-bindgen flow:**
```bash
cargo build --target wasm32-unknown-unknown --profile profiling
wasm-bindgen --target web --out-dir dist target/wasm32-unknown-unknown/profiling/*.wasm
```

**wasm-pack:**
```bash
wasm-pack build --target web --profiling
# To disable wasm-opt passes entirely (avoiding symbol stripping):
WASM_PACK_NO_OPT=1 wasm-pack build --target web --profiling
```

**Trunk (e.g., Yew apps):**
```bash
trunk serve --release -- --profile profiling
```
In your `index.html` you can also add:
```html
<link data-trunk rel="rust" data-wasm-opt="0" />
```
This prevents extra optimization passes during profiling.

---

## 2) Profiling with Chrome DevTools

Chrome DevTools provides two main panels useful for WASM profiling: **Performance** and **Memory**.

### CPU & Timeline Profiling
1. Open your application in Chrome.
2. Go to **DevTools → Performance**.
3. Click **Record**, perform the action you want to profile, then **Stop**.
4. Look at the **Flame chart** (top-down view of function calls over time).
5. Use the **Bottom-Up** tab to see which functions consume the most self time.

With a profiling build, you’ll see actual Rust function names (translated through wasm-bindgen) instead of anonymous IDs.

### Memory Profiling
- Open the **Memory** panel.
- Use **Allocation sampling** to observe where allocations occur on the JavaScript side.
- WASM’s linear memory is visible as an `ArrayBuffer`. You can track its growth over time by comparing heap snapshots before and after certain operations.

---

## 3) Deep Tracing with Perfetto

For performance issues that aren’t obvious from CPU usage (e.g., scheduling delays, frame drops, audio glitches), use **Perfetto**, Chrome’s advanced tracing tool.

Steps:
1. Go to [ui.perfetto.dev](https://ui.perfetto.dev).
2. Click **Record new trace** → **Target: Chrome**.
3. Enable categories: **Scheduler**, **WebAssembly**, **V8**, **WebAudio** (if relevant), **GPU**.
4. Record while reproducing the performance problem.
5. Analyze Main, Worker, AudioWorklet, and GPU threads to find stalls, long tasks, or missed deadlines.

Perfetto traces are especially useful for diagnosing multi-threaded WASM and real-time audio/graphics workloads.

---

## 4) Adding Instrumentation from Rust

You can add **User Timing marks** and `console.time` calls from your Rust code to make specific code sections stand out in the profiler.

Example:
```rust
use wasm_bindgen::prelude::*;
use web_sys::{window, console};

#[wasm_bindgen]
pub fn profiled<F: FnOnce()>(work: F) {
    let perf = window().unwrap().performance().unwrap();
    perf.mark("hot-start").ok();
    console::time_with_label("hot");

    work(); // Your workload

    console::time_end_with_label("hot");
    perf.mark("hot-end").ok();
    perf.measure_with_start_mark_and_end_mark("hot-section", "hot-start", "hot-end").ok();
}
```

- **User Timing marks** appear in DevTools’ **Performance** panel under the **Timings** track.
- **console.time** measurements appear in the JavaScript console.

This helps correlate specific Rust functions with visual markers in the flame chart.

---

## 5) Threads and Environment Setup

If your Rust/WASM build uses **threads** (e.g., via `wasm32-unknown-unknown` + `std::thread`), the browser requires **cross-origin isolation** for `SharedArrayBuffer` to work.

Set HTTP headers on your server:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

Without these headers, threading will be disabled, and your profiling results may not match production behavior.

---

## 6) Minimal Profiling Checklist

- [ ] Build with `--profile profiling` (or `wasm-pack --profiling`) and **do not strip symbols**.
- [ ] In Chrome, record with the **Performance** panel.
- [ ] Sort the **Bottom-Up** view by **Self Time** to find hotspots.
- [ ] Add **User Timing** marks to key code sections.
- [ ] If CPU usage looks fine but performance is bad, capture a **Perfetto trace**.
- [ ] Inspect memory growth using the **Memory** panel and heap snapshots.
- [ ] Match test conditions to production (same data sizes, threads, features).

---

## 7) Notes for Other Browsers

**Firefox**  
- Use `about:profiling` to start a profile.
- Analyze at [profiler.firefox.com](https://profiler.firefox.com).  
Firefox’s profiler is very powerful and can handle long-duration captures well.

**Safari (macOS/iOS)**  
- Enable the **Develop** menu (Preferences → Advanced → “Show Develop menu in menu bar”).
- On iOS, enable remote inspection in Settings → Safari → Advanced.
- Use Web Inspector → **Timelines** & **JavaScript & Memory** instruments.

---

## 8) Common Pitfalls

- **Missing function names:** You stripped debug info or symbols; rebuild as in Section 1.
- **Unrealistic hotspots:** Low optimization levels (`opt-level=0`) may change performance behavior. Use `opt-level=2` with debug info for profiling.
- **Audio issues not visible in CPU profile:** Use Perfetto with WebAudio categories and inspect the AudioWorklet thread.
- **Unexpected wasm-opt stripping:** Disable automatic wasm-opt passes during profiling.

---

By following these steps, you’ll get **clear, actionable profiles** of your Rust/WASM code running in the browser, helping you find and fix performance bottlenecks efficiently.
