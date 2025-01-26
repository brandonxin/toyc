use std::cell::RefCell;
use std::collections::HashMap;

use super::super::Value;

pub struct NestedScope<'m> {
    stack: RefCell<Vec<HashMap<String, &'m dyn Value>>>,
}

impl<'m> NestedScope<'m> {
    pub fn new() -> NestedScope<'m> {
        NestedScope {
            stack: RefCell::new(vec![HashMap::new()]),
        }
    }

    pub fn new_scope(&'m self) -> ScopeGuard<'m> {
        let mut stack = self.stack.borrow_mut();
        stack.push(HashMap::new());
        ScopeGuard::new(self)
    }

    pub fn lookup(&self, name: &str) -> Option<&'m dyn Value> {
        let stack = self.stack.borrow();
        for scope in stack.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(*val);
            }
        }
        None
    }

    pub fn update(&self, name: &str, val: &'m dyn Value) {
        let mut stack = self.stack.borrow_mut();
        stack.last_mut().unwrap().insert(String::from(name), val);
    }
}

pub struct ScopeGuard<'m> {
    scope: &'m NestedScope<'m>,
}

impl<'m> ScopeGuard<'m> {
    fn new(scope: &'m NestedScope<'m>) -> ScopeGuard<'m> {
        ScopeGuard { scope }
    }
}

impl Drop for ScopeGuard<'_> {
    fn drop(&mut self) {
        let mut stack = self.scope.stack.borrow_mut();
        stack.pop();
    }
}
