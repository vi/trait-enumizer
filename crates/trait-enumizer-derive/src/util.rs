use proc_macro2::TokenStream;

use crate::{Argument, AccessMode};

use super::{TheTrait, ReceiverStyle};

use quote::quote as q;

impl std::fmt::Debug for TheTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TheTrait")
            .field("name", &self.name.to_string())
            .field("methods", &self.methods)
            .finish()
    }
}


impl TheTrait {
    pub(crate) fn receiver_style_that_is_the_most_inconvenient_for_caller(&self) -> ReceiverStyle {
        use ReceiverStyle::{Move, Mut, Ref};
        let mut style = Ref;
        for method in &self.methods {
            match (style, method.receiver_style) {
                (_, Move) => style = ReceiverStyle::Move,
                (_, Ref) => (),
                (Move, Mut) => (),
                (Ref, Mut) => style = ReceiverStyle::Mut,
                (Mut, Mut) => (),
            }
        }
        style
    }
    pub(crate) fn receiver_style_that_is_the_most_inconvenient_for_callee(&self) -> ReceiverStyle {
        use ReceiverStyle::{Move, Mut, Ref};
        let mut style = Move;
        for method in &self.methods {
            match (style, method.receiver_style) {
                (_, Ref) => style = ReceiverStyle::Ref,
                (_, Move) => (),
                (Ref, Mut) => (),
                (Move, Mut) => style = ReceiverStyle::Mut,
                (Mut, Mut) => (),
            }
        }
        style
    }

    #[allow(dead_code)]
    pub(crate) fn sole_receiver_style(&self) -> Option<ReceiverStyle> {
        let mut style = None;
        for method in &self.methods {
            match (style, method.receiver_style) {
                (None, x) => style = Some(x),
                (Some(t), x) if x != t => return None,
                (Some(_t), _x) => (),
            }
        }
        style
    }
}


impl std::fmt::Debug for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = &self.ty;
        f.debug_struct("Argument")
            .field("name", &self.name.to_string())
            .field("ty", &format!("{}", q! {#t}))
            .finish()
    }
}


impl ReceiverStyle {
    pub(crate) fn ts(self) -> TokenStream {
        match self {
            ReceiverStyle::Move => q! {self},
            ReceiverStyle::Ref => q! {&self},
            ReceiverStyle::Mut => q! {&mut self},
        }
    }

    pub(crate) fn identpart(self) -> &'static str {
        match self {
            ReceiverStyle::Move => "Once",
            ReceiverStyle::Mut => "Mut",
            ReceiverStyle::Ref => "",
        }
    }

    pub(crate) fn fn_trait(self) -> TokenStream {
         match self {
            ReceiverStyle::Move => q! {FnOnce},
            ReceiverStyle::Mut => q! {FnMut},
            ReceiverStyle::Ref => q! {Fn},
        }
    }
    pub(crate) fn call_fn_name(self) -> &'static str {
        match self {
           ReceiverStyle::Move => "call_once",
           ReceiverStyle::Mut => "call_mut",
           ReceiverStyle::Ref => "call",
       }
   }
}

impl AccessMode {
    pub(crate) fn code(self) -> TokenStream {
        match self {
            AccessMode::Priv => q!{},
            AccessMode::Pub => q!{pub},
            AccessMode::PubCrate => q!{pub(crate)},
        }
    } 
}
