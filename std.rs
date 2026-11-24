pub mod prelude {
}

pub mod intrinsics {
    pub fn vec_last<T>(vector: &Vec<T>) -> Option<&T>;
}

pub mod vec {
    impl<T> Vec<T> {
        pub fn last(&self) -> Option<&T> { crate::intrinsics::vec_last(self) }
    }
}
