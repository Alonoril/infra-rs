use rancor::Error;
use rkyv::Deserialize;
use rkyv::api::high::HighDeserializer;

pub struct Value {
	pub val0: f64,
	pub val1: Option<String>,
}

#[automatically_derived]
#[doc = "An archived [`Value`]"]
#[derive(::rkyv::bytecheck::CheckBytes)]
#[bytecheck( crate   =   ::   rkyv   ::   bytecheck)]
#[repr(C)]
pub struct ArchivedValue
where
	f64: ::rkyv::Archive,
	Option<String>: ::rkyv::Archive,
{
	#[doc = "The archived counterpart of [`Value::val0`]"]
	pub val0: <f64 as ::rkyv::Archive>::Archived,
	#[doc = "The archived counterpart of [`Value::val1`]"]
	pub val1: <Option<String> as ::rkyv::Archive>::Archived,
}
#[automatically_derived]
#[doc = "The resolver for an archived [`Value`]"]
pub struct ValueResolver
where
	f64: ::rkyv::Archive,
	Option<String>: ::rkyv::Archive,
{
	val0: <f64 as ::rkyv::Archive>::Resolver,
	val1: <Option<String> as ::rkyv::Archive>::Resolver,
}
impl ::rkyv::Archive for Value
where
	f64: ::rkyv::Archive,
	Option<String>: ::rkyv::Archive,
{
	type Archived = ArchivedValue;
	type Resolver = ValueResolver;
	const COPY_OPTIMIZATION: ::rkyv::traits::CopyOptimization<Self> = unsafe {
		::rkyv::traits::CopyOptimization::enable_if(
			0 + ::core::mem::size_of::<f64>() + ::core::mem::size_of::<Option<String>>()
				== ::core::mem::size_of::<Value>()
				&& <f64 as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
				&& ::core::mem::offset_of!(Value, val0)
					== ::core::mem::offset_of!(ArchivedValue, val0)
				&& <Option<String> as ::rkyv::Archive>::COPY_OPTIMIZATION.is_enabled()
				&& ::core::mem::offset_of!(Value, val1)
					== ::core::mem::offset_of!(ArchivedValue, val1),
		)
	};
	#[allow(clippy::unit_arg)]
	fn resolve(&self, resolver: Self::Resolver, out: ::rkyv::Place<Self::Archived>) {
		let field_ptr = unsafe { ::core::ptr::addr_of_mut!((*out.ptr()).val0) };
		let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
		<f64 as ::rkyv::Archive>::resolve(&self.val0, resolver.val0, field_out);
		let field_ptr = unsafe { ::core::ptr::addr_of_mut!((*out.ptr()).val1) };
		let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
		<Option<String> as ::rkyv::Archive>::resolve(&self.val1, resolver.val1, field_out);
	}
}
unsafe impl ::rkyv::traits::Portable for ArchivedValue
where
	f64: ::rkyv::Archive,
	Option<String>: ::rkyv::Archive,
	<f64 as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
	<Option<String> as ::rkyv::Archive>::Archived: ::rkyv::traits::Portable,
{
}

fn main() {}
