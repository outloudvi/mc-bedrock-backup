use clap::ValueEnum;

#[derive(PartialEq, ValueEnum, Clone, Debug)]
pub enum EngineMode {
    Bedrock,
    Java,
}
