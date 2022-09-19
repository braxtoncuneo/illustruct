use typed_arena::Arena;

use crate:: kind::Kind;



struct Context <'context> {
    kinds: Arena<Kind<'context>>,
}




impl <'context> Context <'context> {

    pub fn new() -> Self {
        Context {
            kinds:  Arena::new(),
        }
    }

}


