//! Runtime values for the Stroop VM.

use stroop_bytecode::ValueType;

/// A runtime value that can hold any of the supported types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    /// Returns the type of this value.
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
        }
    }

    /// Get as i32, panics if wrong type.
    pub fn as_i32(&self) -> i32 {
        match self {
            Value::I32(v) => *v,
            _ => panic!("expected i32, got {:?}", self.value_type()),
        }
    }

    /// Get as i64, panics if wrong type.
    pub fn as_i64(&self) -> i64 {
        match self {
            Value::I64(v) => *v,
            _ => panic!("expected i64, got {:?}", self.value_type()),
        }
    }

    /// Get as f32, panics if wrong type.
    pub fn as_f32(&self) -> f32 {
        match self {
            Value::F32(v) => *v,
            _ => panic!("expected f32, got {:?}", self.value_type()),
        }
    }

    /// Get as f64, panics if wrong type.
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::F64(v) => *v,
            _ => panic!("expected f64, got {:?}", self.value_type()),
        }
    }

    /// Try to get as i32.
    pub fn try_as_i32(&self) -> Option<i32> {
        match self {
            Value::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as i64.
    pub fn try_as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as f32.
    pub fn try_as_f32(&self) -> Option<f32> {
        match self {
            Value::F32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as f64.
    pub fn try_as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(v) => Some(*v),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
        }
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::I32(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::I64(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::F32(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::F64(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::I32(if v { 1 } else { 0 })
    }
}
