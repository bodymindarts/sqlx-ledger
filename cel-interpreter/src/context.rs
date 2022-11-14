use std::{collections::HashMap, rc::Rc};

use crate::{builtins, error::*, value::*};

type CelFunction = Box<dyn Fn(Vec<CelValue>) -> Result<CelValue, CelError>>;
pub struct CelContext {
    idents: HashMap<String, ContextItem>,
}

impl CelContext {
    pub fn new() -> Self {
        let mut idents = HashMap::new();
        idents.insert(
            "date".to_string(),
            ContextItem::Function(Box::new(builtins::date)),
        );
        idents.insert(
            "uuid".to_string(),
            ContextItem::Function(Box::new(builtins::uuid)),
        );
        Self { idents }
    }
}

pub(crate) enum ContextItem {
    Value(CelValue),
    Function(CelFunction),
}

impl CelContext {
    pub(crate) fn lookup(&self, name: Rc<String>) -> Result<&ContextItem, CelError> {
        self.idents
            .get(name.as_ref())
            .ok_or_else(|| CelError::UnknownIdent(name.clone()))
    }

    pub fn add_variable(&mut self, name: impl Into<String>, value: impl Into<CelValue>) {
        self.idents
            .insert(name.into(), ContextItem::Value(value.into()));
    }
}
