trait Describe<T>: Show
    fn value(self): T

type Box<T>
    ## The stored value.
    export value: T

impl<T: Show> Describe<T> for Box<T>
    fn value(self): T
        self.value

alias Handler = fn(take Text): Int raises Error
