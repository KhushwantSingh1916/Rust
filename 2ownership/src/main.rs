//Ownership is a set of rules that govern how a Rust program manages memory. All programs have to manage the way they use a computer’s memory while running
//memory is managed through a system of ownership with a set of rules that the compiler checks. If any of the rules are violated, the program won’t compile. None of the features of ownership will slow down your program while it’s running.
//Keeping track of what parts of code are using what data on the heap, minimizing the amount of duplicate data on the heap, and cleaning up unused data on the heap so you don’t run out of space are all problems that ownership addresses
//Rules of Ownership:
        //Each value in Rust has an owner.
        // There can only be one owner at a time.
        // When the owner goes out of scope, the value will be dropped.