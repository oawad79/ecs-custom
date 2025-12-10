use crate::entity::Entity;
use std::any::TypeId;
use std::fmt;

#[derive(Debug, Clone)]
pub enum EcsError {
    EntityNotFound(Entity),
    ComponentNotFound(TypeId),
    ArchetypeNotFound(usize),
    InvalidOperation(String),
}

impl fmt::Display for EcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcsError::EntityNotFound(e) => write!(f, "Entity {:?} not found", e),
            EcsError::ComponentNotFound(t) => write!(f, "Component {:?} not found", t),
            EcsError::ArchetypeNotFound(a) => write!(f, "Archetype {} not found", a),
            EcsError::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
        }
    }
}

impl std::error::Error for EcsError {}

pub type Result<T> = std::result::Result<T, EcsError>;
