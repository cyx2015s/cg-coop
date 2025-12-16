#[macro_export]
macro_rules! implement_uniform_block_new {
    // ===================== 无泛型 =====================
    ($struct_name:ident, $($field_name:ident),+ $(,)?) => {
        impl $crate::glium::uniforms::UniformBlock for $struct_name {

            fn matches(
                layout: &$crate::glium::program::BlockLayout,
                base_offset: usize,
            ) -> Result<(), $crate::glium::uniforms::LayoutMismatchError> {
                use $crate::glium::program::BlockLayout;
                use $crate::glium::uniforms::LayoutMismatchError;

                if let BlockLayout::Struct { members } = layout {
                    // layout 中不能有未知字段
                    for (name, _) in members {
                        if $(name != stringify!($field_name) &&)+ true {
                            return Err(LayoutMismatchError::MissingField { name: name.clone() });
                        }
                    }

                    fn matches_from_ty<T: $crate::glium::uniforms::UniformBlock + ?Sized>(_: Option<&T>,
                        layout: &$crate::glium::program::BlockLayout,
                        base_offset: usize,
                    ) -> Result<(), $crate::glium::uniforms::LayoutMismatchError> {
                        <T as $crate::glium::uniforms::UniformBlock>::matches(layout, base_offset)
                    }

                    // 检查每个字段
                    $(
                        let (_, reflected_layout) = members.iter()
                            .find(|(name, _)| name == stringify!($field_name))
                            .ok_or_else(|| LayoutMismatchError::MissingField {
                                name: stringify!($field_name).to_owned()
                            })?;

                        let offset = core::mem::offset_of!($struct_name, $field_name) + base_offset;
                        let field_option = None::<&$struct_name>.map(|v| &v.$field_name);

                        match matches_from_ty(field_option, reflected_layout, offset) {
                            Ok(_) => {},
                            Err(e) => {
                                return Err(LayoutMismatchError::MemberMismatch {
                                    member: stringify!($field_name).to_owned(),
                                    err: Box::new(e),
                                });
                            }
                        }
                    )+

                    Ok(())
                } else {
                    Err(LayoutMismatchError::LayoutMismatch {
                        expected: layout.clone(),
                        obtained: Self::build_layout(base_offset),
                    })
                }
            }

            fn build_layout(base_offset: usize) -> $crate::glium::program::BlockLayout {
                use $crate::glium::program::BlockLayout;

                fn layout_from_ty<T: $crate::glium::uniforms::UniformBlock + ?Sized>(_: Option<&T>, offset: usize) -> BlockLayout {
                    <T as $crate::glium::uniforms::UniformBlock>::build_layout(offset)
                }

                BlockLayout::Struct {
                    members: vec![
                        $(
                            (
                                stringify!($field_name).to_owned(),
                                {
                                    let offset = core::mem::offset_of!($struct_name, $field_name) + base_offset;
                                    let field_option = None::<&$struct_name>.map(|v| &v.$field_name);
                                    layout_from_ty(field_option, offset)
                                }
                            ),
                        )+
                    ],
                }
            }
        }
    };
}
