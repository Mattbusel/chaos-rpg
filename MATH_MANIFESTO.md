# MATH MANIFESTO
## The 10 Sacred Algorithms of CHAOS RPG

*For nerds, masochists, and anyone who has stared at a bifurcation diagram at 3am and felt something.*

---

Every roll in CHAOS RPG passes through a randomly selected chain of 4–10 of these algorithms. The output of each becomes the input of the next. All algorithms map to `[-1, 1]`, but the chaining, the seed perturbations, and the unbounded game-value mappings produce the chaos you experience.

---

## 1. Lorenz Attractor

**The butterfly effect, implemented literally.**

The Lorenz system is a set of three coupled differential equations originally derived to model atmospheric convection:

```
dx/dt = σ(y − x)         σ = 10
dy/dt = x(ρ − z) − y     ρ = 28
dz/dt = xy − βz          β = 8/3
```

Edward Lorenz discovered in 1963 that changing his initial conditions by 0.000127 produced a completely different weather simulation. He called it the butterfly effect. The system has a "strange attractor" — a fractal structure in phase space that trajectories orbit forever without repeating.

**Why it's chaotic:** Two inputs that differ by 0.0001 diverge exponentially. The Lyapunov exponent is positive: λ₁ ≈ 0.9056. Every nearby trajectory separates.

**In the game:** When Lorenz Attractor appears in your chain, tiny differences in your stat values produce wildly different outcomes. Your +1 to STR might be the difference between doing 1 damage and 10,000. The butterfly flaps. The math decides.

---

## 2. Fourier Harmonic Series

**The interference pattern of fate.**

Joseph Fourier proved in 1822 that any periodic function can be decomposed into a sum of sinusoids. The implementation sums 8 harmonics:

```
f(x) = Σ (1/k) * sin(k·π·x + φₖ)   for k = 1..8
```

The phase offsets φₖ are derived from the seed: `φₖ = hash(seed × k) × 2π`. Different seeds produce completely different interference patterns — some produce constructive interference (huge values), others destructive (near-zero).

**Why it's chaotic:** When harmonics add in phase, the output spikes. When they cancel, it zeroes out. The transition between these states is sensitive to both the input frequency and the seed-derived phase shifts.

**In the game:** The Fourier engine can either amplify or annihilate the value feeding into it. Your legendary sword's +5000 damage might enter Fourier and emerge as essentially zero if the harmonics cancel. Or it might double. The physics of waves is now your combat mechanic.

---

## 3. Prime Density Sieve

**The irregularity of prime numbers as a chaos source.**

The Prime Number Theorem predicts that primes near N appear with density ~1/ln(N). But the actual distribution is irregular — sometimes primes cluster, sometimes they're sparse. The implementation:

1. Selects a window near a seed-derived value
2. Runs a mini Sieve of Eratosthenes
3. Compares actual prime density to Li(x) prediction
4. The deviation becomes the output

**Why it's chaotic:** Primes are deterministic but effectively irregular. The gaps between consecutive primes jump unpredictably — twin primes (3,5), (11,13) appear near prime deserts (23...29 gap = 6). The deviation from the smooth Li(x) approximation oscillates without pattern.

**In the game:** Your character's seed might land in a prime-rich region (+output) or a prime gap (-output). A sword found on floor 7 with seed 31337 might hit differently than one with seed 31338 because 31337 is prime and 31338 isn't.

---

## 4. Riemann Zeta Partial Sum

**The mysteries of the critical line.**

The Riemann Hypothesis concerns the zeros of the zeta function. The implementation evaluates the Dirichlet eta function on the critical line s = 0.5 + it:

```
η(s) = Σ (-1)^(n+1) / n^s   for n = 1..50
     = Σ n^(-0.5) · e^(-it·ln(n))
```

The parameter t is derived from `|input| × seed`. The output is the normalized imaginary part of the partial sum.

**Why it's chaotic:** The zeta function has nontrivial zeros all suspected to lie on the critical line Re(s) = 0.5. Between zeros the function oscillates with increasing frequency as t grows. The partial sums overshoot and undershoot the true value in complex patterns — the Gibbs phenomenon of the complex plane.

