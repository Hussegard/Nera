//Any errors related to loading and/or interpretting the policy file
// The rule apsects/Engine in Nera
#[derive(Debug, thiserror::Error)]
pub enum PolicyError{
    //any file system error involved with policy.toml (Reading, Opening, or Writing)
    //inner io::Error handles the details of which file system kind of Error it is
    #[error("io error with policy file: {0}")]
    IoPolicy(#[from] std::io::Error),

    //Reading file was successful however parsing as TOML failed most likely due to syntax 
    #[error("the toml file is malformed")]
    InvalidToml(#[from] toml::de::Error),

    //Policy file is valid (syntax and file system-wise ), but the requested Agent ID doesnt have an entry
    #[error("no policy was defined for the agent: {0}")]
    NoDefinedPolicy(String),
}