use fre_rs::prelude::*;


fre_rs::extension! {
    extern Initializer;
    gen init_ctx, final;
}


fn init_ctx (ctx: &CurrentContext) -> (Option<Box<dyn Any>>, FunctionSet) {
    assert_eq!(ctx.ty().unwrap_or_default().as_str(), "test");
    
    let mut funcs = FunctionSet::with_capacity(1);
    funcs.add(UCStr::from_literal(c"hello").ok(), None, hello);
    (None, funcs)
}


fre_rs::function! {
    hello (ctx, _, args) -> Option<as3::String> {
        if args.len() != 1 {panic!("Extension crashed.")}
        
        let words: &str = args[0].try_into().ok()?;
        let words = words.to_lowercase();
        if words.starts_with("hello") {
            Some(as3::String::new(ctx, "Hello! Flash Runtime."))
        } else {None}
    }
}

