/// Macro to auto-generate trait definitions and delegate implementations
///
/// This macro generates a trait and a delegate implementation for a struct.
/// It simplifies manually defining a trait and then using auto_delegate_trait!.
///
/// # Syntax
///
/// ```rust
/// use sql_infra::autogen_delegate_repo_trait;
///
/// autogen_delegate_repo_trait! {
///     impl TraitName for StructName {
///         delegate_to: method_name();
///
///         // Manually specify all trait method signatures
///         async fn method1(&self, param1: Type1) -> ReturnType1;
///         async fn method2(&self, param1: Type1, param2: Type2) -> ReturnType2;
///         fn sync_method(&self, param: Type) -> ReturnType;
///     }
/// }
/// ```
///
/// # Generated code
///
/// The macro invocation above will generate:
///
/// ```rust
/// #[async_trait::async_trait]
/// pub trait TraitName {
///     async fn method1(&self, param1: Type1) -> ReturnType1;
///     async fn method2(&self, param1: Type1, param2: Type2) -> ReturnType2;
///     fn sync_method(&self, param: Type) -> ReturnType;
/// }
///
/// #[async_trait::async_trait]
/// impl TraitName for StructName {
///     async fn method1(&self, param1: Type1) -> ReturnType1 {
///         self.method_name().method1(param1).await
///     }
///     async fn method2(&self, param1: Type1, param2: Type2) -> ReturnType2 {
///         self.method_name().method2(param1, param2).await
///     }
///     fn sync_method(&self, param: Type) -> ReturnType {
///         self.method_name().sync_method(param)
///     }
/// }
/// ```
///
/// # Features
///
/// - **Auto-generate trait**: Create trait definitions from method signatures
/// - **Auto-generate delegate impl**: Generate delegate implementation for the struct
/// - **Async support**: Automatically handle `async` methods and `.await` calls
/// - **Type safety**: Compile-time checking for signature matching
/// - **Simplified syntax**: One macro to do both
///
/// # Limitations
///
/// - The delegate target must implement the same trait
/// - All method signatures must be specified manually (Rust macro system limitation)
/// - Delegated method calls must be simple (no complex expressions)
/// - The generated trait is always public
#[macro_export]
macro_rules! autogen_delegate_repo_trait {
    // Basic form: impl TraitName for StructName
    (
        impl $trait_name:ident for $struct_name:ident {
            delegate_to: $delegate_method:ident();

            $(
                async fn $async_method_name:ident(
                    &self
                    $(, $async_param_name:ident: $async_param_type:ty)*
                ) -> $async_return_type:ty;
            )*

            $(
                fn $sync_method_name:ident(
                    &self
                    $(, $sync_param_name:ident: $sync_param_type:ty)*
                ) -> $sync_return_type:ty;
            )*
        }
    ) => {
        // First, generate the trait definition
        #[async_trait::async_trait]
        pub trait $trait_name {
            $(
                async fn $async_method_name(&self $(, $async_param_name: $async_param_type)*) -> $async_return_type;
            )*

            $(
                fn $sync_method_name(&self $(, $sync_param_name: $sync_param_type)*) -> $sync_return_type;
            )*
        }

        // Then, generate the delegate implementation
        #[async_trait::async_trait]
        impl $trait_name for $struct_name {
            $(
                async fn $async_method_name(&self $(, $async_param_name: $async_param_type)*) -> $async_return_type {
                    self.$delegate_method().$async_method_name($($async_param_name),*).await
                }
            )*

            $(
                fn $sync_method_name(&self $(, $sync_param_name: $sync_param_type)*) -> $sync_return_type {
                    self.$delegate_method().$sync_method_name($($sync_param_name),*)
                }
            )*
        }
    };
}
