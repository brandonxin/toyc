use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum TypeExpr {
    Void,
    // Bool,                       // unimplemented
    Int64,
    // StringLiteral,              // unimplemented
    Pointer(Rc<TypeExpr>),
    // Slice(Rc<TypeExpr>),        // unimplemented
    // Array(Rc<TypeExpr>, usize), // unimplemented
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_expr() {
        let int64 = Rc::new(TypeExpr::Int64);
        let pointer = TypeExpr::Pointer(int64.clone());
        let pointer2 = TypeExpr::Pointer(Rc::new(TypeExpr::Int64));
        assert_eq!(int64, Rc::new(TypeExpr::Int64));
        assert_eq!(pointer, TypeExpr::Pointer(Rc::new(TypeExpr::Int64)));
        assert_eq!(pointer, pointer2);
    }
}
