use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum TypeSpecifier {
    Void,
    // Bool,                       // unimplemented
    Int64,
    // StringLiteral,              // unimplemented
    Pointer(Rc<TypeSpecifier>),
    // Slice(Rc<TypeExpr>),        // unimplemented
    // Array(Rc<TypeExpr>, usize), // unimplemented
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let int64 = Rc::new(TypeSpecifier::Int64);
        let pointer = TypeSpecifier::Pointer(int64.clone());
        let pointer2 = TypeSpecifier::Pointer(Rc::new(TypeSpecifier::Int64));
        assert_eq!(int64, Rc::new(TypeSpecifier::Int64));
        assert_eq!(
            pointer,
            TypeSpecifier::Pointer(Rc::new(TypeSpecifier::Int64))
        );
        assert_eq!(pointer, pointer2);
    }
}
