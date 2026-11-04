#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthStatus {
    pub status: String,
}

pub struct HealthUseCase;

impl HealthUseCase {
    pub fn execute(&self) -> HealthStatus {
        HealthStatus {
            status: "ok".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_use_case_returns_ok() {
        let use_case = HealthUseCase;
        let result = use_case.execute();

        assert_eq!(result.status, "ok");
    }
}
