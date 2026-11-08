// declares a java vtable as a table of fn pointers plus forwarding methods, a subclass const overrides fields with ..PARENT
#[macro_export]
macro_rules! vtable {
    ($class:ident for $object:ident { $( fn $name:ident ( $( $arg:ident : $ty:ty ),* ) $( -> $ret:ty )?; )* }) => {
        #[derive(Clone, Copy)]
        pub struct $class
        {
            pub m_name: &'static str,
            pub m_parent: Option<&'static $class>,
            pub m_interfaces: &'static [&'static str],
            $( pub $name: fn(&std::rc::Rc<$object> $(, $ty)*) $( -> $ret )?, )*
        }

        impl $class
        {
            pub fn isSubclassOf(&self, name: &str) -> bool
            {
                let mut current: Option<&$class> = Some(self);
                while let Some(class) = current
                {
                    if class.m_name == name || class.m_interfaces.contains(&name)
                    {
                        return true;
                    }
                    current = class.m_parent;
                }
                false
            }
        }

        impl $object
        {
            $(
                pub fn $name(self: &std::rc::Rc<Self> $(, $arg: $ty)*) $( -> $ret )?
                {
                    (self.m_class.$name)(self $(, $arg)*)
                }
            )*

            pub fn instanceOf(&self, name: &str) -> bool { self.m_class.isSubclassOf(name) }

            pub fn className(&self) -> &'static str { self.m_class.m_name }
        }
    };
}
