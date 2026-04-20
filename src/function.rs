use super::*;


/// The function is used to initialize the extension.
///
/// Returns [`None`] if no Extension Data is set.
/// 
pub type Initializer = fn () -> Option<Box<dyn Any>>;

/// The function receives ownership of the Extension Data,
/// allowing it to be saved or disposed.
/// 
/// The runtime does **NOT** guarantee that this function will be called before process termination.
/// 
pub type Finalizer = fn (ext_data: Option<Box<dyn Any>>);

/// The function is used to initialize a [`Context`].
///
/// The first return value sets the Context Data.
/// The second return value sets the methods associated with the [`Context`].
/// 
pub type ContextInitializer = fn (ctx: &CurrentContext) -> (Option<Box<dyn Any>>, FunctionSet);

/// The function is called before the Context Data is disposed,
/// allowing final access and saving data to the Extension Data.
///
pub type ContextFinalizer = fn (ctx: &CurrentContext);

/// The function can be associated with a [`Context`] and treated as its method.
/// 
/// This signature is intended to match the closure accepted by [`CurrentContext::with_method`].
///
/// Although the signature returns `as3::Object<'a>`, implementations may return
/// any type implementing `Into<as3::Object<'a>> + 'a`, primarily to support
/// types like `Option<AsObject<'a>>`.
/// 
pub type Function <'a> = fn (ctx: &CurrentContext<'a>, func_data: Option<&mut dyn Any>, args: &[as3::Object<'a>]) -> as3::Object<'a>;


/// **In typical usage of this crate, instances of this type should not be constructed directly.**
/// 
/// The [`crate::function!`] macro should be used to construct this type, as it provides a safer abstraction.
/// 
#[derive(Debug)]
pub struct FunctionImplementation {
    raw_name: UCStr,
    raw_func: FREFunction,
}
impl FunctionImplementation {
    pub fn raw_name(&self) -> &UCStr {&self.raw_name}
    pub fn raw_func(&self) -> FREFunction {self.raw_func}
    pub const fn new (raw_name: UCStr, raw_func: FREFunction) -> Self {
        Self { raw_name, raw_func }
    }
}


/// A collection of functions associated with a specific context type.
///
/// This type is used to construct a set of functions for a [`Context`].
/// Once registered, these functions are referred to as *methods* within this crate,
/// and can be invoked from ActionScript via `ExtensionContext.call`.
/// 
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

    /// Adds a function that can be registered as a method.
    ///
    /// - `name`: The method name. If [`None`], the raw name of `func_impl` is used.
    /// - `func_data`: Data associated with this method. It will be dropped after [`ContextFinalizer`] returns.
    /// - `func_impl`: The method implementation created by the [`crate::function!`] macro.
    /// 
    /// # Panics
    ///
    /// Panics if a function with the same `name` has already been added.
    ///
    /// Callers must ensure that each `name` is unique within this set. 
    /// 
    pub fn add (
        &mut self,
        name: Option<UCStr>,
        func_data: Option<Box<dyn Any>>,
        func_impl: &FunctionImplementation,
    ) {
        let name = name.unwrap_or(func_impl.raw_name.clone());
        let index = self.list.len();
        self.list.push(FRENamedFunction {
            name: name.as_ptr(),
            functionData: if let Some(func_data) = func_data {crate::data::into_raw(func_data).as_ptr()} else {FREData::default()},
            function: func_impl.raw_func,
        });
        let r = self.map.insert(name, index);
        assert!(r.is_none(), "Method name conflict.");
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