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
pub type ContextInitializer = fn (ctx: &mut CurrentContext) -> (Option<Box<dyn Any>>, FunctionSet);

/// The function is called before the Context Data is disposed,
/// allowing final access and saving data to the Extension Data.
///
pub type ContextFinalizer = fn (ctx: &mut CurrentContext);

/// The function can be associated with a [`Context`] and treated as its method.
///
/// Although the signature returns [`Object<'a>`], implementations may return
/// any type implementing [`Into<Object> + 'a`], primarily to support types like [`Option<AsObject<'a>>`].
/// 
pub type Function <'a> = fn (ctx: &mut CurrentContext<'a>, func_data: Option<&mut dyn Any>, args: &[Object<'a>]) -> Object<'a>;


/// Represents a native function implementation that can be registered as a method.
/// 
/// Can only be constructed through the [`function!`] macro.
/// 
#[derive(Debug)]
pub struct FunctionImplementation {
    name: UCStr,
    func: FREFunction,
}
impl FunctionImplementation {
    pub fn name(&self) -> &UCStr {&self.name}
    pub fn func(&self) -> FREFunction {self.func}
    pub(crate) const fn new (name: UCStr, func: FREFunction) -> Self {Self { name, func }}
}


/// A collection of functions associated with a specific context type.
///
/// This type is used to construct a set of functions for a [`Context`].
/// Once registered, these functions are referred to as *methods* within this crate,
/// and can be invoked from AS3 via `ExtensionContext.call`.
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
    /// # Parameters
    /// 
    /// - `name`: The method name. If [`None`], the raw name of `func_impl` is used.
    /// - `func_data`: Data associated with this method. It will be dropped after [`ContextFinalizer`] returns.
    /// - `func_impl`: The method implementation created by the [`function!`] macro.
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
        func_impl: &'static FunctionImplementation,
    ) {
        let name = name.unwrap_or(func_impl.name.clone());
        let index = self.list.len();
        self.list.push(FRENamedFunction {
            name: name.as_ptr(),
            functionData: if let Some(func_data) = func_data {crate::data::into_raw(func_data).as_ptr()} else {FREData::default()},
            function: func_impl.func,
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


/// Sends a message to the debugger output.
/// 
/// Delivery is not guaranteed; the `message` may not be presented.
/// 
/// # Panics
///
/// Panics if no [`CurrentContext`] exists. The call must occur within an
/// active native function call from the Flash runtime main thread.
/// 
/// # Examples
/// 
/// ```
/// use fre_rs::prelude::*;
/// fn func <'a> (_: CurrentContext<'a>, args: &[Object<'a>]) {
///     trace("Hello, Flash runtime!");
///     trace(args);
///     trace(args[0]);
/// }
/// ```
/// 
pub fn trace(message: impl ToUcstrLossy) {crate::context::stack::current_context().trace(message);}

