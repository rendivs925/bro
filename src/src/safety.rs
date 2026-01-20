use colored::*;
use anyhow::Result;

pub struct SafetyAssessment {
    pub blocked: bool,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl SafetyAssessment {
    pub fn new() -> Self {
        Self {
            blocked: false,
            reasons: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

pub fn assess_command(cmd: &str, ultra_safe: bool) -> SafetyAssessment {
    let mut assessment = SafetyAssessment::new();
    let lower = cmd.to_lowercase();

    // Absolute hard blocks
    if lower.contains("rm -rf /") || lower.contains("rm -rf /*") {
        assessment.blocked = true;
        assessment.reasons.push("Contains 'rm -rf /' which is catastrophic.".to_string());
    }

    if lower.contains("mkfs") {
        assessment.blocked = true;
        assessment.reasons.push("Contains 'mkfs' which can format disks.".to_string());
    }

    if lower.contains("dd if=") && (lower.contains("/dev/sd") || lower.contains("/dev/nvme")) {
        assessment.blocked = true;
        assessment.reasons.push("Contains 'dd' with a block device, potentially destructive.".to_string());
    }

    if lower.contains(">: /dev/sd") || lower.contains(">/dev/sd") || lower.contains(">/dev/nvme") {
        assessment.blocked = true;
        assessment
            .reasons
            .push("Redirecting output to a block device is destructive.".to_string());
    }

    if lower.contains("cryptsetup") {
        assessment.blocked = true;
        assessment
            .reasons
            .push("Contains 'cryptsetup', which can modify encrypted volumes.".to_string());
    }

    if ultra_safe && lower.contains("sudo") {
        assessment.blocked = true;
        assessment
            .reasons
            .push("Contains 'sudo' which is disallowed in ultra-safe mode.".to_string());
    }

    // Warnings
    if lower.contains("rm -rf") && !assessment.blocked {
        assessment
            .warnings
            .push("Uses 'rm -rf' which can be dangerous if misused.".to_string());
    }

    if lower.contains("chmod 777") {
        assessment
            .warnings
            .push("Uses 'chmod 777' which is usually unsafe on shared systems.".to_string());
    }

    if lower.contains("chown -r") {
        assessment
            .warnings
            .push("Uses 'chown -R' which can change many file owners recursively.".to_string());
    }

    assessment
}

pub fn print_assessment(assessment: &SafetyAssessment) {
    if !assessment.reasons.is_empty() {
        println!("
{}", "Blocked for safety:".red().bold());
        for r in &assessment.reasons {
            println!("  - {}", r.red());
        }
    }

    if !assessment.warnings.is_empty() {
        println!("
{}", "Warnings:".yellow().bold());
        for w in &assessment.warnings {
            println!("  - {}", w.yellow());
        }
    }
}

pub fn require_additional_confirmation(assessment: &SafetyAssessment) -> Result<bool> {
    if !assessment.warnings.is_empty() && !assessment.blocked {
        println!("
{}", "This command has warnings.".yellow().bold());
        println!("{}", "Type 'yes' to run anyway, anything else to cancel:".yellow());

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("yes") {
            Ok(true)
        } else {
            println!("{}", "Cancelled due to warnings.".red());
            Ok(false)
        }
    } else {
        Ok(true)
    }
}
