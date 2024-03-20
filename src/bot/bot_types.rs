#[derive(Debug, PartialEq, Eq)]
pub struct Subscription {
    pub chat_id: i64,
    pub location: String,
    pub size: Option<u32>,
    pub yield_goal: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionArgs {
    pub location: String,
    pub size: Option<u32>,
    pub yield_goal: u32,
}
