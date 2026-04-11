use super::*;


pub type Initializer = fn () -> Option<Box<dyn Any>>;
pub type Finalizer = fn (ext_data: Option<Box<dyn Any>>);
pub type ContextInitializer = fn (frt: &FlashRuntime) -> FunctionSet;
pub type ContextFinalizer = fn (frt: &FlashRuntime);
pub type Function <'a> = fn (frt: &FlashRuntime<'a>, func_data: Option<&mut dyn Any>, args: &[Object<'a>]) -> Object<'a>;


/// **In typical usage of this crate, instances of this type should not be constructed directly.**
/// 
/// The [`crate::function!`] macro should be used to construct this type, as it provides a safer abstraction.
#[derive(Debug)]
pub struct FunctionDefinition {
    raw_name: UCStr,
    raw_func: FREFunction,
}
impl FunctionDefinition {
    pub fn raw_name(&self) -> &UCStr {&self.raw_name}
    pub fn raw_func(&self) -> FREFunction {self.raw_func}
    pub const fn new (raw_name: UCStr, raw_func: FREFunction) -> Self {
        Self { raw_name, raw_func }
    }
}


/// A collection of functions associated with a specific context type.
///
/// This type is used to construct and register a set of functions for a [`Context`].
/// Once registered, these functions are referred to as *methods* within this crate,
/// and can be invoked from ActionScript via `ExtensionContext.call`.
#[derive(Debug)]
pub struct FunctionSet {
    list: Vec<FRENamedFunction>,
    map: HashMap<UCStr, usize>,
}
impl FunctionSet {
    pub fn new () -> Self {
        Self {
            list: Vec::new(),
            map: HashMap::new(),
        }
    }
    pub fn with_capacity (capacity: usize) -> Self{
        Self {
            list: Vec::with_capacity(capacity),
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Adds a named function entry to the internal registry.
    ///
    /// # Panics
    ///
    /// Panics if a function with the same `name` has already been added.
    ///
    /// Callers must ensure that each `name` is unique within this registry.
    pub fn add (
        &mut self,
        name: Option<UCStr>,
        func_data: Option<impl Data>,
        func: &FunctionDefinition,
    ) {
        let name = name.unwrap_or(func.raw_name.clone());
        let index = self.list.len();
        self.list.push(FRENamedFunction {
            name: name.as_ptr(),
            functionData: if let Some(func_data) = func_data {func_data.into_raw().as_ptr()} else {FREData::default()},
            function: func.raw_func,
        });
        let r = self.map.insert(name, index);
        assert!(r.is_none());
    }
}
impl Drop for FunctionSet {
    fn drop(&mut self) {
        self.list.iter()
            .map(|i| i.functionData)
            .for_each(|d| {
                if let Some(d) = NonNullFREData::new(d) {
                    unsafe {crate::data::drop_from(d)};
                }
            });
    }
}


#[derive(Debug)]
pub(super) struct MethodSet {
    registry: Box<[FRENamedFunction]>,
    dictionary: HashMap<UCStr, usize>,
}
impl MethodSet {
    /// ## Borrow
    pub(super) fn get(&self, name: &str) -> Option<(FREFunction, FREData)> {
        self.dictionary.get(name)
            .map(|index| {
                let i = &(self.registry[*index]);
                (i.function, i.functionData)
            })
    }
}
impl Drop for MethodSet {
    fn drop(&mut self) {
        self.registry.iter()
            .map(|i| i.functionData)
            .for_each(|d| {
                if let Some(d) = NonNullFREData::new(d) {
                    unsafe {crate::data::drop_from(d)};
                }
            });
    }
}
impl From<FunctionSet> for MethodSet {
    fn from(mut value: FunctionSet) -> Self {
        let registry = std::mem::take(&mut value.list).into_boxed_slice();
        let dictionary = std::mem::take(&mut value.map);
        Self { registry, dictionary }
    }
}
impl AsRef<[FRENamedFunction]> for MethodSet {
    fn as_ref(&self) -> &[FRENamedFunction] {self.registry.as_ref()}
}