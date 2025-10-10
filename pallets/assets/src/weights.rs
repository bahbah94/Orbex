use frame_support::weights::Weight;

pub trait WeightInfo {
    fn deposit() -> Weight;
    fn withdraw() -> Weight;
}

impl WeightInfo for () {
    fn deposit() -> Weight {
        // essentially this right now just intuitive. like 10k for validation, 25000 for reads and 100000 for writes( just an assumption, will change when benchmarks are done)
        Weight::from_parts(10_000,0)
            .saturating_add(Weight::from_parts(25_000,0))
            .saturating_add(Weight::from_parts(100_000,0))
    }

    fn withdraw() -> Weight {
        Weight::from_parts(10_000,0)
            .saturating_add(Weight::from_parts(25_000,0))
            .saturating_add(Weight::from_parts(100_000,0))
    }
}