const FIXED_LIMIT_THRESHOLD: f32 = 0.05;

pub struct Dynamics {}

impl Dynamics {
    pub fn new() -> Self {
        Self {}
    }
    pub fn compress(&self, threshold: f32, ratio: f32, sample: f32) -> f32 {
        let sample_dbfs = get_dbfs_from_f32_sample(sample);

        if sample_dbfs < threshold {
            return sample;
        }

        let delta = sample_dbfs - threshold;
        let compressed_delta = delta * ratio;
        let new_dbfs = threshold + compressed_delta;
        get_f32_sample_from_dbfs(new_dbfs)
    }

    pub fn limit(&self, threshold: f32, sample: f32) -> f32 {
        self.compress(threshold, FIXED_LIMIT_THRESHOLD, sample)
    }

    pub fn clip(&self, threshold: f32, sample: f32) -> f32 {
        let sample_dbfs = get_dbfs_from_f32_sample(sample);

        if sample_dbfs < threshold {
            return sample;
        }

        get_f32_sample_from_dbfs(threshold)
    }
}

fn get_dbfs_from_f32_sample(sample: f32) -> f32 {
    let sample_absolute_value = sample.abs();

    if sample_absolute_value <= f32::EPSILON {
        return f32::NEG_INFINITY;
    }

    20.0 * sample_absolute_value.log10()
}

fn get_f32_sample_from_dbfs(dbfs: f32) -> f32 {
    10.0_f32.powf(dbfs / 20.0)
}
