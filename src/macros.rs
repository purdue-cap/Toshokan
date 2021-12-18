macro_rules! test_fixture {
    ($type:ty, $name:ident, builder{$($mtd:ident($($arg:expr),*)),*}) => {
        ::paste::paste! {
            #[cfg(test)]
            impl $type {
                pub fn [< test_fixture_ $name >]() -> Self {
                    [< $type Builder >]::default()
                        $(.$mtd($($arg),*))*
                        .build().expect("Test fixture")
                }
            }
        }
    };
    ($type:ty, $name:ident, $expr:expr) => {
        ::paste::paste!{
            #[cfg(test)]
            impl $type {
                pub fn [< test_fixture_ $name >]() -> Self {
                    $expr
                }
            }
        }
    };
}