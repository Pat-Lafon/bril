#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
fn main() {
    match bril_grammar::parse("") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 2u32,
                        "bril_grammar::parse(\"\")", & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("from \"test\" import;") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 3u32,
                        "bril_grammar::parse(\"from \\\"test\\\" import;\")", & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("from \"test\" import @Foo;") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 4u32,
                        "bril_grammar::parse(\"from \\\"test\\\" import @Foo;\")", & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("from \"test\" import @Foo as @MyFoo;") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 5u32,
                        "bril_grammar::parse(\"from \\\"test\\\" import @Foo as @MyFoo;\")",
                        & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("from \"test\" import @Foo as @MyFoo, @Bar;") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 6u32,
                        "bril_grammar::parse(\"from \\\"test\\\" import @Foo as @MyFoo, @Bar;\")",
                        & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("from \"test\" import @Foo as @MyFoo, @Bar as @MyBar;") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 10u32,
                        "bril_grammar::parse(\"from \\\"test\\\" import @Foo as @MyFoo, @Bar as @MyBar;\")",
                        & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("@main {}") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 14u32,
                        "bril_grammar::parse(\"@main {}\")", & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("@main : int {}") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 15u32,
                        "bril_grammar::parse(\"@main : int {}\")", & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("@main () {}") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 16u32,
                        "bril_grammar::parse(\"@main () {}\")", & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
    match bril_grammar::parse("@main (i : int) {}") {
        tmp => {
            {
                ::std::io::_eprint(
                    format_args!(
                        "[{0}:{1}] {2} = {3:#?}\n", "src/main.rs", 17u32,
                        "bril_grammar::parse(\"@main (i : int) {}\")", & tmp
                    ),
                );
            };
            tmp
        }
    }
        .unwrap();
}
mod bril_grammar {
    use std::path::PathBuf;
    pub struct AbstractProgram {
        pub imports: Vec<Import>,
        pub functions: Vec<AbstractFunction>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AbstractProgram {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "AbstractProgram",
                "imports",
                &self.imports,
                "functions",
                &&self.functions,
            )
        }
    }
    impl rust_sitter::Extract for AbstractProgram {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractProgram_imports(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Vec<Import> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "imports" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractProgram_functions(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Vec<AbstractFunction> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "functions" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_AbstractProgram(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> AbstractProgram {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                AbstractProgram {
                    imports: extract_AbstractProgram_imports(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    functions: extract_AbstractProgram_functions(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                }
            }
            extract_AbstractProgram(node, source)
        }
    }
    pub struct Import {
        _s: (),
        pub path: PathBuf,
        _t: (),
        pub functions: Vec<ImportedFunction>,
        _e: (),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Import {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "Import",
                "_s",
                &self._s,
                "path",
                &self.path,
                "_t",
                &self._t,
                "functions",
                &self.functions,
                "_e",
                &&self._e,
            )
        }
    }
    impl rust_sitter::Extract for Import {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Import__s(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_s" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Import_path(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> PathBuf {
                fn make_transform() -> impl Fn(&str) -> PathBuf {
                    |v| PathBuf::from(v.trim_matches('\"'))
                }
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "path" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = make_transform()(
                                    node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return make_transform()(
                                    node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return make_transform()(
                                node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return make_transform()(
                        node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                    );
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Import__t(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_t" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Import_functions(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Vec<ImportedFunction> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "functions" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Import__e(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_e" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_Import(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> Import {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                Import {
                    _s: extract_Import__s(&mut cursor, source, &mut last_idx),
                    path: extract_Import_path(&mut cursor, source, &mut last_idx),
                    _t: extract_Import__t(&mut cursor, source, &mut last_idx),
                    functions: extract_Import_functions(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    _e: extract_Import__e(&mut cursor, source, &mut last_idx),
                }
            }
            extract_Import(node, source)
        }
    }
    pub struct ImportedFunction {
        pub name: Func,
        pub alias: Option<Alias>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ImportedFunction {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "ImportedFunction",
                "name",
                &self.name,
                "alias",
                &&self.alias,
            )
        }
    }
    impl rust_sitter::Extract for ImportedFunction {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_ImportedFunction_name(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Func {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "name" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_ImportedFunction_alias(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Option<Alias> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "alias" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_ImportedFunction(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> ImportedFunction {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                ImportedFunction {
                    name: extract_ImportedFunction_name(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    alias: extract_ImportedFunction_alias(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                }
            }
            extract_ImportedFunction(node, source)
        }
    }
    pub struct Alias {
        _a: (),
        alias: Func,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Alias {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "Alias",
                "_a",
                &self._a,
                "alias",
                &&self.alias,
            )
        }
    }
    impl rust_sitter::Extract for Alias {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Alias__a(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_a" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Alias_alias(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Func {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "alias" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_Alias(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> Alias {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                Alias {
                    _a: extract_Alias__a(&mut cursor, source, &mut last_idx),
                    alias: extract_Alias_alias(&mut cursor, source, &mut last_idx),
                }
            }
            extract_Alias(node, source)
        }
    }
    pub struct AbstractFunction {
        name: Func,
        args: Option<ArgumentList>,
        ty: Option<OutputType>,
        _l: (),
        _r: (),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AbstractFunction {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "AbstractFunction",
                "name",
                &self.name,
                "args",
                &self.args,
                "ty",
                &self.ty,
                "_l",
                &self._l,
                "_r",
                &&self._r,
            )
        }
    }
    impl rust_sitter::Extract for AbstractFunction {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractFunction_name(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Func {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "name" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractFunction_args(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Option<ArgumentList> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "args" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractFunction_ty(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Option<OutputType> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "ty" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractFunction__l(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_l" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractFunction__r(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_r" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_AbstractFunction(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> AbstractFunction {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                AbstractFunction {
                    name: extract_AbstractFunction_name(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    args: extract_AbstractFunction_args(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    ty: extract_AbstractFunction_ty(&mut cursor, source, &mut last_idx),
                    _l: extract_AbstractFunction__l(&mut cursor, source, &mut last_idx),
                    _r: extract_AbstractFunction__r(&mut cursor, source, &mut last_idx),
                }
            }
            extract_AbstractFunction(node, source)
        }
    }
    pub struct ArgumentList {
        _l: (),
        args: Vec<Argument>,
        _r: (),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ArgumentList {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "ArgumentList",
                "_l",
                &self._l,
                "args",
                &self.args,
                "_r",
                &&self._r,
            )
        }
    }
    impl rust_sitter::Extract for ArgumentList {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_ArgumentList__l(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_l" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_ArgumentList_args(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Vec<Argument> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "args" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_ArgumentList__r(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_r" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_ArgumentList(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> ArgumentList {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                ArgumentList {
                    _l: extract_ArgumentList__l(&mut cursor, source, &mut last_idx),
                    args: extract_ArgumentList_args(&mut cursor, source, &mut last_idx),
                    _r: extract_ArgumentList__r(&mut cursor, source, &mut last_idx),
                }
            }
            extract_ArgumentList(node, source)
        }
    }
    pub struct Argument {
        name: Ident,
        _c: (),
        arg_type: AbstractType,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Argument {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "Argument",
                "name",
                &self.name,
                "_c",
                &self._c,
                "arg_type",
                &&self.arg_type,
            )
        }
    }
    impl rust_sitter::Extract for Argument {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Argument_name(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Ident {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "name" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Argument__c(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_c" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Argument_arg_type(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> AbstractType {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "arg_type" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_Argument(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> Argument {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                Argument {
                    name: extract_Argument_name(&mut cursor, source, &mut last_idx),
                    _c: extract_Argument__c(&mut cursor, source, &mut last_idx),
                    arg_type: extract_Argument_arg_type(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                }
            }
            extract_Argument(node, source)
        }
    }
    pub struct OutputType {
        _c: AbstractType,
        arg_type: AbstractType,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for OutputType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "OutputType",
                "_c",
                &self._c,
                "arg_type",
                &&self.arg_type,
            )
        }
    }
    impl rust_sitter::Extract for OutputType {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_OutputType__c(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> AbstractType {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_c" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_OutputType_arg_type(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> AbstractType {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "arg_type" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_OutputType(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> OutputType {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                OutputType {
                    _c: extract_OutputType__c(&mut cursor, source, &mut last_idx),
                    arg_type: extract_OutputType_arg_type(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                }
            }
            extract_OutputType(node, source)
        }
    }
    pub enum AbstractType {
        Primitive(Ident),
        Parameterized(Ident, (), Box<AbstractType>, ()),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AbstractType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                AbstractType::Primitive(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Primitive",
                        &__self_0,
                    )
                }
                AbstractType::Parameterized(__self_0, __self_1, __self_2, __self_3) => {
                    ::core::fmt::Formatter::debug_tuple_field4_finish(
                        f,
                        "Parameterized",
                        __self_0,
                        __self_1,
                        __self_2,
                        &__self_3,
                    )
                }
            }
        }
    }
    impl rust_sitter::Extract for AbstractType {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractType_Primitive_0(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Ident {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "0" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_AbstractType_Primitive(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> AbstractType {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                AbstractType::Primitive(
                    extract_AbstractType_Primitive_0(&mut cursor, source, &mut last_idx),
                )
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractType_Parameterized_0(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Ident {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "0" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractType_Parameterized_1(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "1" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractType_Parameterized_2(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> Box<AbstractType> {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "2" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_AbstractType_Parameterized_3(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "3" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_AbstractType_Parameterized(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> AbstractType {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                AbstractType::Parameterized(
                    extract_AbstractType_Parameterized_0(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    extract_AbstractType_Parameterized_1(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    extract_AbstractType_Parameterized_2(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                    extract_AbstractType_Parameterized_3(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                )
            }
            match node.child(0).unwrap().kind() {
                "AbstractType_Primitive" => {
                    extract_AbstractType_Primitive(node.child(0).unwrap(), source)
                }
                "AbstractType_Parameterized" => {
                    extract_AbstractType_Parameterized(node.child(0).unwrap(), source)
                }
                _ => ::core::panicking::panic("explicit panic"),
            }
        }
    }
    pub struct Func {
        pub name: String,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Func {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "Func",
                "name",
                &&self.name,
            )
        }
    }
    impl rust_sitter::Extract for Func {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Func_name(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> String {
                fn make_transform() -> impl Fn(&str) -> String {
                    |v| v.strip_prefix("@").unwrap().to_owned()
                }
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "name" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = make_transform()(
                                    node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return make_transform()(
                                    node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return make_transform()(
                                node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return make_transform()(
                        node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                    );
                }
            }
            #[allow(non_snake_case)]
            fn extract_Func(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> Func {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                Func {
                    name: extract_Func_name(&mut cursor, source, &mut last_idx),
                }
            }
            extract_Func(node, source)
        }
    }
    pub struct Ident {
        pub name: String,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Ident {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "Ident",
                "name",
                &&self.name,
            )
        }
    }
    impl rust_sitter::Extract for Ident {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Ident_name(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> String {
                fn make_transform() -> impl Fn(&str) -> String {
                    |v| v.to_owned()
                }
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "name" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = make_transform()(
                                    node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return make_transform()(
                                    node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return make_transform()(
                                node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return make_transform()(
                        node.and_then(|n| n.utf8_text(source).ok()).unwrap(),
                    );
                }
            }
            #[allow(non_snake_case)]
            fn extract_Ident(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> Ident {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                Ident {
                    name: extract_Ident_name(&mut cursor, source, &mut last_idx),
                }
            }
            extract_Ident(node, source)
        }
    }
    struct Whitespace {
        _whitespace: (),
    }
    impl rust_sitter::Extract for Whitespace {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Whitespace__whitespace(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_whitespace" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_Whitespace(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> Whitespace {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                Whitespace {
                    _whitespace: extract_Whitespace__whitespace(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                }
            }
            extract_Whitespace(node, source)
        }
    }
    struct Comment {
        _comment: (),
    }
    impl rust_sitter::Extract for Comment {
        #[allow(non_snake_case)]
        fn extract(
            node: Option<rust_sitter::tree_sitter::Node>,
            source: &[u8],
            last_idx: usize,
        ) -> Self {
            let node = node.unwrap();
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            fn extract_Comment__comment(
                cursor_opt: &mut Option<rust_sitter::tree_sitter::TreeCursor>,
                source: &[u8],
                last_idx: &mut usize,
            ) -> () {
                if let Some(cursor) = cursor_opt.as_mut() {
                    loop {
                        let n = cursor.node();
                        if let Some(name) = cursor.field_name() {
                            if name == "_comment" {
                                let node: Option<rust_sitter::tree_sitter::Node> = Some(n);
                                let out = rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                                if !cursor.goto_next_sibling() {
                                    *cursor_opt = None;
                                }
                                *last_idx = n.end_byte();
                                return out;
                            } else {
                                let node: Option<rust_sitter::tree_sitter::Node> = None;
                                return rust_sitter::Extract::extract(
                                    node,
                                    source,
                                    *last_idx,
                                );
                            }
                        } else {
                            *last_idx = n.end_byte();
                        }
                        if !cursor.goto_next_sibling() {
                            let node: Option<rust_sitter::tree_sitter::Node> = None;
                            return rust_sitter::Extract::extract(
                                node,
                                source,
                                *last_idx,
                            );
                        }
                    }
                } else {
                    let node: Option<rust_sitter::tree_sitter::Node> = None;
                    return rust_sitter::Extract::extract(node, source, *last_idx);
                }
            }
            #[allow(non_snake_case)]
            fn extract_Comment(
                node: rust_sitter::tree_sitter::Node,
                source: &[u8],
            ) -> Comment {
                let mut last_idx = node.start_byte();
                let mut parent_cursor = node.walk();
                let mut cursor = if parent_cursor.goto_first_child() {
                    Some(parent_cursor)
                } else {
                    None
                };
                Comment {
                    _comment: extract_Comment__comment(
                        &mut cursor,
                        source,
                        &mut last_idx,
                    ),
                }
            }
            extract_Comment(node, source)
        }
    }
    extern "C" {
        fn tree_sitter_bril() -> rust_sitter::tree_sitter::Language;
    }
    fn language() -> rust_sitter::tree_sitter::Language {
        unsafe { tree_sitter_bril() }
    }
    pub fn parse(
        input: &str,
    ) -> core::result::Result<AbstractProgram, Vec<rust_sitter::errors::ParseError>> {
        let mut parser = rust_sitter::tree_sitter::Parser::new();
        parser.set_language(language()).unwrap();
        let tree = parser.parse(input, None).unwrap();
        let root_node = tree.root_node();
        if root_node.has_error() {
            let mut errors = ::alloc::vec::Vec::new();
            rust_sitter::errors::collect_parsing_errors(
                &root_node,
                input.as_bytes(),
                &mut errors,
            );
            Err(errors)
        } else {
            use rust_sitter::Extract;
            Ok(rust_sitter::Extract::extract(Some(root_node), input.as_bytes(), 0))
        }
    }
}
