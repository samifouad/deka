use core::Context;

mod build;
mod env;
mod extensions;
mod run;
mod serve;

pub fn run(context: &Context) {
    run::run(context);
}

pub fn build(context: &Context) {
    build::build(context);
}

pub fn serve(context: &Context) {
    serve::serve(context);
}
