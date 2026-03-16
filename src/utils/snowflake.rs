use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::CONFIG;

const EPOCH: u64 = 1_735_689_600_000;
const MACHINE_BITS: u64 = 10;
const SEQUENCE_BITS: u64 = 12;
const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock went backwards")
        .as_millis() as u64
}

pub fn next_id() -> i64 {
    let machine_id = CONFIG.machine_id as u64 & ((1 << MACHINE_BITS) - 1);

    loop {
        let ts = now_ms().saturating_sub(EPOCH);
        let last = LAST_TIMESTAMP.load(Ordering::SeqCst);

        if ts < last {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        let seq = if ts == last {
            let s = SEQUENCE.fetch_add(1, Ordering::SeqCst) & MAX_SEQUENCE;
            if s == 0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            s
        } else {
            LAST_TIMESTAMP.store(ts, Ordering::SeqCst);
            SEQUENCE.store(0, Ordering::SeqCst);
            0
        };

        let id = (ts << (MACHINE_BITS + SEQUENCE_BITS)) | (machine_id << SEQUENCE_BITS) | seq;
        return id as i64;
    }
}
