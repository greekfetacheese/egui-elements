//! Tiny dependency-free PRNG used by the particle systems (module-private).

/// xorshift64* stream, seeded through splitmix64; any seed (incl. 0) is valid.
#[derive(Debug, Clone, Copy)]
pub(super) struct ParticleRng {
   state: u64,
}

impl ParticleRng {
   pub(super) fn new(seed: u64) -> Self {
      // splitmix64 finalizer to spread the seed; nudge away from the all-zero state
      let mut z = seed.wrapping_add(0x9E3779B97F4A7C15);
      z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
      z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
      z ^= z >> 31;
      Self {
         state: if z == 0 { 0x9E3779B97F4A7C15 } else { z },
      }
   }

   pub(super) fn next_u64(&mut self) -> u64 {
      let mut x = self.state;
      x ^= x >> 12;
      x ^= x << 25;
      x ^= x >> 27;
      self.state = x;
      x.wrapping_mul(0x2545F4914F6CDD1D)
   }

   /// Uniform in [0.0, 1.0).
   pub(super) fn f32(&mut self) -> f32 {
      ((self.next_u64() >> 40) as f32) / (1u64 << 24) as f32
   }

   /// Uniform in [min, max); returns `min` when `min >= max` (callers normally pre-normalize).
   pub(super) fn range(&mut self, min: f32, max: f32) -> f32 {
      if min >= max {
         min
      } else {
         min + self.f32() * (max - min)
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn same_seed_same_sequence() {
      let mut a = ParticleRng::new(42);
      let mut b = ParticleRng::new(42);
      for _ in 0..16 {
         assert_eq!(a.next_u64(), b.next_u64());
      }
   }

   #[test]
   fn different_seeds_differ() {
      let mut a = ParticleRng::new(1);
      let mut b = ParticleRng::new(2);
      let a_seq: Vec<u64> = (0..16).map(|_| a.next_u64()).collect();
      let b_seq: Vec<u64> = (0..16).map(|_| b.next_u64()).collect();
      assert_ne!(a_seq, b_seq);
   }

   #[test]
   fn range_stays_within_bounds() {
      let mut rng = ParticleRng::new(7);
      let mut spread = false;
      for _ in 0..1000 {
         let v = rng.range(0.3, 1.5);
         assert!(
            (0.3..1.5).contains(&v),
            "value {v} out of [0.3, 1.5)"
         );
         spread |= v != 0.3;
      }
      for _ in 0..1000 {
         let v = rng.range(-20.0, 20.0);
         assert!(
            (-20.0..20.0).contains(&v),
            "value {v} out of [-20.0, 20.0)"
         );
         spread |= v != -20.0;
      }
      assert!(
         spread,
         "range returned the lower bound every draw"
      );
   }

   #[test]
   fn seed_zero_does_not_lock() {
      let mut rng = ParticleRng::new(0);
      let outputs: Vec<u64> = (0..10).map(|_| rng.next_u64()).collect();
      let all_equal = outputs.iter().all(|v| *v == outputs[0]);
      assert!(!all_equal, "seed 0 produced a constant stream");
   }
}
