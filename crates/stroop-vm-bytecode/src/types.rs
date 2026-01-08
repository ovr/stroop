/// Represents the four numeric value types supported by the bytecode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ValueType {
    /// 32-bit integer
    I32 = 0x7F,
    /// 64-bit integer
    I64 = 0x7E,
    /// 32-bit floating point
    F32 = 0x7D,
    /// 64-bit floating point
    F64 = 0x7C,
}

impl ValueType {
    /// Returns the size in bytes for this value type.
    pub const fn size_bytes(self) -> usize {
        match self {
            ValueType::I32 | ValueType::F32 => 4,
            ValueType::I64 | ValueType::F64 => 8,
        }
    }

    /// Returns true if this is an integer type.
    pub const fn is_integer(self) -> bool {
        matches!(self, ValueType::I32 | ValueType::I64)
    }

    /// Returns true if this is a floating-point type.
    pub const fn is_float(self) -> bool {
        matches!(self, ValueType::F32 | ValueType::F64)
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::I32 => write!(f, "i32"),
            ValueType::I64 => write!(f, "i64"),
            ValueType::F32 => write!(f, "f32"),
            ValueType::F64 => write!(f, "f64"),
        }
    }
}

/// Value stored in the constant pool (only 64-bit types).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstPoolValue {
    I64(i64),
    F64(f64),
}

impl Eq for ConstPoolValue {}

impl std::hash::Hash for ConstPoolValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            ConstPoolValue::I64(v) => v.hash(state),
            ConstPoolValue::F64(v) => v.to_bits().hash(state),
        }
    }
}

impl ConstPoolValue {
    /// Get as i64, panics if wrong type.
    pub fn as_i64(&self) -> i64 {
        match self {
            ConstPoolValue::I64(v) => *v,
            _ => panic!("expected i64 constant"),
        }
    }

    /// Get as f64, panics if wrong type.
    pub fn as_f64(&self) -> f64 {
        match self {
            ConstPoolValue::F64(v) => *v,
            _ => panic!("expected f64 constant"),
        }
    }
}

/// Function signature with parameter and result types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl FuncType {
    /// Create a new function type with no parameters or results.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a function type with the given parameters and results.
    pub fn with_params_results(params: Vec<ValueType>, results: Vec<ValueType>) -> Self {
        Self { params, results }
    }
}

impl std::fmt::Display for FuncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(func")?;
        if !self.params.is_empty() {
            write!(f, " (param")?;
            for p in &self.params {
                write!(f, " {}", p)?;
            }
            write!(f, ")")?;
        }
        if !self.results.is_empty() {
            write!(f, " (result")?;
            for r in &self.results {
                write!(f, " {}", r)?;
            }
            write!(f, ")")?;
        }
        write!(f, ")")
    }
}
