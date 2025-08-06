/// 自动生成 trait 定义和委托实现的宏
///
/// 这个宏可以同时生成 trait 定义和为指定结构体生成委托实现。
/// 它简化了需要手动定义 trait 然后再使用 auto_delegate_trait! 的流程。
///
/// # 语法
///
/// ```rust
/// use sql_infra::autogen_delegate_repo_trait;
///
/// autogen_delegate_repo_trait! {
///     impl TraitName for StructName {
///         delegate_to: method_name();
///
///         // 手动指定 trait 的所有方法签名
///         async fn method1(&self, param1: Type1) -> ReturnType1;
///         async fn method2(&self, param1: Type1, param2: Type2) -> ReturnType2;
///         fn sync_method(&self, param: Type) -> ReturnType;
///     }
/// }
/// ```
///
/// # 生成的代码
///
/// 上述宏调用会生成：
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
/// # 特性
///
/// - **自动生成 trait**: 根据方法签名自动生成 trait 定义
/// - **自动生成委托实现**: 为指定结构体生成委托实现
/// - **异步支持**: 自动处理 `async` 方法和 `.await` 调用
/// - **类型安全**: 编译时检查方法签名匹配
/// - **简化语法**: 一个宏调用完成两个任务
///
/// # 限制
///
/// - 委托目标必须实现相同的 trait
/// - 需要手动指定所有方法签名（由于 Rust 宏系统限制）
/// - 委托方法调用必须是简单的方法调用（不支持复杂表达式）
/// - 生成的 trait 总是 public 的
#[macro_export]
macro_rules! autogen_delegate_repo_trait {
    // 基本形式：impl TraitName for StructName
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
        // 首先生成 trait 定义
        #[async_trait::async_trait]
        pub trait $trait_name {
            $(
                async fn $async_method_name(&self $(, $async_param_name: $async_param_type)*) -> $async_return_type;
            )*

            $(
                fn $sync_method_name(&self $(, $sync_param_name: $sync_param_type)*) -> $sync_return_type;
            )*
        }

        // 然后生成委托实现
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
