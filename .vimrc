let $RUST_BACKTRACE=1
map !! iprintln!(
set makeprg=cargo\ run\ --bin\ rust_walker
set wildignore+=target