**In the game:** When t lands near a Riemann zero, the output becomes maximally uncertain. Your attack passing through this engine might be amplified by the zero's nearby oscillation — or zeroed out by it. The unsolved nature of the Riemann Hypothesis means even mathematicians can't fully predict this engine's behavior.

---

## 5. Fibonacci Golden Spiral

**The irrationality of φ as a deterministic wobble.**

The golden ratio φ = (1 + √5) / 2 ≈ 1.6180339887... is the "most irrational" number — its continued fraction [1; 1, 1, 1, ...] converges slowest of all. The implementation uses Binet's formula:

```
F(n) = (φⁿ − ψⁿ) / √5    where ψ = (1 − √5) / 2
```

Then maps through the golden angle (2π/φ²) spiral:

```
output = sin(x · φ · golden_angle + F(n) · 0.001) + φ-fractional-harmonic
```

**Why it's chaotic:** The golden angle (≈137.5°) is irrational, meaning no two steps in the spiral ever align. Sunflower seeds pack this way precisely because it avoids clustering. Input-output relationships follow the Fibonacci sequence's pattern — smooth locally, complex globally.

**In the game:** The Fibonacci engine produces outputs that are beautifully distributed but never periodic. Your stat value 60 and stat value 61 will produce outputs that bear no obvious relationship. The sunflower has decided your fate today.

---

## 6. Mandelbrot Escape Velocity

**The boundary of mathematical infinity.**

The Mandelbrot set is defined by iterating z_{n+1} = z² + c and checking if the orbit escapes to infinity. Points inside the set (|z| never exceeds 2) are in the set; points outside escape at some iteration count. The implementation uses smooth coloring:

```
smooth_iter = iter − ln(ln(|z|²)) / ln(2)
```

The starting point c is derived from `seed + input` near the Mandelbrot boundary (specifically near the Seahorse Valley at c ≈ −0.7269 + 0.1889i).

**Why it's chaotic:** The Mandelbrot boundary is a fractal of infinite complexity — zoom in arbitrarily far and new structures appear. Points infinitesimally close to the boundary have wildly different escape times. The smooth coloring formula makes this continuous, but the underlying structure is still infinitely complex.

**In the game:** Points inside the set (never escape) return negative values — cursed outcomes. Points near the boundary return high values — maximum chaos. Points far outside the set escape quickly — moderate, predictable outcomes. If your item rolls a Mandelbrot step, you want to be near the boundary but outside it. Inside the set is a bad day.

---

## 7. Logistic Map Bifurcation

**Period doubling to infinity.**

The logistic map x_{n+1} = r·x·(1−x) is the simplest nonlinear system to exhibit chaos. At r < 3.0: stable fixed point. At r = 3.0: period-2 bifurcation. At r ≈ 3.57: chaos onset. At r = 4.0: fully chaotic. The implementation:

```
r = 3.57 + (seed % 1000) / 1000 × 0.43     [in chaotic regime]
x₀ = |input|.fract()                         [initial condition]
iterate 60 times: burn-in then sample 10
```

The Feigenbaum constant δ ≈ 4.6692... governs the ratio of bifurcation intervals: each period-doubling happens at 1/δ the width of the previous. This ratio appears universally in all chaotic systems — it's a mathematical constant like π.

**Why it's chaotic:** Two starting values differing by 10⁻¹⁵ produce completely different orbits after ~50 iterations. Lyapunov exponent is positive for r > 3.57. The long-term behavior is deterministic but indistinguishable from random.

**In the game:** The logistic engine is the most "random-feeling" of all ten. Your character's precise stats determine r and x₀ to machine precision, but 60 iterations later the output is effectively random. This is the engine most likely to surprise you.

---

## 8. Euler's Totient Function

**The irregularity of multiplicative number theory.**

Euler's totient function φ(n) counts integers from 1 to n that share no common factor with n. Key properties:
- If p is prime: φ(p) = p − 1
- φ(12) = 4 (only 1, 5, 7, 11 are coprime to 12)
- φ(n)/n approaches 6/π² ≈ 0.6079 on average

