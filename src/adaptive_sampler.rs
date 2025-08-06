// adaptive_sampler.rs
#![allow(dead_code)]

pub struct AdaptiveSampler {
    max_sampling_rate: f64,
    min_sampling_rate: f64,
    base_safety_factor: f64,       // 基础安全系数
    min_safety_factor: f64,        // 最小安全系数
    max_safety_factor: f64,        // 最大安全系数
    freq_threshold_low: f64,       // 低频阈值
    freq_threshold_high: f64,      // 高频阈值
}

impl AdaptiveSampler {
    /// 创建新的自适应采样器
    ///
    /// # 参数
    /// - `max_sampling_rate`: 最大允许采样率 (Hz)
    /// - `min_sampling_rate`: 最小允许采样率 (Hz)
    pub fn new(max_sampling_rate: f64, min_sampling_rate: f64) -> Self {
        Self {
            max_sampling_rate,
            min_sampling_rate,
            base_safety_factor: 10.0,         // 基础安全系数
            min_safety_factor: 4.0,           // 最小安全系数
            max_safety_factor: 50.0,          // 最大安全系数
            freq_threshold_low: 1.0,          // 低频阈值(Hz)
            freq_threshold_high: 100.0,       // 高频阈值(Hz)
        }
    }

    /// 根据信号频率计算采样率（无状态版本）
    ///
    /// # 参数
    /// - `signal_freq`: 当前信号频率 (Hz)
    ///
    /// # 返回
    /// 计算出的采样率 (Hz)
    pub fn compute_sampling_rate(&self, signal_freq: f64) -> f64 {
        // 根据频率范围动态调整安全系数
        let dynamic_safety = self.calculate_dynamic_safety(signal_freq);

        // 计算基础目标采样率
        let mut target_sr = signal_freq * dynamic_safety;

        // 确保满足奈奎斯特采样定理
        target_sr = target_sr.max(signal_freq * 2.0);

        // 限制在硬件范围内
        target_sr.clamp(self.min_sampling_rate, self.max_sampling_rate)
    }

    /// 根据信号频率计算动态安全系数
    fn calculate_dynamic_safety(&self, signal_freq: f64) -> f64 {
        // 低频区域：使用更高的安全系数保证波形精度
        if signal_freq < self.freq_threshold_low {
            // 频率越低，安全系数越高（对数尺度）
            let ratio = (self.freq_threshold_low / signal_freq.max(0.01)).ln().clamp(1.0, 5.0);
            return self.base_safety_factor * ratio.clamp(1.5, 3.0);
        }
        // 高频区域：使用较低的安全系数节省资源
        else if signal_freq > self.freq_threshold_high {
            // 频率越高，安全系数越低（但保持最小安全系数）
            let ratio = (signal_freq / self.freq_threshold_high).sqrt().clamp(1.0, 3.0);
            return (self.base_safety_factor / ratio).max(self.min_safety_factor);
        }
        // 中频区域：使用基础安全系数
        self.base_safety_factor
    }

    /// 设置基础安全系数
    pub fn set_base_safety_factor(&mut self, factor: f64) {
        self.base_safety_factor = factor.clamp(self.min_safety_factor, self.max_safety_factor);
    }

    /// 设置频率阈值
    pub fn set_frequency_thresholds(&mut self, low: f64, high: f64) {
        self.freq_threshold_low = low.max(0.01);
        self.freq_threshold_high = high.max(low * 2.0);
    }
}

pub fn initialize() {
    log::debug!("初始化自适应采样器");
}