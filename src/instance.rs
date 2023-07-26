// extern crate proc_macro;

// use proc_macro::TokenStream;

// use quote::quote;
// use syn::{parse_macro_input, DeriveInput};

// #[proc_macro_derive(InstanceData)]
// pub fn instance_data(input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree
//     let input = parse_macro_input!(input as DeriveInput);

//     // Build the output, possibly using quasi-quotation
//     let expanded = quote! {
//         // ...
//     };

//     // Hand the output tokens back to the compiler
//     TokenStream::from(expanded)
// }

use nalgebra::{Matrix4, Vector4};
use std::fmt::Debug;

use crate::plain::Plain;

// TODO: rename to just `Instance`
pub trait InstanceData: Debug {
    fn data(&self) -> &[u8];
    // {
    // self.as_bytes()
    // }
}

impl InstanceData for Box<dyn InstanceData> {
    fn data(&self) -> &[u8] {
        self.as_ref().data()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BasicInstance {
    pub transform: Matrix4<f32>,
    pub atlas_coords: Vector4<f32>,
}

impl InstanceData for BasicInstance {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

unsafe impl Plain for BasicInstance {}
