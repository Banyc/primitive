use std::collections::HashMap;

pub type Key = &'static str;

pub trait Stub: core::fmt::Debug {
    #[must_use]
    fn name(&self) -> Key;
    #[must_use]
    fn deps(&self) -> &[Key];
    fn build(&self, deps: &[&Box<dyn core::any::Any>]) -> anyhow::Result<Box<dyn core::any::Any>>;
}

/// # Example
///
/// ```rust
/// use primitive::dep_inj::{Key, Stub, DepAssembly};
///
/// let mut dep_asm = DepAssembly::new();
/// dep_asm.insert_stub(Box::new(ParentStub::default()));
/// dep_asm.insert_stub(Box::new(ChildStub::default()));
/// let parent = dep_asm.build(core::any::type_name::<Parent>()).unwrap();
/// let parent: &Parent = parent.downcast_ref().unwrap();
/// assert_eq!(parent.child.name, "child");
///
/// #[derive(Clone)]
/// struct Child {
///     pub name: String,
/// }
/// #[derive(Debug, Default)]
/// struct ChildStub {
///     deps: Vec<Key>,
/// }
/// impl Stub for ChildStub {
///     fn name(&self) -> Key {
///         core::any::type_name::<Child>()
///     }
///     fn deps(&self) -> &[Key] {
///         &self.deps
///     }
///     fn build(
///         &self,
///         deps: &[&Box<dyn core::any::Any>],
///     ) -> anyhow::Result<Box<dyn core::any::Any>> {
///         assert!(deps.is_empty());
///         Ok(Box::new(Child {
///             name: "child".into(),
///         }))
///     }
/// }
///
/// struct Parent {
///     pub child: Child,
/// }
/// #[derive(Debug)]
/// struct ParentStub {
///     deps: Vec<Key>,
/// }
/// impl Default for ParentStub {
///     fn default() -> Self {
///         Self {
///             deps: vec![core::any::type_name::<Child>()],
///         }
///     }
/// }
/// impl Stub for ParentStub {
///     fn name(&self) -> Key {
///         core::any::type_name::<Parent>()
///     }
///     fn deps(&self) -> &[Key] {
///         &self.deps
///     }
///     fn build(
///         &self,
///         deps: &[&Box<dyn core::any::Any>],
///     ) -> anyhow::Result<Box<dyn core::any::Any>> {
///         assert_eq!(deps.len(), 1);
///         let child: &Child = deps[0].downcast_ref().unwrap();
///         if child.name == "unknown" {
///             return Err(anyhow::anyhow!("child unknown"));
///         }
///         Ok(Box::new(Parent {
///             child: child.clone(),
///         }))
///     }
/// }
/// ```
#[derive(Debug)]
pub struct DepAssembly {
    stubs: HashMap<Key, Box<dyn Stub>>,
    deps: HashMap<Key, Box<dyn core::any::Any>>,
}
impl DepAssembly {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stubs: HashMap::new(),
            deps: HashMap::new(),
        }
    }

    pub fn insert_stub(&mut self, stub: Box<dyn Stub>) {
        self.stubs.insert(stub.name(), stub);
    }
    pub fn insert_dep(&mut self, name: Key, dep: Box<dyn core::any::Any>) {
        self.deps.insert(name, dep);
    }

    pub fn build(&mut self, name: Key) -> anyhow::Result<&Box<dyn core::any::Any>> {
        enum NextStep {
            AskDep,
            Build,
        }
        let mut stack = vec![(name, NextStep::AskDep)];
        while let Some((name, state)) = stack.pop() {
            if self.deps.contains_key(name) {
                continue;
            }
            match state {
                NextStep::AskDep => {
                    let Some(stub) = self.stubs.get(name) else {
                        return Err(anyhow::anyhow!("missing stub `{name}`"));
                    };
                    stack.push((name, NextStep::Build));
                    let children = stub.deps();
                    for child in children {
                        stack.push((child, NextStep::AskDep));
                    }
                }
                NextStep::Build => {
                    let stub = &self.stubs[name];
                    let children = stub.deps();
                    let mut deps_buf = vec![];
                    for child in children {
                        deps_buf.push(&self.deps[child]);
                    }
                    let dep = stub.build(&deps_buf)?;
                    self.deps.insert(name, dep);
                }
            }
        }
        Ok(&self.deps[name])
    }
}
impl Default for DepAssembly {
    fn default() -> Self {
        Self::new()
    }
}
