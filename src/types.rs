use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug)]
pub enum EngineMode {
    Bedrock,
    Java,
}
