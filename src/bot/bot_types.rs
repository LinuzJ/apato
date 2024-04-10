#[derive(Debug, PartialEq, Eq)]
pub struct Subscription {
    pub chat_id: i64,
    pub location: String,
    pub target_yield: u32,
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionArgs {
    pub location: String,
    pub target_yield: Option<u32>,
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
}
