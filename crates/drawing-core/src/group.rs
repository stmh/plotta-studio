//! Group - a collection of elements that transform together

use serde::{Deserialize, Serialize};

use crate::Element;

/// A group of elements that transform together
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Group {
    pub children: Vec<Element>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, element: Element) -> Self {
        self.children.push(element);
        self
    }

    pub fn push(&mut self, element: Element) {
        self.children.push(element);
    }

    pub fn extend(&mut self, elements: impl IntoIterator<Item = Element>) {
        self.children.extend(elements);
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}
