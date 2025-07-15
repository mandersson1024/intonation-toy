# Safe and Efficient Audio Data Transfer via postMessage in AudioWorklet

---

## ✅ Goal

Transfer audio data from the audio thread (`AudioWorkletProcessor`) to the main thread using `postMessage()` without blocking the audio thread or causing GC-related stutter.

---

## 🛠 Safe and Efficient Approach

### 1. ✅ Use `postMessage(..., [buffer])` with `Transferable`

Use a `Float32Array` and **transfer** its buffer to the main thread to avoid copying:

```js
const array = new Float32Array(128);
for (let i = 0; i < 128; i++) {
    array[i] = ...; // your processed audio sample
}
this.port.postMessage({ type: "audio", buffer: array.buffer }, [array.buffer]);
```

> ✅ Transferable objects (like `ArrayBuffer`) are moved across threads, not cloned — this avoids copying and reduces GC pressure.

---

### 2. ✅ Throttle messages to reduce overhead

Sending every 128-sample block (~344 messages/sec at 44.1kHz) may overload the main thread.

#### Batch multiple blocks

Instead of sending every frame, accumulate samples in a buffer and send once it's full. The size of the buffer is set when the worklet is constructed, for example 1024 bytes = 128 * 8

---

### 3. ✅ On the main thread: Create `Float32Array` from received buffer

```js
node.port.onmessage = (e) => {
    const audioBuffer = new Float32Array(e.data.buffer);
    // Use or copy the buffer for visualization/analysis
};
```

> 💡 Once transferred, the buffer is detached in the processor — do not reuse it there.

---

## ✅ Summary

| Task                        | Best Practice                                      |
|-----------------------------|----------------------------------------------------|
| Buffer Transfer             | Use `postMessage(..., [array.buffer])`             |
| Frequency                   | Throttle or batch to reduce load                  |
| Allocation                  | Reuse or preallocate buffers                      |
| GC Avoidance                | Don’t create fresh arrays every 128 samples       |
| Receiving Data              | Wrap buffer with `Float32Array` on main thread    |

---

This approach enables low-latency, real-time-safe audio data transfer from AudioWorklet to the main thread — perfect for waveform visualizations, pitch tracking, and more.