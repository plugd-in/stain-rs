#[macro_export]
#[doc(hidden)]
/// **Internal:** Non-public API
///
/// Maps from an external (e.g. super) visibility to an internal
/// visibility equal to the external one. This is useful for
/// macro paradigms that generate a `mod` for its generated code
/// that also has some customizable visibility.
macro_rules! __vis_mod_mapper {
    (
        vis: pub,

        pub: {
            $($pub:tt)* // Capture ANY number of token trees
        };
        pub(crate): {
            $($pub_crate:tt)*
        };
        pub(super): {
            $($pub_super:tt)*
        };
        pub(self): {
            $($pub_self:tt)*
        };
    ) => {
        $crate::__vis_mod_mapper! {
            @map {
                vis: pub;

                // We wrap the captured sequences in braces {} so they become
                // a SINGLE token tree when passed to the next stage.
                pub: { $($pub)* },
                pub(crate): { $($pub_crate)* },
                pub(super): { $($pub_super)* },
                pub(self): { $($pub_self)* },
            }
        }
    };

    (
        vis: pub(crate),

        pub: {
            $($pub:tt)* // Capture ANY number of token trees
        };
        pub(crate): {
            $($pub_crate:tt)*
        };
        pub(super): {
            $($pub_super:tt)*
        };
        pub(self): {
            $($pub_self:tt)*
        };
    ) => {
        $crate::__vis_mod_mapper! {
            @map {
                vis: pub(crate);

                // We wrap the captured sequences in braces {} so they become
                // a SINGLE token tree when passed to the next stage.
                pub: { $($pub)* },
                pub(crate): { $($pub_crate)* },
                pub(super): { $($pub_super)* },
                pub(self): { $($pub_self)* },
            }
        }
    };

    (
        vis: pub(super),

        pub: {
            $($pub:tt)* // Capture ANY number of token trees
        };
        pub(crate): {
            $($pub_crate:tt)*
        };
        pub(super): {
            $($pub_super:tt)*
        };
        pub(self): {
            $($pub_self:tt)*
        };
    ) => {
        $crate::__vis_mod_mapper! {
            @map {
                vis: pub(super);

                // We wrap the captured sequences in braces {} so they become
                // a SINGLE token tree when passed to the next stage.
                pub: { $($pub)* },
                pub(crate): { $($pub_crate)* },
                pub(super): { $($pub_super)* },
                pub(self): { $($pub_self)* },
            }
        }
    };

    (
        vis: pub(self),

        pub: {
            $($pub:tt)* // Capture ANY number of token trees
        };
        pub(crate): {
            $($pub_crate:tt)*
        };
        pub(super): {
            $($pub_super:tt)*
        };
        pub(self): {
            $($pub_self:tt)*
        };
    ) => {
        $crate::__vis_mod_mapper! {
            @map {
                vis: pub(self);

                // We wrap the captured sequences in braces {} so they become
                // a SINGLE token tree when passed to the next stage.
                pub: { $($pub)* },
                pub(crate): { $($pub_crate)* },
                pub(super): { $($pub_super)* },
                pub(self): { $($pub_self)* },
            }
        }
    };

    (
        vis: pub(in self),

        pub: {
            $($pub:tt)* // Capture ANY number of token trees
        };
        pub(crate): {
            $($pub_crate:tt)*
        };
        pub(super): {
            $($pub_super:tt)*
        };
        pub(self): {
            $($pub_self:tt)*
        };
    ) => {
        $crate::__vis_mod_mapper! {
            @map {
                vis: pub(self);

                // We wrap the captured sequences in braces {} so they become
                // a SINGLE token tree when passed to the next stage.
                pub: { $($pub)* },
                pub(crate): { $($pub_crate)* },
                pub(super): { $($pub_super)* },
                pub(self): { $($pub_self)* },
            }
        }
    };

    (
        vis: $_catch_all:vis, // At this point, we just assume empty vis.

        pub: {
            $($pub:tt)* // Capture ANY number of token trees
        };
        pub(crate): {
            $($pub_crate:tt)*
        };
        pub(super): {
            $($pub_super:tt)*
        };
        pub(self): {
            $($pub_self:tt)*
        };
    ) => {
        $crate::__vis_mod_mapper! {
            @map {
                vis: pub(self);

                // We wrap the captured sequences in braces {} so they become
                // a SINGLE token tree when passed to the next stage.
                pub: { $($pub)* },
                pub(crate): { $($pub_crate)* },
                pub(super): { $($pub_super)* },
                pub(self): { $($pub_self)* },
            }
        }
    };

    // --- Internal Rules ---

    (@map {
        vis: pub;

        // We match the braces we created in the step above
        pub: { $($pub:tt)* },
        pub(crate): $pub_crate:tt,
        pub(super): $pub_super:tt,
        pub(self): $pub_self:tt,
    }) => {
        $($pub)*
    };

    (@map {
        vis: pub(crate);

        pub: $pub:tt,
        pub(crate): { $($pub_crate:tt)* },
        pub(super): $pub_super:tt,
        pub(self): $pub_self:tt,
    }) => {
        $($pub_crate)*
    };

    (@map {
        vis: pub(super);

        pub: $pub:tt,
        pub(crate): $pub_crate:tt,
        pub(super): { $($pub_super:tt)* },
        pub(self): $pub_self:tt,
    }) => {
        $($pub_super)*
    };

    (@map {
        vis: pub(self); // pub(self) is empty visibility

        pub: $pub:tt,
        pub(crate): $pub_crate:tt,
        pub(super): $pub_super:tt,
        pub(self): { $($pub_self:tt)* },
    }) => {
        $($pub_self)*
    };
}
