```python
from collections import deque
import math
from typing import Iterable, Optional, List

class AdaptiveEMA:
   """
   Adaptive EMA that:
     - Smooths small jitter strongly
     - Stays responsive on larger moves
   Optional pre-steps:
     - median-of-3 prefilter (cheap jitter killer)
     - Hampel outlier suppression (removes transient spikes)

   Parameters
   ----------
   alpha_min : float
       Lower bound on EMA factor (0<alpha_min<1). Small value => strong smoothing.
   alpha_max : float
       Upper bound on EMA factor (alpha_min<=alpha_max<1). Large value => snappier on big moves.
   d : float
       Soft threshold for "jitter size" in the sigmoid mapping (typical noise scale).
   s : float
       Softness of the transition (smaller => steeper).
   use_median3 : bool
       If True, applies median-of-3 on the input stream before filtering.
   use_hampel : bool
       If True, applies Hampel outlier suppression before filtering.
   hampel_window : int
       Odd window size for the Hampel filter (e.g., 5, 7). Used causally (past-only).
   hampel_nsigma : float
       Sensitivity for Hampel; larger => fewer points flagged as outliers.
   deadband : Optional[float]
       If set, when |x - y_prev| < deadband, force alpha_t = alpha_min (extra smoothing near stillness).
   hysteresis : Optional[tuple[float, float]]
       (d_down, d_up) two-threshold hysteresis. If provided, it will adjust the effective 'd'
       depending on whether we're in "quiet" or "moving" state to reduce flicker.

   Usage
   -----
   # Streaming:
   f = AdaptiveEMA(alpha_min=0.02, alpha_max=0.6, d=0.5, s=0.15,
                   use_median3=True, use_hampel=True, hampel_window=7, hampel_nsigma=3.0)
   for x in stream:
       y = f.update(x)

   # Batch:
   ys = f.filter_series(xs)
   """

   def __init__(
       self,
       alpha_min: float = 0.02,
       alpha_max: float = 0.6,
       d: float = 0.5,
       s: float = 0.15,
       *,
       use_median3: bool = False,
       use_hampel: bool = False,
       hampel_window: int = 7,
       hampel_nsigma: float = 3.0,
       deadband: Optional[float] = None,
       hysteresis: Optional[tuple] = None,
   ):
       assert 0.0 < alpha_min <= alpha_max < 1.0
       assert d > 0.0 and s > 0.0
       if use_hampel:
           assert hampel_window >= 3 and hampel_window % 2 == 1

       self.alpha_min = float(alpha_min)
       self.alpha_max = float(alpha_max)
       self.d_base = float(d)
       self.s = float(s)

       self.use_median3 = use_median3
       self.m3_buf = deque(maxlen=3) if use_median3 else None

       self.use_hampel = use_hampel
       self.hampel_window = hampel_window
       self.hampel_nsigma = float(hampel_nsigma)
       self.h_buf = deque(maxlen=hampel_window) if use_hampel else None

       self.deadband = deadband
       self.hysteresis = hysteresis
       self._mode = "quiet"  # used if hysteresis is enabled

       self._y = None  # last filtered value

   @staticmethod
   def _median(vals: Iterable[float]) -> float:
       vals = sorted(vals)
       n = len(vals)
       mid = n // 2
       if n % 2:
           return vals[mid]
       return 0.5 * (vals[mid - 1] + vals[mid])

   @staticmethod
   def _mad(vals: List[float], med: float) -> float:
       # Median Absolute Deviation
       return AdaptiveEMA._median([abs(v - med) for v in vals])

   def _median3(self, x: float) -> float:
       self.m3_buf.append(x)
       if len(self.m3_buf) < 3:
           return x
       return self._median(self.m3_buf)

   def _hampel(self, x: float) -> float:
       # Causal (uses past-only window)
       self.h_buf.append(x)
       vals = list(self.h_buf)
       med = self._median(vals)
       mad = self._mad(vals, med)
       # 1.4826 factor makes MAD ~ std for normal dist
       sigma = 1.4826 * mad if mad > 0 else 0.0
       if sigma == 0:
           # Not enough variation yet; don't flag
           return x
       if abs(x - med) > self.hampel_nsigma * sigma:
           # Replace outlier with median (or clipped)
           return med
       return x

   def _compute_alpha(self, x: float, y_prev: float) -> float:
       delta = abs(x - y_prev)

       # Optional deadband override
       if self.deadband is not None and delta < self.deadband:
           return self.alpha_min

       # Optional hysteresis by shifting the effective threshold d
       d_eff = self.d_base
       if self.hysteresis is not None:
           d_down, d_up = self.hysteresis
           # Update mode first based on current delta
           if self._mode == "quiet" and delta > d_up:
               self._mode = "moving"
           elif self._mode == "moving" and delta < d_down:
               self._mode = "quiet"

           # Bias 'd' depending on mode
           d_eff = d_down if self._mode == "quiet" else d_up

       # Sigmoid mapping from delta -> alpha
       u = (delta - d_eff) / self.s
       sig = 1.0 / (1.0 + math.exp(-u))
       a = self.alpha_min + (self.alpha_max - self.alpha_min) * sig
       # Safety clamp
       return max(self.alpha_min, min(self.alpha_max, a))

   def update(self, x: float) -> float:
       """Process a single sample and return the filtered value."""
       # Optional prefilters
       if self.use_median3:
           x = self._median3(x)
       if self.use_hampel:
           x = self._hampel(x)

       if self._y is None:
           self._y = x
           return x

       a = self._compute_alpha(x, self._y)
       self._y = (1.0 - a) * self._y + a * x
       return self._y

   def filter_series(self, xs: Iterable[float]) -> List[float]:
       """Filter a sequence (batch mode). Resets internal state."""
       # reset state for deterministic batch runs
       self._y = None
       if self.use_median3 and self.m3_buf is not None:
           self.m3_buf.clear()
       if self.use_hampel and self.h_buf is not None:
           self.h_buf.clear()
       self._mode = "quiet"

       ys = []
       for x in xs:
           ys.append(self.update(x))
       return ys


# -----------------------------
# Minimal example & defaults
# -----------------------------
if __name__ == "__main__":
   import random

   # Create a synthetic signal: piecewise steps + jitter + occasional spikes
   xs = []
   val = 0.0
   for t in range(300):
       if t == 100: val = 5.0
       if t == 200: val = 2.5
       # jitter
       x = val + 0.15 * (random.random() - 0.5)
       # rare spike
       if t in (60, 140, 220):
           x += 3.0 * (1 if random.random() < 0.5 else -1)
       xs.append(x)

   f = AdaptiveEMA(
       alpha_min=0.02, alpha_max=0.6,
       d=0.3, s=0.1,
       use_median3=True,
       use_hampel=True, hampel_window=7, hampel_nsigma=3.0,
       deadband=0.05,
       hysteresis=(0.25, 0.45)  # quiet/moving thresholds
   )
   ys = f.filter_series(xs)

   # Optional: quick ASCII peek (replace with a real plot in your environment)
   for i in range(0, len(xs), 25):
       print(f"{i:03d}: x={xs[i]:6.3f} -> y={ys[i]:6.3f}")
```

#### Tuning tips
- Start with alpha_min=0.02, alpha_max=0.6, d ~ jitter amplitude, s = d/3.
- Turn on use_median3=True for cheap jitter reduction.
- For occasional spikes, add use_hampel=True with hampel_window=5.. nine and hampel_nsigma=3.0.
- If small “twitches” still sneak through, set deadband to a small value (e.g., 0.05).
- To avoid flicker near the threshold, use hysteresis=(d_down, d_up) with d_down < d_up.
