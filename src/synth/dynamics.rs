const FIXED_LIMIT_THRESHOLD: f32 = 0.05;

pub struct Dynamics {}

impl Dynamics {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compress(&self, output_level: f32, threshold: f32, ratio: f32, sample: f32) -> f32 {
        let sample_dbfs = get_dbfs_from_f32_sample(sample);

        if sample_dbfs <= threshold || output_level <= threshold {
            return sample * self.get_makeup_gain(threshold * (1.0 - ratio), output_level);
        }

        let delta = sample_dbfs - threshold;
        let compressed_delta = delta * ratio;
        let new_dbfs = threshold + compressed_delta;
        let mut compressed_sample = get_f32_sample_from_dbfs(new_dbfs);
        compressed_sample *= compressed_sample.signum();

        compressed_sample * self.get_makeup_gain(threshold * (1.0 - ratio), output_level)
    }

    pub fn limit(&self, output_level: f32, threshold: f32, sample: f32) -> f32 {
        self.compress(output_level, threshold, FIXED_LIMIT_THRESHOLD, sample)
    }

    pub fn clip(&self, output_level: f32, threshold: f32, sample: f32) -> f32 {
        let sample_dbfs = get_dbfs_from_f32_sample(sample);

        if sample_dbfs <= threshold || output_level <= threshold {
            return sample * self.get_makeup_gain(threshold, output_level);
        }

        let mut clipped_sample = get_f32_sample_from_dbfs(threshold);
        clipped_sample *= clipped_sample.signum();

        clipped_sample * self.get_makeup_gain(threshold, output_level)
    }

    pub fn wave_fold(&self, output_level: f32, threshold: f32, ratio: f32, sample: f32) -> f32 {
        let sample_dbfs = get_dbfs_from_f32_sample(sample);

        if sample_dbfs <= threshold || output_level <= threshold {
            return sample * self.get_makeup_gain(threshold, output_level);
        }

        let delta = sample_dbfs - threshold;
        let compressed_delta = delta * ratio;
        let new_dbfs = threshold - compressed_delta;
        let mut compressed_sample = get_f32_sample_from_dbfs(new_dbfs);
        compressed_sample *= compressed_sample.signum();

        compressed_sample * self.get_makeup_gain(threshold, output_level)
    }

    pub fn get_makeup_gain(&self, threshold: f32, output_level: f32) -> f32 {
        if output_level <= threshold {
            return 1.0;
        }
        get_f32_sample_from_dbfs(output_level - threshold)
    }
}

pub fn get_dbfs_from_f32_sample(sample: f32) -> f32 {
    let sample_absolute_value = sample.abs();

    if sample_absolute_value <= f32::EPSILON {
        return f32::NEG_INFINITY;
    }

    20.0 * sample_absolute_value.log10()
}

pub fn get_f32_sample_from_dbfs(dbfs: f32) -> f32 {
    10.0_f32.powf(dbfs / 20.0)
}
