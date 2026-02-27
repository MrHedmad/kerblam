#[derive(Debug, Clone)]
pub enum ExecutionStrategy {
    Make,
    Shell,
}

impl Copy for ExecutionStrategy {}

impl ExecutionStrategy {
    pub fn to_command_vec(self) {
        todo!();
    }
}
