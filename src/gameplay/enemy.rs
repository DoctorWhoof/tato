use crate::*;

#[derive(Default)]
pub enum AIState {
    #[default] Chasing,
    // Searching,
    // Shooting,
    // Stunned,
    // Dead
}

pub struct Enemy {
    pub id: EntityID,
    pub state: CharState,
    pub ai_state: AIState
}