The implementation computes φ(n) for a seed-derived n using the multiplicative formula:
```
φ(n) = n × Π (1 − 1/p)    for each prime p dividing n
```

Then returns the deviation from the average: `(φ(n)/n − 6/π²) / (6/π²)`

**Why it's chaotic:** The ratio φ(n)/n oscillates wildly. Primes give (p−1)/p ≈ 1.0. Highly composite numbers (with many small prime factors) give very low ratios. The sequence φ(1), φ(2), φ(3),... has no simple pattern. n=510510 (product of first 7 primes) gives φ/n ≈ 0.229 — a 60% deviation from average.

**In the game:** Whether your seed-derived n is prime (high ratio, positive deviation) or highly composite (low ratio, negative deviation) determines whether this engine helps or hurts you. This is why two characters with nearly identical seeds can have completely different outcomes.

---

## 9. Collatz Conjecture Chain

**The unsolved problem as a damage multiplier.**

The 3n+1 problem: start with any positive integer. If even, divide by 2. If odd, multiply by 3 and add 1. Repeat. The conjecture (unproven) is that all starting values eventually reach 1.

The implementation tracks stopping time for a seed-derived starting value n ∈ [3, 100003]. The most notorious case: n=27 takes 111 steps to reach 1, reaching a peak of 9232 — 342× its starting value.

**Why it's chaotic:** No pattern predicts stopping time. n=27 takes 111 steps; n=28 takes 18 steps. The altitude ratio (peak/start) varies from ~3 for simple cases to >300 for particularly cursed values. The problem appears in number theory, computational complexity, and the study of halting problems.

**In the game:** When this engine fires, your seed is mapped to a starting value. If it's one of the "long journey" numbers (like 27, 703, 871, ...), your chain length spikes — producing either huge positive or negative outputs depending on what came before. If it's a "short trip" number, the engine is relatively stable. The chaos is that you can't predict which without running the sequence.

---

## 10. Modular Exponentiation Hash

**The avalanche effect of modular arithmetic.**

Computes a^b mod m where:
- a is derived from input × seed
- b is derived from a Knuth multiplicative hash of seed
- m = 1,000,003 (a prime)

Fast exponentiation by repeated squaring:
```
result = a^b mod m    via binary exponentiation
```

**Why it's chaotic:** The discrete logarithm problem (recovering b from a^b mod m) is computationally hard — this is the basis of RSA cryptography. The output is essentially a cryptographic hash: smooth, continuous inputs map to pseudo-random outputs. Changing a by 1 changes the result unpredictably.

**In the game:** This is the mixing engine. It's the final chaos in a chain — taking whatever wild value came before and redistributing it across the output range in a way that makes prediction essentially impossible. When the Modular Exp Hash is the last engine in your chain, all bets are off.

---

## The Chain: How They Combine

When you roll for anything in the game, the pipeline:

1. Picks 4–8 engines (or all 10 for Destiny Rolls)
2. Seeds each with a deterministic perturbation of the run seed
3. Feeds the output of engine N as input to engine N+1
4. Maps the final [-1, 1] output to a game value

The compound effect means 4 engines creates chaos squared. 10 engines creates chaos to the 10th power. A Destiny Roll (all 10) on your character creation is why two players with seeds differing by 1 can get a Mortal and a Godlike.

The same input passes through 10 different chaotic transformations. The Lorenz butterfly flaps. The Fourier harmonics interfere. The Mandelbrot boundary bleeds through. The Collatz chain resolves. The modular hash finalizes.

This is your fate. It was always going to be this number.

---

## Seeded Runs

```bash
CHAOS_SEED=31337 cargo run --release
```

Same seed = same fate. Share seeds. Compare outcomes. Report particularly cursed or blessed runs.

The seed is the only thing you control. Everything else is math.

---

*"The most beautiful thing we can experience is the mysterious. It is the source of all true art and science." — Albert Einstein*

*He didn't have to fight THE HEAT DEATH on floor 47.*
