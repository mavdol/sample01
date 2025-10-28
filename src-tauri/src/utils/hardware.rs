use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppleChip {
    M1,
    M2,
    M3,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ChipVariant {
    Base,
    Pro,
    Max,
    Ultra,
}

fn detect_apple_chip(cpu_brand: &str) -> (AppleChip, ChipVariant) {
    let cpu_lower = cpu_brand.to_lowercase();

    let chip = if cpu_lower.contains("m3") {
        AppleChip::M3
    } else if cpu_lower.contains("m2") {
        AppleChip::M2
    } else if cpu_lower.contains("m1") {
        AppleChip::M1
    } else {
        AppleChip::Unknown
    };

    let variant = if cpu_lower.contains("ultra") {
        ChipVariant::Ultra
    } else if cpu_lower.contains("max") {
        ChipVariant::Max
    } else if cpu_lower.contains("pro") {
        ChipVariant::Pro
    } else {
        ChipVariant::Base
    };

    (chip, variant)
}

pub fn detect_optimal_gpu_layers() -> u32 {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            if let Ok(cpu_brand) = String::from_utf8(output.stdout) {
                let (chip, variant) = detect_apple_chip(&cpu_brand);

                let layers = match (chip, variant) {
                    (AppleChip::M3, ChipVariant::Ultra | ChipVariant::Max) => 99,
                    (AppleChip::M3, ChipVariant::Pro) => 60,
                    (AppleChip::M3, ChipVariant::Base) => 35,

                    (AppleChip::M2, ChipVariant::Ultra | ChipVariant::Max) => 80,
                    (AppleChip::M2, ChipVariant::Pro) => 50,
                    (AppleChip::M2, ChipVariant::Base) => 28,

                    (AppleChip::M1, ChipVariant::Ultra | ChipVariant::Max) => 65,
                    (AppleChip::M1, ChipVariant::Pro) => 45,
                    (AppleChip::M1, ChipVariant::Base) => 25,

                    (AppleChip::Unknown, _) => 20,
                };

                return layers;
            }
        }

        return 20;
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("nvidia-smi")
            .arg("--query-gpu=memory.total")
            .arg("--format=csv,noheader,nounits")
            .output()
        {
            if output.status.success() {
                if let Ok(vram_str) = String::from_utf8(output.stdout) {
                    if let Ok(vram_mb) = vram_str.trim().parse::<u32>() {
                        return match vram_mb {
                            0..=3999 => 8,
                            4000..=7999 => 20,
                            8000..=11999 => 28,
                            12000..=15999 => 35,
                            16000..=23999 => 45,
                            _ => 60,
                        };
                    }
                }
            }
        }

        return 12;
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("nvidia-smi")
            .arg("--query-gpu=memory.total")
            .arg("--format=csv,noheader,nounits")
            .output()
        {
            if output.status.success() {
                if let Ok(vram_str) = String::from_utf8(output.stdout) {
                    if let Ok(vram_mb) = vram_str.trim().parse::<u32>() {
                        return match vram_mb {
                            0..=3999 => 8,
                            4000..=7999 => 20,
                            8000..=11999 => 28,
                            12000..=15999 => 35,
                            16000..=23999 => 45,
                            _ => 60,
                        };
                    }
                }
            }
        }

        return 12;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        return 10;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_optimal_gpu_layers() {
        let layers = detect_optimal_gpu_layers();

        assert!(
            layers >= 8 && layers <= 99,
            "GPU layers should be between 8 and 99, got {}",
            layers
        );
    }
}
