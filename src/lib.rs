//! This library generates snowflakes based off Twitter's snowflake format with some modifications.
//!
//! <https://github.com/twitter-archive/snowflake/tree/snowflake-2010>
//!
//! # Changes
//! * unsigned 128 bit integers are used
//! * atomic 16 bit counters are used to allow up to 65,536 IDs to be generated every millisecond
//! * response time must be less than 5 microseconds
//!
//! # Format
//! * Bits 0 to 63: milliseconds since the Ferris Epoch (01/01/2022 00:00:00.0000+00:00).
//! Range of around 600,000,000 years.
//! * Bits 64 to 71: the type of model (i.e. user, channel, guild)
//! * Bits 73 to 85: internal 16-bit atomic counter
//! * Bits 86 to 93: the API version this ID was generated with
//! * Bits 94 to 109: the node this ID was generated on
//! * Bits 110 to 127: unused
//!
//! # Crate Features
//! * `time-safety-checks`: checks that the system clock has not rolled back since the last
//! snowflake generated and if it has, blocks until the time is after the time of the last snowflake.
//! Adds a slight performance penalty but isn't that noticeable. Enabled by default.

use std::sync::atomic::{AtomicU16, Ordering};
#[cfg(feature = "time-safety-checks")]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

/// A internal atomic counter that helps guarantee snowflakes will be globally unique.
static INTERNAL_COUNTER: AtomicU16 = AtomicU16::new(0);
/// The start of the Ferris Epoch in milliseconds since the Unix Epoch
pub const FERRIS_EPOCH: u128 = 1_577_836_800_000;

#[cfg(feature = "time-safety-checks")]
/// A static variable to store the timestamp of the last snowflake generated.
static mut LAST_TIME_CREATED: u128 = 0;

/// Generates a snowflake from the current API version, the model type, and the node ID.
///
/// # Panics
/// Panics if the current time is behind the Unix Epoch.
///
/// # Examples
/// ```rust
/// use ferrischat_snowflake_generator::generate_snowflake;
/// assert_ne!(generate_snowflake::<0>(0, 0), generate_snowflake::<0>(0, 0));
/// ```
#[inline]
pub fn generate_snowflake<const API_VERSION: u8>(model_type: u8, node_id: u16) -> u128 {
    #[cfg(feature = "time-safety-checks")]
    let mut current_time = get_epoch_time();
    #[cfg(not(feature = "time-safety-checks"))]
    let current_time = get_epoch_time();

    #[cfg(feature = "time-safety-checks")]
    {
        // SAFETY: this is a variable we honestly do not care much about: if it's raced, we don't
        // have a issue whatsoever with that as long as the timestamp is not stored too late
        // which should not be possible because we update the timestamp after sleeping
        // to add to that, atomic u128s are not available on some platforms
        if current_time < unsafe { LAST_TIME_CREATED } {
            let sleep_for = unsafe { LAST_TIME_CREATED + 1 } - current_time;
            eprintln!(
                "detected system clock rolling back, not generating snowflakes for {}ms",
                sleep_for
            );
            std::thread::sleep(Duration::from_millis(sleep_for as u64));
        }
        current_time = get_epoch_time();
        unsafe {
            LAST_TIME_CREATED = current_time;
        }
    }
    (current_time << 64)
        + ((model_type as u128) << 56)
        // fetch_add wraps on overflow: this is what we want
        + ((INTERNAL_COUNTER.fetch_add(1, Ordering::Relaxed) as u128) << 42)
        + ((API_VERSION as u128) << 34)
        + ((node_id as u128) << 18)
}

/// Returns the current Ferris Epoch time.
///
/// Returns 0 if before the epoch, as well as at the first millisecond of the epoch.
///
/// # Panics
/// Panics if the current time is behind the Unix Epoch.
#[inline]
pub fn get_epoch_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("you're behind the Unix Epoch")
        .as_millis()
        .saturating_sub(FERRIS_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn unequal_snowflakes() {
        let snowflake_1 = generate_snowflake::<0>(0, u16::MAX);
        let snowflake_2 = generate_snowflake::<0>(0, u16::MAX);
        println!("{:b}, {:b}", &snowflake_1, snowflake_2);
        assert_ne!(snowflake_1, snowflake_2);
    }

    #[test]
    fn all_unequal_snowflakes() {
        // this code would panic until the current Ferris time reaches 0
        // since before that date this is designed exclusively for testing
        // and never to be used in production
        // to solve that we have a different way of doing things
        let max = if get_epoch_time() == 0 {
            u16::MAX as usize
        } else {
            1_000_000
        };
        let mut seen = HashSet::with_capacity(1_000_000);
        for _ in 1..max {
            let sf = generate_snowflake::<0>(0, 0);
            assert!(
                seen.insert(sf),
                "generated {} snowflakes at failure, snowflake was {}",
                seen.len(),
                sf
            );
        }
    }
}
